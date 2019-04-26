// Copyright 2017, 2018 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Derives serialization and deserialization codec for complex structs for simple marshalling.

#![recursion_limit="256"]

extern crate proc_macro;
extern crate proc_macro2;

#[macro_use]
extern crate syn;

#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::{DeriveInput, Generics, GenericParam, Ident};

mod decode;
mod encode;
mod prefixable;
mod hash;

#[cfg(feature = "std")]
mod alloc {
	pub use std::boxed;
	pub use std::vec;
}

const ENCODE_ERR: &str = "derive(SszEncode) failed";

#[proc_macro_derive(Ssz, attributes(ssz))]
pub fn derive(input: TokenStream) -> TokenStream {
	let input: DeriveInput = syn::parse(input).expect(ENCODE_ERR);
	let name = &input.ident;

	let hash_param_ = quote!(H);
	let digest_param_ = quote!(D);

	let no_bounds = no_bound(&input.attrs);

	let prefixable_generics = add_trait_bounds(input.generics.clone(), parse_quote!(::ssz::Prefixable), &no_bounds);
	let (prefixable_impl_generics, prefixable_ty_generics, prefixable_where_clause) = prefixable_generics.split_for_impl();

	let encode_generics = add_trait_bounds(input.generics.clone(), parse_quote!(::ssz::Encode), &no_bounds);
	let (encode_impl_generics, encode_ty_generics, encode_where_clause) = encode_generics.split_for_impl();

	let decode_generics = add_trait_bounds(input.generics.clone(), parse_quote!(::ssz::Decode), &no_bounds);
	let (decode_impl_generics, decode_ty_generics, decode_where_clause) = decode_generics.split_for_impl();

	let hash_generics = add_trait_bounds(input.generics.clone(), parse_quote!(::ssz::Hashable<#hash_param_>), &no_bounds);
	let mut hash_impl_generics = hash_generics.clone();
	let mut hash_param: syn::TypeParam = parse_quote!(#hash_param_);
	hash_param.bounds.push(parse_quote!(::ssz::hash_db::Hasher));
	hash_impl_generics.params.push(hash_param.into());
	let (_, hash_ty_generics, hash_where_clause) = hash_generics.split_for_impl();

	let digest_generics = add_trait_bounds(input.generics.clone(), parse_quote!(::ssz::Digestible<#digest_param_>), &no_bounds);
	let mut digest_impl_generics = digest_generics.clone();
	let mut digest_param: syn::TypeParam = parse_quote!(#digest_param_);
	digest_param.bounds.push(parse_quote!(::ssz::digest::Digest));
	digest_impl_generics.params.push(digest_param.into());
	let (_, digest_ty_generics, digest_where_clause) = digest_generics.split_for_impl();

	let composite_generics = input.generics.clone();
	let (composite_impl_generics, composite_ty_generics, composite_where_clause) = composite_generics.split_for_impl();

	let self_ = quote!(self);
	let dest_ = quote!(dest);
	let input_ = quote!(input);
	let sorted = has_attr(&input.attrs, "sorted");
	let no_decode = has_attr(&input.attrs, "no_decode");
	let no_encode = has_attr(&input.attrs, "no_encode");

	let prefixing = prefixable::quote(&input.data, &dest_);
	let (encoding, decodable) = encode::quote(&input.data, &self_, &dest_, sorted);
	let decoding = decode::quote(&input.data, name, &input_, sorted);
	let hashing = hash::quote(&input.data, &self_, &dest_, &hash_param_, false, false);
	let digesting = hash::quote(&input.data, &self_, &dest_, &digest_param_, false, true);
	let truncate_hashing = hash::quote(&input.data, &self_, &dest_, &hash_param_, true, false);
	let truncate_digesting = hash::quote(&input.data, &self_, &dest_, &digest_param_, true, true);

	let decode = if no_decode || !decodable {
		quote! { }
	} else {
		quote! {
			impl #decode_impl_generics ::ssz::Decode for #name #decode_ty_generics #decode_where_clause {
				fn decode_as<DecIn: ::ssz::Input>(#input_: &mut DecIn) -> Option<(Self, usize)> {
					use ::ssz::Prefixable;

					let mut l = 0;
					let len = if Self::prefixed() {
						let (len, i) = <u32 as ::ssz::Decode>::decode_as(#input_)?;
						l += i;
						Some(len as usize)
					} else {
						None
					};
					let ol = l;
					let value = #decoding;
					if let Some(len) = len {
						if l - ol != len {
							return None
						}
					}
					Some((value, l))
				}
			}
		}
	};

	let encode = if no_encode {
		quote! { }
	} else {
		quote! {
			impl #encode_impl_generics ::ssz::Encode for #name #encode_ty_generics #encode_where_clause {
				fn encode_to<EncOut: ::ssz::Output>(&#self_, d: &mut EncOut) {
					use ::ssz::Prefixable;

					if Self::prefixed() {
						let mut bytes = ::ssz::prelude::Vec::new();
						{
							let #dest_ = &mut bytes;
							#encoding
						}
						::ssz::Encode::encode_to(&(bytes.len() as u32), d);
						d.write(&bytes);
					} else {
						let #dest_ = d;
						#encoding
					}
				}
			}
		}
	};

	let expanded = quote! {
		#[allow(unused_imports)]
		impl #prefixable_impl_generics ::ssz::Prefixable for #name #prefixable_ty_generics #prefixable_where_clause {
			fn prefixed() -> bool {
				let mut #dest_ = false;
				#prefixing
				#dest_
			}
		}

		#encode

		#decode

		impl #composite_impl_generics ::ssz::Composite for #name #composite_ty_generics #composite_where_clause { }

		impl #hash_impl_generics ::ssz::Hashable< #hash_param_ > for #name #hash_ty_generics #hash_where_clause {
			fn hash(&self) -> #hash_param_ :: Out {
				let mut #dest_ = ::ssz::prelude::Vec::new();
				#hashing
				let len = #dest_.len() as u32;
				::ssz::hash::hash_db_hasher::merkleize :: <#hash_param_> (
					#dest_
				)
			}

			fn truncated_hash(&self) -> #hash_param_ :: Out {
				let mut #dest_ = ::ssz::prelude::Vec::new();
				#truncate_hashing
				let len = #dest_.len() as u32;
				::ssz::hash::hash_db_hasher::merkleize :: <#hash_param_> (
					#dest_
				)
			}
		}

		impl #digest_impl_generics ::ssz::Digestible< #digest_param_ > for #name #digest_ty_generics #digest_where_clause {
			fn hash(&self) -> ::ssz::generic_array::GenericArray< u8, #digest_param_ :: OutputSize > {
				let mut #dest_ = ::ssz::prelude::Vec::new();
				#digesting
				let len = #dest_.len() as u32;
				::ssz::hash::digest_hasher::merkleize :: <#digest_param_> (
					#dest_
				)
			}

			fn truncated_hash(&self) -> ::ssz::generic_array::GenericArray< u8, #digest_param_ :: OutputSize > {
				let mut #dest_ = ::ssz::prelude::Vec::new();
				#truncate_digesting
				let len = #dest_.len() as u32;
				::ssz::hash::digest_hasher::merkleize :: <#digest_param_> (
					#dest_
				)
			}
		}
	};

	expanded.into()
}

fn add_trait_bounds(mut generics: Generics, bounds: syn::TypeParamBound, no_bounds: &[Ident]) -> Generics {
	for param in &mut generics.params {
		if let GenericParam::Type(ref mut type_param) = *param {
			if !no_bounds.iter().any(|ident| ident.to_string() == type_param.ident.to_string()) {
				type_param.bounds.push(bounds.clone());
			}
		}
	}
	generics
}

fn no_bound(attrs: &[syn::Attribute]) -> Vec<Ident> {
	let mut ret = Vec::new();
	for attr in attrs {
		attr.path.segments.first().map(|pair| {
			let seg = pair.value();

			if seg.ident == Ident::new("ssz", seg.ident.span()) {
				assert_eq!(attr.path.segments.len(), 1);

				let meta = attr.interpret_meta();
				if let Some(syn::Meta::List(ref l)) = meta {
					for a in &l.nested {
						if let syn::NestedMeta::Meta(syn::Meta::List(ref l)) = a {
							if l.ident == Ident::new("no_bound", l.ident.span()) {
								for v in &l.nested {
									if let syn::NestedMeta::Meta(syn::Meta::Word(ref w)) = v {
										ret.push(w.clone());
									} else {
										panic!("Invalid ssz attribute syntax");
									}
								}
							}
						}
					}
				}
			}
		});
	}

	ret
}

fn has_attr(attrs: &[syn::Attribute], s: &str) -> bool {
	attrs.iter().any(|attr| {
		attr.path.segments.first().map(|pair| {
			let seg = pair.value();

			if seg.ident == Ident::new("ssz", seg.ident.span()) {
				assert_eq!(attr.path.segments.len(), 1);

				let meta = attr.interpret_meta();
				if let Some(syn::Meta::List(ref l)) = meta {
					for a in &l.nested {
						if let syn::NestedMeta::Meta(syn::Meta::Word(ref w)) = a {
							if w == &Ident::new(s, w.span()) {
								return true
							}
						}
					}
					false
				} else {
					panic!("Invalid syntax for `ssz` attribute.");
				}
			} else {
				false
			}
		}).unwrap_or(false)
	})
}
