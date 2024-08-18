extern crate proc_macro;

use heck::{ToLowerCamelCase, ToUpperCamelCase};
use params::*;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use std::ops::Deref;
use syn::punctuated::Punctuated;
use syn::Expr::Struct;
use syn::{
    parse_macro_input, Data, DataEnum, DeriveInput, Expr, ExprStruct, Lit, Member, Meta, Token,
};

mod params;

// TODO: Skipping fields
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
                RPParamType::EnumParam => {
                    let ident = Ident::new(
                        format!(
                            "{}{}",
                            params.ident.to_string().to_upper_camel_case(),
                            param.ident.to_string().to_upper_camel_case()
                        )
                            .as_str(),
                        Span::call_site(),
                    );
                    quote! {#ident}
                }
            };
            let ident = Ident::new(
                param.ident.to_string().to_upper_camel_case().as_str(),
                Span::call_site(),
            );
            if let RPParamType::EnumParam = param.ty {
                quote! {
                    #[ts(skip)]
                    #ident(#ty)
                }
            } else {
                quote! {
                    #ident(#ty)
                }
            }
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
            let id = ident.to_string();

            let ty = &param.ty;

            if let RPParamType::EnumParam = ty {
                let enum_ident = Ident::new(
                    format!(
                        "{}{}",
                        params.ident.to_string().to_upper_camel_case(),
                        ident.to_string().to_upper_camel_case()
                    )
                        .as_str(),
                    Span::call_site(),
                );

                quote! {
                    #[id = #id]
                    pub #ident: EnumParam<#enum_ident>
                }
            } else {
                quote! {
                    #[id = #id]
                    pub #ident: #ty
                }
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
                RPParamType::FloatParam => { vec!["name", "value", "range"] }
                RPParamType::IntParam => { vec!["name", "value", "range"] }
                RPParamType::BoolParam => { vec!["name", "value"] }
                RPParamType::EnumParam => { vec!["name", "value"] }
            };

            let (required_params, modifier_params): (Vec<_>, Vec<_>) = param.fields.iter()
                .partition(|field| required.contains(&&*field.ident.to_string()));

            let mut args: Punctuated<Expr, Token![,]> = Punctuated::new();

            required.iter().for_each(|name| {
                let expr = required_params.iter()
                    .find(|field| *field.ident.to_string() == name.to_string())
                    .unwrap().expr.clone();

                if name == &"range" {
                    if let Expr::Call(call) = &expr {
                        if let Expr::Path(path) = call.func.deref() {
                            if path.path.segments.last().unwrap().ident == "Reversed" {
                                let expr = call.args.first().unwrap();

                                args.push(Expr::Verbatim(quote! {#path(&#expr)}));
                                return;
                            }
                        }
                    }
                }

                args.push(expr);
            });

            if let RPParamType::EnumParam = ty {
                let enum_ident = Ident::new(
                    format!(
                        "{}{}",
                        params.ident.to_string().to_upper_camel_case(),
                        ident.to_string().to_upper_camel_case()
                    )
                        .as_str(),
                    Span::call_site(),
                ).to_token_stream();

                let value = args.get(1).unwrap().to_token_stream().clone();

                *args.get_mut(1).unwrap() = Expr::Verbatim(quote! {#enum_ident::#value});
            }

            let modifiers = modifier_params.iter().filter_map(|param| -> Option<TokenStream> {
                let mut arg = &param.expr.clone();

                // TODO: Use a more sophisticated system

                let ident = &param.ident.to_string();

                let modifier = Ident::new(format!("with_{}", ident).as_str(), Span::call_site());

                if ident == "variants" {
                    return None;
                }

                Some(quote! {.#modifier(#arg)})
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

            quote! {
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

    let enum_params = {
        params.params.iter().filter_map(|param| {
            if let RPParamType::EnumParam = param.ty {
                let ident = &param.ident;
                let enum_ident = Ident::new(
                    format!(
                        "{}{}",
                        params.ident.to_string().to_upper_camel_case(),
                        ident.to_string().to_upper_camel_case()
                    )
                        .as_str(),
                    Span::call_site(),
                );
                let variants_field = &param
                    .fields
                    .iter()
                    .find(|field| field.ident == "variants")
                    .expect(format!("No variants field for param \"{}\" provided!", ident).as_str())
                    .expr;

                let param_enum = if let Struct(s) = variants_field {
                    let variants = s.fields.iter().map(|field| {
                        let id = if let Member::Named(ident) = &field.member {
                            ident
                        } else {
                            panic!("Invalid syntax for \"variants\" field");
                        };

                        if let Expr::Lit(name) = &field.expr {
                            quote! {
                                #[name = #name]
                                #id
                            }
                        } else {
                            id.to_token_stream()
                        }
                    });

                    quote! {
                        #[derive(nih_plug::prelude::Enum, PartialEq, Clone, Copy, serde::Serialize, serde::Deserialize)]
                        pub enum #enum_ident {
                            #(#variants),*
                        }
                    }
                } else {
                    panic!("Invalid syntax for \"variants\" field");
                };

                Some(param_enum)
            } else {
                None
            }
        })
    };

    let test_block = {
        let declarations = params
            .params
            .iter()
            .map(|param| {
                let ident = &param.ident;
                let ty = &param.ty;
                let id = ident.to_string().to_lower_camel_case();

                format!("{}: ReactPlug.Parameters.{}", id, ty.to_token_stream())
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

            let optional_field_by_id = |id: &str| {
                param
                    .fields
                    .iter()
                    .find(|field| field.ident == id)
                    .map(|field| field.expr.clone())
            };

            let field_by_id = |id: &str| {
                optional_field_by_id(id)
                    .unwrap_or_else(|| panic!("No value for param field \"{}\" provided!", id))
                    .clone()
            };

            match &param.ty {
                RPParamType::FloatParam => {
                    expressions.push(field_by_id("name"));
                    expressions.push(field_by_id("value"));

                    fn range_to_ts(range: &Expr, expressions: &mut Vec<Expr>) -> String {
                        match range {
                            Expr::Struct(range) => {
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
                                        expressions.push(field_by_id(&range, "min"));
                                        expressions.push(field_by_id(&range, "max"));
                                        "new ReactPlug.Ranges.LinearRange({}, {})".to_string()
                                    }
                                    "Skewed" => {
                                        expressions.push(field_by_id(&range, "min"));
                                        expressions.push(field_by_id(&range, "max"));
                                        expressions.push(field_by_id(&range, "factor"));
                                        "new ReactPlug.Ranges.SkewedRange({}, {}, {})".to_string()
                                    }
                                    "SymmetricalSkewed" => {
                                        expressions.push(field_by_id(&range, "min"));
                                        expressions.push(field_by_id(&range, "max"));
                                        expressions.push(field_by_id(&range, "factor"));
                                        expressions.push(field_by_id(&range, "center"));
                                        "new ReactPlug.Ranges.SymmetricalSkewedRange({}, {}, {}, {})"
                                            .to_string()
                                    }
                                    r => panic!("Invalid range type: {}", r),
                                }
                            }
                            Expr::Call(call) => {
                                if let Expr::Path(path) = call.func.deref() {
                                    let path = path
                                        .path
                                        .segments
                                        .iter()
                                        .map(|segment| segment.ident.to_string())
                                        .collect::<Vec<String>>();

                                    if match &path[..] {
                                        [.., r, t] => {
                                            if r != "FloatRange" {
                                                panic!("Invalid range type: {}", r);
                                            }
                                            t
                                        }
                                        [t] => t,
                                        _ => panic!("Invalid range type"),
                                    } == "Reversed" {
                                        format!("new ReactPlug.Ranges.ReversedRange({})", range_to_ts(call.args.first().expect("No range provided for ReversedRange"), expressions))
                                    } else {
                                        panic!("Invalid range");
                                    }
                                } else {
                                    panic!("Invalid range");
                                }
                            }
                            _ => panic!("Range is not a struct"),
                        }
                    }

                    let range = range_to_ts(&field_by_id("range"), &mut expressions);

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

                    fn range_to_ts(range: &Expr, expressions: &mut Vec<Expr>) -> String {
                        match range {
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
                                        r => panic!("Invalid range type: {}", r),
                                    }
                                };

                                range_to_ts(&range)
                            }
                            Expr::Call(call) => {
                                if let Expr::Path(path) = call.func.deref() {
                                    let path = path
                                        .path
                                        .segments
                                        .iter()
                                        .map(|segment| segment.ident.to_string())
                                        .collect::<Vec<String>>();

                                    if match &path[..] {
                                        [.., r, t] => {
                                            if r != "IntRange" {
                                                panic!("Invalid range type: {}", r);
                                            }
                                            t
                                        }
                                        [t] => t,
                                        _ => panic!("Invalid range type"),
                                    } == "Reversed" {
                                        format!("new ReactPlug.Ranges.ReversedRange({})", range_to_ts(call.args.first().expect("No range provided for ReversedRange"), expressions))
                                    } else {
                                        panic!("Invalid range");
                                    }
                                } else {
                                    panic!("Invalid range");
                                }
                            }
                            _ => panic!("Range is not a struct"),
                        }
                    }

                    let range = range_to_ts(&field_by_id("range"), &mut expressions);

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
                RPParamType::EnumParam => {
                    expressions.push(field_by_id("name"));
                    &param.fields;

                    let value = format!(
                        r#""{}""#,
                        field_by_id("value").to_token_stream()
                    );

                    let variants = if let Expr::Struct(variants) = field_by_id("variants") {
                        variants.fields.iter().map(|field| {
                            let id = if let Member::Named(ident) = &field.member {
                                ident.to_token_stream().to_string()
                            } else {
                                panic!("Invalid syntax for \"variants\" field");
                            };

                            let name = if let Expr::Lit(name) = &field.expr {
                                if let Lit::Str(s) = &name.lit {
                                    s.value()
                                } else {
                                    panic!("Invalid syntax for \"variants\" field");
                                }
                            } else {
                                id.clone()
                            };

                            format!(r#"{id}: "{name}""#)
                        }).collect::<Vec<String>>().join(", ")
                    } else {
                        panic!("Invalid syntax for \"variants\" field");
                    };

                    let s = format!(
                        r#"new ReactPlug.Parameters.EnumParam("{}", "{{}}", {}, {{{{ {} }}}}), "#,
                        param.ident.to_string().to_upper_camel_case(),
                        value,
                        variants
                    );

                    initializer.push_str(&s);
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

import * as ReactPlug from '@exa04/react-plug';

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
          (param as unknown as ReactPlug.Parameters.FloatParam)._setDisplayedValue(value as unknown as number);
        else if (param.type == 'IntParam')
          (param as unknown as ReactPlug.Parameters.IntParam)._setDisplayedValue(value as unknown as number);
        else if (param.type == 'BoolParam')
          (param as unknown as ReactPlug.Parameters.BoolParam)._setDisplayedValue(value as unknown as boolean);
        else if (param.type == 'EnumParam')
          (param as unknown as ReactPlug.Parameters.EnumParam)._setDisplayedValue(value as unknown as string);
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

                            let path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("gui/src/bindings");
                            std::fs::create_dir_all(&path).expect("Couldn't create directory for bindings");
                            let mut file = File::create(path.join("PluginProvider.tsx")).unwrap();
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

            #(#enum_params)*

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
            new_variants.push(syn::parse_quote! { ParameterChange(<#param as react_plug::Parameters>::ParamType) });

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
                #[derive(serde::Serialize, serde::Deserialize, ts_rs::TS)]
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
