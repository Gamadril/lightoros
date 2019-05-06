use serde::{Deserialize};

use std::ops::Deref;
use std::io::prelude::*;

use lightoros_plugin::{PluginInfo, PluginOutputTrait, RgbData};

#[derive(Deserialize, Debug)]
struct Config {
    port: String,
}

struct SerialTpm2Output {
    config: Config,
}

impl SerialTpm2Output {
    fn new(config: &serde_json::Value) -> SerialTpm2Output {
        let cfg: serde_json::Value = config.clone();
        let config = match serde_json::from_value(cfg) {
            Ok(config) => config,
            Err(err) => {
                panic!("Error deserializing configuration: {}", err);
            }
        };

        SerialTpm2Output {
            config: config
        }
    }
}

impl PluginOutputTrait for SerialTpm2Output {
    fn send(&self, data: &RgbData) -> bool {
        let mut serial = match serial::open(&self.config.port) {
            Ok(s) => s,
            Err(err) => {
                eprintln!("Couldn't open serial port '{}': {}", self.config.port, err);
                return false;
            }
        };

        let rgb_data = data.deref();
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
            eprintln!(
                "Couldn't write to serial port '{}': {}",
                self.config.port,
                result.err().unwrap()
            );
            return false;
        }
        true
    }
}

#[no_mangle]
pub fn create(
    config: &serde_json::Value
) -> Box<PluginOutputTrait> {
    let plugin = SerialTpm2Output::new(config);
    Box::new(plugin)
}

#[no_mangle]
pub fn info() -> PluginInfo {
    PluginInfo {
        api_version: 1,
        name: "SerialOutputTPM2",
        filename: env!("CARGO_PKG_NAME"),
    }
}
