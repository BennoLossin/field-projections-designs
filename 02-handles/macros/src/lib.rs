use proc_macro::TokenStream;
use syn::parse_macro_input;

mod adt_reflect;

#[proc_macro]
pub fn adt_reflect(input: TokenStream) -> TokenStream {
    adt_reflect::expand(parse_macro_input!(input as _)).into()
}
