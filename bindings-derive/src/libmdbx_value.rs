use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Attribute, DeriveInput, Meta, Path, parse_quote};

pub fn parse(mut item: DeriveInput) -> syn::Result<TokenStream2> {
    let mut existing_derives: Vec<Path> = Vec::new();
    let mut other_attrs: Vec<Attribute> = Vec::new();

    // Process attributes
    for attr in item.attrs.drain(..) {
        if attr.path().is_ident("derive") {
            if let Meta::List(meta_list) = attr.meta.clone() {
                meta_list.tokens.into_iter().for_each(|token| {
                    if let Ok(path) = syn::parse2::<Path>(TokenStream2::from(token)) {
                        existing_derives.push(path);
                    }
                });
            }
        } else {
            other_attrs.push(attr);
        }
    }

    // Add additional derives
    let additional_derives: Vec<Path> = vec![
        parse_quote!(libmdbx_bindings::re_export_rkyv::Serialize),
        parse_quote!(libmdbx_bindings::re_export_rkyv::Deserialize),
        parse_quote!(libmdbx_bindings::re_export_serde::Serialize),
        parse_quote!(libmdbx_bindings::re_export_serde::Deserialize),
        parse_quote!(libmdbx_bindings::Archive),
    ];

    // Combine existing and additional derives
    let combined_derives = existing_derives
        .into_iter()
        .chain(additional_derives)
        .collect::<Vec<_>>();

    // Restore non-derive attributes
    item.attrs = other_attrs;

    // Generate the output
    let output = quote! {
        #[derive(#(#combined_derives),*)]
        #item
    };

    Ok(output)
}
