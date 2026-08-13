use clap::Parser;
use std::ffi::OsString;
use std::os::unix::process::ExitStatusExt;
use std::process::{ExitCode, ExitStatus};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, trailing_var_arg = true)]
struct Args {
    /// Ensures child process (COMMAND) dies when bwrap's parent dies. Kills
    /// (SIGKILL) all bwrap sandbox processes in sequence from parent to child
    /// including COMMAND process when bwrap or bwrap's parent dies. See
    /// prctl, PR_SET_PDEATHSIG.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    die_with_parent: bool,

    /// Create a new user namespace
    #[arg(long, action = clap::ArgAction::SetTrue)]
    unshare_user: bool,

    /// Create a new PID namespace
    #[arg(long, action = clap::ArgAction::SetTrue)]
    unshare_pid: bool,

    /// Create a new network namespace
    #[arg(long, action = clap::ArgAction::SetTrue)]
    unshare_net: bool,

    /// Create a new terminal session for COMMAND
    #[arg(long, action = clap::ArgAction::SetTrue)]
    new_session: bool,

    /// Change directory to DIR
    #[arg(long, value_name = "DIR")]
    chdir: Option<String>,

    /// Bind mount the host path SRC readonly on DEST
    #[arg(long, num_args = 2, value_names = ["SRC", "DEST"])]
    ro_bind: Option<Vec<String>>,

    /// Mount new tmpfs on DEST. If the previous option was --perms, it sets
    /// the mode of the tmpfs. Otherwise, the tmpfs has mode 0755.
    #[arg(long, value_name = "DEST")]
    tmpfs: Option<String>,

    /// Bind mount the host path SRC on DEST
    #[arg(long, num_args = 2, value_names = ["SRC", "DEST"])]
    bind: Option<Vec<String>>,

    /// Create a directory at DEST. If the directory already exists, its
    /// permissions are unmodified, ignoring --perms (use --chmod if the
    /// permissions of an existing directory need to be changed). If the
    /// directory is newly created and the previous option was --perms, it
    /// sets the mode of the directory. Otherwise, newly-created directories
    /// have mode 0755.
    #[arg(long, value_name = "DEST")]
    dir: Option<String>,

    /// Remount the path DEST as readonly. It works only on the specified
    /// mount point, without changing any other mount point under the
    /// specified path
    #[arg(long, value_name = "DEST")]
    remount_ro: Option<String>,

    /// Program and arguments to execute in the sandbox
    #[arg(required = true, num_args = 1.., allow_hyphen_values = true)]
    command: Vec<OsString>,
}

fn main() -> ExitCode {
    let args = Args::parse();

    if args.ro_bind.is_some()
        || args.tmpfs.is_some()
        || args.bind.is_some()
        || args.dir.is_some()
        || args.remount_ro.is_some()
    {
        eprintln!("bubblewrap: filesystem options are not implemented yet");
        return ExitCode::from(2);
    }

    if args.die_with_parent
        && let Err(error) = bubblewrap::die_with_parent()
    {
        eprintln!("bubblewrap: failed to configure parent-death signal: {error}");
        return ExitCode::from(125);
    }

    let (program, command_args) = args.command.split_first().expect("command is required");
    let mut command = bubblewrap::Command::new(program);
    command.args(command_args);
    if args.unshare_user {
        command.unshare_user();
    }
    if args.unshare_pid {
        command.unshare_pid();
    }
    if args.unshare_net {
        command.unshare_net();
    }
    if args.new_session {
        command.new_session();
    }
    if args.die_with_parent {
        command.die_with_parent();
    }
    if let Some(directory) = args.chdir {
        command.current_dir(directory);
    }

    match command.status() {
        Ok(status) => status_exit_code(status),
        Err(error) => {
            eprintln!("bubblewrap: {error}");
            ExitCode::from(125)
        }
    }
}

fn status_exit_code(status: ExitStatus) -> ExitCode {
    let code = status
        .code()
        .unwrap_or_else(|| 128 + status.signal().unwrap_or(127));
    ExitCode::from(u8::try_from(code).unwrap_or(255))
}
