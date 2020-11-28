use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::thread;
use std::time::Duration;
use std::collections::HashMap;

use lightoros_plugin_base::*;
use lightoros_plugin_base::input::{PluginInputTrait, CreateInputPluginResult};

use serde::Deserialize;

const NAME: &str = "FilesystemInputTPM2";

#[derive(Deserialize, Debug)]
struct Config {
    files: Vec<String>,
    repeat: bool,
    delay_frame: u64,
    delay_file: u64,
}

struct FilesystemInput {
    config: Config,
    frame_offset: usize,
    file_index: usize
}

impl FilesystemInput {
    fn create(config: &serde_json::Value) -> CreateInputPluginResult {
        let config = plugin_config_or_return!(config.clone());

        let plugin = FilesystemInput {
            config,
            frame_offset: 0,
            file_index: 0,
        };

        Ok(Box::new(plugin))
    }
}

impl std::fmt::Display for FilesystemInput {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        NAME.fmt(f)
    }
}

impl PluginInputTrait for FilesystemInput {
    fn init(&mut self) -> PluginResult<()> {
        Ok(())
    }

    fn get(&mut self) -> PluginResult<TraitData> {
        if self.config.files.len() == 0 {
            return plugin_err!("File list empty");
        }

        if self.file_index == self.config.files.len() {
            self.file_index = 0;
            thread::sleep(Duration::from_millis(self.config.delay_file));
        }

        let ref current_file = self.config.files[self.file_index];
        let path = Path::new(current_file);
        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(error) => {
                return plugin_err!("Error opening file '{}': {}", current_file, error);
            }
        };

        let mut data_in: Vec<u8> = Vec::new();
        match file.read_to_end(&mut data_in) {
            Err(error) => {
                return plugin_err!("Error reading file '{}': {}", current_file, error);
            }
            Ok(_) => {}
        }

        let mut index = self.frame_offset;
        if data_in[index] != 0xC9 || data_in[index + 1] != 0xDA {
            return plugin_err!("Error: '{}' is not a tpm2 file - start marker missing.", current_file);
        }

        let size: u16 = (data_in[index + 2] as u16) << 8 | data_in[index + 3] as u16;
        let mut data_out: Vec<RGB> = Vec::with_capacity(size as usize / 3);
        index = index + 4;
        for _ in 0..(size / 3) {
            data_out.push(RGB {
                r: data_in[index],
                g: data_in[index + 1],
                b: data_in[index + 2],
            });
            index = index + 3;
        }

        if data_in[index] != 0x36 {
            return plugin_err!("Error: '{}' is not a tpm2 file - frame end marker missing.", current_file);
        }

        index += 1;

        if index == data_in.len() {
            self.frame_offset = 0;
            self.file_index = self.file_index + 1;
        } else {
            self.frame_offset = index;
            thread::sleep(Duration::from_millis(self.config.delay_frame));
        }

        let result: TraitData = TraitData {
            rgb: data_out,
            meta: HashMap::new(),
        }; 
        Ok(result)
    }
}

#[no_mangle]
pub fn create(config: &serde_json::Value) -> CreateInputPluginResult {
    FilesystemInput::create(config)
}

#[no_mangle]
pub fn info() -> PluginInfo {
    plugin_info!(1, NAME, PluginKind::Input)
}
