use std::ffi::CString;
use std::fs::File;
use std::io::prelude::*;
use std::io::Error;
use std::os::raw::{c_char, c_int, c_ushort};
use std::path::Path;

use lightoros_plugin_base::input::{CreateInputPluginResult, PluginInputTrait};
use lightoros_plugin_base::*;

use serde::Deserialize;

const NAME: &str = "RemoteScreenPipeGrabberInput";

#[derive(Deserialize, Debug)]
struct Config {
    path: String,
}

struct RemoteScreenPipeGrabberInput {
    path: String,
}

impl std::fmt::Display for RemoteScreenPipeGrabberInput {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        NAME.fmt(f)
    }
}

impl RemoteScreenPipeGrabberInput {
    fn create(config: &serde_json::Value) -> CreateInputPluginResult {
        let config = plugin_config_or_return!(config.clone());

        let plugin = RemoteScreenPipeGrabberInput {
            path: config.path.clone(),
        };
        Ok(Box::new(plugin))
    }
}

impl PluginInputTrait for RemoteScreenPipeGrabberInput {
    fn init(&mut self) -> PluginResult<()> {
        // create pipe since android has no API for doing it
        let pipe_path = Path::new(&self.path);
        if pipe_path.exists() {
            let result = std::fs::remove_file(pipe_path);
            if result.is_err() {
                return plugin_err!(
                    "File '{}' already exists and could not be removed: {}",
                    &self.path,
                    result.err().unwrap()
                );
            }
        }

        let path: CString = CString::new(self.path.as_bytes()).unwrap();
        let result = unsafe { mkfifo(path.as_ptr(), 0o644) };
        if result != 0 {
            let err = Error::last_os_error();
            return plugin_err!("Failed to create named pipe '{}': {}", &self.path, err);
        }

        Ok(())
    }

    fn get(&mut self) -> PluginResult<TraitData> {
        let path = Path::new(&self.path);
        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(err) => {
                return plugin_err!("Error opening file '{}': {}", &self.path, err);
            }
        };

        // get incoming data size
        let mut data_in_len: Vec<u8> = vec![0; 4];
        match file.read_exact(&mut data_in_len) {
            Ok(_) => (),
            Err(err) => {
                return plugin_err!("Error reading file '{}': {}", &self.path, err);
            }
        };

        let in_data_size = get_u32!(data_in_len, 0) as usize;

        let mut data_in: Vec<u8> =  vec!(0; in_data_size);
        match file.read_exact(&mut data_in) {
            Ok(_) => (),
            Err(err) => {
                return plugin_err!("Error reading file '{}': {}", &self.path, err);
            }
        };

        // get width and height
        let width = get_u16!(data_in, 0);
        let height = get_u16!(data_in, 2);

        let size = (width * height) as usize;
        let mut data_out: Vec<RGB> = Vec::with_capacity(size * 3);

        let offset = 4;
        for i in 0..size {
            data_out.push(RGB {
                r: data_in[i * 3 + offset],
                g: data_in[i * 3 + 1 + offset],
                b: data_in[i * 3 + 2 + offset],
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
    RemoteScreenPipeGrabberInput::create(config)
}

#[no_mangle]
pub fn info() -> PluginInfo {
    plugin_info!(1, NAME, PluginKind::Input)
}

extern "C" {
    pub fn mkfifo(pathname: *const c_char, mode: c_ushort) -> c_int;
}
