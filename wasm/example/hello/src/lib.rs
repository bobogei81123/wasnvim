wit_bindgen::generate!("plugin");

struct MyPlugin;

impl Plugin for MyPlugin {
    fn run() {
        let _ = nvim_exec("echo \"Hello, from rust through WASM!\"");
        let _ = nvim_exec("r !echo 'Hello, from rust through WASM\\!'");
    }
}

export_plugin!(MyPlugin);
