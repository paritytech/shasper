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

#![recursion_limit="128"]

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

	let prefixable_generics = add_trait_bounds(input.generics.clone(), parse_quote!(::ssz::Prefixable));
	let (prefixable_impl_generics, prefixable_ty_generics, prefixable_where_clause) = prefixable_generics.split_for_impl();

	let encode_generics = add_trait_bounds(input.generics.clone(), parse_quote!(::ssz::Encode));
	let (encode_impl_generics, encode_ty_generics, encode_where_clause) = encode_generics.split_for_impl();

	let decode_generics = add_trait_bounds(input.generics.clone(), parse_quote!(::ssz::Decode));
	let (decode_impl_generics, decode_ty_generics, decode_where_clause) = decode_generics.split_for_impl();

	let hash_generics = add_trait_bounds(input.generics.clone(), parse_quote!(::ssz::Hashable));
	let (hash_impl_generics, hash_ty_generics, hash_where_clause) = hash_generics.split_for_impl();

	let self_ = quote!(self);
	let dest_ = quote!(dest);
	let input_ = quote!(input);
	let hash_param_ = quote!(H);
	let sorted = sorted(&input.attrs);
	let prefixing = prefixable::quote(&input.data, &dest_);
	let encoding = encode::quote(&input.data, &self_, &dest_, sorted);
	let decoding = decode::quote(&input.data, name, &input_, sorted);
	let hashing = hash::quote(&input.data, &self_, &dest_, &hash_param_, false);
	let truncate_hashing = hash::quote(&input.data, &self_, &dest_, &hash_param_, true);

	let expanded = quote! {
		impl #prefixable_impl_generics ::ssz::Prefixable for #name #prefixable_ty_generics #prefixable_where_clause {
			fn prefixed() -> bool {
				let mut #dest_ = false;
				#prefixing
				#dest_
			}
		}

		impl #encode_impl_generics ::ssz::Encode for #name #encode_ty_generics #encode_where_clause {
			fn encode_to<EncOut: ::ssz::Output>(&#self_, d: &mut EncOut) {
				if Self::prefixed() {
					let mut bytes = ::ssz::prelude::Vec::new();
					{
						let #dest_ = &mut bytes;
						#encoding
					}
					(bytes.len() as u32).encode_to(d);
					d.write(&bytes);
				} else {
					let #dest_ = d;
					#encoding
				}
			}
		}

		impl #decode_impl_generics ::ssz::Decode for #name #decode_ty_generics #decode_where_clause {
			fn decode_as<DecIn: ::ssz::Input>(#input_: &mut DecIn) -> Option<(Self, usize)> {
				let mut l = 0;
				let len = if Self::prefixed() {
					use ::ssz::Decode;

					let (len, i) = <u32>::decode_as(#input_)?;
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

		impl #hash_impl_generics ::ssz::Hashable for #name #hash_ty_generics #hash_where_clause {
			fn hash<#hash_param_ : ::ssz::hash_db::Hasher>(&self) -> H::Out {
				let mut #dest_ = ::ssz::prelude::Vec::new();
				#hashing
				#hash_param_ :: hash(& #dest_ )
			}

			fn truncated_hash<#hash_param_ : ::ssz::hash_db::Hasher>(&self) -> H::Out {
				let mut #dest_ = ::ssz::prelude::Vec::new();
				#truncate_hashing
				#hash_param_ :: hash(& #dest_ )
			}
		}
	};

	expanded.into()
}

fn add_trait_bounds(mut generics: Generics, bounds: syn::TypeParamBound) -> Generics {
	for param in &mut generics.params {
		if let GenericParam::Type(ref mut type_param) = *param {
			type_param.bounds.push(bounds.clone());
		}
	}
	generics
}

fn sorted(attrs: &[syn::Attribute]) -> bool {
	attrs.iter().any(|attr| {
		attr.path.segments.first().map(|pair| {
			let seg = pair.value();

			if seg.ident == Ident::new("ssz", seg.ident.span()) {
				assert_eq!(attr.path.segments.len(), 1);

				let meta = attr.interpret_meta();
				if let Some(syn::Meta::List(ref l)) = meta {
					if let syn::NestedMeta::Meta(syn::Meta::Word(ref w)) = l.nested.last().unwrap().value() {
						assert_eq!(w, &Ident::new("sorted", w.span()));
						true
					} else {
						panic!("Invalid syntax for `ssz` attribute: Expected sorted.");
					}
				} else {
					panic!("Invalid syntax for `ssz` attribute: Expected sorted.");
				}
			} else {
				false
			}
		}).unwrap_or(false)
	})
}
