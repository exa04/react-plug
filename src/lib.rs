pub mod editor;

/// Re-export of [serde]
pub use serde as sd;
/// Re-export of [ts_rs]
pub use ts_rs as ts;

pub mod prelude {
    pub use crate::editor::ReactPlugEditor;
    pub use react_plug_derive::*;

    /// Re-export of [serde]'s Serialize and Deserialize
    pub use crate::sd::{Deserialize, Serialize};
    /// Re-export of [ts_rs::TS]
    pub use crate::ts::TS;
}

use nih_plug::params::Params;
pub use react_plug_derive::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub type MessageChannel<M> = Arc<(crossbeam_channel::Sender<M>, crossbeam_channel::Receiver<M>)>;

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
