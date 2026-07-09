use proc_macro::TokenStream;
use syn::parse_macro_input;

use crate::summary::SummaryArgs;

mod adt_reflect;
mod summary;

#[proc_macro_attribute]
pub fn summary(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as SummaryArgs);
    match args {
        SummaryArgs::Toplevel => summary::expand(parse_macro_input!(input as _)).into(),
        SummaryArgs::Skip => input,
    }
}

#[proc_macro]
pub fn adt_reflect(input: TokenStream) -> TokenStream {
    adt_reflect::expand(parse_macro_input!(input as _)).into()
}
