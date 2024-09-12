extern crate proc_macro;

use crate::params::RPParamType;
use heck::ToLowerCamelCase;
use params::*;
use quote::{format_ident, quote, ToTokens};
use std::ops::Deref;
use syn::Expr::Struct;
use syn::{Expr, Member};

mod params;

fn try_find_field(param: &RPParam, ident: &str) -> Result<Expr, String> {
    param
        .fields
        .iter()
        .find(|field| field.ident == ident)
        .map(|field| field.expr.clone())
        .ok_or(format!(
            r#"Couldn't find field "{}" in parameter "{}""#,
            ident, param.ident
        ))
}

fn find_field(param: &RPParam, ident: &str) -> Expr {
    try_find_field(param, ident).unwrap_or_else(|err| panic!("{}", err))
}

fn find_fields<'a>(
    param: &'a RPParam,
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

        let ty = if ty == &RPParamType::EnumParam {
            let associated = if let Struct(s) = find_field(param, "variants") {
                s.path
            } else {
                panic!("Invalid syntax for \"variants\" field!");
            };
            quote! {#ty<#associated>}
        } else {
            quote! {#ty}
        };

        try_find_field(param, "id")
            .map(|id| {
                quote! {
                    #[id = #id]
                    pub #name: #ty
                }
            })
            .unwrap_or_else(|_| {
                let id = name.to_string();
                quote! {
                    #[id = #id]
                    pub #name: #ty
                }
            })
    });

    let param_enums = params
        .params
        .iter()
        .filter(|param| param.ty == RPParamType::EnumParam)
        .map(|param| {
            let (ident, variants) = if let Struct(s) = find_field(param, "variants") {
                (
                    s.path,
                    s.fields.into_iter().map(|field| {
                        let id = if let Member::Named(ident) = &field.member {
                            ident
                        } else {
                            panic!(
                                "Invalid syntax for \"variants\" field! Expected a named field."
                            );
                        };

                        if let Expr::Lit(name) = &field.expr {
                            quote! {
                                #[name = #name]
                                #id
                            }
                        } else {
                            id.to_token_stream()
                        }
                    }),
                )
            } else {
                panic!("Invalid syntax for \"variants\" field!");
            };

            quote! {
                #[derive(nih_plug::params::enums::Enum, PartialEq)]
                pub enum #ident {
                    #(#variants),*
                }
            }
        });

    let defaults = params.params.iter().map(|param| {
        let ident = &param.ident;
        let ty = &param.ty;

        let args = if ty == &RPParamType::EnumParam {
            let name = find_field(param, "name");
            let default_value = find_field(param, "default_value");
            let ident = if let Struct(s) = find_field(param, "variants") {
                s.path
            } else {
                panic!("Invalid syntax for \"variants\" field!");
            };

            quote! { #name, #ident::#default_value }
        } else {
            let field_values = find_fields(
                param,
                match ty {
                    RPParamType::FloatParam | RPParamType::IntParam => {
                        vec!["name", "default_value", "range"]
                    }
                    _ => vec!["name", "default_value"],
                },
            );
            quote! { #(#field_values),* }
        };

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
            #ident: #ty::new(#args)#(#modifier_idents)*
        }
    });

    let bindings = generate_ts_bindings(&params);

    {
        quote! {
            #[derive(nih_plug::params::Params)]
            pub struct #ident {
                #(#fields),*
            }

            #(#param_enums)*

            impl Default for #ident {
                fn default() -> Self {
                    Self {
                        #(#defaults),*
                    }
                }
            }

            #bindings
        }
    }
        .into()
}

fn generate_ts_bindings(params: &RPParams) -> proc_macro2::TokenStream {
    let mut args = vec![];

    let param_type_def = params
        .params
        .iter()
        .map(|param| {
            format!(
                "{}: ReactPlug.Parameters.{}",
                param.ident.to_token_stream().to_string(),
                param.ty.to_token_stream().to_string()
            )
        })
        .collect::<Vec<_>>()
        .join(", \n    ");

    let param_defaults = params
        .params
        .iter()
        .map(|param| {
            let param_ident = &param.ident.to_string();
            let param_ty = &param.ty.to_token_stream().to_string();
            let mut options = vec![];

            param
                .fields
                .iter()
                .for_each(|field| match field.ident.to_string().as_str() {
                    "smoother" => {}
                    "range" => {
                        let (range_options, range_args) = range_to_ts(&field.expr, &param.ty);
                        options.push(format!("range: {}", range_options));
                        args.extend(range_args);
                    }
                    "value_to_string" => {
                        if let Expr::Call(call) = &field.expr {
                            if call
                                .func
                                .to_token_stream()
                                .to_string()
                                .split("::")
                                .next()
                                .unwrap()
                                .trim()
                                == "formatters"
                            {
                                call.args
                                    .iter()
                                    .for_each(|arg| args.push(arg.to_token_stream()));
                                options.push(format!(
                                    "value_to_string: ReactPlug.Formatters.{}({})",
                                    call.func
                                        .to_token_stream()
                                        .to_string()
                                        .split("::")
                                        .last()
                                        .unwrap()
                                        .trim(),
                                    "{}".repeat(call.args.len())
                                ));
                            }
                        }
                    }
                    "string_to_value" => {
                        // TODO
                    }
                    "variants" => {
                        let variants = if let Expr::Struct(variants) = &field.expr {
                            variants
                                .fields
                                .iter()
                                .map(|field| {
                                    let id = if let Member::Named(ident) = &field.member {
                                        ident.to_token_stream().to_string()
                                    } else {
                                        panic!("Invalid syntax for \"variants\" field");
                                    };

                                    let name = if let Expr::Lit(name) = &field.expr {
                                        if let syn::Lit::Str(s) = &name.lit {
                                            s.value()
                                        } else {
                                            panic!("Invalid syntax for \"variants\" field");
                                        }
                                    } else {
                                        id.clone()
                                    };

                                    format!(r#""{id}": "{name}""#)
                                })
                                .collect::<Vec<String>>()
                                .join(", ")
                        } else {
                            panic!("Invalid syntax for \"variants\" field");
                        };

                        options.push(format!("variants: {{{{ {} }}}}", variants))
                    }
                    "default_value" => {
                        if param.ty == RPParamType::EnumParam {
                            options.push(format!(
                                "defaultValue: \"{}\"",
                                field.expr.to_token_stream().to_string()
                            ));
                        } else {
                            options.push("defaultValue: {}".to_string());
                            args.push(field.expr.to_token_stream());
                        }
                    }
                    ident => {
                        options.push(format!(
                            "{}: {{:?}}",
                            ident.to_string().to_lower_camel_case()
                        ));
                        args.push(field.expr.to_token_stream());
                    }
                });

            if let Err(_) = try_find_field(param, "id") {
                options.push(format!("id: \"{}\"", param_ident));
            }

            format!(
                "{}: new ReactPlug.Parameters.{}({{{{ {} }}}})",
                param_ident,
                param_ty,
                options.join(", ")
            )
        })
        .collect::<Vec<_>>()
        .join(", \n    ");

    let params_bindings = format!(
        r#"import * as ReactPlug from "@exa04/react-plug";

export type Params = {{{{
    {}
}}}};

export const createParameters: () => Params = () => ({{{{
    {}
}}}});
"#,
        param_type_def, param_defaults
    );

    quote! {
        #[cfg(test)]
        mod bindings {
            use super::*;
            use std::fs::{File, create_dir_all};
            use std::env;
            use std::path::Path;
            use std::io::prelude::*;

            #[test]
            fn generate_bindings() {
                let ts = format!(#params_bindings, #(#args),*);

                let path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("gui/src/bindings");
                create_dir_all(&path).expect("Couldn't create directory for bindings");
                let mut file = File::create(path.join("Params.ts")).unwrap();
                file.write_all(ts.as_bytes()).unwrap();
            }
        }
    }
}

fn range_to_ts(expr: &Expr, param_type: &RPParamType) -> (String, Vec<proc_macro2::TokenStream>) {
    let mut args = vec![];
    match expr {
        Expr::Struct(structure) => {
            let range_type = structure
                .path
                .segments
                .last()
                .unwrap()
                .ident
                .to_token_stream();

            let constructor_args = structure
                .fields
                .iter()
                .map(|field| {
                    args.push(field.expr.to_token_stream());
                    format!("{}: {{}}", field.member.to_token_stream())
                })
                .collect::<Vec<String>>()
                .join(", ");

            (
                format!(
                    "new ReactPlug.Ranges.{}{}Range({{{{ {} }}}})",
                    range_type,
                    match param_type {
                        RPParamType::FloatParam => "Float",
                        RPParamType::IntParam => "Int",
                        _ => {
                            panic!("This parameter type doesn't support ranges.")
                        }
                    },
                    constructor_args
                ),
                args,
            )
        }
        Expr::Call(call) => {
            if let Expr::Path(path) = call.func.deref() {
                if path.path.segments.last().unwrap().ident == "Reversed" {
                    let mut expr = call.args.first().unwrap();

                    if let Expr::Reference(reference) = expr {
                        expr = &*reference.expr;
                    }

                    let (inner_options, inner_args) = range_to_ts(&expr, param_type);
                    (
                        format!(
                            "new ReactPlug.Ranges.Reversed{}Range({})",
                            match param_type {
                                RPParamType::FloatParam => "Float",
                                RPParamType::IntParam => "Int",
                                _ => {
                                    panic!("This parameter type doesn't support ranges.")
                                }
                            },
                            inner_options
                        ),
                        inner_args,
                    )
                } else {
                    panic!("No valid range provided!")
                }
            } else {
                panic!("No valid range provided!")
            }
        }
        _ => ("".to_string(), args),
    }
}
