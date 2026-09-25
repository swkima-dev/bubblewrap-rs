use crate::constant::EXIT_INTERNAL_FAILURE;
use nix::libc::_exit;
use nix::sched::{CloneFlags, unshare};
use nix::unistd::{Gid, Uid, getgid, getuid};
use std::fs::File;
use std::io::Write;

pub fn apply_user_namespace(internal_uid: &Uid, internal_gid: &Gid) -> Result<(), i32> {
    let real_uid_in_parental_ns = getuid();
    let real_gid_in_parental_ns = getgid();

    if let Err(_) = unshare(CloneFlags::CLONE_NEWUSER) {
        return Err(EXIT_INTERNAL_FAILURE);
    }

    if let Err(_) = write_file(
        &format!("/proc/self/uid_map"),
        &format!("{} {} 1", internal_uid, real_uid_in_parental_ns),
    ) {
        unsafe { _exit(EXIT_INTERNAL_FAILURE) }
    }

    if let Err(_) = write_file(&format!("/proc/self/setgroups"), &format!("deny")) {
        unsafe { _exit(EXIT_INTERNAL_FAILURE) }
    }

    if let Err(_) = write_file(
        &format!("/proc/self/gid_map"),
        &format!("{} {} 1", internal_gid, real_gid_in_parental_ns),
    ) {
        unsafe { _exit(EXIT_INTERNAL_FAILURE) }
    }

    Ok(())
}

fn write_file(path: &str, content: &str) -> std::io::Result<()> {
    let mut file = File::create(&path)?;

    file.write_all(content.as_bytes())
}
