use serde::Deserialize;

use std::net;
use std::ops::Deref;

use lightoros_plugin_base::*;
use lightoros_plugin_base::output::{PluginOutputTrait, CreateOutputPluginResult};

const NAME: &str = "NetUdpOutputTPM2";

#[derive(Deserialize, Debug)]
struct Config {
    ip: String,
    port: u16,
    max_packet_length: u16,
}

struct NetUdpTpm2Output {
    address: net::SocketAddr,
    max_packet_length: u16
}

impl std::fmt::Display for NetUdpTpm2Output {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        NAME.fmt(f)
    }
}

impl NetUdpTpm2Output {
    fn create(config: &serde_json::Value) -> CreateOutputPluginResult {
        let config = plugin_config_or_return!(config.clone());
                
        let addr_str = format!("{}:{}", config.ip, config.port);
        let address = match addr_str.parse::<net::SocketAddr>() {
            Ok(address) => address,
            Err(err) => return plugin_err!("Error parsing IP address: {}", err),
        };

        // FIXME max_packet_length must be at least long enough for a minimal TPM2 packet
        let plugin = NetUdpTpm2Output {
            address: address,
            max_packet_length: config.max_packet_length,
        };
        Ok(Box::new(plugin))
    }
}

impl PluginOutputTrait for NetUdpTpm2Output {
    fn init(&mut self) -> PluginResult<()> {
        Ok(())
    }
    
    fn send(&self, data: &TraitData) -> PluginResult<()> {
        let socket = match net::UdpSocket::bind("0.0.0.0:0") {
            Ok(socket) => socket,
            Err(err) => return plugin_err!("Error creating UDP Socket: {}", err)
        };

        let rgb_data = data.rgb.deref();
        let out_size = rgb_data.len() * 3;

        if out_size + 5 > self.max_packet_length as usize {
            // split into chunks, use TMP2.net protocol
            // FIXME length of all packets should be ideally the same to make life(memory allocation) on the receiver side easier
            let max_frame_size: usize = self.max_packet_length as usize - 7;
            let rgb_per_frame: usize = max_frame_size / 3;

            let mut number_of_packets: usize = rgb_data.len() / rgb_per_frame;
            if rgb_data.len() % rgb_per_frame != 0 {
                number_of_packets += 1
            }

            if number_of_packets > 255 {
                return plugin_err!("Error sending TPM2.net message: the number of total packets needed ({}) exceeds the allowed limit of 255", number_of_packets);
            }

            for i in 0..number_of_packets {
                let rgb_frame = if i == number_of_packets - 1 { rgb_data.len() - i * rgb_per_frame } else {  rgb_per_frame };
                let frame_size = rgb_frame * 3;
                
                let mut out: Vec<u8> = Vec::with_capacity(frame_size + 5);
                out.push(0x9C);
                out.push(0xDA);
                out.push(((frame_size >> 8) & 0xFF) as u8);
                out.push((frame_size & 0xFF) as u8);
                out.push((i + 1) as u8);
                out.push(number_of_packets as u8);

                let idx_start = i * rgb_per_frame;
                let idx_end = idx_start + rgb_frame;
                for idx in idx_start..idx_end {
                    let rgb = rgb_data[idx];
                    out.push(rgb.g);
                    out.push(rgb.r);
                    out.push(rgb.b);
                }
    
                out.push(0x36);
                let result = socket.send_to(&out, self.address);
                if result.is_err() {
                    return plugin_err!("Error sending UDP message: {}", result.err().unwrap());
                }        
            }
        } else {
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
                return plugin_err!("Error sending UDP message: {}", result.err().unwrap());
            }
        }

        Ok(())
    }
}

#[no_mangle]
pub fn create(config: &serde_json::Value) -> CreateOutputPluginResult {
    NetUdpTpm2Output::create(config)
}

#[no_mangle]
pub fn info() -> PluginInfo {
    plugin_info!(1, NAME, PluginKind::Output)
}
