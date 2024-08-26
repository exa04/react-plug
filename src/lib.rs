pub mod editor;

pub mod prelude {
    pub use crate::editor::ReactPlugEditor;
    pub use react_plug_derive::*;
}

pub use react_plug_derive::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub type MessageChannel<M> = Arc<(crossbeam_channel::Sender<M>, crossbeam_channel::Receiver<M>)>;

pub trait ParamType: serde::Serialize + serde::Deserialize<'static> + ts_rs::TS {}

#[derive(Serialize, Deserialize, Debug)]
pub struct ParamChange {
    pub id: String,
    pub value: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PluginMessage<M> {
    ParamChange(ParamChange),
    Message(M),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GuiMessage<M> {
    ParamChange(ParamChange),
    Init,
    Message(M),
}
