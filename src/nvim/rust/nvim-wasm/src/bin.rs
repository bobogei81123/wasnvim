use wasm_rs::{wasm_rs_init, wasm_run_impl};

pub fn main() {
  wasm_rs_init();
  wasm_run_impl("/home/meteor/Documents/Project/neovim/wasm/example/hello/plugin.wasm");
}
