use nvim::api::nvim_api::{nvim_err_write, nvim_out_write};

wit_bindgen::generate!("plugin");

struct MyPlugin;

impl Plugin for MyPlugin {
    fn run(_: Vec<Object>) -> Object {
        std::panic::set_hook(Box::new(|panic_info| {
            nvim_err_write(&format!("{panic_info}\n"));
        }));
        nvim_out_write("Hello, from WASM!\n");

        Object::Nil
    }
}

export_plugin!(MyPlugin);
