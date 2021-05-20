use std::sync::Arc;

use lightoros_plugin_base::*;

#[derive(serde::Deserialize)]
pub(crate) struct InputPipeDescription {
    pub name: String,
    pub priority: u8,
    pub members: Vec<PluginDescription>,
}

#[derive(serde::Deserialize)]
pub(crate) struct OutputPipeDescription {
    pub name: String,
    pub members: Vec<PluginDescription>,
}

#[derive(serde::Deserialize)]
pub(crate) struct PluginDescription {
    pub kind: String,
    pub config: serde_json::Value,
}

#[derive(serde::Deserialize)]
pub(crate) struct Config {
    pub description: String,
    pub max_input_inactivity_period: u64,
    pub input: Vec<InputPipeDescription>,
    pub output: Vec<OutputPipeDescription>,
}

pub(crate) struct InputEvent {
    pub priority: u8,
    pub data: Arc<TraitData>,
}

impl InputEvent {
    pub fn create(data: Arc<TraitData>, priority: u8) -> InputEvent {
        InputEvent {
            data,
            priority,
        }
    }
}
