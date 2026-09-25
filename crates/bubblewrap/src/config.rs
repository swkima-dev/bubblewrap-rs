use std::ffi::OsString;

use nix::unistd::{Gid, Uid, getgid, getuid};

#[derive(Clone, Debug)]
pub struct Config {
    pub(crate) program: OsString,
    pub(crate) args: Vec<OsString>,
    pub(crate) internal_uid: Uid,
    pub(crate) internal_gid: Gid,
}

impl Config {
    pub fn new(program: OsString) -> Self {
        Self {
            program,
            args: Vec::new(),
            internal_uid: getuid(),
            internal_gid: getgid(),
        }
    }

    pub fn args(&mut self, args: Vec<OsString>) -> &mut Self {
        self.args = args;
        self
    }

    pub fn internal_uid(&mut self, uid: u32) {
        self.internal_uid = Uid::from_raw(uid);
    }

    pub fn internal_gid(&mut self, gid: u32) {
        self.internal_gid = Gid::from_raw(gid);
    }
}
