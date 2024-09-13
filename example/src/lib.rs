mod params;

use crate::params::*;

use nih_plug::prelude::*;
use react_plug::prelude::*;

use include_dir::{include_dir, Dir};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ts_rs::TS;

pub struct ExamplePlugin {
    params: Arc<ExampleParams>,
}

impl Default for ExamplePlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(ExampleParams::default()),
        }
    }
}

static EDITOR_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/gui/dist");

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../gui/src/bindings/GuiMessage.ts")]
enum GuiMessage {
    Ping,
    Foo(String),
    Bar { a: f32, b: f32 },
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../gui/src/bindings/PluginMessage.ts")]
enum PluginMessage {
    Pong,
    Oof(String),
    Baz { a: f32, b: f32 },
}

impl Plugin for ExamplePlugin {
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            let gain = self.params.gain.smoothed.next();
            for sample in channel_samples {
                *sample *= gain;
            }
        }

        ProcessStatus::Normal
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        ReactPlugEditor::<PluginMessage, GuiMessage>::new(
            self.params.clone(),
            &EDITOR_DIR,
            (1000, 800),
        )
        .with_background_color((0, 0, 0, 255))
        .with_developer_mode(true)
        .with_message_handler(|gui_message, send| match gui_message {
            GuiMessage::Ping => {
                let _ = send(PluginMessage::Pong);
            }
            GuiMessage::Foo(s) => {
                let _ = send(PluginMessage::Oof(s.chars().rev().collect::<String>()));
            }
            GuiMessage::Bar { a, b } => {
                let _ = send(PluginMessage::Baz { a: a / b, b: a * b });
            }
        })
        .into()
    }

    const NAME: &'static str = "Example Plugin";
    const VENDOR: &'static str = "223230";
    const URL: &'static str = "https://github.com/223230";
    const EMAIL: &'static str = "223230@pm.me";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;
}

impl Vst3Plugin for ExamplePlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"AUDIOPLUGIN\0\0\0\0\0";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_vst3!(ExamplePlugin);
