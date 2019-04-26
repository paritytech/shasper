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

use proc_macro2::{Span, TokenStream};
use syn::{
	Data, Field, Fields, Ident, Index,
	punctuated::Punctuated,
	spanned::Spanned,
	token::Comma,
};
use super::has_attr;

type FieldsList = Punctuated<Field, Comma>;

fn encode_fields<F>(
	dest: &TokenStream,
	fields: &FieldsList,
	field_name: F,
	sorted: bool,
) -> (TokenStream, bool) where
	F: Fn(usize, &Option<Ident>) -> TokenStream,
{
	let mut fields: Vec<_> = fields.iter().collect();

	if sorted {
		fields.sort_by(|a, b| {
			a.ident.cmp(&b.ident)
		})
	}

	let mut decodable = true;

	let recurse = fields.iter().enumerate().map(|(i, f)| {
		let field = field_name(i, &f.ident);
		let use_fixed = has_attr(&f.attrs, "use_fixed");
		let skip = has_attr(&f.attrs, "skip_default");

		if skip {
			quote! {
				();
			}
		} else if use_fixed {
			decodable = false;
			quote_spanned! { f.span() =>
				::ssz::Encode::encode_to(&::ssz::Fixed(#field.as_ref()), #dest);
			}
		} else {
			quote_spanned! { f.span() =>
				::ssz::Encode::encode_to(#field, #dest);
			}
		}
	});

	(quote! {
		#( #recurse )*
	}, decodable)
}

pub fn quote(data: &Data, self_: &TokenStream, dest: &TokenStream, sorted: bool) -> (TokenStream, bool) {
	let call_site = Span::call_site();
	match *data {
		Data::Struct(ref data) => match data.fields {
			Fields::Named(ref fields) => encode_fields(
				dest,
				&fields.named,
				|_, name| quote_spanned!(call_site => &#self_.#name),
				sorted,
			),
			Fields::Unnamed(ref fields) => encode_fields(
				dest,
				&fields.unnamed,
				|i, _| {
					let index = Index { index: i as u32, span: call_site };
					quote_spanned!(call_site => &#self_.#index)
				},
				sorted,
			),
			Fields::Unit => (quote_spanned! { call_site =>
				drop(#dest);
			}, true),
		},
		Data::Enum(_) => panic!("Enum types are not supported."),
		Data::Union(_) => panic!("Union types are not supported."),
	}
}
