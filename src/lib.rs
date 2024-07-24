pub mod editor;

pub mod prelude {
    pub use react_plug_derive::rp_params;
    pub use crate::editor::create_editor;
}

// TODO: Add a macro for deriving this
pub trait PluginToGuiMessage<P: ParamType>:
serde::Serialize +
serde::de::DeserializeOwned +
Send
{
}

pub trait Parameters: nih_plug::params::Params {
    type ParamType: ParamType;
    type PluginToGuiMessage: PluginToGuiMessage<Self::ParamType>;

    fn send_all(&self, sender: crossbeam_channel::Sender<Self::PluginToGuiMessage>);
    fn set_param(&self, setter: &nih_plug::context::gui::ParamSetter, param: Self::ParamType);
}

pub trait ParamType:
    serde::Serialize +
    serde::Deserialize<'static>
{ }

pub trait RPPlugin: nih_plug::plugin::Plugin {
    type PluginToGuiMessage: PluginToGuiMessage<Self::ParamType>;
    type ParamType: ParamType;

    fn editor_channel(&self) -> (crossbeam_channel::Sender<Self::PluginToGuiMessage>, crossbeam_channel::Receiver<Self::PluginToGuiMessage>);
}