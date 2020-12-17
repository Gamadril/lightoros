use std::io::prelude::*;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};

use lightoros_plugin_base::input::{CreateInputPluginResult, PluginInputTrait};
use lightoros_plugin_base::*;

use serde::Deserialize;

const NAME: &str = "RemoteScreenTcpGrabberInput";

const CMD_SIZE: u8 = 0x20;
const CMD_NAME: u8 = 0x10;

#[derive(Deserialize, Debug)]
struct Config {
    client_name: String,
    address: String,
    port: u16,
    width: u16,
    height: u16,
}

struct RemoteScreenTcpGrabberInput {
    config: Config,
    address: SocketAddr,
    stream: Option<TcpStream>,
}

impl std::fmt::Display for RemoteScreenTcpGrabberInput {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        NAME.fmt(f)
    }
}

impl RemoteScreenTcpGrabberInput {
    fn create(config: &serde_json::Value) -> CreateInputPluginResult {
        let config = plugin_config_or_return!(config.clone());

        let addr = (config.address.as_str(), config.port);
        let address = match addr.to_socket_addrs() {
            Ok(mut addresses) => addresses.next().unwrap(),
            Err(err) => return plugin_err!("Error parsing IP address: {}", err),
        };

        let plugin = RemoteScreenTcpGrabberInput {
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
            let data_size = name_bytes.len() + 1; // +1 : CMD byte
            let mut message = Vec::<u8>::with_capacity(data_size + 2);
            message.push(((data_size & 0xFF00) >> 8) as u8);
            message.push((data_size & 0xFF) as u8);
            message.push(CMD_NAME);
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
            let data_size = 5; // u16 width, u16 height, u8 CMD
            let mut message = Vec::<u8>::with_capacity(data_size + 2);
            message.push(((data_size & 0xFF00) >> 8) as u8);
            message.push((data_size & 0xFF) as u8);
            message.push(CMD_SIZE);
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

impl PluginInputTrait for RemoteScreenTcpGrabberInput {
    fn init(&mut self) -> PluginResult<()> {
        self.stream = match TcpStream::connect(&self.address) {
            Ok(stream) => Some(stream),
            Err(_err) => return Ok(()), // will try it again in get()
        };

        self.send_name()?;
        self.send_size()?;

        Ok(())
    }

    fn get(&mut self) -> PluginResult<TraitData> {
        if self.stream.is_none() {
            let stream = match TcpStream::connect(&self.address) {
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

        // create the buffer with the expected data size. however there is no guarantee that the grabber respects it
        let size = (self.config.width * self.config.height) as usize;
        let mut data_out: Vec<RGB> = Vec::with_capacity(size * 3);

        let mut read_buffer = vec![0u8; data_out.capacity() + 8]; // u32: data size, u16: width, u16: height
        let bytes_read = match stream_ref.read_to_end(&mut read_buffer) {
            Ok(bytes_read) => bytes_read,
            Err(err) => {
                self.stream = None;
                return plugin_err!("Error reading from remote grabber server: {}", err);
            }
        };

        if bytes_read != read_buffer.len() {
            self.stream = None;
            return plugin_err!("Error reading from remote grabber server: Too few bytes received");
        }

        // get incoming data size
        let inc_data_size = get_u32!(read_buffer, 0);

        if bytes_read - 4 != inc_data_size as usize {
            self.stream = None;
            return plugin_err!("Error reading from remote grabber server: Too few bytes received");
        }

        // get width and height
        let width = get_u16!(read_buffer, 4);
        let height = get_u16!(read_buffer, 6);

        if  width != self.config.width || height != self.config.height {
            self.stream = None;
            return plugin_err!("Remote grabber is not using set resolution. Config: {}x{}, Grabber: {}x{}", self.config.width, self.config.height, width, height);
        }
        
        let offset = 8;
        for i in 0..size {
            data_out.push(RGB {
                r: read_buffer[i * 3 + offset],
                g: read_buffer[i * 3 + 1 + offset],
                b: read_buffer[i * 3 + 2 + offset],
            });
        }

        let result = plugin_data!(data_out, {
            "width" => width,
            "height" => height,
        });
        Ok(result)
    }
}

#[no_mangle]
pub fn create(config: &serde_json::Value) -> CreateInputPluginResult {
    RemoteScreenTcpGrabberInput::create(config)
}

#[no_mangle]
pub fn info() -> PluginInfo {
    plugin_info!(1, NAME, PluginKind::Input)
}
