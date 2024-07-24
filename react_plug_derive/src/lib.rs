extern crate proc_macro;

use std::str::FromStr;

use heck::ToUpperCamelCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{braced, Error, Expr, Token, token, Type};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;

const SENDER_NAME: &str = "sender";

struct RPParams {
    pub ident: Ident,
    pub brace_token: token::Brace,
    pub params: Punctuated<RPParam, Token![,]>
}

impl Parse for RPParams {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self{
            ident: input.parse()?,
            brace_token: braced!(content in input),
            params: content.parse_terminated(RPParam::parse, Token![,])?,
        })
    }
}
struct RPParam {
    pub ident: Ident,
    pub colon_token: Token![:],
    pub ty: RPParamType,
    pub brace_token: token::Brace,
    pub fields: Punctuated<RPParamField, Token![,]>
}

impl Parse for RPParam {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;

        Ok(Self{
            ident: input.parse()?,
            colon_token: input.parse()?,
            ty: input.parse()?,
            brace_token: braced!(content in input),
            fields: content.parse_terminated(RPParamField::parse, Token![,])?,
        })
    }
}

struct RPParamField {
    pub ident: Ident,
    pub colon_token: Token![:],
    pub expr: Expr,
}

impl Parse for RPParamField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self{
            ident: input.parse()?,
            colon_token: input.parse()?,
            expr: input.parse()?,
        })
    }
}

enum RPParamType {
    FloatParam,
    IntParam,
    BoolParam
}

impl ToTokens for RPParamType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = match self {
            RPParamType::FloatParam => Ident::new("FloatParam", Span::call_site()),
            RPParamType::IntParam => Ident::new("IntParam", Span::call_site()),
            RPParamType::BoolParam => Ident::new("BoolParam", Span::call_site()),
        };
        ident.to_tokens(tokens);
    }
}

impl Parse for RPParamType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ty: Type = input
            .parse::<Type>()?;

        let type_name = ty
            .to_token_stream()
            .to_string();

        match type_name.as_str() {
            "FloatParam" => Ok(RPParamType::FloatParam),
            "IntParam" => Ok(RPParamType::IntParam),
            "BoolParam" => Ok(RPParamType::BoolParam),
            _ => Err(Error::new(ty.span(), format!("Unknown param type: {}", type_name)))
        }
    }
}


#[proc_macro]
pub fn rp_params(
    input: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    let params = match syn::parse::<RPParams>(input) {
        Ok(params) => { params },
        Err(E) => { panic!("{}", E) }
    };

    let ident = &params.ident;

    let param_enum_ident = Ident::new(
        format!("{}Type", ident.to_string().to_upper_camel_case()).as_str(),
        Span::call_site()
    );

    fn variant(ident: &Ident) -> Ident { Ident::new(
        ident.to_string().to_upper_camel_case().as_str(),
        Span::call_site()
    ) }

    /// Fields of the param enum
    let param_enum_fields = params.params.iter().map(|param| {
        let ty = match &param.ty {
            RPParamType::FloatParam => {quote!{f32}}
            RPParamType::IntParam => {quote!{i32}}
            RPParamType::BoolParam => {quote!{bool}}
        };
        let ident = Ident::new(
            param.ident.to_string().to_upper_camel_case().as_str(),
            Span::call_site()
        );
        quote!{#ident(#ty)}
    });

    /// Fields of the param struct
    let param_struct_fields = params.params.iter().map(|param| {
        let ident = &param.ident;
        let ty = &param.ty;
        let id = ident.to_string();

        quote! {
            #[id = #id]
            #ident: #ty
        }
    });

    /// Content of Self declaration in new() constructor
    let param_struct_initializers = params.params.iter().map(|param| {
        let ident = &param.ident;
        let ty = &param.ty;

        fn find_arg(fields: &Punctuated<RPParamField, Token![,]>, name: String) -> Option<Expr> {
            fields
                .iter()
                .find(|field| field.ident.to_token_stream().to_string() == name)
                .map(|field| field.expr.clone())
        }

        let required = match ty {
            RPParamType::FloatParam => { vec!["name", "value", "range"] },
            RPParamType::IntParam => { vec!["name", "value", "range"] },
            RPParamType::BoolParam => { vec!["name", "value"] }
        };

        let (required_params, modifier_params): (Vec<_>, Vec<_>) = param.fields
            .iter()
            .partition(|field| required.contains(&&*field.ident.to_string()));

        let mut args: Punctuated<Expr, Token![,]> = Punctuated::new();

        required.iter().for_each(|name| {
            let expr = required_params.iter()
                .find(|field| *field.ident.to_string() == name.to_string())
                .unwrap().expr.clone();
            args.push(expr);
        });

        let modifiers = modifier_params.iter().map(|param| -> TokenStream {
            let arg = &param.expr;

            let modifier = Ident::new(format!("with_{}", &param.ident.to_string()).as_str(), Span::call_site());

            quote! {.#modifier(#arg)}
        });

        let variant = variant(&param.ident);

        quote! {
            #ident: #ty::new(
                #args
            )
                #(#modifiers)*
                .with_callback({
                    let sender = sender.clone();
                    std::sync::Arc::new(move |value| {
                        sender.send(PluginMessage::ParameterChange(#param_enum_ident::#variant(value))).unwrap();
                    })
                })
        }
    });

    /// Content of send_all fn
    let send_value_fns = params.params.iter().map(|param| {
        let ident = &param.ident;
        let variant = variant(&param.ident);

        quote!{
            sender.send(PluginMessage::ParameterChange(#param_enum_ident::#variant(self.#ident.value()))).unwrap();
        }
    });

    {quote! {
        #[derive(Params)]
        pub struct #ident {
            #(#param_struct_fields),*
        }

        pub enum #param_enum_ident {
            #(#param_enum_fields),*
        }

        impl #ident {
            pub fn new(sender: &crossbeam_channel::Sender<PluginMessage>) -> Self {
                Self {
                    #(#param_struct_initializers),*
                }
            }
            pub fn send_all(&self, sender: crossbeam_channel::Sender<PluginMessage>) {
                #(#send_value_fns);*
            }
        }
    }}.into()
}
