use nvim::api::nvim_api::{nvim_err_write, nvim_out_write};

wit_bindgen::generate!({
    world: "my-plugin",
    exports: {
        world: MyPluginImp,
    }
});

struct MyPluginImp;

impl MyPlugin for MyPluginImp {
    fn run(_: Vec<Object>) -> Object {
        std::panic::set_hook(Box::new(|panic_info| {
            nvim_err_write(&format!("{panic_info}\n"));
        }));
        nvim_out_write("Hello, from WASM!\n");

        Object::Nil
    }
}
