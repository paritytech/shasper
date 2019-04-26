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

use proc_macro2::{Span, TokenStream, Ident};
use syn::{
	Data, Fields,
	spanned::Spanned,
};
use super::has_attr;

pub fn quote(data: &Data, type_name: &Ident, input: &TokenStream, sorted: bool) -> TokenStream {
	let call_site = Span::call_site();
	match *data {
		Data::Struct(ref data) => match data.fields {
			Fields::Named(_) | Fields::Unnamed(_) => create_instance(
				call_site,
				quote! { #type_name },
				input,
				&data.fields,
				sorted,
			),
			Fields::Unit => {
				quote_spanned! {call_site =>
					#type_name
				}
			},
		},
		Data::Enum(_) => panic!("Enum types are not supported."),
		Data::Union(_) => panic!("Union types are not supported."),
	}
}

fn create_instance(call_site: Span, name: TokenStream, input: &TokenStream, fields: &Fields, sorted: bool) -> TokenStream {
	match *fields {
		Fields::Named(ref fields) => {
			let mut named_fields: Vec<_> = fields.named.iter().collect();

			if sorted {
				named_fields.sort_by(|a, b| {
					a.ident.cmp(&b.ident)
				})
			}

			let recurse = named_fields.iter().map(|f| {
				let name = &f.ident;
				let field = quote_spanned!(call_site => #name);
				let skip = has_attr(&f.attrs, "skip_default");

				if skip {
					quote_spanned! { f.span() => #field: Default::default() }
				} else {
					quote_spanned! {
						f.span() =>
							#field: {
								let (value, i) = ::ssz::Decode::decode_as(#input)?;
								l += i;
								value
							}
					}
				}
			});

			quote_spanned! {call_site =>
				#name {
					#( #recurse, )*
				}
			}
		},
		Fields::Unnamed(ref fields) => {
			let recurse = fields.unnamed.iter().map(|f| {
				let skip = has_attr(&f.attrs, "skip_default");

				if skip {
					quote! { Default::default() }
				} else {
					quote_spanned! {
						f.span() => {
							let (value, i) = ::ssz::Decode::decode_as(#input)?;
							l += i;
							value
						}
					}
				}
			});

			quote_spanned! {call_site =>
				#name (
					#( #recurse, )*
				)
			}
		},
		Fields::Unit => {
			quote_spanned! {call_site =>
				#name
			}
		},
	}
}
