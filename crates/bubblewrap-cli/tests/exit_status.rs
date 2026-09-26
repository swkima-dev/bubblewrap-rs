mod common;

use common::{describe, run};
use std::process::Command;

#[test]
fn exit_code_of_command_is_propagated() {
    let output = run(&["--", "/bin/sh", "-c", "exit 42"]);

    assert_eq!(output.status.code(), Some(42), "{}", describe(&output));
}

#[test]
fn signal_termination_is_reported_as_128_plus_signal() {
    let output = run(&["--", "/bin/sh", "-c", "kill -9 $$"]);

    assert_eq!(output.status.code(), Some(128 + 9), "{}", describe(&output));
}

#[test]
fn nonexistent_program_fails_with_message() {
    let output = run(&["--", "/nonexistent/program"]);

    assert_eq!(output.status.code(), Some(1), "{}", describe(&output));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("No such file or directory"),
        "{}",
        describe(&output)
    );
}

/// Simulates a host where user namespaces are unavailable by running the CLI
/// inside a user namespace whose `max_user_namespaces` is 0. This works
/// without root, but requires util-linux `unshare`.
/// The marker is assembled by printf because the CLI echoes its argv to stdout.
#[test]
fn user_namespace_setup_failure_is_reported() {
    let script = format!(
        r#"echo 0 > /proc/sys/user/max_user_namespaces || exit 100
exec "{}" -- /bin/sh -c 'printf "COMMAND_%s\n" RAN'"#,
        env!("CARGO_BIN_EXE_bubblewrap-cli")
    );
    let output = Command::new("unshare")
        .args(["--user", "--map-root-user", "/bin/sh", "-c", &script])
        .output()
        .expect("failed to spawn unshare (util-linux)");

    assert_ne!(
        output.status.code(),
        Some(100),
        "could not disable user namespaces for the test\n{}",
        describe(&output)
    );
    assert_eq!(output.status.code(), Some(255), "{}", describe(&output));
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains("COMMAND_RAN"),
        "{}",
        describe(&output)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("user namespace"),
        "{}",
        describe(&output)
    );
}
