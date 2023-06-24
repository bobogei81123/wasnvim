use nvim::api::nvim_api::{nvim_exec2, ExecOpts};

wit_bindgen::generate!("plugin");

struct MyPlugin;

impl Plugin for MyPlugin {
    fn run(_: Vec<Object>) -> Object {
        let _ = nvim_exec2(
            "echo \"Hello, from rust through WASM!\"",
            ExecOpts {
                output: &Object::Nil,
            },
        );
        let _ = nvim_exec2(
            "r !echo 'Hello, from rust through WASM\\!'",
            ExecOpts {
                output: &Object::Nil,
            },
        );

        Object::Nil
    }
}

export_plugin!(MyPlugin);
