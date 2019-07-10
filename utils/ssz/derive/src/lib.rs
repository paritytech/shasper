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
use deriving::{struct_fields, attribute_value};

use proc_macro::TokenStream;

#[proc_macro_derive(Ssz, attributes(ssz, bm))]
pub fn into_tree_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let config_trait = attribute_value("bm", &input.attrs, "config_trait");

    let encode_fields = struct_fields(&input.data)
        .expect("Not supported derive type")
        .iter()
        .map(|f| {
            let name = &f.ident;
			let ty = &f.ty;

            quote_spanned! { f.span() => {
				if <#ty as ssz::SizeType>::is_fixed() {
					series.0.push(ssz::SeriesItem::Fixed(ssz::Encode::encode(&self.#name)));
				} else {
					series.0.push(ssz::SeriesItem::Variable(ssz::Encode::encode(&self.#name)));
				}
            } }
        });

	let is_fixed_fields = struct_fields(&input.data)
        .expect("Not supported derive type")
        .iter()
        .map(|f| {
			let ty = &f.ty;

			quote_spanned! { f.span() => {
				<#ty as ssz::SizeType>::is_fixed()
			} }
		});

    let basic_expanded = quote! {
		impl ssz::SizeType for #name {
			fn is_fixed() -> bool {
				[#(#is_fixed_fields),*].iter().all(|v| *v)
			}
		}

        impl ssz::Composite for #name { }

		impl ssz::Encode for #name {
			fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
				let mut series = ssz::Series::default();
				#(#encode_fields)*
				f(&series.encode())
			}
		}
    };

	let config_expanded = if let Some(config_trait) = config_trait {
		let config_trait = config_trait.parse::<syn::TraitBound>().expect("Invalid syntax");

		let size_fields = struct_fields(&input.data)
			.expect("Not supported derive type")
			.iter()
			.map(|f| {
				let ty = &f.ty;

				quote_spanned! { f.span() => {
					<#ty as ssz::SizeFromConfig<C>>::size_from_config(config)
				} }
			}).collect::<Vec<_>>();

		let size_fields2 = size_fields.clone();

		let decode_fields = struct_fields(&input.data)
			.expect("Not supported derive type")
			.iter()
			.enumerate()
			.map(|(i, f)| {
				let name = &f.ident;
				let ty = &f.ty;

                quote_spanned! {
                    f.span() =>
                        #name: match &series.0[#i] {
							ssz::SeriesItem::Fixed(ref fixed) => {
								if <#ty as ssz::SizeType>::is_fixed() {
									<#ty as ssz::DecodeWithConfig<C>>::decode_with_config(
										fixed,
										config
									)?
								} else {
									return Err(ssz::Error::InvalidType)
								}
							},
							ssz::SeriesItem::Variable(ref variable) => {
								if <#ty as ssz::SizeType>::is_variable() {
									<#ty as ssz::DecodeWithConfig<C>>::decode_with_config(
										variable,
										config
									)?
								} else {
									return Err(ssz::Error::InvalidType)
								}
							},
						},
                }
			});

		quote! {
			impl<C: #config_trait> ssz::SizeFromConfig<C> for #name {
				fn size_from_config(config: &C) -> Option<usize> {
					[#(#size_fields),*].iter().fold(Some(0), |acc, size| {
						match size {
							Some(size) => acc.map(|acc| acc + size),
							None => None
						}
					})
				}
			}

			impl<C: #config_trait> ssz::DecodeWithConfig<C> for #name {
				fn decode_with_config(value: &[u8], config: &C) -> Result<Self, ssz::Error> {
					let types = [#(#size_fields2),*];
					let series = ssz::Series::decode_vector(value, &types)?;
					Ok(Self {
						#(#decode_fields)*
					})
				}
			}
		}
	} else {
		let size_fields = struct_fields(&input.data)
			.expect("Not supported derive type")
			.iter()
			.map(|f| {
				let ty = &f.ty;

				quote_spanned! { f.span() => {
					<#ty as ssz::KnownSize>::size()
				} }
			}).collect::<Vec<_>>();

		let size_fields2 = size_fields.clone();

		let decode_fields = struct_fields(&input.data)
			.expect("Not supported derive type")
			.iter()
			.enumerate()
			.map(|(i, f)| {
				let name = &f.ident;
				let ty = &f.ty;

                quote_spanned! {
                    f.span() =>
                        #name: match &series.0[#i] {
							ssz::SeriesItem::Fixed(ref fixed) => {
								if <#ty as ssz::SizeType>::is_fixed() {
									<#ty as ssz::Decode>::decode(
										fixed,
									)?
								} else {
									return Err(ssz::Error::InvalidType)
								}
							},
							ssz::SeriesItem::Variable(ref variable) => {
								if <#ty as ssz::SizeType>::is_variable() {
									<#ty as ssz::Decode>::decode(
										variable,
									)?
								} else {
									return Err(ssz::Error::InvalidType)
								}
							},
						},
                }
			});

		quote! {
			impl ssz::KnownSize for #name {
				fn size() -> Option<usize> {
					[#(#size_fields),*].iter().fold(Some(0), |acc, size| {
						match size {
							Some(size) => acc.map(|acc| acc + size),
							None => None
						}
					})
				}
			}

			impl<C> ssz::SizeFromConfig<C> for #name {
				fn size_from_config(_config: &C) -> Option<usize> {
					<Self as ssz::KnownSize>::size()
				}
			}

			ssz::impl_decode_with_empty_config!(#name);
			impl ssz::Decode for #name {
				fn decode(value: &[u8]) -> Result<Self, ssz::Error> {
					let types = [#(#size_fields2),*];
					let series = ssz::Series::decode_vector(value, &types)?;
					Ok(Self {
						#(#decode_fields)*
					})
				}
			}
		}
	};

	let expanded = quote! {
		#basic_expanded

		#config_expanded
	};

    proc_macro::TokenStream::from(expanded)
}
