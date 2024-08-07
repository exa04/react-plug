use std::any::Any;
use std::sync::Arc;

use include_dir::Dir;
use nih_plug::editor::{Editor, ParentWindowHandle};
use nih_plug::nih_log;
use nih_plug::prelude::GuiContext;
use nih_plug_webview::{EventStatus, HTMLSource, KeyboardEvent, MouseEvent, WebViewEditor};
use nih_plug_webview::http::{Request, Response};

use crate::{Parameters, PluginMsg, GuiMsg};

pub struct ReactPlugEditor<P, PM>
where
    P: Parameters,
    PM: PluginMsg<P::ParamType> + 'static,
{
    editor: WebViewEditor,
    params: Arc<P>,
    editor_channel: Arc<(crossbeam_channel::Sender<PM>, crossbeam_channel::Receiver<PM>)>,
    dir: &'static Dir<'static>,
}

impl<P, PM> ReactPlugEditor<P, PM>
where
    P: Parameters,
    PM: PluginMsg<P::ParamType> + 'static,
{
    pub fn new<GM: GuiMsg<P::ParamType> + 'static>(
        params: Arc<P>,
        dir: &'static Dir,
        editor_channel: (crossbeam_channel::Sender<PM>, crossbeam_channel::Receiver<PM>),
    ) -> Self {
        let plugin_sender = editor_channel.0.clone();
        let plugin_receiver = editor_channel.1.clone();
        let protocol: &'static str = "reactplug";

        #[cfg(target_os = "windows")]
        let url_scheme = "http://reactplug.localhost";
        // TODO: Not tested on Linux / MacOS
        #[cfg(not(target_os = "windows"))]
        let url_scheme = "reactplug://localhost";

        let url = HTMLSource::URL(url_scheme.into());

        let editor_params = params.clone();

        // TODO: Size is not relative to current DPI
        let editor = WebViewEditor::new(url, (700, 500))
            // TODO: Run with hot-reload Vite server in development? Maybe?
            .with_custom_protocol(protocol.parse().unwrap(), |req| {
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
            .with_event_loop(move |ctx, setter, _window| {
                while let Ok(value) = ctx.next_event() {
                    if let Ok(message) = serde_json::from_value::<GM>(value) {
                        if message.is_init() {
                            editor_params.send_all(plugin_sender.clone());
                        }
                        message.is_param_update_and(|param| {
                            editor_params.set_param(&setter, param);
                        });
                    } else { nih_log!("Received invalid message from GUI!") }
                }
                while !plugin_receiver.is_empty() {
                    let message = serde_json::to_value(plugin_receiver.recv().unwrap())
                        .expect("Failed to serialize message to JSON");
                    ctx.send_json(message)
                        .expect("Failed to send message to GUI");
                }
            });

        Self {
            editor,
            params,
            editor_channel: Arc::new(editor_channel),
            dir,
        }
    }

    pub fn with_message_handler<GM: GuiMsg<P::ParamType> + 'static>(mut self, handler: impl Fn(GM) + 'static + Send + Sync) -> Self {
        let params = self.params.clone();
        let plugin_sender = self.editor_channel.0.clone();
        let plugin_receiver = self.editor_channel.1.clone();
        let handler = Arc::new(handler);

        self.editor = self.editor.with_event_loop(move |ctx, setter, _window| {
            while let Ok(value) = ctx.next_event() {
                if let Ok(message) = serde_json::from_value::<GM>(value) {
                    if message.is_init() {
                        params.send_all(plugin_sender.clone());
                    }
                    message.is_param_update_and(|param| {
                        params.set_param(&setter, param);
                    });
                    handler(message);
                } else { nih_log!("Received invalid message from GUI!") }
            }
            while !plugin_receiver.is_empty() {
                let message = serde_json::to_value(plugin_receiver.clone().recv().unwrap())
                    .expect("Failed to serialize message to JSON");
                ctx.send_json(message)
                    .expect("Failed to send message to GUI");
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
        #[cfg(target_os = "windows")]
        let url_scheme = format!("http://{}.localhost", protocol);
        // TODO: Not tested on Linux / MacOS
        #[cfg(not(target_os = "windows"))]
        let url_scheme = format!("{}://localhost", protocol);

        let dir = self.dir.clone();

        self.editor = self.editor.with_custom_protocol(protocol.parse().unwrap(), move |req| {
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
        });
        self
    }
}

impl<P, PM> Editor for ReactPlugEditor<P, PM>
where
    P: Parameters,
    PM: PluginMsg<P::ParamType> + 'static,
{
    fn spawn(&self, parent: ParentWindowHandle, context: Arc<dyn GuiContext>) -> Box<dyn Any + Send> {
        self.editor.spawn(parent, context)
    }

    fn size(&self) -> (u32, u32) {
        self.editor.size()
    }

    fn set_scale_factor(&self, factor: f32) -> bool {
        self.editor.set_scale_factor(factor)
    }

    fn param_value_changed(&self, id: &str, normalized_value: f32) {
        self.editor.param_value_changed(id, normalized_value)
    }

    fn param_modulation_changed(&self, id: &str, modulation_offset: f32) {
        self.editor.param_modulation_changed(id, modulation_offset)
    }

    fn param_values_changed(&self) {
        self.editor.param_values_changed()
    }
}

impl<P, PM> From<ReactPlugEditor<P, PM>> for Option<Box<dyn Editor + 'static>>
where
    P: Parameters,
    PM: PluginMsg<P::ParamType> + 'static,
{
    fn from(editor: ReactPlugEditor<P, PM>) -> Self {
        Some(Box::new(editor))
    }
}