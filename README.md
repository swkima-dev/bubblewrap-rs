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
