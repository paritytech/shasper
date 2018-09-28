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

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc))]

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

#[cfg(feature = "std")]
extern crate core;

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

#[cfg(feature = "std")]
pub mod alloc {
	pub use std::boxed;
	pub use std::vec;
}

const ENCODE_ERR: &str = "derive(SszEncode) failed";

#[proc_macro_derive(SszEncode, attributes(ssz_codec))]
pub fn encode_derive(input: TokenStream) -> TokenStream {
	let input: DeriveInput = syn::parse(input).expect(ENCODE_ERR);
	let name = &input.ident;

	let generics = add_trait_bounds(input.generics, parse_quote!(::ssz::Encode));
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	let self_ = quote!(self);
	let dest_ = quote!(dest);
	let sorted = sorted(&input.attrs);
	let encoding = encode::quote(&input.data, name, &self_, &dest_, sorted);

	let expanded = quote! {
		impl #impl_generics ::ssz::Encode for #name #ty_generics #where_clause {
			fn encode_to<EncOut: ::ssz::Output>(&#self_, #dest_: &mut EncOut) {
				#encoding
			}
		}
	};

	expanded.into()
}

#[proc_macro_derive(SszDecode, attributes(ssz_codec))]
pub fn decode_derive(input: TokenStream) -> TokenStream {
	let input: DeriveInput = syn::parse(input).expect(ENCODE_ERR);
	let name = &input.ident;

	let generics = add_trait_bounds(input.generics, parse_quote!(::ssz::Decode));
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	let input_ = quote!(input);
	let sorted = sorted(&input.attrs);
	let decoding = decode::quote(&input.data, name, &input_, sorted);

	let expanded = quote! {
		impl #impl_generics ::ssz::Decode for #name #ty_generics #where_clause {
			fn decode<DecIn: ::ssz::Input>(#input_: &mut DecIn) -> Option<Self> {
				#decoding
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

			if seg.ident == Ident::new("ssz_codec", seg.ident.span()) {
				assert_eq!(attr.path.segments.len(), 1);

				let meta = attr.interpret_meta();
				if let Some(syn::Meta::List(ref l)) = meta {
					if let syn::NestedMeta::Meta(syn::Meta::Word(ref w)) = l.nested.last().unwrap().value() {
						assert_eq!(w, &Ident::new("sorted", w.span()));
						true
					} else {
						panic!("Invalid syntax for `ssz_codec` attribute: Expected sorted.");
					}
				} else {
					panic!("Invalid syntax for `ssz_codec` attribute: Expected sorted.");
				}
			} else {
				false
			}
		}).unwrap_or(false)
	})
}

fn index(v: &syn::Variant, i: usize) -> proc_macro2::TokenStream {
	// look for an index in attributes
	let index = v.attrs.iter().filter_map(|attr| {
		let pair = attr.path.segments.first()?;
		let seg = pair.value();

		if seg.ident == Ident::new("ssz_codec", seg.ident.span()) {
			assert_eq!(attr.path.segments.len(), 1);

			let meta = attr.interpret_meta();
			if let Some(syn::Meta::List(ref l)) = meta {
				if let syn::NestedMeta::Meta(syn::Meta::NameValue(ref nv)) = l.nested.last().unwrap().value() {
					assert_eq!(nv.ident, Ident::new("index", nv.ident.span()));
					if let syn::Lit::Str(ref s) = nv.lit {
						let byte: u8 = s.value().parse().expect("Numeric index expected.");
						return Some(byte)
					}
					panic!("Invalid syntax for `ssz_codec` attribute: Expected string literal.")
				}
			}
			panic!("Invalid syntax for `ssz_codec` attribute: Expected `name = value` pair.")
		} else {
			None
		}
	}).next();

	// then fallback to discriminant or just index
	index.map(|i| quote! { #i })
		.unwrap_or_else(|| v.discriminant
			.as_ref()
			.map(|&(_, ref expr)| quote! { #expr })
			.unwrap_or_else(|| quote! { #i })
		)
}
