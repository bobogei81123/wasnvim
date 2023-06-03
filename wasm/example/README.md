This directories contain some WASM examples that can be run with the WASM
interface.

# Compiling WASM binaries

Before building the WASM binaries, you'll need:
* Rust toolchain with the latest stable rustc and `wasm32-unknown-unknown`
  target installed. You can get it from [rustup](https://rustup.rs/) and run
  `rustup target add wasm32-unknown-unknown`.
* [`wasm-tools`](https://github.com/bytecodealliance/wasm-tools). Once you have
  rust and cargo you can get it by `cargo install wasm-tools`.

Follow these steps to build the WASM binaries: 

```bash
# Can be hello, call_func, etc.
EXAMPLE_NAME=hello
cd ${EXAMPLE_NAME}
cargo build --release
wasm-tools component new ./target/wasm32-unknown-unknown/release/${EXAMPLE_NAME}.wasm -o plugin.wasm
```

After that, `plugin.wasm` will be created that can be run with the new `:wasm` command.
