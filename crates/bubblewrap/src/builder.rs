use std::{ffi::OsString, io::Result};

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
        std::process::Command::new(self.program.clone())
            .args(self.args.clone())
            .status()?;
        Ok(())
    }
}
