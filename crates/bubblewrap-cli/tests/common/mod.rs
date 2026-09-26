use std::process::{Command, Output};

/// Spawns the CLI as a separate process instead of calling the library
/// in-process. The test harness is multi-threaded, and forking it is only
/// safe until exec, which the sandbox setup code does not guarantee.
pub fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bubblewrap-cli"))
        .args(args)
        .output()
        .expect("failed to spawn bubblewrap-cli")
}

/// Formats the output for assertion messages.
pub fn describe(output: &Output) -> String {
    format!(
        "status: {}\n--- stdout ---\n{}\n--- stderr ---\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}
