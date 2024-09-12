use crate::{GuiMessage, MessageChannel, ParamChange, PluginMessage};
use include_dir::Dir;
use nih_plug::editor::{Editor, ParentWindowHandle};
use nih_plug::params::Params;
use nih_plug::prelude::{GuiContext, ParamPtr};
use nih_plug::{nih_dbg, nih_log, nih_warn};
use nih_plug_webview::http::Response;
use nih_plug_webview::{EventStatus, HTMLSource, KeyboardEvent, MouseEvent, WebViewEditor};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::any::Any;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

pub struct ReactPlugEditor<PM, GM>
where
    PM: Serialize + DeserializeOwned,
    GM: Serialize + DeserializeOwned,
{
    editor: WebViewEditor,
    plugin_msg_channel: MessageChannel<PluginMessage<PM>>,
    dir: &'static Dir<'static>,
    gui_messages: PhantomData<GM>,
    params: Arc<dyn Params>,
}

impl<PM, GM> ReactPlugEditor<PM, GM>
where
    PM: serde::Serialize + DeserializeOwned + Send + Sync + Debug + 'static,
    GM: serde::Serialize + DeserializeOwned + Send + Sync + Debug + 'static,
{
    pub fn new(params: Arc<impl Params>, dir: &'static Dir, size: (u32, u32)) -> Self {
        let plugin_msg_channel = Arc::new(crossbeam_channel::unbounded());
        let pm_channel_editor = plugin_msg_channel.clone();
        let param_map = params.param_map();

        let editor = if cfg!(rp_dev) {
            WebViewEditor::new(HTMLSource::URL("http://localhost:5173"), size)
        } else {
            let protocol: &'static str = "reactplug";

            #[cfg(target_os = "windows")]
            let url_scheme = "http://reactplug.localhost";
            // TODO: Not tested on Linux / MacOS
            #[cfg(not(target_os = "windows"))]
            let url_scheme = "reactplug://localhost";

            let url = HTMLSource::URL(url_scheme);

            WebViewEditor::new(url, size).with_custom_protocol(protocol.parse().unwrap(), |req| {
                let path = req.uri().path();

                let path = if path == "/" {
                    "index.html"
                } else {
                    &path[1..]
                };

                let mime_type = mime_guess::from_path(path)
                    .first_or_text_plain()
                    .to_string();

                if let Some(file) = dir.get_file(path) {
                    let content = file.contents();

                    Response::builder()
                        .header("content-type", mime_type)
                        .header("Access-Control-Allow-Origin", "*")
                        .body(content.into())
                        .map_err(Into::into)
                } else {
                    Response::builder()
                        .header("content-type", "text/plain")
                        .header("Access-Control-Allow-Origin", "*")
                        .body("404 Not Found".as_bytes().into())
                        .map_err(Into::into)
                }
            })
        }
            .with_event_loop(move |ctx, setter, _window| unsafe {
                while let Ok(value) = ctx.next_event() {
                    if let Ok(message) = serde_json::from_value::<GuiMessage<GM>>(value.clone()) {
                        match message {
                            GuiMessage::Init => {
                                param_map.iter().for_each(|(id, param, _)| {
                                    pm_channel_editor
                                        .0
                                        .send(PluginMessage::ParamChange(ParamChange {
                                            id: id.to_string(),
                                            value: param.modulated_normalized_value(),
                                        }))
                                        .unwrap()
                                });
                            }
                            GuiMessage::ParamChange(param_change) => {
                                let param = if let Some(param) =
                                    param_map.iter().find(|(id, _, _)| id == &param_change.id)
                                {
                                    param.1
                                } else {
                                    nih_warn!("Couldn't find parameter with id: {}", param_change.id);
                                    continue;
                                };

                                match param {
                                    ParamPtr::FloatParam(p) => setter.begin_set_parameter(&*p),
                                    ParamPtr::IntParam(p) => setter.begin_set_parameter(&*p),
                                    ParamPtr::BoolParam(p) => setter.begin_set_parameter(&*p),
                                    ParamPtr::EnumParam(p) => setter.begin_set_parameter(&*p),
                                }

                                match param {
                                    ParamPtr::FloatParam(p) => {
                                        setter.set_parameter_normalized(&*p, param_change.value)
                                    }
                                    ParamPtr::IntParam(p) => {
                                        setter.set_parameter_normalized(&*p, param_change.value)
                                    }
                                    ParamPtr::BoolParam(p) => {
                                        setter.set_parameter_normalized(&*p, param_change.value)
                                    }
                                    ParamPtr::EnumParam(p) => {
                                        setter.set_parameter_normalized(&*p, param_change.value)
                                    }
                                }

                                match param {
                                    ParamPtr::FloatParam(p) => setter.end_set_parameter(&*p),
                                    ParamPtr::IntParam(p) => setter.end_set_parameter(&*p),
                                    ParamPtr::BoolParam(p) => setter.end_set_parameter(&*p),
                                    ParamPtr::EnumParam(p) => setter.end_set_parameter(&*p),
                                }
                            }
                            GuiMessage::Message(message) => {}
                        }
                    } else {
                        nih_warn!("Couldn't deserialize message from GUI: {:?}", value);
                    }
                }
                while !pm_channel_editor.1.is_empty() {
                    let message = pm_channel_editor.1.recv().unwrap();
                    let message_json = serde_json::to_value(&message);
                    if let Ok(message_json) = message_json {
                        ctx.send_json(message_json).unwrap_or_else(|err| {
                            nih_warn!(
                            r#"Couldn't send message {:?} to GUI!
{}"#,
                            &message,
                            err
                        );
                        })
                    } else {
                        nih_warn!(
                        r#"Message couldn't be sent to GUI! Couldn't serialize {:?}"#,
                        message
                    );
                    }
                }
            });

        Self {
            editor,
            plugin_msg_channel,
            dir,
            gui_messages: PhantomData,
            params,
        }
    }

    pub fn with_message_handler(
        mut self,
        handler: impl Fn(
            GM,
            Arc<
                dyn Fn(PM) -> Result<(), crossbeam_channel::TrySendError<PluginMessage<PM>>>
                + 'static,
            >,
        ) + Send
        + Sync
        + 'static,
    ) -> Self {
        let pm_channel = self.plugin_msg_channel.clone();
        let param_map = self.params.param_map();

        self.editor = self
            .editor
            .with_event_loop(move |ctx, setter, _window| unsafe {
                while let Ok(value) = ctx.next_event() {
                    if let Ok(message) = serde_json::from_value::<GuiMessage<GM>>(value.clone()) {
                        match message {
                            GuiMessage::Init => {
                                param_map.iter().for_each(|(id, param, _)| {
                                    pm_channel
                                        .0
                                        .send(PluginMessage::ParamChange(ParamChange {
                                            id: id.to_string(),
                                            value: param.modulated_normalized_value(),
                                        }))
                                        .unwrap()
                                });
                            }
                            GuiMessage::ParamChange(param_change) => {
                                let param = if let Some(param) =
                                    param_map.iter().find(|(id, _, _)| id == &param_change.id)
                                {
                                    param.1
                                } else {
                                    nih_warn!(
                                        "Couldn't find parameter with id: {}",
                                        param_change.id
                                    );
                                    continue;
                                };

                                match param {
                                    ParamPtr::FloatParam(p) => setter.begin_set_parameter(&*p),
                                    ParamPtr::IntParam(p) => setter.begin_set_parameter(&*p),
                                    ParamPtr::BoolParam(p) => setter.begin_set_parameter(&*p),
                                    ParamPtr::EnumParam(p) => setter.begin_set_parameter(&*p),
                                }

                                match param {
                                    ParamPtr::FloatParam(p) => {
                                        setter.set_parameter_normalized(&*p, param_change.value)
                                    }
                                    ParamPtr::IntParam(p) => {
                                        setter.set_parameter_normalized(&*p, param_change.value)
                                    }
                                    ParamPtr::BoolParam(p) => {
                                        setter.set_parameter_normalized(&*p, param_change.value)
                                    }
                                    ParamPtr::EnumParam(p) => {
                                        setter.set_parameter_normalized(&*p, param_change.value)
                                    }
                                }

                                match param {
                                    ParamPtr::FloatParam(p) => setter.end_set_parameter(&*p),
                                    ParamPtr::IntParam(p) => setter.end_set_parameter(&*p),
                                    ParamPtr::BoolParam(p) => setter.end_set_parameter(&*p),
                                    ParamPtr::EnumParam(p) => setter.end_set_parameter(&*p),
                                }
                            }
                            GuiMessage::Message(message) => {
                                let sender = pm_channel.0.clone();
                                handler(
                                    message,
                                    Arc::new(move |pm| sender.try_send(PluginMessage::Message(pm))),
                                )
                            }
                        }
                    } else {
                        nih_warn!("Couldn't deserialize message from GUI: {:?}", value);
                    }
                }
                while !pm_channel.1.is_empty() {
                    let message = pm_channel.1.recv().unwrap();
                    let message_json = serde_json::to_value(&message);
                    if let Ok(message_json) = message_json {
                        ctx.send_json(message_json).unwrap_or_else(|err| {
                            nih_warn!(
                                r#"Couldn't send message {:?} to GUI!
{}"#,
                                &message,
                                err
                            );
                        })
                    } else {
                        nih_warn!(
                            r#"Message couldn't be sent to GUI! Couldn't serialize {:?}"#,
                            message
                        );
                    }
                }
            });
        self
    }

    pub fn with_background_color(mut self, background_color: (u8, u8, u8, u8)) -> Self {
        self.editor = self.editor.with_background_color(background_color);
        self
    }

    pub fn with_developer_mode(mut self, mode: bool) -> Self {
        self.editor = self.editor.with_developer_mode(mode);
        self
    }

    pub fn with_keyboard_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(KeyboardEvent) -> bool + Send + Sync + 'static,
    {
        self.editor = self.editor.with_keyboard_handler(handler);
        self
    }

    pub fn with_mouse_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(MouseEvent) -> EventStatus + Send + Sync + 'static,
    {
        self.editor = self.editor.with_mouse_handler(handler);
        self
    }

    pub fn with_protocol(mut self, protocol: &'static str) -> Self {
        todo!()
    }
}

impl<PM, GM> Editor for ReactPlugEditor<PM, GM>
where
    PM: serde::Serialize + DeserializeOwned + Send + Sync + Debug + 'static,
    GM: serde::Serialize + DeserializeOwned + Send + Sync + Debug + 'static,
{
    fn spawn(
        &self,
        parent: ParentWindowHandle,
        context: Arc<dyn GuiContext>,
    ) -> Box<dyn Any + Send> {
        self.editor.spawn(parent, context)
    }

    fn size(&self) -> (u32, u32) {
        self.editor.size()
    }

    fn set_scale_factor(&self, factor: f32) -> bool {
        self.editor.set_scale_factor(factor)
    }

    fn param_value_changed(&self, id: &str, normalized_value: f32) {
        self.plugin_msg_channel
            .0
            .send(PluginMessage::ParamChange(ParamChange {
                id: id.to_string(),
                value: normalized_value,
            }))
            .expect("Couldn't send parameter update message through internal channel!");
    }

    fn param_modulation_changed(&self, id: &str, modulation_offset: f32) {
        self.editor.param_modulation_changed(id, modulation_offset)
    }

    fn param_values_changed(&self) {
        // TODO: Does this need to be handled too?
    }
}

impl<PM, GM> From<ReactPlugEditor<PM, GM>> for Option<Box<dyn Editor + 'static>>
where
    PM: serde::Serialize + DeserializeOwned + Send + Sync + Debug + 'static,
    GM: serde::Serialize + DeserializeOwned + Send + Sync + Debug + 'static,
{
    fn from(editor: ReactPlugEditor<PM, GM>) -> Self {
        Some(Box::new(editor))
    }
}
