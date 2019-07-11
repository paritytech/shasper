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

use quote::{quote, quote_spanned};
use syn::{parse_macro_input, DeriveInput};
use syn::spanned::Spanned;
use deriving::{struct_fields, has_attribute};

use proc_macro::TokenStream;

#[proc_macro_derive(Codec, attributes(ssz, bm))]
pub fn codec_derive(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

	let fields = struct_fields(&input.data)
		.expect("Not supported derive type")
		.iter()
		.map(|f| {
			let ty = &f.ty;

			quote_spanned! { f.span() => <#ty as ssz::Codec>::Size }
		});

	let expanded = quote! {
		impl ssz::Codec for #name {
			type Size = ssz::sum!(#(#fields),*);
		}
	};

	proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(Encode, attributes(ssz, bm))]
pub fn encode_derive(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

	let fields = struct_fields(&input.data)
        .expect("Not supported derive type")
        .iter()
        .map(|f| {
            let name = &f.ident;
			let ty = &f.ty;

			let encode = if has_attribute("bm", &f.attrs, "compact") {
				quote_spanned! { f.span() => {
					ssz::Encode::encode(&ssz::CompactRef(&self.#name))
				} }
			} else {
				quote_spanned! { f.span() => {
					ssz::Encode::encode(&self.#name)
				} }
			};

            quote_spanned! { f.span() => {
				if <<#ty as ssz::Codec>::Size as ssz::Size>::is_fixed() {
					series.0.push(ssz::SeriesItem::Fixed(#encode));
				} else {
					series.0.push(ssz::SeriesItem::Variable(#encode));
				}
            } }
        });

	let expanded = quote! {
		impl ssz::Encode for #name {
			fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
				let mut series = ssz::Series::default();
				#(#fields)*
				f(&series.encode())
			}
		}
	};

	proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(Decode, attributes(ssz, bm))]
pub fn decode_derive(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

	let size_fields = struct_fields(&input.data)
        .expect("Not supported derive type")
        .iter()
        .map(|f| {
			let ty = &f.ty;

			quote_spanned! {
				f.span() =>
					<<#ty as ssz::Codec>::Size as ssz::Size>::size()
			}
		});

	let fields = struct_fields(&input.data)
        .expect("Not supported derive type")
        .iter()
		.enumerate()
        .map(|(i, f)| {
            let name = &f.ident;
			let ty = &f.ty;

			let decode = if has_attribute("bm", &f.attrs, "compact") {
				quote_spanned! { f.span() => {
					<ssz::Compact<#ty> as ssz::Decode>::decode(item)?.0
				} }
			} else {
				quote_spanned! { f.span() => {
					<#ty as ssz::Decode>::decode(item)?
				} }
			};

			quote_spanned! {
                f.span() =>
                    #name: match &series.0[#i] {
						ssz::SeriesItem::Fixed(ref item) => {
							if <<#ty as ssz::Codec>::Size as ssz::Size>::is_fixed() {
								#decode
							} else {
								return Err(ssz::Error::InvalidType)
							}
						},
						ssz::SeriesItem::Variable(ref item) => {
							if <<#ty as ssz::Codec>::Size as ssz::Size>::is_variable() {
								#decode
							} else {
								return Err(ssz::Error::InvalidType)
							}
						},
					},
            }
		});

	let expanded = quote! {
		impl ssz::Decode for #name {
			fn decode(value: &[u8]) -> Result<Self, ssz::Error> {
				let types = [#(#size_fields),*];
				let series = ssz::Series::decode_vector(value, &types)?;
				Ok(Self {
					#(#fields)*
				})
			}
		}
	};

	proc_macro::TokenStream::from(expanded)
}
