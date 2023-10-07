use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    meta::ParseNestedMeta, parenthesized, spanned::Spanned, Error, LitInt,
};

#[derive(Clone, Copy)]
pub enum IntRepr {
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
}

impl ToTokens for IntRepr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::I8 => tokens.append_all(quote! { i8 }),
            Self::I16 => tokens.append_all(quote! { i16 }),
            Self::I32 => tokens.append_all(quote! { i32 }),
            Self::I64 => tokens.append_all(quote! { i64 }),
            Self::I128 => tokens.append_all(quote! { i128 }),
            Self::U8 => tokens.append_all(quote! { u8 }),
            Self::U16 => tokens.append_all(quote! { u16 }),
            Self::U32 => tokens.append_all(quote! { u32 }),
            Self::U64 => tokens.append_all(quote! { u64 }),
            Self::U128 => tokens.append_all(quote! { u128 }),
        }
    }
}

#[derive(Clone, Copy)]
pub enum BaseRepr {
    C,
    // structs only
    Transparent,
    // enums only
    Int(IntRepr),
}

impl ToTokens for BaseRepr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            BaseRepr::C => tokens.append_all(quote! { C }),
            BaseRepr::Transparent => tokens.append_all(quote! { transparent }),
            BaseRepr::Int(int_repr) => tokens.append_all(quote! { #int_repr }),
        }
    }
}

#[derive(Clone)]
pub enum Modifier {
    // structs only
    Packed,
    Align(LitInt),
}

impl ToTokens for Modifier {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Modifier::Packed => tokens.append_all(quote! { packed }),
            Modifier::Align(n) => tokens.append_all(quote! { align(#n) }),
        }
    }
}

#[derive(Clone, Default)]
pub struct Repr {
    pub base_repr: Option<(BaseRepr, Span)>,
    pub modifier: Option<(Modifier, Span)>,
}

impl Repr {
    fn try_set_modifier<S: ToTokens>(
        &mut self,
        modifier: Modifier,
        spanned: S,
    ) -> Result<(), Error> {
        if self.modifier.is_some() {
            Err(Error::new_spanned(
                spanned,
                "only one repr modifier may be specified",
            ))
        } else {
            self.modifier = Some((modifier, spanned.span()));
            Ok(())
        }
    }

    fn try_set_base_repr<S: ToTokens>(
        &mut self,
        repr: BaseRepr,
        spanned: S,
    ) -> Result<(), Error> {
        if self.base_repr.is_some() {
            Err(Error::new_spanned(
                spanned,
                "only one repr may be specified",
            ))
        } else {
            self.base_repr = Some((repr, spanned.span()));
            Ok(())
        }
    }

    pub fn parse_list_meta(
        &mut self,
        meta: ParseNestedMeta<'_>,
    ) -> Result<(), Error> {
        if meta.path.is_ident("packed") {
            return self.try_set_modifier(Modifier::Packed, meta.path);
        } else if meta.path.is_ident("align") {
            let content;
            parenthesized!(content in meta.input);
            let alignment = content.parse()?;

            if !content.is_empty() {
                return Err(content.error("align requires only one argument"));
            }

            return self
                .try_set_modifier(Modifier::Align(alignment), meta.path);
        }

        let parsed_repr = if meta.path.is_ident("transparent") {
            BaseRepr::Transparent
        } else if meta.path.is_ident("C") {
            BaseRepr::C
        } else if meta.path.is_ident("i8") {
            BaseRepr::Int(IntRepr::I8)
        } else if meta.path.is_ident("i16") {
            BaseRepr::Int(IntRepr::I16)
        } else if meta.path.is_ident("i32") {
            BaseRepr::Int(IntRepr::I32)
        } else if meta.path.is_ident("i64") {
            BaseRepr::Int(IntRepr::I64)
        } else if meta.path.is_ident("i128") {
            BaseRepr::Int(IntRepr::I128)
        } else if meta.path.is_ident("u8") {
            BaseRepr::Int(IntRepr::U8)
        } else if meta.path.is_ident("u16") {
            BaseRepr::Int(IntRepr::U16)
        } else if meta.path.is_ident("u32") {
            BaseRepr::Int(IntRepr::U32)
        } else if meta.path.is_ident("u64") {
            BaseRepr::Int(IntRepr::U64)
        } else if meta.path.is_ident("u128") {
            BaseRepr::Int(IntRepr::U128)
        } else {
            let msg =
                "invalid repr, available reprs are transparent, C, i* and u*";

            return Err(Error::new_spanned(meta.path, msg));
        };

        self.try_set_base_repr(parsed_repr, meta.path)
    }
}

impl ToTokens for Repr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let base_repr = self.base_repr.as_ref().map(|(b, _)| b);
        let base_repr_iter = base_repr.iter();
        let modifier = self.modifier.as_ref().map(|(m, _)| m);
        let modifier_iter = modifier.iter();
        tokens.append_all(
            quote! { #[repr(#(#base_repr_iter,)* #(#modifier_iter,)*)] },
        );
    }
}
