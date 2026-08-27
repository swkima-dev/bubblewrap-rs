use crate::sandbox::create::Sandbox;
use anyhow::Result;
use nix::{libc::exit, unistd::write};

impl Sandbox {
    pub fn init(&self) -> Result<()> {
        write(std::io::stdout(), "I'm a new init process\n".as_bytes()).ok();
        unsafe { exit(0) };
    }
}
