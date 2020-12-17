use std::io::prelude::*;
use std::os::unix::net::UnixStream;
use uds::*;

use lightoros_plugin_base::input::{CreateInputPluginResult, PluginInputTrait};
use lightoros_plugin_base::*;

use serde::Deserialize;

const NAME: &str = "RemoteScreenUnixSocketGrabberInput";

#[derive(Deserialize, Debug)]
struct Config {
    client_name: String,
    path: String,
    width: u16,
    height: u16,
}

struct RemoteScreenUnixSocketGrabberInput {
    config: Config,
    address: uds::UnixSocketAddr,
    stream: Option<UnixStream>,
}

impl std::fmt::Display for RemoteScreenUnixSocketGrabberInput {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        NAME.fmt(f)
    }
}

impl RemoteScreenUnixSocketGrabberInput {
    fn create(config: &serde_json::Value) -> CreateInputPluginResult {
        let config = plugin_config_or_return!(config.clone());

        let address = match uds::UnixSocketAddr::from_abstract(&config.path) {
            Ok(address) => address,
            Err(err) => return plugin_err!("Unable to parse path as abstract socket address {}: {}", &config.path, err)
        };

        let plugin = RemoteScreenUnixSocketGrabberInput {
            config,
            address,
            stream: None,
        };
        Ok(Box::new(plugin))
    }

    fn send_name(&mut self) -> PluginResult<()> {
        if self.stream.is_some() {
            let stream_ref = &self.stream.as_ref();
            let name_bytes = self.config.client_name.as_bytes();
            let msg_size = name_bytes.len() + 1;
            let mut message = Vec::<u8>::with_capacity(msg_size + 2);
            message.push(((msg_size & 0xFF00) >> 8) as u8);
            message.push((msg_size & 0xFF) as u8);
            message.push(0x10);
            for i in 0..name_bytes.len() {
                message.push(name_bytes[i]);
            }

            match stream_ref.unwrap().write_all(&message) {
                Ok(_) => return Ok(()),
                Err(err) => return plugin_err!("Error sending data to remote grabber: {}", err),
            };
        }
        Ok(())
    }

    fn send_size(&mut self) -> PluginResult<()> {
        if self.stream.is_some() {
            let stream_ref = &self.stream.as_ref();
            let msg_size = 5;
            let mut message = Vec::<u8>::with_capacity(msg_size + 2);
            message.push(((msg_size & 0xFF00) >> 8) as u8);
            message.push((msg_size & 0xFF) as u8);
            message.push(0x20);
            message.push(((self.config.width & 0xFF00) >> 8) as u8);
            message.push((self.config.width & 0xFF) as u8);
            message.push(((self.config.height & 0xFF00) >> 8) as u8);
            message.push((self.config.height & 0xFF) as u8);
            match stream_ref.unwrap().write_all(&message) {
                Ok(_) => return Ok(()),
                Err(err) => return plugin_err!("Error sending data to remote grabber: {}", err),
            };
        }
        Ok(())
    }
}

impl PluginInputTrait for RemoteScreenUnixSocketGrabberInput {
    fn init(&mut self) -> PluginResult<()> {
        self.stream = match UnixStream::connect_to_unix_addr(&self.address) {
            Ok(stream) => Some(stream),
            Err(_err) => return Ok(()), // will try it again in get()
        };

        self.send_name()?;
        self.send_size()?;

        Ok(())
    }

    fn get(&mut self) -> PluginResult<TraitData> {
        if self.stream.is_none() {
            let stream = match UnixStream::connect_to_unix_addr(&self.address) {
                Ok(stream) => stream,
                Err(err) => {
                    return plugin_err!("Error connecting to remote grabber server: {}", err)
                }
            };
            self.stream = Some(stream);
            self.send_name()?;
            self.send_size()?;
        }

        let mut stream_ref = self.stream.as_ref().unwrap();

        let size = (self.config.width * self.config.height) as usize;
        let data_size: usize = size * 3;
        let mut data_out: Vec<RGB> = Vec::with_capacity(size);

        let mut read_buffer = vec![0u8; data_size];
        let bytes_read = match stream_ref.read(&mut read_buffer) {
            Ok(size) => size,
            Err(err) => {
                self.stream = None;
                return plugin_err!("Error reading from remote grabber server: {}", err);
            }
        };

        if bytes_read != data_size {
            self.stream = None;
            return plugin_err!("Error reading from remote grabber server: Too few bytes received");
        }
        for i in 0..size {
            data_out.push(RGB {
                r: read_buffer[i * 3],
                g: read_buffer[i * 3 + 1],
                b: read_buffer[i * 3 + 2],
            });
        }

        let result = plugin_data!(data_out, {
            "width" => self.config.width,
            "height" => self.config.height,
        });
        Ok(result)
    }
}

#[no_mangle]
pub fn create(config: &serde_json::Value) -> CreateInputPluginResult {
    RemoteScreenUnixSocketGrabberInput::create(config)
}

#[no_mangle]
pub fn info() -> PluginInfo {
    plugin_info!(1, NAME, PluginKind::Input)
}
