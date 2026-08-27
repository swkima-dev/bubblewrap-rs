use crate::sandbox::create::Sandbox;
use anyhow::Result;
use std::ffi::OsString;

pub struct Command {
    program: OsString,
    args: Vec<OsString>,
}

impl Command {
    pub fn new(program: OsString) -> Self {
        Self {
            program,
            args: Vec::new(),
        }
    }

    pub fn args(&mut self, args: Vec<OsString>) -> &mut Self {
        self.args = args;
        self
    }

    pub fn exec(&self) -> Result<()> {
        let sandbox = Sandbox::new();
        sandbox.create()
        // std::process::Command::new(self.program.clone())
        //     .args(self.args.clone())
        //     .status()?;
    }
}
