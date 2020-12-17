use lightoros_plugin_base::output::PluginOutputTrait;
use lightoros_plugin_base::transform::PluginTransformTrait;
use lightoros_plugin_base::input::PluginInputTrait;
use std::thread::JoinHandle;
use std::thread::ThreadId;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;

use lightoros_plugin_base::*;

#[derive(serde::Deserialize)]
pub(crate) struct InputPipeDescription {
    pub name: String,
    pub priority: u8,
    pub members: Vec<PluginDescription>,
}

#[derive(serde::Deserialize)]
pub(crate) struct OutputPipeDescription {
    pub name: String,
    pub members: Vec<PluginDescription>,
}

#[derive(serde::Deserialize)]
pub(crate) struct PluginDescription {
    pub kind: String,
    pub config: serde_json::Value,
}

#[derive(serde::Deserialize)]
pub(crate) struct Config {
    pub max_input_inactivity_period: u64,
    pub input: Vec<InputPipeDescription>,
    pub output: Vec<OutputPipeDescription>,
}

pub(crate) struct InputEvent {
    pub thread_id: ThreadId,
    pub data: Arc<TraitData>,
}

impl InputEvent {
    pub fn new(data: Arc<TraitData>) -> InputEvent {
        InputEvent {
            thread_id: std::thread::current().id(),
            data,
        }
    }
}

pub(crate) struct InputThread {
    pub priority: u8,
    pub handle: JoinHandle<()>,
    pub id: ThreadId,
}

pub(crate) struct OutputThread {
    pub handle: JoinHandle<()>,
    pub id: ThreadId,
}

pub(crate) struct InputPipe {
    pub priority: u8,
    //handle: Option<JoinHandle<()>>,
    //id: ThreadId,
    pub name: String,
    pub channel_out: Sender<InputEvent>,
    pub input: Box<dyn PluginInputTrait>,
    pub transformations: Vec<Box<dyn PluginTransformTrait>>,
}

pub(crate) struct OutputPipe {
    //handle: Option<JoinHandle<()>>,
    //id: ThreadId,
    pub channel_in: Receiver<Arc<TraitData>>,
    pub name: String,
    pub output: Box<dyn PluginOutputTrait>,
    pub transformations: Vec<Box<dyn PluginTransformTrait>>,
}

impl InputThread {
    pub fn new(priority: u8, handle: JoinHandle<()>) -> InputThread {
        let id = handle.thread().id();
        InputThread {
            priority,
            handle,
            id,
        }
    }
}

impl OutputThread {
    pub fn new(handle: JoinHandle<()>) -> OutputThread {
        let id = handle.thread().id();
        OutputThread { handle, id }
    }
}

impl InputPipe {
    pub fn new(input: Box<dyn PluginInputTrait>, channel: Sender<InputEvent>) -> InputPipe {
        InputPipe {
            priority: 0,
            //handle: None,
            name: String::new(),
            input,
            channel_out: channel,
            transformations: Vec::new(),
        }
    }
}

impl OutputPipe {
    pub fn new(output: Box<dyn PluginOutputTrait>, channel: Receiver<Arc<TraitData>>) -> OutputPipe {
        OutputPipe {
            //handle: None,
            name: String::new(),
            output,
            channel_in: channel,
            transformations: Vec::new(),
        }
    }
}

impl std::fmt::Display for InputThread {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[ Name: {}, ID: {:?} ]",
            self.handle.thread().name().unwrap_or(""),
            self.id
        )
    }
}

impl std::fmt::Display for OutputThread {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[ Name: {}, ID: {:?} ]",
            self.handle.thread().name().unwrap_or(""),
            self.id
        )
    }
}

macro_rules! create_input_plugin {
    ($lib:expr, $config:expr) => {{
        let func: Symbol<fn(&serde_json::Value) -> lightoros_plugin_base::input::CreateInputPluginResult> =
            unsafe { $lib.get(b"create").unwrap() };
        func($config)
    }};
}

macro_rules! create_transform_plugin {
    ($lib:expr, $config:expr) => {{
        let func: Symbol<fn(&serde_json::Value) -> lightoros_plugin_base::transform::CreateTransformPluginResult> =
            unsafe { $lib.get(b"create").unwrap() };
        func($config)
    }};
}

macro_rules! create_output_plugin {
    ($lib:expr, $config:expr) => {{
        let func: Symbol<fn(&serde_json::Value) -> lightoros_plugin_base::output::CreateOutputPluginResult> =
            unsafe { $lib.get(b"create").unwrap() };
        func($config)
    }};
}