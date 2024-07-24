use std::sync::Arc;

use include_dir::Dir;
use nih_plug_webview::{HTMLSource, WebViewEditor};
use nih_plug_webview::http::Response;

use crate::{Parameters, ParamType, PluginToGuiMessage};

pub fn create_editor<M: PluginToGuiMessage<P> + 'static, P: ParamType>(
    _params: Arc<impl Parameters>,
    editor_channel: (crossbeam_channel::Sender<M>, crossbeam_channel::Receiver<M>),
    _protocol: Option<&'static str>,
    dir: &'static Dir
) -> WebViewEditor {
    let _plugin_sender = editor_channel.0.clone();
    let plugin_receiver = editor_channel.1.clone();
    let protocol: &'static str = None.unwrap_or("plugin".into());

    #[cfg(target_os = "windows")]
    let url_scheme = format!("http://{}.localhost", protocol);
    #[cfg(not(target_os = "windows"))]
    let url_scheme = format!("{}://localhost", protocol); // TODO: Not tested on Linux / MacOS

    let url = HTMLSource::URL(Box::leak(url_scheme.into_boxed_str()));

    // TODO: Size is not relative to current DPI
    // TODO: Leaking the URL scheme string, is this safe?
    WebViewEditor::new(url, (700, 500))
        // TODO: Run with hot-reload Vite server in development? Maybe?
        .with_custom_protocol("plugin".into(), move |req| {
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
            while !plugin_receiver.is_empty() {
                let message = serde_json::to_value(plugin_receiver.recv().unwrap())
                    .expect("Failed to serialize message to JSON");
                ctx.send_json(message)
                    .expect("Failed to send message to GUI");
            }
        })
        // while let Ok(value) = ctx.next_event() {
        //     if let Ok(action) = serde_json::from_value::<M>(value) {
        //         if action.is_init() {
        //             nih_log!("GUI Opened, sending initial data..");
        //             params.send_all(plugin_sender.clone());
        //         }
        //         action.is_update_param_and(|param| {
        //             params.set_param(&setter, param);
        //         });
        //     } else {
        //         panic!("Invalid action received from web UI.")
        //     }
        // }
}