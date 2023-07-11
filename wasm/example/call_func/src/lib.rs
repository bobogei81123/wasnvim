use nvim::api::{
    nvim_api::{nvim_call_function, nvim_err_write, nvim_exec2},
    nvim_keysets::KeysetExecOpts,
};

wit_bindgen::generate!("plugin");

struct MyPlugin;

impl Plugin for MyPlugin {
    fn run(_args: Vec<Object>) -> Object {
        std::panic::set_hook(Box::new(|panic_info| {
            nvim_err_write(&format!("{panic_info}\n"));
        }));

        let Object::String(val) =
          nvim_call_function("input", &[&Object::String("Enter a number: ".to_owned())]).unwrap() else { return Object::Nil; };

        let val = val.parse::<f64>().unwrap();
        let Ok(Object::Float(ans)) = nvim_call_function("sin", &[&Object::Float(val)]) else { return Object::Nil; };
        nvim_exec2(
            &format!("echo ' => sin({val}) = {ans}'"),
            KeysetExecOpts {
                output: &Object::Nil,
            },
        )
        .unwrap();

        Object::Nil
    }
}

export_plugin!(MyPlugin);
