use serde::Deserialize;

use std::io::prelude::*;
use std::ops::Deref;

use lightoros_plugin_base::*;
use lightoros_plugin_base::output::{PluginOutputTrait, CreateOutputPluginResult};

const NAME: &str = "SerialOutputTPM2";

#[derive(Deserialize, Debug)]
struct Config {
    port: String,
}

struct SerialTpm2Output {
    config: Config,
}

impl std::fmt::Display for SerialTpm2Output {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        NAME.fmt(f)
    }
}

impl SerialTpm2Output {
    fn create(config: &serde_json::Value) -> CreateOutputPluginResult {
        let config = plugin_config_or_return!(config.clone());

        let plugin = SerialTpm2Output { config };

        Ok(Box::new(plugin))
    }
}

impl PluginOutputTrait for SerialTpm2Output {
    fn init(&mut self) -> PluginResult<()> {
        Ok(())
    }

    fn send(&self, data: &TraitData) -> PluginResult<()> {
        let mut serial = match serial::open(&self.config.port) {
            Ok(serial) => serial,
            Err(err) => {
                return plugin_err!(
                    "Could not open serial port '{}': {}",
                    self.config.port,
                    err
                )
            }
        };

        let rgb_data = data.rgb.deref();
        let out_size = rgb_data.len() * 3;

        let mut out: Vec<u8> = Vec::with_capacity(out_size + 5);
        out.push(0xC9);
        out.push(0xDA);
        out.push(((out_size >> 8) & 0xFF) as u8);
        out.push((out_size & 0xFF) as u8);

        for rgb in rgb_data {
            out.push(rgb.g);
            out.push(rgb.r);
            out.push(rgb.b);
        }
        out.push(0x36);

        let result = serial.write(&out);
        if result.is_err() {
            return plugin_err!(
                "Could not write to serial port '{}': {}",
                self.config.port,
                result.err().unwrap()
            );
        }
        Ok(())
    }
}

#[no_mangle]
pub fn create(config: &serde_json::Value) -> CreateOutputPluginResult {
    SerialTpm2Output::create(config)
}

#[no_mangle]
pub fn info() -> PluginInfo {
    plugin_info!(1, NAME, PluginKind::Output)
}
