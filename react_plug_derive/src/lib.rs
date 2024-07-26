extern crate proc_macro;
use heck::ToUpperCamelCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Data, DataEnum, DeriveInput, Expr, Meta, parse_macro_input, Token};
use syn::punctuated::Punctuated;
use params::*;

mod params;

// TODO: Support attributes and macros
// TODO: Skipping fields
// TODO: EnumParam support
#[proc_macro]
pub fn rp_params(
    input: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    let params = syn::parse::<RPParams>(input).unwrap();

    let struct_ident = &params.ident;

    let enum_ident = Ident::new(
        format!("{}Type", struct_ident.to_string().to_upper_camel_case()).as_str(),
        Span::call_site()
    );

    fn variant(ident: &Ident) -> Ident { Ident::new(
        ident.to_string().to_upper_camel_case().as_str(),
        Span::call_site()
    ) }

    let param_enum = {
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

        quote!{
            #[derive(ts_rs::TS, serde::Serialize, serde::Deserialize)]
            #[ts(export, export_to = "Param.ts")]
            pub enum #enum_ident {
                #(#param_enum_fields),*
            }
        }
    };

    let param_struct = {
        let param_struct_fields = params.params.iter().map(|param| {
            let ident = &param.ident;
            let ty = &param.ty;
            let id = ident.to_string();

            quote! {
            #[id = #id]
            pub #ident: #ty
        }
        });
        
        quote! {
            #[derive(Params)]
            pub struct #struct_ident {
                #(#param_struct_fields),*
            }
        }
    };

    let impl_block = {
        let initializers = params.params.iter().map(|param| {
            let ident = &param.ident;
            let ty = &param.ty;

            let required = match ty {
                RPParamType::FloatParam => { vec!["name", "value", "range"] },
                RPParamType::IntParam => { vec!["name", "value", "range"] },
                RPParamType::BoolParam => { vec!["name", "value"] }
            };

            let (required_params, modifier_params): (Vec<_>, Vec<_>) = param.fields.iter()
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
                            sender.send(PM::parameter_change(#enum_ident::#variant(value))).unwrap();
                        })
                    })
            }
        });

        quote! {
            impl #struct_ident {
                pub fn new<PM: react_plug::PluginMsg<#enum_ident> + 'static>(sender: &crossbeam_channel::Sender<PM>) -> Self {
                    Self {
                        #(#initializers),*
                    }
                }
            }
        }
    };

    let impl_parameters_block = {
        let send_value_fns = params.params.iter().map(|param| {
            let ident = &param.ident;
            let variant = variant(&param.ident);

            quote!{
                sender.send(PM::parameter_change(#enum_ident::#variant(self.#ident.value()))).unwrap();
            }
        });

        let set_param_match_arms = params.params.iter().map(|param| {
            let field = &param.ident;
            let variant = variant(&param.ident);

            quote! {
                #enum_ident::#variant(value) => {
                    setter.begin_set_parameter(&self.#field);
                    setter.set_parameter(&self.#field, *value);
                    setter.end_set_parameter(&self.#field);
                }
            }
        });

        quote! {
            impl react_plug::Parameters for #struct_ident {
                type ParamType = #enum_ident;

                fn send_all<PM: react_plug::PluginMsg<Self::ParamType> + 'static>(&self, sender: crossbeam_channel::Sender<PM>) {
                    #(#send_value_fns)*
                }

                fn set_param(&self, setter: &ParamSetter, param: &#enum_ident) {
                    match param {
                        #(#set_param_match_arms)*,
                    }
                }
            }
        }
    };

    {quote! {
        #param_enum

        impl react_plug::ParamType for #enum_ident { }

        #param_struct

        #impl_block

        #impl_parameters_block
    }}.into()
}

#[proc_macro_attribute]
pub fn plugin_message(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args with Punctuated::<Meta, syn::Token![,]>::parse_terminated);
    let input = parse_macro_input!(input as DeriveInput);

    let param = &args.iter()
        .find(|arg| arg.path().is_ident("params"))
        .expect("Missing params argument")
        .require_name_value()
        .expect("Params argument needs to be given a value")
        .value;

    let name = input.ident;
    let expanded = match input.data {
        Data::Enum(DataEnum { variants, .. }) => {
            let mut new_variants = variants.clone();

            // Define new variants
            new_variants.push(syn::parse_quote! { ParameterChange(ExampleParamsType) });

            // Generate the enum with the new variants
            quote! {
                #[derive(serde::Serialize, serde::Deserialize, ts_rs::TS)]
                #[ts(export, export_to = "PluginMessage.ts")]
                pub enum #name {
                    #new_variants
                }

                impl react_plug::PluginMsg<<#param as react_plug::Parameters>::ParamType> for #name {
                    fn parameter_change(param_type: <#param as react_plug::Parameters>::ParamType) -> Self {
                        Self::ParameterChange(param_type)
                    }
                }
            }
        },
        _ => panic!("Not an enum"),
    };

    expanded.into()
}

#[proc_macro_attribute]
pub fn gui_message(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args with Punctuated::<Meta, syn::Token![,]>::parse_terminated);
    let input = parse_macro_input!(input as DeriveInput);

    let param = &args.iter()
        .find(|arg| arg.path().is_ident("params"))
        .expect("Missing params argument")
        .require_name_value()
        .expect("Params argument needs to be given a value")
        .value;

    let name = input.ident;
    let expanded = match input.data {
        Data::Enum(DataEnum { variants, .. }) => {
            let mut new_variants = variants.clone();

            // Define new variants
            new_variants.push(syn::parse_quote! { Init });
            new_variants.push(syn::parse_quote! { ParameterChange(<#param as react_plug::Parameters>::ParamType) });

            // Generate the enum with the new variants
            quote! {
                #[derive(Serialize, Deserialize, ts_rs::TS)]
                #[ts(export, export_to = "GuiMessage.ts")]
                pub enum #name {
                    #new_variants
                }

                impl react_plug::GuiMsg<<#param as react_plug::Parameters>::ParamType> for #name {
                    fn is_init(&self) -> bool {
                        matches!(self, Self::Init)
                    }
                    fn is_param_update_and<F: FnOnce(&<#param as react_plug::Parameters>::ParamType)>(&self, action: F) {
                        if let Self::ParameterChange(param_type) = self {
                            action(param_type);
                        }
                    }
                }
            }
        },
        _ => panic!("Not an enum"),
    };

    expanded.into()
}