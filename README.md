# Rust Fuzzer

A simple file format fuzzer for unix based systems written in rust

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

