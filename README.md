# About

Run `cargo llvm-cov` with ignore capabilities and simple report

- Ignore uncovered lines by appending `//cargo-llvm-cov-ignore:line` to the line

- Ignore whole files by including `//cargo-llvm-cov-ignore:file` in the file

- Exit code:
  - LSB 1 is set if there are unignored uncovered lines
  - LSB 2 is set if there are unused ignored lines
  - LSB 3 is set if there are unused ignored files

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

Usage: cargo llvm-cov-ignore

Options:
  -h, --help     Print help
  -V, --version  Print version
```

# Version

```bash
cargo llvm-cov-ignore -V
```

```text
cargo-llvm-cov-ignore 0.1.0
```

# Example

```bash
cargo llvm-cov-ignore
```

```text
# Code Coverage

## Uncovered Lines

There are zero files with uncovered lines.

## Unused Ignored Lines

There are zero files with unused ignored lines.

## Unused Ignored Files

There are zero unused ignored files.
```

