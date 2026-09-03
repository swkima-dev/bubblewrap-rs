use crate::{config::Config, sandbox::Sandbox};
use std::{ffi::OsString, io::Result, process::ExitStatus};

pub struct Command {
    config: Config,
}

impl Command {
    pub fn new(program: OsString) -> Self {
        Self {
            config: Config::new(program),
        }
    }

    pub fn args(&mut self, args: Vec<OsString>) -> &mut Self {
        self.config.args(args);
        self
    }

    // TODO: user namespace's builder function herer.
    // TODO: implement user namespace is should task

    pub fn exec(&self) -> Result<ExitStatus> {
        let sandbox = Sandbox::new(self.config.clone());
        sandbox.create()
    }
}
