mod libmdbx_value;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_attribute]
pub fn derive_libmdbx_value(
    _: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> TokenStream {
    let i_struct = parse_macro_input!(item as DeriveInput);
    libmdbx_value::parse(i_struct)
        .map(Into::into)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
