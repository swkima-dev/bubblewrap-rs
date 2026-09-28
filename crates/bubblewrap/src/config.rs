use std::ffi::OsString;

use nix::{
    sched::CloneFlags,
    unistd::{Gid, Uid},
};

#[derive(Clone, Debug)]
pub struct Config {
    pub(crate) program: OsString,
    pub(crate) args: Vec<OsString>,
    pub(crate) internal_uid: Option<Uid>,
    pub(crate) internal_gid: Option<Gid>,
    pub(crate) share_user: bool,
    pub(crate) namespaces: CloneFlags,
}

impl Config {
    pub fn new(program: OsString) -> Self {
        Self {
            program,
            args: Vec::new(),
            internal_uid: None,
            internal_gid: None,
            share_user: false,
            // Whether or not to create a new User Namespace is not controlled by these CloneFlags, but solely by `share_user`.
            // This is because creating a new user namespace is the initial step for an unprivileged user to create a new namespace,
            // and due to PID namespace constraints—it must be applied at an intermediate process stage.
            namespaces: CloneFlags::empty().union(CloneFlags::CLONE_NEWUSER),
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

    pub fn unshare_pid(&mut self) {
        self.namespaces.insert(CloneFlags::CLONE_NEWPID);
    }
}
