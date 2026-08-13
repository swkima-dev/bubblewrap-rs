use clap::{ArgMatches, CommandFactory, FromArgMatches, Parser};
use std::ffi::OsString;
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
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
    #[arg(long, num_args = 2, value_names = ["SRC", "DEST"], action = clap::ArgAction::Append)]
    ro_bind: Vec<String>,

    /// Mount new tmpfs on DEST. If the previous option was --perms, it sets
    /// the mode of the tmpfs. Otherwise, the tmpfs has mode 0755.
    #[arg(long, value_name = "DEST")]
    #[arg(action = clap::ArgAction::Append)]
    tmpfs: Vec<String>,

    /// Mount a proc filesystem on DEST
    #[arg(long, value_name = "DEST", action = clap::ArgAction::Append)]
    proc: Vec<String>,

    /// Bind mount the host path SRC on DEST
    #[arg(long, num_args = 2, value_names = ["SRC", "DEST"], action = clap::ArgAction::Append)]
    bind: Vec<String>,

    /// Create a directory at DEST. If the directory already exists, its
    /// permissions are unmodified, ignoring --perms (use --chmod if the
    /// permissions of an existing directory need to be changed). If the
    /// directory is newly created and the previous option was --perms, it
    /// sets the mode of the directory. Otherwise, newly-created directories
    /// have mode 0755.
    #[arg(long, value_name = "DEST")]
    #[arg(action = clap::ArgAction::Append)]
    dir: Vec<String>,

    /// Remount the path DEST as readonly. It works only on the specified
    /// mount point, without changing any other mount point under the
    /// specified path
    #[arg(long, value_name = "DEST")]
    #[arg(action = clap::ArgAction::Append)]
    remount_ro: Vec<String>,

    /// Program and arguments to execute in the sandbox
    #[arg(required = true, num_args = 1.., allow_hyphen_values = true)]
    command: Vec<OsString>,
}

fn main() -> ExitCode {
    let matches = Args::command().get_matches();
    let args = Args::from_arg_matches(&matches).unwrap_or_else(|error| error.exit());
    let filesystem = filesystem_operations(&args, &matches);

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
    for operation in filesystem {
        match operation {
            FilesystemArgument::Bind {
                source,
                destination,
                readonly: false,
            } => {
                command.bind(source, destination);
            }
            FilesystemArgument::Bind {
                source,
                destination,
                readonly: true,
            } => {
                command.ro_bind(source, destination);
            }
            FilesystemArgument::Tmpfs(destination) => {
                command.tmpfs(destination);
            }
            FilesystemArgument::Proc(destination) => {
                command.proc(destination);
            }
            FilesystemArgument::Dir(destination) => {
                command.dir(destination);
            }
            FilesystemArgument::RemountReadonly(destination) => {
                command.remount_readonly(destination);
            }
        }
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

#[derive(Debug, Eq, PartialEq)]
enum FilesystemArgument {
    Bind {
        source: PathBuf,
        destination: PathBuf,
        readonly: bool,
    },
    Tmpfs(PathBuf),
    Proc(PathBuf),
    Dir(PathBuf),
    RemountReadonly(PathBuf),
}

fn filesystem_operations(args: &Args, matches: &ArgMatches) -> Vec<FilesystemArgument> {
    let mut indexed = Vec::new();

    add_bind_operations(&mut indexed, matches, "ro_bind", &args.ro_bind, true);
    add_bind_operations(&mut indexed, matches, "bind", &args.bind, false);
    add_path_operations(&mut indexed, matches, "tmpfs", &args.tmpfs, |path| {
        FilesystemArgument::Tmpfs(path)
    });
    add_path_operations(&mut indexed, matches, "proc", &args.proc, |path| {
        FilesystemArgument::Proc(path)
    });
    add_path_operations(&mut indexed, matches, "dir", &args.dir, |path| {
        FilesystemArgument::Dir(path)
    });
    add_path_operations(
        &mut indexed,
        matches,
        "remount_ro",
        &args.remount_ro,
        FilesystemArgument::RemountReadonly,
    );

    indexed.sort_by_key(|(index, _)| *index);
    indexed
        .into_iter()
        .map(|(_, operation)| operation)
        .collect()
}

fn add_bind_operations(
    operations: &mut Vec<(usize, FilesystemArgument)>,
    matches: &ArgMatches,
    id: &str,
    values: &[String],
    readonly: bool,
) {
    let Some(indices) = matches.indices_of(id) else {
        return;
    };
    for (indices, values) in indices
        .collect::<Vec<_>>()
        .chunks_exact(2)
        .zip(values.chunks_exact(2))
    {
        operations.push((
            indices[0],
            FilesystemArgument::Bind {
                source: PathBuf::from(&values[0]),
                destination: PathBuf::from(&values[1]),
                readonly,
            },
        ));
    }
}

fn add_path_operations(
    operations: &mut Vec<(usize, FilesystemArgument)>,
    matches: &ArgMatches,
    id: &str,
    values: &[String],
    make_operation: impl Fn(PathBuf) -> FilesystemArgument,
) {
    let Some(indices) = matches.indices_of(id) else {
        return;
    };
    operations.extend(
        indices
            .zip(values)
            .map(|(index, value)| (index, make_operation(PathBuf::from(value)))),
    );
}

fn status_exit_code(status: ExitStatus) -> ExitCode {
    let code = status
        .code()
        .unwrap_or_else(|| 128 + status.signal().unwrap_or(127));
    ExitCode::from(u8::try_from(code).unwrap_or(255))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filesystem_options_preserve_command_line_order() {
        let matches = Args::command()
            .try_get_matches_from([
                "bubblewrap",
                "--dir",
                "/workspace",
                "--ro-bind",
                "/usr",
                "/usr",
                "--tmpfs",
                "/tmp",
                "--proc",
                "/proc",
                "/bin/true",
            ])
            .unwrap();
        let args = Args::from_arg_matches(&matches).unwrap();

        assert_eq!(
            filesystem_operations(&args, &matches),
            vec![
                FilesystemArgument::Dir(PathBuf::from("/workspace")),
                FilesystemArgument::Bind {
                    source: PathBuf::from("/usr"),
                    destination: PathBuf::from("/usr"),
                    readonly: true,
                },
                FilesystemArgument::Tmpfs(PathBuf::from("/tmp")),
                FilesystemArgument::Proc(PathBuf::from("/proc")),
            ]
        );
    }
}
