use nix::{sys::wait::waitpid, unistd::Pid};

use crate::sandbox::create::Sandbox;

impl Sandbox {
    pub fn monitor(&self, child: Pid) {
        println!(
            "Continuing execution in parent process, new child has pid: {}",
            child
        );
        waitpid(child, None).unwrap();
    }
}
