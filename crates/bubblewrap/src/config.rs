use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct SandboxConfig {
    pub(crate) program: OsString,
    pub(crate) args: Vec<OsString>,
    pub(crate) current_dir: Option<PathBuf>,
    pub(crate) unshare_pid: bool,
    pub(crate) unshare_net: bool,
    pub(crate) new_session: bool,
    pub(crate) die_with_parent: bool,
}
