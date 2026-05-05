# WASI Multi-Utility & Shell (sl-rust)

A collection of high-performance, modular utilities and an interactive shell, all optimized for WASI (WebAssembly System Interface).

## Project Overview

This workspace contains several crates designed to provide a rich command-line environment in WASI-compatible runtimes (like wasmtime, wasmer, or the browser).

- **`wasibox`**: An umbrella multi-call binary (similar to BusyBox) that integrates all utilities.
- **`wasi-shell`**: A modular REPL with support for piping (`|`) and redirection (`>`, `>>`).
- **`wasibox-core`**: A shared library of essential POSIX-like utilities (ls, cat, grep, etc.) using an injectable `IoContext`.
- **`sl-rust`**: A Rust implementation of the classic SL (Steam Locomotive) animation.

## Features

- **Piping & Redirection**: Full support for standard shell IO operators in `wasi-shell`.
- **Modular Utilities**: Over 30 utilities that can be used as standalone binaries or linked into the `wasibox`.
- **WASI Optimized**: Designed from the ground up to respect WASI capabilities and limitations (e.g., restricted filesystem access).
- **Cross-Platform**: Works on Native (Windows/Linux/macOS) and WASI.

## Installation

```bash
cargo install wasibox
cargo install wasi-shell
```

## Usage

### Interactive Shell
```bash
wasi-shell
```

### Multi-call Utility
```bash
wasibox ls -al
wasibox grep "pattern" file.txt
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
