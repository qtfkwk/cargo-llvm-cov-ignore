# About

Run `cargo llvm-cov` with ignore capabilities and simple report

- Ignore uncovered lines by appending `//cargo-llvm-cov-ignore:line` to the line

- Ignore whole files by including `//cargo-llvm-cov-ignore:file` in the file

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
!run:../target/release/cargo-llvm-cov-ignore llvm-cov-ignore -h
```

# Version

```bash
cargo llvm-cov-ignore -V
```

```text
!run:../target/release/cargo-llvm-cov-ignore llvm-cov-ignore -V
```

# Example

```bash
cargo llvm-cov-ignore
```

```text
!run:cd .. && target/release/cargo-llvm-cov-ignore llvm-cov-ignore
```

