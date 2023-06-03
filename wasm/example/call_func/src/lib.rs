wit_bindgen::generate!("plugin");

struct MyPlugin;

impl Plugin for MyPlugin {
    fn run() {
        let Object::String(val) =
            nvim_call_function("input", &[&Object::String("Enter a number: ".to_owned())]).unwrap() else { return; };

        let val = val.parse::<f64>().unwrap();
        let Ok(Object::Float(ans)) = nvim_call_function("sin", &[&Object::Float(val)]) else { return; };
        nvim_exec("echo ''").unwrap();
        nvim_exec(&format!("echo '\nsin({val}) = {ans}'")).unwrap();
    }
}

export_plugin!(MyPlugin);
