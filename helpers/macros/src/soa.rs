// Linked from `../../../examples/struct_of_arrays/src/derive_soa.rs`. This
// macro is a helper macro for implementing the `struct_of_arrays` example, as a
// declarative macro would be too much pain and a separate macro crate is
// annoying.

use std::{
    format as f,
    mem,
};

use proc_macro2::TokenStream;
use quote::{
    format_ident,
    quote,
};
use syn::{
    Data,
    DataStruct,
    DeriveInput,
    Fields,
    GenericParam,
    Generics,
    Ident,
    Token,
    Type,
    TypeArray,
    Visibility,
    parse_quote,
    spanned::Spanned,
    token::Bracket,
};

use crate::utils::HumanList;

macro_rules! error {
    ($span:expr, $($msg:tt)*) => {
        ::syn::Error::new(
            $span,
            ::std::format!($($msg)*),
        ).into_compile_error()
    };
}

pub(crate) fn expand(input: DeriveInput) -> TokenStream {
    let inputs = match get_inputs(input) {
        Ok(inputs) => inputs,
        Err(err) => return err,
    };
    let info = match gen_info(&inputs) {
        Ok(info) => info,
        Err(err) => return err,
    };
    let soa_struct = gen_struct(&inputs, &info);
    let soa_impl = gen_impl(&inputs, &info);
    let index_impl = gen_index_impl(&inputs, &info);
    quote! {
        #soa_struct
        #soa_impl
        #index_impl
    }
}

fn gen_struct(
    Inputs {
        vis,
        struct_token,
        semi_token,
        ..
    }: &Inputs,
    SoAInfo {
        soa_ident,
        soa_generics,
        soa_fields,
        ..
    }: &SoAInfo,
) -> TokenStream {
    quote! {
        #vis #struct_token #soa_ident #soa_generics #soa_fields #semi_token
    }
}

fn gen_impl(
    Inputs { ident, generics, .. }: &Inputs,
    SoAInfo {
        soa_ident,
        soa_generics,
        soa_len,
        ..
    }: &SoAInfo,
) -> TokenStream {
    let (impl_gen, ty_gen, whr) = generics.split_for_impl();
    let (_, soa_ty_gen, _) = soa_generics.split_for_impl();
    quote! {
        impl #impl_gen SoA for #ident #ty_gen #whr {
            type SoA<const #soa_len: ::core::primitive::usize>
                = #soa_ident #soa_ty_gen;

            type ArrayField<
                ___Field: ::design::place::Field<
                    Source = Self,
                    Target: ::core::marker::Sized,
                >,
                const #soa_len: ::core::primitive::usize,
            > = ::design::place::TransmutedField<
                    ___Field,
                    #soa_ident #soa_ty_gen,
                    [<___Field as ::design::place::Subplace>::Target; #soa_len]
                >;

            fn array_field_from_struct<___Field, const N: ::core::primitive::usize>(
                _field: ___Field,
            ) -> Self::ArrayField<___Field, N>
            where
                ___Field: ::design::place::Field<Source = Self, Target: Sized>
            {
                <
                    ::design::place::TransmutedField<
                        ___Field,
                        #soa_ident #soa_ty_gen,
                        [<___Field as ::design::place::Subplace>::Target; #soa_len]
                    > as ::core::default::Default
                >::default()
            }
        }
    }
}

fn gen_index_impl(
    Inputs { ident, generics, .. }: &Inputs,
    SoAInfo {
        soa_ident,
        soa_len,
        soa_generics,
        ..
    }: &SoAInfo,
) -> TokenStream {
    let (_, ty_gen, _) = generics.split_for_impl();
    let (soa_impl_gen, soa_ty_gen, soa_whr) = soa_generics.split_for_impl();
    let mut index_generics = soa_generics.clone();
    let idx = index_generics
        .params
        .iter()
        .position(|param| !matches!(param, GenericParam::Lifetime(_)))
        .unwrap_or(index_generics.params.len());
    index_generics.params.insert(
        idx,
        parse_quote!(
            ___Handle: ::design::ops::place::PlaceHandle<
                Target = #soa_ident #soa_ty_gen,
            >
        ),
    );
    let (index_impl_gen, _, index_whr) = index_generics.split_for_impl();
    quote! {
        impl #soa_impl_gen
            ::design::ops::place::Indexable<::core::primitive::usize>
            for #soa_ident #soa_ty_gen
        #soa_whr
        {
            type Element = #ident #ty_gen;
        }

        unsafe impl #index_impl_gen
            ::design::ops::place::IndexPlace<
                ::core::primitive::usize,
                ___Handle,
                ::design::ops::place::borrowck::Instant, // TODO
                ::design::ops::place::borrowck::Instant,
            >
            for #soa_ident #soa_ty_gen
        #index_whr
        {
            type ElementHandle = SoAHandle<#ident #ty_gen, ___Handle, #soa_len>;

            const POINTEE_ACCESS: ::design::ops::place::borrowck::AccessKind
                = ::design::ops::place::borrowck::AccessKind::Shared;
            const POINTER_ACCESS: ::design::ops::place::borrowck::AccessKind
                = ::design::ops::place::borrowck::AccessKind::Shared;

            const SAFE: ::core::primitive::bool = true;

            fn index(
                handle: ___Handle,
                idx: ::core::primitive::usize,
            ) -> Self::ElementHandle {
                unsafe { SoAHandle::from_parts(handle, idx) }
            }
        }
    }
}

struct Inputs {
    vis: Visibility,
    ident: Ident,
    generics: Generics,
    struct_token: Token![struct],
    fields: Fields,
    semi_token: Option<Token![;]>,
}

struct SoAInfo {
    soa_ident: Ident,
    soa_len: Ident,
    soa_generics: Generics,
    soa_fields: Fields,
}

fn get_inputs(input: DeriveInput) -> Result<Inputs, TokenStream> {
    let DeriveInput {
        attrs: _,
        vis,
        ident,
        generics,
        data,
    } = input;
    let DataStruct {
        struct_token,
        fields,
        semi_token,
    } = match data {
        Data::Struct(struct_) => struct_,
        Data::Enum(enum_) => {
            return Err(error!(enum_.enum_token.span(), "expected a struct"));
        }
        Data::Union(union) => {
            return Err(error!(union.union_token.span(), "expected a struct"));
        }
    };
    Ok(Inputs {
        vis,
        ident,
        generics,
        struct_token,
        fields,
        semi_token,
    })
}

fn gen_info(
    Inputs {
        ident, generics, fields, ..
    }: &Inputs,
) -> Result<SoAInfo, TokenStream> {
    let soa_ident = format_ident!("SoA{ident}");
    const SOA_LEN_NAMES: [&'static str; 3] = ["N", "LEN", "SOA_LEN"];
    let Some(soa_len) = SOA_LEN_NAMES
        .into_iter()
        .filter(|name| {
            for param in &generics.params {
                let ident = match param {
                    GenericParam::Lifetime(_) => continue,
                    GenericParam::Type(ty) => &ty.ident,
                    GenericParam::Const(cnst) => &cnst.ident,
                };
                if ident == name {
                    return false;
                }
            }
            true
        })
        .map(|name| Ident::new(name, generics.span()))
        .next()
    else {
        return Err(error!(
            generics.span(),
            "expected one of {} to be available for the SoA length.",
            HumanList(SOA_LEN_NAMES.map(|name| f!("`{name}`"))),
        ));
    };
    let soa_generics = {
        let mut tmp = generics.clone();
        let idx = tmp
            .params
            .iter()
            .position(|param| match param {
                GenericParam::Lifetime(_) => false,
                GenericParam::Type(ty) => ty.default.is_some(),
                GenericParam::Const(const_) => const_.default.is_some(),
            })
            .unwrap_or(tmp.params.len());
        tmp.params.insert(
            idx,
            parse_quote!(const #soa_len: ::core::primitive::usize),
        );
        tmp
    };
    let soa_fields = {
        let mut tmp = fields.clone();
        for field in &mut tmp {
            let ty = mem::replace(&mut field.ty, parse_quote!(_));
            field.ty = Type::Array(TypeArray {
                bracket_token: Bracket::default(),
                elem: Box::new(ty),
                semi_token: Default::default(),
                len: parse_quote!(#soa_len),
            });
        }
        tmp
    };
    Ok(SoAInfo {
        soa_ident,
        soa_len,
        soa_generics,
        soa_fields,
    })
}
