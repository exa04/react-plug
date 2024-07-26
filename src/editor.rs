use std::sync::Arc;

use include_dir::Dir;
use nih_plug::nih_log;
use nih_plug_webview::{HTMLSource, WebViewEditor};
use nih_plug_webview::http::Response;

use crate::{Parameters, PluginMsg, GuiMsg};

pub fn create_editor<P, PM, GM, F>
(
    params: Arc<P>,
    protocol: Option<&'static str>,
    dir: &'static Dir,
    editor_channel: (crossbeam_channel::Sender<PM>, crossbeam_channel::Receiver<PM>),
    gui_message_handler: F,
) -> WebViewEditor
where
    P: Parameters,
    PM: PluginMsg<P::ParamType> + 'static,
    GM: GuiMsg<P::ParamType> + 'static,
    F: Fn(GM, crossbeam_channel::Sender<PM>) + 'static + Send + Sync,
{
    let plugin_sender = editor_channel.0.clone();
    let plugin_receiver = editor_channel.1.clone();
    let protocol: &'static str = protocol.unwrap_or("plugin");

    #[cfg(target_os = "windows")]
    let url_scheme = format!("http://{}.localhost", protocol);
    // TODO: Not tested on Linux / MacOS
    #[cfg(not(target_os = "windows"))]
    let url_scheme = format!("{}://localhost", protocol);

    let url = HTMLSource::URL(Box::leak(url_scheme.into_boxed_str()));

    // TODO: Size is not relative to current DPI
    WebViewEditor::new(url, (700, 500))
        // TODO: Run with hot-reload Vite server in development? Maybe?
        .with_custom_protocol(protocol.parse().unwrap(), move |req| {
            let path = req.uri().path();

            let path = if path == "/" {
                "index.html"
            } else {
                &path[1..]
            };

            let mime_type = mime_guess::from_path(path)
                .first_or_text_plain()
                .to_string();

            if let Some(file) = dir.clone().get_file(path) {
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
        }).with_event_loop(move |ctx, setter, _window| {
            while let Ok(value) = ctx.next_event() {
                if let Ok(message) = serde_json::from_value::<GM>(value) {
                    if message.is_init() {
                        params.send_all(plugin_sender.clone());
                    }
                    message.is_param_update_and(|param| {
                        params.set_param(&setter, param);
                    });
                    gui_message_handler(message, plugin_sender.clone());
                } else { nih_log!("Received invalid message from GUI!") }
            }
            while !plugin_receiver.is_empty() {
                let message = serde_json::to_value(plugin_receiver.recv().unwrap())
                    .expect("Failed to serialize message to JSON");
                ctx.send_json(message)
                    .expect("Failed to send message to GUI");
            }
        })
}