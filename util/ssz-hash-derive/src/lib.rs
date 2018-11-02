// Copyright 2018 Parity Technologies (UK) Ltd.
// This file is part of Substrate Shasper.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

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
use syn::{
	DeriveInput, Generics, GenericParam,
	Data, Field, Fields, Ident, Index,
	punctuated::Punctuated,
	spanned::Spanned,
	token::Comma,
};

#[cfg(feature = "std")]
mod alloc {
	pub use std::boxed;
	pub use std::vec;
}

const HASH_ERR: &str = "derive(SszHash) failed";

#[proc_macro_derive(SszHash)]
pub fn hash_derive(input: TokenStream) -> TokenStream {
	let input: DeriveInput = syn::parse(input).expect(HASH_ERR);
	let name = &input.ident;

	let hash_param_ = quote!(H);
	let generics = add_trait_bounds(input.generics, parse_quote!(::ssz_hash::SpecHash < #hash_param_ >));
	let mut generics_for_impl = generics.clone();
	let (_, ty_generics, where_clause) = generics.split_for_impl();
	let hash_param_ = quote!(H);
	generics_for_impl.params.push(parse_quote!(#hash_param_ : ::hash_db::Hasher));
	let (impl_generics, _, _) = generics_for_impl.split_for_impl();

	let self_ = quote!(self);
	let dest_ = quote!(agg);

	let hashing = quote(&input.data, &self_, &dest_, &hash_param_);

	let expanded = quote! {
		impl #impl_generics ::ssz_hash::SpecHash<#hash_param_> for #name #ty_generics #where_clause {
			fn spec_hash(&self) -> H::Out {
				let mut #dest_ = ::ssz_hash::alloc::vec::Vec::new();
				#hashing
				#hash_param_ :: hash(& #dest_ )
			}
		}
	};

	expanded.into()
}

type FieldsList = Punctuated<Field, Comma>;

fn encode_fields<F>(
	dest: &proc_macro2::TokenStream,
	generic_param: &proc_macro2::TokenStream,
	fields: &FieldsList,
	field_name: F,
) -> proc_macro2::TokenStream where
	F: Fn(usize, &Option<Ident>) -> proc_macro2::TokenStream,
{
	let fields: Vec<_> = fields.iter().collect();

	let recurse = fields.iter().enumerate().map(|(i, f)| {
		let field = field_name(i, &f.ident);

		quote_spanned! { f.span() =>
			#dest.append(&mut ::ssz_hash::SpecHash::< #generic_param >::spec_hash(#field).as_ref().to_vec());
		}
	});

	quote! {
		#( #recurse )*
	}
}

fn quote(data: &Data, self_: &proc_macro2::TokenStream, dest: &proc_macro2::TokenStream, generic_param: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
	let call_site = proc_macro2::Span::call_site();
	match *data {
		Data::Struct(ref data) => match data.fields {
			Fields::Named(ref fields) => encode_fields(
				dest,
				generic_param,
				&fields.named,
				|_, name| quote_spanned!(call_site => &#self_.#name),
			),
			Fields::Unnamed(ref fields) => encode_fields(
				dest,
				generic_param,
				&fields.unnamed,
				|i, _| {
					let index = Index { index: i as u32, span: call_site };
					quote_spanned!(call_site => &#self_.#index)
				},
			),
			Fields::Unit => quote_spanned! { call_site =>
				drop(#dest);
			},
		},
		Data::Enum(_) => panic!("Enum types are not supported."),
		Data::Union(_) => panic!("Union types are not supported."),
	}
}

fn add_trait_bounds(mut generics: Generics, bounds: syn::TypeParamBound) -> Generics {
	for param in &mut generics.params {
		if let GenericParam::Type(ref mut type_param) = *param {
			type_param.bounds.push(bounds.clone());
		}
	}
	generics
}
