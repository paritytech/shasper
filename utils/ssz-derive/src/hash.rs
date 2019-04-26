use proc_macro2::TokenStream;
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
	generic_param: &TokenStream,
	fields: &FieldsList,
	field_name: F,
	skip_truncated: bool,
	is_digest: bool,
) -> TokenStream where
	F: Fn(usize, &Option<Ident>) -> TokenStream,
{
	let fields: Vec<_> = fields.iter().collect();

	let recurse = fields.iter().enumerate().map(|(i, f)| {
		let truncate = if skip_truncated {
			has_attr(&f.attrs, "truncate")
		} else {
			false
		};
		let use_fixed = has_attr(&f.attrs, "use_fixed");
		let skip = has_attr(&f.attrs, "skip_default");
		let field = field_name(i, &f.ident);

		if truncate || skip {
			quote! { (); }
		} else if use_fixed {
			if is_digest {
				quote_spanned! {
					f.span() =>
						#dest.push(::ssz::Digestible::< #generic_param >::hash(&::ssz::Fixed(#field.as_ref())));
				}
			} else {
				quote_spanned! {
					f.span() =>
						#dest.push(::ssz::hash::hash_db_hasher::hash_to_array(::ssz::Hashable::< #generic_param >::hash(&::ssz::Fixed(#field.as_ref()))));
				}
			}
		} else {
			if is_digest {
				quote_spanned! {
					f.span() =>
						#dest.push(::ssz::Digestible::< #generic_param >::hash(#field));
				}
			} else {
				quote_spanned! {
					f.span() =>
						#dest.push(::ssz::hash::hash_db_hasher::hash_to_array(::ssz::Hashable::< #generic_param >::hash(#field)));
				}
			}
		}
	});

	quote! {
		#( #recurse )*
	}
}

pub fn quote(data: &Data, self_: &TokenStream, dest: &TokenStream, generic_param: &TokenStream, skip_truncated: bool, is_digest: bool) -> TokenStream {
	let call_site = proc_macro2::Span::call_site();
	match *data {
		Data::Struct(ref data) => match data.fields {
			Fields::Named(ref fields) => encode_fields(
				dest,
				generic_param,
				&fields.named,
				|_, name| quote_spanned!(call_site => &#self_.#name),
				skip_truncated,
				is_digest,
			),
			Fields::Unnamed(ref fields) => encode_fields(
				dest,
				generic_param,
				&fields.unnamed,
				|i, _| {
					let index = Index { index: i as u32, span: call_site };
					quote_spanned!(call_site => &#self_.#index)
				},
				skip_truncated,
				is_digest,
			),
			Fields::Unit => quote! { (); },
		},
		Data::Enum(_) => panic!("Enum types are not supported."),
		Data::Union(_) => panic!("Union types are not supported."),
	}
}
