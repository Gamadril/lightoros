//use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::thread::ThreadId;
use std::fmt;

use std::collections::HashMap;

pub struct TraitData {
    pub rgb: Vec<RGB>,
    pub meta: HashMap<String, String>,
}

pub struct InputEvent {
    pub thread_id: ThreadId,
    pub data: Arc<TraitData>,
}

impl InputEvent {
    pub fn new(data: Arc<TraitData>) -> InputEvent {
        InputEvent {
            thread_id: thread::current().id(),
            data,
        }
    }
}

pub struct PluginInfo {
    pub api_version: u8,
    pub name: &'static str,
    pub filename: &'static str,
}

#[derive(Copy, Clone, Debug)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub trait PluginInputTrait: Send {
    fn init(&mut self) -> bool;
    fn get(&mut self) -> Option<TraitData>;
}

pub trait PluginOutputTrait: Send {
    fn send(&self, data: &TraitData) -> bool;
}

pub trait PluginTransformTrait: Send {
    fn transform(&self, data: &TraitData) -> TraitData;
}

impl fmt::Display for RGB {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "({},{},{})",
            self.r,
            self.g,
            self.b
        )
    }
}