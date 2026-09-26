use std::{ffi::CString, io::Result, os::unix::ffi::OsStrExt};

use crate::sandbox::Sandbox;
use nix::unistd::execve;

impl Sandbox {
    pub fn init(&self) -> Result<()> {
        let program = CString::new(self.config.program.as_bytes())?;
        let mut args = vec![program.clone()];
        for arg in &self.config.args {
            args.push(CString::new(arg.as_bytes())?);
        }
        let env: Vec<CString> = Vec::new();
        // TODO: Drop capabilities before execve. Upstream bwrap drops all of them
        // by default (unless --cap-add is given), but here the program keeps every
        // capability in the new user namespace when mapped to uid 0 (e.g. --uid 0),
        // and root keeps its capabilities in the host namespace with share_user.
        // Clear the bounding and ambient sets as well and set PR_SET_NO_NEW_PRIVS.
        // `execve` returns `Result<Infallible>`: on success the process image is
        // replaced, so the only way it returns is with an error.
        let Err(e) = execve(&program, &args, &env);
        Err(e.into())
    }
}
