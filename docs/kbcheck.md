# kbcheck

`kbcheck` is a tool for interacting with the knowledge base.

## Requirements

`kbcheck` is written in [Rust](https://www.rust-lang.org/) and
requires [Cargo](https://doc.rust-lang.org/cargo/). If you don't
already have Rust installed (e.g. from a gecko build environment) it's
recommended to install it using [rustup](https://rustup.rs/).

## Usage

The easiest way to build and run `kbcheck` is using `cargo run` in the
git root. Note that command line options must be passed after a `--`
e.g.

```
cargo run -- --help
```
will show the `kbcheck` help.

Alterntively the binary may be built, again using cargo and then run from the `target` directory:

```
cargo build --release
./target/release/kbcheck
```

### Command Line Options

The full set of options can be obtained through the built-in
help. Only the most commonly used options are additionally documented
here.

* `--root-path` - Sets the root of the knowledge-base checkout. By
  default this is assumed to be the current directory.

## Commands

`kbcheck` is organised into a set of subcommands for different tasks.

### tags

`cargo run -- tags`

This prints a list of all tags that are currently being used in the
knowledge base, and their count.

### validate

`cargo run -- validate`

Validates the knowledge base entries against the YAML schema, and
against additional lint rules that are defined in code.
