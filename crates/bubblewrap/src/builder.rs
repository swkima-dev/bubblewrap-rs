use crate::config::{FilesystemOperation, SandboxConfig};
use crate::process;
use crate::{Error, Result};
use nix::unistd::Pid;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::ExitStatus;

/// Builder for a process isolated with Linux namespaces.
#[derive(Debug)]
pub struct Command {
    program: OsString,
    args: Vec<OsString>,
    current_dir: Option<PathBuf>,
    unshare_user: bool,
    unshare_pid: bool,
    unshare_net: bool,
    new_session: bool,
    die_with_parent: bool,
    filesystem: Vec<FilesystemOperation>,
}

impl Command {
    pub fn new<S: AsRef<OsStr>>(program: S) -> Self {
        Self {
            program: program.as_ref().to_owned(),
            args: Vec::new(),
            current_dir: None,
            unshare_user: false,
            unshare_pid: false,
            unshare_net: false,
            new_session: false,
            die_with_parent: false,
            filesystem: Vec::new(),
        }
    }

    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.args.push(arg.as_ref().to_owned());
        self
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.args
            .extend(args.into_iter().map(|arg| arg.as_ref().to_owned()));
        self
    }

    pub fn current_dir<P: AsRef<Path>>(&mut self, directory: P) -> &mut Self {
        self.current_dir = Some(directory.as_ref().to_owned());
        self
    }

    pub fn unshare_user(&mut self) -> &mut Self {
        self.unshare_user = true;
        self
    }

    pub fn unshare_pid(&mut self) -> &mut Self {
        self.unshare_pid = true;
        self
    }

    pub fn unshare_net(&mut self) -> &mut Self {
        self.unshare_net = true;
        self
    }

    pub fn new_session(&mut self) -> &mut Self {
        self.new_session = true;
        self
    }

    pub fn die_with_parent(&mut self) -> &mut Self {
        self.die_with_parent = true;
        self
    }

    pub fn bind<P: AsRef<Path>, Q: AsRef<Path>>(&mut self, source: P, destination: Q) -> &mut Self {
        self.filesystem.push(FilesystemOperation::Bind {
            source: source.as_ref().to_owned(),
            destination: destination.as_ref().to_owned(),
            readonly: false,
        });
        self
    }

    pub fn ro_bind<P: AsRef<Path>, Q: AsRef<Path>>(
        &mut self,
        source: P,
        destination: Q,
    ) -> &mut Self {
        self.filesystem.push(FilesystemOperation::Bind {
            source: source.as_ref().to_owned(),
            destination: destination.as_ref().to_owned(),
            readonly: true,
        });
        self
    }

    pub fn tmpfs<P: AsRef<Path>>(&mut self, destination: P) -> &mut Self {
        self.filesystem.push(FilesystemOperation::Tmpfs {
            destination: destination.as_ref().to_owned(),
        });
        self
    }

    pub fn proc<P: AsRef<Path>>(&mut self, destination: P) -> &mut Self {
        self.filesystem.push(FilesystemOperation::Proc {
            destination: destination.as_ref().to_owned(),
        });
        self
    }

    pub fn dir<P: AsRef<Path>>(&mut self, destination: P) -> &mut Self {
        self.filesystem.push(FilesystemOperation::Dir {
            destination: destination.as_ref().to_owned(),
        });
        self
    }

    pub fn remount_readonly<P: AsRef<Path>>(&mut self, destination: P) -> &mut Self {
        self.filesystem.push(FilesystemOperation::RemountReadonly {
            destination: destination.as_ref().to_owned(),
        });
        self
    }

    /// Launch the sandbox and return once its init process is ready.
    ///
    /// The current launcher performs setup after `fork(2)`. Until it is moved
    /// into a separately exec'd helper, callers should invoke this before
    /// starting additional threads in the process.
    pub fn spawn(&mut self) -> Result<Child> {
        if !self.unshare_user {
            return Err(Error::InvalidConfig(
                "rootless sandboxing requires a user namespace",
            ));
        }
        validate_filesystem_paths(&self.filesystem)?;

        let config = SandboxConfig {
            program: self.program.clone(),
            args: self.args.clone(),
            current_dir: self.current_dir.clone(),
            unshare_pid: self.unshare_pid,
            unshare_net: self.unshare_net,
            new_session: self.new_session,
            die_with_parent: self.die_with_parent,
            filesystem: self.filesystem.clone(),
        };

        process::spawn(config).map(|pid| Child { pid, waited: false })
    }

    pub fn status(&mut self) -> Result<ExitStatus> {
        self.spawn()?.wait()
    }
}

fn validate_filesystem_paths(operations: &[FilesystemOperation]) -> Result<()> {
    for operation in operations {
        match operation {
            FilesystemOperation::Bind {
                source,
                destination,
                ..
            } => {
                validate_absolute_path(source, "bind source must be an absolute path")?;
                validate_absolute_path(
                    destination,
                    "bind destination must be an absolute normalized path",
                )?;
            }
            FilesystemOperation::Tmpfs { destination }
            | FilesystemOperation::Proc { destination }
            | FilesystemOperation::Dir { destination }
            | FilesystemOperation::RemountReadonly { destination } => {
                validate_absolute_path(
                    destination,
                    "filesystem destination must be an absolute normalized path",
                )?;
            }
        }
    }
    Ok(())
}

fn validate_absolute_path(path: &Path, message: &'static str) -> Result<()> {
    use std::path::Component;

    if !path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::RootDir | Component::Normal(_)))
    {
        return Err(Error::InvalidConfig(message));
    }
    Ok(())
}

#[derive(Debug)]
pub struct Child {
    pid: Pid,
    waited: bool,
}

impl Child {
    /// The host PID of the intermediate sandbox process.
    pub fn id(&self) -> u32 {
        self.pid.as_raw() as u32
    }

    pub fn wait(&mut self) -> Result<ExitStatus> {
        if self.waited {
            return Err(Error::InvalidConfig(
                "sandbox process was already waited on",
            ));
        }
        let status = process::wait(self.pid)?;
        self.waited = true;
        Ok(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rootless_command_requires_user_namespace() {
        let error = Command::new("true").spawn().unwrap_err();
        assert!(matches!(error, Error::InvalidConfig(_)));
    }
}
