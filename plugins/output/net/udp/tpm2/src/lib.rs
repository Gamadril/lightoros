use serde::Deserialize;

use std::net;
use std::ops::Deref;

use lightoros_plugin::{PluginInfo, PluginOutputTrait, TraitData};

#[derive(Deserialize, Debug)]
struct Config {
    ip: String,
    port: u16,
}

struct NetUdpTpm2Output {
    address: net::SocketAddr,
}

impl NetUdpTpm2Output {
    fn new(config: &serde_json::Value) -> NetUdpTpm2Output {
        let cfg: serde_json::Value = config.clone();
        let config: Config = match serde_json::from_value(cfg) {
            Ok(config) => config,
            Err(err) => {
                panic!("Error deserializing configuration: {}", err);
            }
        };

        let addr_str = format!("{}:{}", config.ip, config.port);
        let address = match addr_str.parse::<net::SocketAddr>() {
            Ok(address) => address,
            Err(err) => {
                panic!("Error parsing IP address: {}", err);
            }
        };

        NetUdpTpm2Output { address: address }
    }
}

impl PluginOutputTrait for NetUdpTpm2Output {
    fn send(&self, data: &TraitData) -> bool {
        let socket = match net::UdpSocket::bind("0.0.0.0:0") {
            Ok(socket) => socket,
            Err(err) => {
                eprintln!("Error creating UDP Socket: {}", err);
                return false;
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

        let result = socket.send_to(&out, self.address);
        if result.is_err() {
            eprintln!("Error sending UDP message: {}", result.err().unwrap());
            return false;
        }
        true
    }
}

#[no_mangle]
pub fn create(config: &serde_json::Value) -> Box<dyn PluginOutputTrait> {
    let plugin = NetUdpTpm2Output::new(config);
    Box::new(plugin)
}

#[no_mangle]
pub fn info() -> PluginInfo {
    PluginInfo {
        api_version: 1,
        name: "NetUdpOutputTPM2",
        filename: env!("CARGO_PKG_NAME"),
    }
}
