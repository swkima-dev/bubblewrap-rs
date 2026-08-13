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
  --unshare-user --unshare-pid --unshare-net --proc /proc \
  /bin/sh -c 'id && echo "sandbox PID: $$"'
```

The user namespace maps the invoking user's UID and GID to root inside the
sandbox. The sandbox init process runs as PID 1 and reaps descendants, while
the requested command normally runs as PID 2. As in Bubblewrap, `--proc /proc`
mounts a procfs instance belonging to the new PID namespace.

When filesystem operations are present, the sandbox root is assembled using
Bubblewrap's two-pivot layout: a tmpfs mounted at `/tmp` becomes a staging
root, host sources are visible below `/oldroot`, destinations are built below
`/newroot`, and a second `pivot_root` makes `/newroot` the final root while the
staging root is detached. Bind mounts, read-only bind mounts, tmpfs and procfs
mounts, directory creation and read-only remounts are currently supported.

Capabilities and seccomp are not implemented yet.

The library launcher currently performs setup after `fork(2)`, so `spawn()`
should be called before the embedding process starts additional threads.
