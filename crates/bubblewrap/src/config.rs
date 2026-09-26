use std::ffi::OsString;

use nix::unistd::{Gid, Uid};

#[derive(Clone, Debug)]
pub struct Config {
    pub(crate) program: OsString,
    pub(crate) args: Vec<OsString>,
    pub(crate) internal_uid: Option<Uid>,
    pub(crate) internal_gid: Option<Gid>,
    pub(crate) share_user: bool,
}

impl Config {
    pub fn new(program: OsString) -> Self {
        Self {
            program,
            args: Vec::new(),
            internal_uid: None,
            internal_gid: None,
            share_user: false,
        }
    }

    pub fn args(&mut self, args: Vec<OsString>) -> &mut Self {
        self.args = args;
        self
    }

    pub fn internal_uid(&mut self, uid: u32) {
        self.internal_uid = Some(Uid::from_raw(uid));
    }

    pub fn internal_gid(&mut self, gid: u32) {
        self.internal_gid = Some(Gid::from_raw(gid));
    }

    pub fn share_user(&mut self) {
        self.share_user = true;
    }
}
