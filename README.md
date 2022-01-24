# Rust Fuzzer

A simple file format fuzzer for unix based systems written in rust

**WARNING**: WSL 2 (WSL 1 untested) may cause large amounts of disk space (GB/s) to be eaten up on some target files (including the test file) when they crash with not a known way to get back other than resetting the WSL 2 filesystem. This fuzzer works fine for virtual machines.

## Reqirements
- Cargo (>=1.53.0)

## Setup
- Put input files into the `./corpus/` directory

To set up a test configuration with a test target file `./objdump` and a small corpus:
```
cp test_files/* .
```

## Run
Run with target command and options as argument
```
cargo run --release [target] [options]...
```

Build with optimizations
```
cargo build --release
cp target/release/rust_fuzzer .
./rust_fuzzer [target] [options]...
```

Install to path
```
cargo install -path .
rust_fuzzer [target] [options]...
```
