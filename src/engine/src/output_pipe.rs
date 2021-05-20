use libloading::{Library, Symbol};
use lightoros_plugin_base::output::PluginOutputTrait;
use lightoros_plugin_base::transform::PluginTransformTrait;
use lightoros_plugin_base::*;
use std::thread::JoinHandle;

use super::utils::*;
use super::*;

pub(crate) struct OutputPipe {
    description: OutputPipeDescription,
    channel: Option<Receiver<Arc<TraitData>>>,
    _libs: Vec<Library>,
    output: Option<Box<dyn PluginOutputTrait>>,
    transformations: Option<Vec<Box<dyn PluginTransformTrait>>>,
    handle: Option<JoinHandle<()>>,
}

impl OutputPipe {
    pub fn create(
        description: OutputPipeDescription,
        channel: Receiver<Arc<TraitData>>,
        plugins_folder: &str,
    ) -> Result<OutputPipe, PluginError> {
        let name = &description.name;
        let mut libs: Vec<Library> = Vec::new();

        let output_plugin_info = match description.members.last() {
            Some(member) => member,
            None => return plugin_err!("Pipe {} is empty.", name),
        };
        let output_plugin_library = get_plugin(&output_plugin_info.kind, plugins_folder)?;
        let get_info: Symbol<fn() -> PluginInfo> =
            unsafe { output_plugin_library.get(b"info").unwrap() };
        let info = get_info();
        if info.kind != PluginKind::Output {
            return plugin_err!("Last member of a pipe ({}) must be an output plugin.", name);
        }

        libs.push(output_plugin_library); // prevent deallocation
        let lib = libs.last().unwrap();
        let mut output_plugin = match create_output_plugin!(lib, &output_plugin_info.config) {
            Ok(plugin) => plugin,
            Err(err) => {
                return plugin_err!(
                    "Error creating output plugin '{}': {}",
                    output_plugin_info.kind,
                    err
                )
            }
        };

        match output_plugin.init() {
            Ok(_) => (),
            Err(err) => {
                return plugin_err!(
                    "Failed to initialize output plugin '{}' for pipe '{}': {}",
                    info.name,
                    name,
                    err
                )
            }
        }

        let mut transformations = Vec::new();

        if description.members.len() > 1 {
            for i in 0..description.members.len() - 1 {
                let transform_plugin_info = &description.members[i];
                let transform_plugin_library =
                    get_plugin(&transform_plugin_info.kind, plugins_folder)?;
                libs.push(transform_plugin_library); // prevent deallocation
                let lib = libs.last().unwrap();
                let transform_plugin =
                    match create_transform_plugin!(lib, &transform_plugin_info.config) {
                        Ok(plugin) => plugin,
                        Err(err) => {
                            return plugin_err!(
                                "Error creating transform plugin '{}': {}",
                                transform_plugin_info.kind,
                                err
                            );
                        }
                    };
                //pipe.transformations.push(transform_plugin);
                transformations.push(transform_plugin);
            }
        }

        Ok(OutputPipe {
            description,
            channel: Some(channel),
            _libs: libs,
            output: Some(output_plugin),
            transformations: Some(transformations),
            handle: None,
        })
    }

    pub fn start(&mut self) -> Result<(), PluginError> {
        if self.handle.is_some() {
            return plugin_err!(
                "Cannot start output pipe '{}', because it's already running.",
                self.description.name
            );
        }

        let channel = self.channel.take().unwrap(); // TODO error handling
        let mut output = self.output.take().unwrap(); // TODO error handling
        let transformations = self.transformations.take().unwrap(); // TODO error handling
        let pipe_name = self.description.name.clone();

        let handle = std::thread::Builder::new()
            .name(pipe_name.clone())
            .spawn(move || loop {
                // wait for incoming data from engine thread
                let data_in: Arc<TraitData> = match channel.recv() {
                    Ok(data) => data,
                    Err(error) => {
                        eprintln!("[{}] Error receiving data: {}", pipe_name, error);
                        std::thread::sleep(Duration::from_millis(5000));
                        continue;
                    }
                };
                let mut data_ref: &TraitData = &data_in;

                if data_ref.rgb.len() == 0 {
                    // stop command from the engine
                    break;
                }

                let mut data_out: TraitData;
                // transform data if necessary
                for transformator in &transformations {
                    data_out = match transformator.transform(data_ref) {
                        Ok(data) => data,
                        Err(err) => {
                            eprintln!(
                                "[{}] Failed transforming data in '{}': {}. sleeping",
                                pipe_name, transformator, err
                            );
                            std::thread::sleep(Duration::from_millis(5000));
                            continue;
                        }
                    };
                    data_ref = &data_out;
                }
                let result = output.send(data_ref);
                if result.is_err() {
                    eprintln!(
                        "[{}] Failes sending data to '{}': {}. sleeping",
                        pipe_name,
                        output,
                        result.err().unwrap()
                    );
                    std::thread::sleep(Duration::from_millis(5000));
                }
            })
            .unwrap();

        self.handle = Some(handle);

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), PluginError> {
        if self.handle.is_none() {
            return plugin_err!(
                "Cannot stop output pipe '{}', because it's not running.",
                self.description.name
            );
        }

        self.handle
            .take()
            .expect("Called stop on non-running thread")
            .join()
            .expect("Could not join spawned thread");

        self.handle = None;

        Ok(())
    }
}

impl std::fmt::Display for OutputPipe {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let name = &self.description.name;
        match &self.handle {
            Some(handle) => {
                write!(
                    f,
                    "[ Name: {}, State: {}, ID: {:?} ]",
                    name,
                    "running",
                    handle.thread().id(),
                )
            }
            None => {
                write!(f, "[ Name: {}, State: {} ]", name, "stopped")
            }
        }
    }
}
