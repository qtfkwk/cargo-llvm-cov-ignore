# About

Run `cargo llvm-cov` with ignore capabilities and simple report

- Ignore an uncovered line by appending `//cargo-llvm-cov-ignore:line` to the line

- Ignore all uncovered lines in a file by including `//cargo-llvm-cov-ignore:file` in the file

- Exit code:
  - LSB 1 is set if there are unignored uncovered lines
  - LSB 2 is set if there are unused ignored lines
  - LSB 3 is set if there are unused ignored files

See also:

- `cargo-llvm-cov`
  - <https://crates.io/crates/cargo-llvm-cov>
  - <https://github.com/taiki-e/cargo-llvm-cov>
  - <https://github.com/taiki-e/cargo-llvm-cov/issues/524>

# Install

```bash
cargo install cargo-llvm-cov cargo-llvm-cov-ignore
```

# Usage

```bash
cargo llvm-cov-ignore -h
```

```text
Run `cargo llvm-cov` with ignore capabilities and simple report

Usage: cargo llvm-cov-ignore [OPTIONS]

Options:
      --color <auto|always|never>  Color output
  -h, --help                       Print help
  -V, --version                    Print version
```

# Version

```bash
cargo llvm-cov-ignore -V
```

```text
cargo-llvm-cov-ignore 0.3.1
```

# Example

```bash
cargo llvm-cov-ignore
```

```text
# Code Coverage

- There are zero files with uncovered lines.
- There are zero files with unused ignored lines.
- There are zero unused ignored files.
```

