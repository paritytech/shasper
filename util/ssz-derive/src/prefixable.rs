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

use proc_macro2::TokenStream;
use syn::{
	Data, Field, Fields,
	punctuated::Punctuated,
	spanned::Spanned,
	token::Comma,
};

type FieldsList = Punctuated<Field, Comma>;

fn encode_fields(
	dest: &TokenStream,
	fields: &FieldsList,
) -> TokenStream {
	let fields: Vec<_> = fields.iter().collect();

	let recurse = fields.iter().map(|f| {
		let ty = &f.ty;

		quote_spanned! { f.span() =>
			{
				use ::ssz::Prefixable;
				#dest = #dest || <#ty>::prefixed();
			}
		}
	});

	quote! {
		#( #recurse )*
	}
}

pub fn quote(data: &Data, dest: &TokenStream) -> TokenStream {
	match *data {
		Data::Struct(ref data) => match data.fields {
			Fields::Named(ref fields) => encode_fields(
				dest,
				&fields.named,
			),
			Fields::Unnamed(ref fields) => encode_fields(
				dest,
				&fields.unnamed,
			),
			Fields::Unit => quote! { (); },
		},
		Data::Enum(_) => panic!("Enum types are not supported."),
		Data::Union(_) => panic!("Union types are not supported."),
	}
}
