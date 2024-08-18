use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{braced, token, Error, Expr, Token, Type};

/// A Params declaration. An identifier, followed by a braced declaration of all
/// parameters as [RPParams](RPParam).
///
/// ## Example
///
/// ```rust
/// ExampleParams {
///     gain: FloatParam {
///         name: "Gain",
///         value: util::db_to_gain(0.0),
///         range: FloatRange::Skewed {
///             min: util::db_to_gain(-30.0),
///             max: util::db_to_gain(30.0),
///             factor: FloatRange::gain_skew_factor(-30.0, 30.0),
///         },
///         smoother: SmoothingStyle::Logarithmic(50.0),
///         unit: " dB",
///         value_to_string: formatters::v2s_f32_gain_to_db(2),
///         string_to_value: formatters::s2v_f32_gain_to_db(),
///     },
///     bool_test: BoolParam {
///         name: "Bool Test",
///         value: false
///     },
///     int_test: IntParam {
///         name: "Int Test",
///         value: 0,
///         range: IntRange::Linear { min: 0, max: 10 }
///     }
/// }
/// ```
pub struct RPParams {
    pub ident: Ident,
    pub brace_token: token::Brace,
    pub params: Punctuated<RPParam, Token![,]>,
}

impl Parse for RPParams {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            ident: input.parse()?,
            brace_token: braced!(content in input),
            params: content.parse_terminated(RPParam::parse, Token![,])?,
        })
    }
}

/// A single parameter declaration. An identifier, a colon, a [RPParamType], and a
/// braced declaration of [RPParamFields](RPParamField).
///
/// ## Example
///
/// ```rust
/// gain: FloatParam {
///     name: "Gain",
///     value: util::db_to_gain(0.0),
///     range: FloatRange::Skewed {
///         min: util::db_to_gain(-30.0),
///         max: util::db_to_gain(30.0),
///         factor: FloatRange::gain_skew_factor(-30.0, 30.0),
///     },
///     smoother: SmoothingStyle::Logarithmic(50.0),
///     unit: " dB",
///     value_to_string: formatters::v2s_f32_gain_to_db(2),
///     string_to_value: formatters::s2v_f32_gain_to_db(),
/// }
/// ```
pub struct RPParam {
    pub ident: Ident,
    pub colon_token: Token![:],
    pub ty: RPParamType,
    pub brace_token: token::Brace,
    pub fields: Punctuated<RPParamField, Token![,]>,
}

impl Parse for RPParam {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;

        Ok(Self {
            ident: input.parse()?,
            colon_token: input.parse()?,
            ty: input.parse()?,
            brace_token: braced!(content in input),
            fields: content.parse_terminated(RPParamField::parse, Token![,])?,
        })
    }
}

/// A single field of a parameter declaration. An identifier, a colon, and an
/// expression.
///
/// ## Examples
///
/// A string field:
///
/// ```rust
/// name: "Gain"
/// ```
///
/// A field that is assigned by a function call:
///
/// ```rust
/// value: util::db_to_gain(0.0)
/// ```
pub struct RPParamField {
    pub ident: Ident,
    pub colon_token: Token![:],
    pub expr: Expr,
}

impl Parse for RPParamField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            ident: input.parse()?,
            colon_token: input.parse()?,
            expr: input.parse()?,
        })
    }
}

/// The types of parameters.
pub enum RPParamType {
    FloatParam,
    IntParam,
    BoolParam,
    EnumParam,
}

impl ToTokens for RPParamType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            RPParamType::FloatParam => quote! {FloatParam},
            RPParamType::IntParam => quote! {IntParam},
            RPParamType::BoolParam => quote! {BoolParam},
            RPParamType::EnumParam => quote! {EnumParam},
        }
        .to_tokens(tokens);
    }
}

impl Parse for RPParamType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ty: Type = input.parse::<Type>()?;

        let type_name = ty.to_token_stream().to_string();

        match type_name.as_str() {
            "FloatParam" => Ok(RPParamType::FloatParam),
            "IntParam" => Ok(RPParamType::IntParam),
            "BoolParam" => Ok(RPParamType::BoolParam),
            "EnumParam" => Ok(RPParamType::EnumParam),
            _ => Err(Error::new(
                ty.span(),
                format!("Unknown param type: {}", type_name),
            )),
        }
    }
}
