extern crate proc_macro;

use heck::{ToLowerCamelCase, ToUpperCamelCase};
use params::*;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Data, DataEnum, DeriveInput, Expr, ExprStruct, Meta, Token};

mod params;

// TODO: Skipping fields
// TODO: EnumParam support
#[proc_macro]
pub fn rp_params<'a>(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let params = syn::parse::<RPParams>(input).unwrap();

    let struct_ident = &params.ident;

    let enum_ident = Ident::new(
        format!("{}Type", struct_ident.to_string().to_upper_camel_case()).as_str(),
        Span::call_site(),
    );

    fn variant(ident: &Ident) -> Ident {
        Ident::new(
            ident.to_string().to_upper_camel_case().as_str(),
            Span::call_site(),
        )
    }

    let param_enum = {
        let param_enum_fields = params.params.iter().map(|param| {
            let ty = match &param.ty {
                RPParamType::FloatParam => {
                    quote! {f32}
                }
                RPParamType::IntParam => {
                    quote! {i32}
                }
                RPParamType::BoolParam => {
                    quote! {bool}
                }
            };
            let ident = Ident::new(
                param.ident.to_string().to_upper_camel_case().as_str(),
                Span::call_site(),
            );
            quote! {#ident(#ty)}
        });

        quote! {
            #[derive(ts_rs::TS, serde::Serialize, serde::Deserialize)]
            #[ts(export, export_to = "../gui/src/bindings/Param.ts")]
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

    let test_block = {
        let declarations = params
            .params
            .iter()
            .map(|param| {
                let ident = &param.ident;
                let ty = &param.ty;
                let id = ident.to_string().to_lower_camel_case();

                match ty {
                    RPParamType::FloatParam => format!("{}: ReactPlug.Parameters.FloatParam", id),
                    RPParamType::IntParam => format!("{}: ReactPlug.Parameters.IntParam", id),
                    RPParamType::BoolParam => format!("{}: ReactPlug.Parameters.BoolParam", id),
                }
            })
            .collect::<Vec<String>>()
            .join(",\n  ");

        let mut initializer: String = String::new();
        let mut expressions: Vec<Expr> = vec![];

        params.params.iter().for_each(|param| {
            initializer.push_str(&format!(
                "{}: ",
                (&param.ident).to_string().to_lower_camel_case()
            ));

            let field_by_id = |id: &str| {
                param
                    .fields
                    .iter()
                    .find(|field| field.ident == id)
                    .unwrap_or_else(|| panic!("No value for param field \"{}\" provided!", id))
                    .expr
                    .clone()
            };

            let optional_field_by_id = |id: &str| {
                param
                    .fields
                    .iter()
                    .find(|field| field.ident == id)
                    .map(|field| field.expr.clone())
            };

            match &param.ty {
                RPParamType::FloatParam => {
                    expressions.push(field_by_id("name"));
                    expressions.push(field_by_id("value"));

                    let range = match field_by_id("range") {
                        Expr::Struct(range) => {
                            let mut range_to_ts = |range: &ExprStruct| {
                                let field_by_id = |range: &ExprStruct, id: &str| -> Expr {
                                    range
                                        .fields
                                        .iter()
                                        .find(|field| {
                                            field.member.to_token_stream().to_string() == id
                                        })
                                        .unwrap_or_else(|| {
                                            panic!("No value for param field \"{}\" provided!", id)
                                        })
                                        .expr
                                        .clone()
                                };

                                let path = range
                                    .path
                                    .segments
                                    .iter()
                                    .map(|segment| segment.ident.to_string())
                                    .collect::<Vec<String>>();

                                let range_type = match &path[..] {
                                    [.., r, t] => {
                                        if r != "FloatRange" {
                                            panic!("Invalid range type: {}", r);
                                        }
                                        t
                                    }
                                    [t] => t,
                                    _ => panic!("Invalid range type"),
                                };

                                match range_type.as_str() {
                                    "Linear" => {
                                        expressions.push(field_by_id(range, "min"));
                                        expressions.push(field_by_id(range, "max"));
                                        "new ReactPlug.Ranges.LinearRange({}, {})".to_string()
                                    }
                                    "Skewed" => {
                                        expressions.push(field_by_id(range, "min"));
                                        expressions.push(field_by_id(range, "max"));
                                        expressions.push(field_by_id(range, "factor"));
                                        "new ReactPlug.Ranges.SkewedRange({}, {}, {})".to_string()
                                    }
                                    "SymmetricalSkewed" => {
                                        expressions.push(field_by_id(range, "min"));
                                        expressions.push(field_by_id(range, "max"));
                                        expressions.push(field_by_id(range, "factor"));
                                        expressions.push(field_by_id(range, "center"));
                                        "new ReactPlug.Ranges.SymmetricalSkewedRange({}, {}, {}, {})"
                                            .to_string()
                                    }
                                    "Reversed" => {
                                        todo!()
                                    }
                                    r => panic!("Invalid range type: {}", r),
                                }
                            };

                            range_to_ts(&range)
                        }
                        Expr::Call(_) => {
                            todo!()
                        }
                        _ => panic!("Range is not a struct"),
                    };

                    let mut options = String::new();

                    if let Some(expr) = optional_field_by_id("unit") {
                        expressions.push(expr);
                        options.push_str(r#"unit: "{}", "#);
                    }

                    if let Some(expr) = optional_field_by_id("stepSize") {
                        expressions.push(expr);
                        options.push_str(r#"stepSize: {}, "#);
                    }

                    if let Some(Expr::Call(call)) = optional_field_by_id("value_to_string") {
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
                            expressions.push(call.args.first().unwrap().clone());
                            options.push_str(&format!(
                                r#"formatter: ReactPlug.Formatters.{}({{}}), "#,
                                call.func
                                    .to_token_stream()
                                    .to_string()
                                    .split("::")
                                    .last()
                                    .unwrap()
                                    .trim()
                            ));
                        }
                    }

                    //                                                     name   id     value range options
                    initializer.push_str(&format!(
                        r#"new ReactPlug.Parameters.FloatParam("{}", "{{}}", {{}}, {}, {{{{ {}}}}}), "#,
                        param.ident.to_string().to_upper_camel_case(),
                        range,
                        options
                    ));
                }
                RPParamType::IntParam => {
                    expressions.push(field_by_id("name"));
                    expressions.push(field_by_id("value"));

                    let range = match field_by_id("range") {
                        Expr::Struct(range) => {
                            let mut range_to_ts = |range: &ExprStruct| {
                                let field_by_id = |range: &ExprStruct, id: &str| -> Expr {
                                    range
                                        .fields
                                        .iter()
                                        .find(|field| {
                                            field.member.to_token_stream().to_string() == id
                                        })
                                        .unwrap_or_else(|| {
                                            panic!("No value for param field \"{}\" provided!", id)
                                        })
                                        .expr
                                        .clone()
                                };

                                let path = range
                                    .path
                                    .segments
                                    .iter()
                                    .map(|segment| segment.ident.to_string())
                                    .collect::<Vec<String>>();

                                let range_type = match &path[..] {
                                    [.., r, t] => {
                                        if r != "IntRange" {
                                            panic!("Invalid range type: {}", r);
                                        }
                                        t
                                    }
                                    [t] => t,
                                    _ => panic!("Invalid range type"),
                                };

                                match range_type.as_str() {
                                    "Linear" => {
                                        expressions.push(field_by_id(range, "min"));
                                        expressions.push(field_by_id(range, "max"));
                                        "new ReactPlug.Ranges.LinearRange({}, {})".to_string()
                                    }
                                    "Reversed" => {
                                        todo!()
                                    }
                                    r => panic!("Invalid range type: {}", r),
                                }
                            };

                            range_to_ts(&range)
                        }
                        Expr::Call(_) => {
                            todo!()
                        }
                        _ => panic!("Range is not a struct"),
                    };

                    let mut options = String::new();

                    if let Some(expr) = optional_field_by_id("unit") {
                        expressions.push(expr);
                        options.push_str(r#"unit: "{}", "#);
                    }

                    if let Some(Expr::Call(call)) = optional_field_by_id("value_to_string") {
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
                            expressions.push(call.args.first().unwrap().clone());
                            options.push_str(&format!(
                                r#"formatter: formatters.{}({{}}), "#,
                                call.func
                                    .to_token_stream()
                                    .to_string()
                                    .split("::")
                                    .last()
                                    .unwrap()
                                    .trim()
                            ));
                        }
                    }

                    //                                                    name  id     value range options
                    initializer.push_str(&format!(
                        r#"new ReactPlug.Parameters.IntParam("{}", "{{}}", {{}}, {}, {{{{ {}}}}}), "#,
                        param.ident.to_string().to_upper_camel_case(),
                        range,
                        options
                    ));
                }
                RPParamType::BoolParam => {
                    expressions.push(field_by_id("name"));
                    expressions.push(field_by_id("value"));

                    let mut options = String::new();

                    if let Some(expr) = optional_field_by_id("unit") {
                        expressions.push(expr);
                        options.push_str(r#"unit: "{}", "#);
                    }

                    if let Some(Expr::Call(call)) = optional_field_by_id("value_to_string") {
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
                            expressions.push(call.args.first().unwrap().clone());
                            options.push_str(&format!(
                                r#"formatter: ReactPlug.Formatters.{}({{}}), "#,
                                call.func
                                    .to_token_stream()
                                    .to_string()
                                    .split("::")
                                    .last()
                                    .unwrap()
                                    .trim()
                            ));
                        }
                    }

                    initializer.push_str(&format!(
                        r#"new ReactPlug.Parameters.BoolParam("{}", "{{}}", {{}}, {{{{ {}}}}}), "#,
                        param.ident.to_string().to_upper_camel_case(),
                        options
                    ));
                }
            };
        });

        // TODO: User-defined export path for bindings?

        quote! {
                    #[cfg(test)]
                    mod test {
                        use super::*;
                        use std::fs::File;
                        use std::env;
                        use std::path::Path;
                        use std::io::prelude::*;

                        #[test]
                        fn generate_provider() {
                            let init = format!(#initializer, #(#expressions),*);
                            let ts =
        format!(r#"import {{createContext, FC, ReactNode, useContext, useEffect, useRef}} from 'react';

import {{EventEmitter}} from 'events';

import * as ReactPlug from 'react-plug';

import {{GuiMessage}} from "./GuiMessage";
import {{PluginMessage}} from "./PluginMessage";

interface ContextType {{
  parameters: Params;
  sendToPlugin: (message: GuiMessage) => void;
  addMessageListener: (action: (message: PluginMessage) => void) => void;
  removeMessageListener: (action: (message: PluginMessage) => void) => void;
}}

const PluginContext = createContext<ContextType | undefined>(undefined);

type Params = {{
  {}
}};

const PluginProvider: FC<{{ children: ReactNode }}> = ({{children}}) => {{
  const eventEmitter = useRef(new EventEmitter());

  const addMessageListener = (action: (message: PluginMessage) => void) => eventEmitter.current.on('pluginMessage', action);
  const removeMessageListener = (action: (message: PluginMessage) => void) => eventEmitter.current.off('pluginMessage', action);

  const parameters: Params = {{
    {}
  }};

  useEffect(() => {{
    ReactPlug.util.sendToPlugin('Init');

    // TODO: This kinda sucks
    (window as any).onPluginMessage = (message: Object) => {{
      console.log('Message (Plugin -> GUI)', message);
      if (ReactPlug.util.isParameterChange(message)) {{
        const [id, value] = Object.entries(message.ParameterChange)[0];

        const param = Object.values(parameters)
          .find((p) => p.id == id);

        if (param === undefined)
          throw new Error('usePluginContext must be used within a provider');

        if (param.type == 'FloatParam')
          (param as ReactPlug.Parameters.FloatParam)._setDisplayedValue(value as unknown as number);
        else if (param.type == 'IntParam')
          (param as ReactPlug.Parameters.IntParam)._setDisplayedValue(value as unknown as number);
        else if (param.type == 'BoolParam')
          (param as ReactPlug.Parameters.BoolParam)._setDisplayedValue(value as unknown as boolean);
      }} else {{
        eventEmitter.current.emit('pluginMessage', message as PluginMessage);
      }}
    }};
  }}, []);

  return (
    <PluginContext.Provider value={{{{
      parameters,
      sendToPlugin: ReactPlug.util.sendToPlugin,
      addMessageListener,
      removeMessageListener
    }}}}>
      {{children}}
    </PluginContext.Provider>
  );
}};

export const usePluginContext = () => {{
  const context = useContext(PluginContext);
  if (!context) {{
    throw new Error('usePluginContext must be used within a provider');
  }}
  return context;
}};

export default PluginProvider;
"#, #declarations, init);


                            let path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("gui/src/bindings/PluginProvider.tsx");
                            let mut file = File::create(path).unwrap();
                            file.write_all(ts.as_bytes()).unwrap();
                        }
                    }
                }
    };

    {
        quote! {
            #param_enum

            impl react_plug::ParamType for #enum_ident { }

            #param_struct

            #impl_block

            #impl_parameters_block

            #test_block
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn plugin_message(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args with Punctuated::<Meta, syn::Token![,]>::parse_terminated);
    let input = parse_macro_input!(input as DeriveInput);

    let param = &args
        .iter()
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
                #[ts(export, export_to = "../gui/src/bindings/PluginMessage.ts")]
                pub enum #name {
                    #new_variants
                }

                impl react_plug::PluginMsg<<#param as react_plug::Parameters>::ParamType> for #name {
                    fn parameter_change(param_type: <#param as react_plug::Parameters>::ParamType) -> Self {
                        Self::ParameterChange(param_type)
                    }
                }
            }
        }
        _ => panic!("Not an enum"),
    };

    expanded.into()
}

#[proc_macro_attribute]
pub fn gui_message(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args with Punctuated::<Meta, syn::Token![,]>::parse_terminated);
    let input = parse_macro_input!(input as DeriveInput);

    let param = &args
        .iter()
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
                #[ts(export, export_to = "../gui/src/bindings/GuiMessage.ts")]
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
        }
        _ => panic!("Not an enum"),
    };

    expanded.into()
}
