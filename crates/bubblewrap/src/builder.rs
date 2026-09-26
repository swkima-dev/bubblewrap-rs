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

    pub fn share_user(&mut self) -> &mut Self {
        self.config.share_user();
        self
    }

    pub fn exec(&self) -> Result<ExitStatus> {
        self.config_validate()?;

        let sandbox = Sandbox::new(self.config.clone());
        sandbox.create()
    }

    fn config_validate(&self) -> Result<()> {
        if self.config.share_user
            && (self.config.internal_uid.is_some() || self.config.internal_gid.is_some())
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "internal_uid/internal_gid cannot be set with share_user",
            ));
        }

        Ok(())
    }
}
