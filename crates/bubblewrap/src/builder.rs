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

    pub fn internal_uid(&mut self, uid: u32) -> &mut Self {
        self.config.internal_uid(uid);
        self
    }

    pub fn internal_gid(&mut self, gid: u32) -> &mut Self {
        self.config.internal_gid(gid);
        self
    }

    pub fn exec(&self) -> Result<ExitStatus> {
        let sandbox = Sandbox::new(self.config.clone());
        sandbox.create()
    }
}
