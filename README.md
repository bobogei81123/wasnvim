This fork of [Neovim](https://github.com/neovim/neovim) aims to add a
[WASM](https://webassembly.org/) plugin interface.

This is **not** to make Neovim able to be compiled into WASM.

Build
--------
Run `make` in this top-level directory. The built neovim will be located at
`build/bin/nvim`. Try it with `build/bin/nvim -u NONE`.

After building the neovim from source, run `:wasm <path-to-wasm>` to run a WASM
binary.


Examples
--------

There are some example WASM binaries located in `wasm/example`. See the [README
file](wasm/example/README.md) for building instructions.

WASM
--------

[WebAssembly](https://webassembly.org/) (WASM) is a portable binary format. It
is designed to be executed in a sandboxed virtual machines (hosts) safely.
Moreover, nearly all programming language can target WASM, this includes C, C++,
Rust, and many more.

WASM Features
--------

This implementation uses the following non-standardized features:

* [Component Model](https://github.com/WebAssembly/component-model): Without
  the component model proposal, we will need to document how data are
  represented and passed across the virtual machine boundary, and either Neovim
  or the people writing WASM binary will need to implement the "glue code"
  translating the representations. The proposal is in Phase 1: *Feature
  Proposal* so many changes is likely before this feature is standardized.

Features that will be helpful

* [WebAssembly C and C++ API](https://github.com/WebAssembly/wasm-c-api): This
  will create a common interface for WASM runtimes so it is possible to
  plug in different runtimes seamlessly.

WASM Runtimes
--------

The current implmementation uses
[Wasmtime](https://github.com/bytecodealliance/wasmtime) because it seems to be
the only one now (2023-06) that supports the component model feature.

Nvim WASM module
--------

The main code for the new WASM module is located at
[`src/nvim/rust`](src/nvim/rust). It is written in Rust because only Wasmtime
Rust API supports Component model for now (2023-06). The C bindings is located
at [`src/nvim/wasm`](src/nvim/wasm).


<!-- vim: set tw=80: -->
