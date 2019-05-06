//use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::thread::ThreadId;

pub type RgbData = Vec<RGB>;

pub struct InputEvent {
    pub thread_id: ThreadId,
    pub data: Arc<RgbData>,
}

impl InputEvent {
    pub fn new(data: Arc<RgbData>) -> InputEvent {
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
    fn get(&mut self) -> Option<RgbData>;
}

pub trait PluginOutputTrait: Send {
    fn send(&self, data: &RgbData) -> bool;
}

pub trait PluginTransformTrait: Send {
    fn transform(&self, data: &RgbData) -> RgbData;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
