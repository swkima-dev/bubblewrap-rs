# bubblewrap-rs

A low-level sandboxing tool that provides ephemeral rootless containers in Rust.

in short, Rust rewrite of [Bubblewrap](https://github.com/containers/bubblewrap)

## General purpose of this project

The three main goals of this project are:

- Prioritize maintainability and readability.
- Maintain partial compatibility with Bubblewrap.
- Be usable not only as a command-line tool, but also as a Rust crate (that is, as a library).

Conversely, this project does not aim to:

- Maintain a direct correspondence with the Bubblewrap source code.
- Provide full compatibility with Bubblewrap.

## How Is This Different from a Container Runtime?

Strictly speaking, Bubblewrap is a tool with the following characteristics:

- It is designed to run rootlessly (that is, using a user namespace).
- The directory used as the target of `pivot_root` is always bind-mounted under `/tmp`.
- It provides containers—in the sense of isolated processes—that do not conform to the OCI Runtime Specification.

Therefore, Bubblewrap should be described as a sandboxing tool rather than a low-level container runtime.

## Current implementation

The current implementation can launch a command in rootless user, mount and
PID namespaces, with an optional network namespace:

```console
cargo run -p bubblewrap-cli -- \
  --unshare-user --unshare-pid --unshare-net \
  /bin/sh -c 'id && echo "sandbox PID: $$"'
```

The user namespace maps the invoking user's UID and GID to root inside the
sandbox. The sandbox init process runs as PID 1 and reaps descendants, while
the requested command normally runs as PID 2. A procfs instance belonging to
the new PID namespace is mounted at `/proc`. Filesystem construction,
`pivot_root`, capabilities and seccomp are not implemented yet; filesystem
options are rejected rather than silently ignored.

The library launcher currently performs setup after `fork(2)`, so `spawn()`
should be called before the embedding process starts additional threads.
