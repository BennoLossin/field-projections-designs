use proc_macro::TokenStream;
use syn::parse_macro_input;

mod summary;

#[proc_macro_attribute]
pub fn summary(args: TokenStream, input: TokenStream) -> TokenStream {
    parse_macro_input!(args as syn::parse::Nothing);
    summary::expand(parse_macro_input!(input as _)).into()
}
