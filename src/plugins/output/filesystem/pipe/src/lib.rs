use std::ffi::CString;
use std::fs::File;
use std::io::prelude::*;
use std::io::Error;
use std::os::raw::{c_char, c_int, c_ushort};
use std::path::Path;

use std::ops::Deref;

use lightoros_plugin_base::*;
use lightoros_plugin_base::output::{PluginOutputTrait, CreateOutputPluginResult};
use serde::Deserialize;

const NAME: &str = "FilesystemPipeOutput";

#[derive(Deserialize, Debug)]
struct Config {
    path: String,
}

struct FilesystemPipeOutput {
    path: String,
}

impl std::fmt::Display for FilesystemPipeOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        NAME.fmt(f)
    }
}

impl FilesystemPipeOutput {
    fn create(config: &serde_json::Value) -> CreateOutputPluginResult {
        let config = plugin_config_or_return!(config.clone());

        let plugin = FilesystemPipeOutput {
            path: config.path.clone(),
        };
        Ok(Box::new(plugin))
    }
}

impl PluginOutputTrait for FilesystemPipeOutput {
    fn init(&mut self) -> PluginResult<()> {
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

    fn send(&self, data: &TraitData) -> PluginResult<()> {
        let path = Path::new(&self.path);
        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(err) => {
                return plugin_err!("Error opening file '{}': {}", &self.path, err);
            }
        };

        let rgb_data = data.rgb.deref();
        let out_size = rgb_data.len() * 3;

        let mut out: Vec<u8> = Vec::with_capacity(out_size);

        for rgb in rgb_data {
            out.push(rgb.g);
            out.push(rgb.r);
            out.push(rgb.b);
        }

        let result = file.write(&out);

        if result.is_err() {
            return plugin_err!(
                "Could not write to file '{}': {}",
                &self.path,
                result.err().unwrap()
            );
        }
        
        Ok(())
    }
}

#[no_mangle]
pub fn create(config: &serde_json::Value) -> CreateOutputPluginResult {
    FilesystemPipeOutput::create(config)
}

#[no_mangle]
pub fn info() -> PluginInfo {
    plugin_info!(1, NAME, PluginKind::Output)
}

extern "C" {
    pub fn mkfifo(pathname: *const c_char, mode: c_ushort) -> c_int;
}
