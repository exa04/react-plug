pub mod editor;

pub mod prelude {
    pub use react_plug_derive::*;
    pub use crate::editor::create_editor;
    pub use crate::RPPlugin;
}

pub use react_plug_derive::*;

pub trait PluginMsg<P: ParamType>:
serde::Serialize +
serde::de::DeserializeOwned +
Send +
ts_rs::TS
{
    fn parameter_change(param_type: P) -> Self;
}

pub trait GuiMsg<P: ParamType>:
serde::Serialize +
serde::de::DeserializeOwned +
Send +
ts_rs::TS
{
    fn is_init(&self) -> bool;
    fn is_param_update_and<F: FnOnce(&P)>(&self, action: F);
}

pub trait Parameters: nih_plug::params::Params {
    type ParamType: ParamType;

    fn send_all<PM: PluginMsg<Self::ParamType> + 'static>(&self, sender: crossbeam_channel::Sender<PM>);
    fn set_param(&self, setter: &nih_plug::context::gui::ParamSetter, param: &Self::ParamType);
}

pub trait ParamType:
    serde::Serialize + 
    serde::Deserialize<'static> +
    ts_rs::TS
{ }

pub trait RPPlugin: nih_plug::plugin::Plugin {
    type Parameters: Parameters;
    type PluginMsg: PluginMsg<<Self::Parameters as Parameters>::ParamType>;
    type GuiMsg: GuiMsg<<Self::Parameters as Parameters>::ParamType>;

    fn editor_channel(&self) -> (
        crossbeam_channel::Sender<Self::PluginMsg>,
        crossbeam_channel::Receiver<Self::PluginMsg>
    );
}