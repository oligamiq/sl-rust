# sl-rust 🚂

`sl-rust` is a faithful Rust port of the classic Unix joke command `sl` (Steam Locomotive). 
If you accidentally type `sl` instead of `ls`, a steam locomotive will run across your terminal.

![sl-rust animation](https://raw.githubusercontent.com/mtoyoda/sl/master/sl.gif) *(Original sl animation)*

## Features

- **Faithful Reproduction:** Faithfully reproduces the original behavior, ASCII art (including D51, C51, and SL Logo), and smoke animations.
- **Cross-Platform:** Runs perfectly on Linux, macOS, and Windows.
- **WASI Support:** Full support for `wasm32-wasip1`. Runs smoothly on WebAssembly runtimes like Wasmtime, Wasmer, and Node.js without relying on `wasm-bindgen`.
- **Unstoppable:** Just like the original, it ignores `Ctrl+C` and other inputs by default. You must watch the train pass!

## Installation

You can install `sl-rust` directly from crates.io using Cargo:

```bash
cargo install sl-rust
```

## Usage

Simply run:

```bash
sl
```

### Options

`sl-rust` supports the same classic flags as the original `sl`:

- `sl -a` : An accident occurs! (People crying for help)
- `sl -l` : Shows a smaller locomotive (SL Logo).
- `sl -F` : The locomotive flies into the sky!
- `sl -c` : Runs the C51 steam locomotive instead of the default D51.

These flags can also be combined, e.g., `sl -alF`.

## WASI Usage

To run `sl-rust` in a WASI environment:

```bash
# Build for WASI
cargo build --target wasm32-wasip1 --release

# Run using Wasmtime (you can pass terminal dimensions via environment variables)
wasmtime run --env COLUMNS=100 --env LINES=30 target/wasm32-wasip1/release/sl.wasm
```

## Debug Mode

By default, `sl-rust` ignores `Ctrl+C` and cannot be paused. If you want to enable keyboard controls for debugging or testing purposes, you can build with the `debug` feature:

```bash
cargo build --features debug
```

When built with the `debug` feature:
- Press `Space` or `P` to pause/resume the animation.
- Press `Ctrl+C` or `Q` to quit.

## Authors

- **oligamiq** <nziq53@gmail.com> (Rust port maintainer)
- **Toyoda Masashi** <mtoyoda@acm.org> (Original `sl` creator)

## License

This project is licensed under the MIT License.
