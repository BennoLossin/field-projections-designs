use std::{
    io::{self, Write},
    process::{Command, Stdio},
};

use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    AttrStyle, Attribute, Error, Expr, ExprLit, Item, ItemMod, Lit,
    parse::{End, Parse},
    parse_quote, parse2,
    punctuated::Punctuated,
    spanned::Spanned,
    visit_mut::{VisitMut, visit_item_mut, visit_use_tree_mut},
};

#[derive(Debug)]
pub(crate) enum SummaryArgs {
    Toplevel,
    Skip,
}

mod kw {
    use syn::custom_keyword;

    custom_keyword!(skip);
}

impl Parse for SummaryArgs {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let lh = input.lookahead1();
        if lh.peek(End) {
            Ok(Self::Toplevel)
        } else if lh.peek(kw::skip) {
            let _: kw::skip = input.parse()?;
            Ok(Self::Skip)
        } else {
            Err(lh.error())
        }
    }
}

pub(crate) fn expand(mut module: ItemMod) -> TokenStream {
    let ItemMod {
        ref attrs,
        content: Some((_, ref items)),
        ..
    } = module
    else {
        let err = Error::new(
            module.span(),
            "must be an inline module, use `#![summary]` inside of files",
        )
        .into_compile_error();
        return quote!(#err #module);
    };
    let Some(anchor) = find_summary_anchor(attrs) else {
        return module.into_token_stream();
    };
    let mut full_summary: Vec<String> = vec![];
    let mut errors = vec![];
    for item in items {
        if let Some(last) = full_summary.last()
            && !last.is_empty()
        {
            full_summary.push(String::new());
        }
        let item_summary = generate_summary(item, &mut errors);
        full_summary.extend(item_summary);
    }
    module.attrs.splice(
        anchor..=anchor,
        full_summary
            .into_iter()
            .map(|content| parse_quote!(#![doc = #content])),
    );
    let errors = errors.into_iter().map(|err| err.into_compile_error());
    quote!(#(#errors)* #module)
}

fn find_summary_anchor<'a>(attrs: &[Attribute]) -> Option<usize> {
    let mut res = None;
    for (idx, attr) in attrs.iter().enumerate() {
        if matches!(attr.style, AttrStyle::Inner(_))
            && let Ok(meta) = attr.meta.require_name_value()
            && meta.path.is_ident("doc")
            && let Expr::Lit(ExprLit {
                lit: Lit::Str(contents),
                ..
            }) = &meta.value
            && contents.value() == " ====== SUMMARY ANCHOR ======"
        {
            if res.is_some() {
                todo!("report error that there are two anchors!");
            }
            res = Some(idx);
        }
    }
    res
}

fn generate_summary(item: &Item, errors: &mut Vec<Error>) -> Vec<String> {
    let mut stripped = item.clone();
    visit_item_mut(&mut StripDistractions { skip: false }, &mut stripped);
    let content = stripped.into_token_stream().to_string();
    match rustfmt(&content) {
        Ok(Ok(formatted)) => {
            let formatted = formatted.trim();
            if formatted.is_empty() {
                vec![]
            } else {
                formatted
                    .split('\n')
                    .map(|s| format!(" {}", s.trim_end()))
                    .collect()
            }
        }
        Ok(Err((code, stderr))) => {
            errors.push(Error::new(
                def_span(item),
                format!(
                    "While fromatting the summary for this item, \
                    encountered an error: code={code:?}, stderr:\n{stderr}"
                ),
            ));
            vec![]
        }
        Err(err) => {
            errors.push(Error::new(
                def_span(item),
                format!(
                    "While formatting the summary of this item, \
                    encountered an error: {err}"
                ),
            ));
            vec![]
        }
    }
}

struct StripDistractions {
    skip: bool,
}

impl VisitMut for StripDistractions {
    fn visit_attributes_mut(&mut self, i: &mut Vec<Attribute>) {
        fn is_skip(attr: &Attribute) -> bool {
            if let Ok(syn::MetaList {
                path: syn::Path { segments, .. },
                tokens,
                ..
            }) = attr.meta.require_list()
                && segments.iter().all(|seg| seg.arguments.is_empty())
                && segments
                    .iter()
                    .rev()
                    .zip(["summary", "macros"])
                    .all(|(seg, expected)| seg.ident == expected)
                && let Ok(SummaryArgs::Skip) = parse2(tokens.clone())
            {
                true
            } else {
                false
            }
        }
        if i.iter().any(|a| is_skip(a)) {
            self.skip = true;
        }
        i.clear();
    }

    fn visit_use_tree_mut(&mut self, i: &mut syn::UseTree) {
        if self.skip {
            *i = syn::UseTree::Group(syn::UseGroup {
                brace_token: Default::default(),
                items: Punctuated::default(),
            });
        } else {
            visit_use_tree_mut(self, i);
            self.skip = false;
        }
    }

    fn visit_item_mut(&mut self, i: &mut Item) {
        if self.skip {
            *i = Item::Verbatim(quote!());
        } else {
            visit_item_mut(self, i);
            self.skip = false;
        }
    }
}

fn rustfmt(txt: &str) -> io::Result<Result<String, (Option<i32>, String)>> {
    let mut cmd = Command::new("rustfmt");
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut fmt = cmd.spawn()?;
    fmt.stdin
        .as_mut()
        .expect("command to have stdin")
        .write_all(txt.as_bytes())?;
    let output = fmt.wait_with_output()?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        return Ok(Err((output.status.code(), err)));
    }
    let contents = String::from_utf8(output.stdout).expect("rustfmt to have valid utf8 output");
    Ok(Ok(contents))
}

fn def_span(item: &Item) -> Span {
    match item {
        Item::Const(syn::ItemConst {
            vis,
            const_token,
            ident,
            ..
        }) => quote!(#vis #const_token #ident).span(),
        Item::Enum(syn::ItemEnum {
            vis,
            enum_token,
            ident,
            ..
        }) => quote!(#vis #enum_token #ident).span(),
        Item::ExternCrate(syn::ItemExternCrate {
            vis,
            extern_token,
            crate_token,
            ident,
            ..
        }) => quote!(#vis #extern_token, #crate_token #ident).span(),
        Item::Fn(syn::ItemFn { vis, sig, .. }) => quote!(#vis #sig).span(),
        Item::ForeignMod(syn::ItemForeignMod { unsafety, abi, .. }) => {
            quote!(#unsafety #abi).span()
        }
        Item::Impl(syn::ItemImpl {
            defaultness,
            unsafety,
            impl_token,
            generics,
            trait_,
            self_ty,
            ..
        }) => match trait_ {
            None => quote!(#defaultness #unsafety #impl_token #generics #self_ty).span(),
            Some((not, path, for_)) => {
                quote!(#defaultness #unsafety #impl_token #generics #not #path #for_ #self_ty)
                    .span()
            }
        },
        Item::Macro(syn::ItemMacro { ident, .. }) => quote!(#ident).span(),
        Item::Mod(syn::ItemMod {
            vis,
            unsafety,
            mod_token,
            ident,
            ..
        }) => quote!(#vis #unsafety #mod_token #ident).span(),
        Item::Static(syn::ItemStatic {
            vis,
            static_token,
            mutability,
            ident,
            ..
        }) => quote!(#vis #static_token #mutability #ident).span(),
        Item::Struct(syn::ItemStruct {
            vis,
            struct_token,
            ident,
            ..
        }) => quote!(#vis #struct_token #ident).span(),
        Item::Trait(syn::ItemTrait {
            vis,
            unsafety,
            auto_token,
            trait_token,
            ident,
            ..
        }) => quote!(#vis #unsafety #auto_token #trait_token #ident).span(),
        Item::TraitAlias(syn::ItemTraitAlias {
            vis,
            trait_token,
            ident,
            ..
        }) => quote!(#vis #trait_token #ident).span(),
        Item::Type(syn::ItemType {
            vis,
            type_token,
            ident,
            ..
        }) => quote!(#vis #type_token #ident).span(),
        Item::Union(syn::ItemUnion {
            vis,
            union_token,
            ident,
            ..
        }) => quote!(#vis #union_token #ident).span(),
        Item::Use(item_use) => item_use.use_token.span,
        Item::Verbatim(ts) => ts.span(),
        _ => {
            eprintln!(
                "WARN: novel `syn::Item` variant not handled in `{}:{} fn def_span`",
                file!(),
                line!()
            );
            item.span()
        }
    }
}
