use libloading::{Library, Symbol};
use lightoros_plugin_base::input::PluginInputTrait;
use lightoros_plugin_base::transform::PluginTransformTrait;
use lightoros_plugin_base::*;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;

use super::data_types::*;
use super::utils::*;
use super::*;

pub(crate) struct InputPipe {
    description: InputPipeDescription,
    channel: Option<Sender<InputEvent>>,
    _libs: Vec<Library>,
    input: Option<Box<dyn PluginInputTrait>>,
    transformations: Option<Vec<Box<dyn PluginTransformTrait>>>,
    should_stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl InputPipe {
    pub fn create(
        description: InputPipeDescription,
        channel: Sender<InputEvent>,
        plugins_folder: &str,
    ) -> Result<InputPipe, PluginError> {
        let name = &description.name;
        let mut libs: Vec<Library> = Vec::new();

        let input_plugin_info = match description.members.first() {
            Some(member) => member,
            None => return plugin_err!("Pipe {} is empty.", name),
        };

        let input_plugin_library = get_plugin(&input_plugin_info.kind, plugins_folder)?;
        let get_info: Symbol<fn() -> PluginInfo> =
            unsafe { input_plugin_library.get(b"info").unwrap() };
        let info = get_info();
        if info.kind != PluginKind::Input {
            return plugin_err!("First member of a pipe ({}) must be an input plugin.", name);
        }

        libs.push(input_plugin_library); // prevent deallocation
        let lib = libs.last().unwrap();
        let mut input_plugin = match create_input_plugin!(lib, &input_plugin_info.config) {
            Ok(plugin) => plugin,
            Err(err) => {
                return plugin_err!(
                    "Error creating input plugin '{}': {}",
                    input_plugin_info.kind,
                    err
                );
            }
        };

        match input_plugin.init() {
            Ok(_) => (),
            Err(err) => {
                return plugin_err!(
                    "Failed to initialize input plugin '{}' for pipe '{}': {}",
                    info.name,
                    name,
                    err
                );
            }
        }

        let mut transformations = Vec::new();

        if description.members.len() > 1 {
            for i in 1..description.members.len() {
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
                            )
                        }
                    };
                transformations.push(transform_plugin);
            }
        }

        Ok(InputPipe {
            description,
            channel: Some(channel),
            _libs: libs,
            input: Some(input_plugin),
            transformations: Some(transformations),
            should_stop: Arc::new(AtomicBool::new(false)),
            handle: None,
        })
    }

    pub fn start(&mut self) -> Result<(), PluginError> {
        if self.handle.is_some() {
            return plugin_err!(
                "Cannot start input pipe '{}', because it's already running.",
                self.description.name
            );
        }

        let name = self.description.name.clone();
        let priority = self.description.priority;
        let should_stop = self.should_stop.clone();
        let channel = self.channel.take().unwrap(); // TODO error handling
        let mut input = self.input.take().unwrap(); // TODO error handling
        let transformations = self.transformations.take().unwrap(); // TODO error handling

        should_stop.store(false, Ordering::SeqCst);

        // run pipe in a thread
        let handle = std::thread::Builder::new()
            .name(name.clone())
            .spawn(move || {
                while should_stop.load(Ordering::SeqCst) != true {
                    // get data from input
                    let mut data_in = match input.get() {
                        Ok(data) => data,
                        Err(err) => {
                            eprintln!("[{}] Failed getting data from '{}'. sleeping", name, err);
                            std::thread::sleep(Duration::from_millis(5000));
                            continue;
                        }
                    };
                    // transform data if necessary
                    for transformator in &transformations {
                        data_in = match transformator.transform(&data_in) {
                            Ok(data) => data,
                            Err(err) => {
                                eprintln!(
                                    "[{}] Failed transforming data in '{}': {}. sleeping",
                                    name, transformator, err
                                );
                                std::thread::sleep(Duration::from_millis(5000));
                                continue;
                            }
                        };
                    }
                    // send data to the engine thread
                    let event = InputEvent::create(Arc::new(data_in), priority);
                    if channel.send(event).is_err() {
                        // should only happen when the main engine stopped
                        eprintln!("[{}] Failed sending data to engine.", name);
                        break;
                    }
                }
            })
            .unwrap();

        self.handle = Some(handle);

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), PluginError> {
        if self.handle.is_none() {
            return plugin_err!(
                "Cannot stop input pipe '{}', because it's not running.",
                self.description.name
            );
        }

        self.should_stop.store(true, Ordering::SeqCst);

        self.handle
            .take()
            .expect("Called stop on non-running thread")
            .join()
            .expect("Could not join spawned thread");

        self.handle = None;

        Ok(())
    }
}

impl std::fmt::Display for InputPipe {
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
