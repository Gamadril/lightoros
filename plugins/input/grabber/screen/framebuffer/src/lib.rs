use std::time::Duration;
use std::thread;

use lightoros_plugin::{PluginInfo, PluginInputTrait, RGB};

use serde::{Deserialize};

#[derive(Deserialize, Debug)]
struct Config {
}

struct ScreenGrabberInput {
    config: Config
}

impl ScreenGrabberInput {
    fn new(config: &serde_json::Value) -> ScreenGrabberInput {
        let cfg = config.clone();
        let config = match serde_json::from_value(cfg) {
            Ok(config) => config,
            Err(err) => {
                panic!("Error deserializing configuration: {}", err);
            }
        };

        ScreenGrabberInput {
            config: config
        }
    }
}

impl PluginInputTrait for ScreenGrabberInput {
    fn init(&mut self) -> bool {
        
        true
    }
    
    fn get(&mut self) -> Option<Vec<RGB>> {
        let mut out: Vec<RGB> = Vec::with_capacity(24);

        Some(out)
    }
}

#[no_mangle]
pub fn create(config: &serde_json::Value) -> Box<dyn PluginInputTrait> {
    let plugin = ScreenGrabberInput::new(config);
    Box::new(plugin)
}

#[no_mangle]
pub fn info() -> PluginInfo {
    PluginInfo {
        api_version: 1,
        name: "ScreenInput",
        filename: env!("CARGO_PKG_NAME"),
    }
}
