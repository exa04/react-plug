extern crate proc_macro;

use heck::{ToLowerCamelCase, ToUpperCamelCase};
use params::*;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use std::ops::Deref;
use syn::punctuated::Punctuated;
use syn::Expr::Struct;
use syn::{
    parse_macro_input, Data, DataEnum, DeriveInput, Expr, ExprStruct, Lit, Member, Meta, Token,
};

mod params;

fn find_field(param: &RPParam, ident: &str) -> Expr {
    param
        .fields
        .iter()
        .find(|field| field.ident == ident)
        .map(|field| field.expr.clone())
        .expect(&format!(
            r#"Couldn't find field "{}" in parameter "{}""#,
            ident, param.ident
        ))
}

fn find_fields<'a>(
    param: &'a params::RPParam,
    idents: impl IntoIterator<Item=&'static str> + 'a,
) -> impl Iterator<Item=Expr> + 'a {
    idents
        .into_iter()
        .map(move |ident| find_field(param, ident))
}

// TODO: Skipping fields
// TODO: Arbitrary data types (that are ignored by TS)
#[proc_macro]
pub fn define_params(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let params = syn::parse::<RPParams>(input).unwrap();
    let ident = &params.ident;

    let fields = params.params.iter().map(|param| {
        let name = &param.ident;
        let ty = &param.ty;

        quote! { pub #name: #ty }
    });

    let defaults = params.params.iter().map(|param| {
        let ident = &param.ident;
        let ty = &param.ty;

        let args = find_fields(
            param,
            match ty {
                RPParamType::FloatParam | RPParamType::IntParam => {
                    vec!["name", "defaultValue", "range"]
                }
                _ => vec!["name", "defaultValue"],
            },
        );

        let modifier_idents = match ty {
            RPParamType::FloatParam => vec![
                "callback",
                "poly_modulation_id",
                "smoother",
                "step_size",
                "string_to_value",
                "unit",
                "value_to_string",
            ],
            RPParamType::IntParam => vec![
                "callback",
                "poly_modulation_id",
                "smoother",
                "string_to_value",
                "unit",
                "value_to_string",
            ],
            RPParamType::BoolParam => vec![
                "callback",
                "poly_modulation_id",
                "string_to_value",
                "value_to_string",
            ],
            RPParamType::EnumParam => vec!["callback", "poly_modulation_id"],
        }
            .into_iter()
            .filter_map(|ident| {
                param
                    .fields
                    .iter()
                    .find(|field| field.ident.to_string() == ident)
                    .map(|field| {
                        let ident = format_ident!("with_{}", ident);
                        let expr = field.expr.clone();
                        quote! { .#ident(#expr) }
                    })
            });

        quote! {
            #ident: #ty::new(#(#args),*)#(#modifier_idents)*
        }
    });

    // TODO: Get EnumParams to work ('variants' field needs to be implemented)

    // TODO: Parameter IDs

    // TODO: Generate Bindings (via test or within this macro)

    {
        quote! {
            pub struct #ident {
                #(#fields),*
            }
            impl Default for #ident {
                fn default() -> Self {
                    Self {
                        #(#defaults),*
                    }
                }
            }
        }
    }
        .into()
}
