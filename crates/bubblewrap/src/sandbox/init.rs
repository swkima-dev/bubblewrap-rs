use std::{ffi::CString, os::unix::ffi::OsStrExt};

use crate::sandbox::Sandbox;
use anyhow::Result;
use nix::unistd::execve;

impl Sandbox {
    pub fn init(&self) -> Result<()> {
        let program = CString::new(self.config.program.as_bytes())?;
        let mut args = vec![program.clone()];
        for arg in &self.config.args {
            args.push(CString::new(arg.as_bytes())?);
        }
        let env: Vec<CString> = Vec::new();
        // `execve` returns `Result<Infallible>`: on success the process image is
        // replaced, so the only way it returns is with an error.
        let Err(e) = execve(&program, &args, &env);
        Err(e.into())
    }
}
