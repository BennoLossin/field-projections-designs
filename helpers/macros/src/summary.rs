use proc_macro2::TokenStream;
use quote::quote;
use syn::File;

pub(crate) fn expand(file: File) -> TokenStream {
    quote!(#file)
}
