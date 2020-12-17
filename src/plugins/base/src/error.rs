use std::error::Error;

#[derive(Debug)]
pub struct PluginError {
    msg: String
}

impl PluginError {
    pub fn new(msg: String) -> PluginError {
        PluginError {
            msg
        }
    }
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.msg.fmt(f)
    }
}

impl Error for PluginError {}
