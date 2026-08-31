use std::{ffi::CString, os::unix::ffi::OsStrExt};

use crate::sandbox::Sandbox;
use nix::{
    libc::{EXIT_FAILURE, exit},
    unistd::execve,
};

impl Sandbox {
    pub fn init(&self) {
        let program = CString::new(self.config.program.as_bytes()).unwrap();
        let mut args = vec![program.clone()];
        args.extend(
            self.config
                .args
                .iter()
                .map(|arg| CString::new(arg.as_bytes()).unwrap()),
        );
        let env: Vec<CString> = Vec::new();
        let Err(_) = execve(&program, &args, &env);
        unsafe { exit(EXIT_FAILURE) };
    }
}
