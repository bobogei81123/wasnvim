use nvim::api::{nvim_api, nvim_keysets::KeysetFloatConfig};

wit_bindgen::generate!("plugin");

struct MyPlugin;

impl Plugin for MyPlugin {
    fn run(_args: Vec<Object>) -> Object {
        std::panic::set_hook(Box::new(|panic_info| {
            nvim_api::nvim_err_write(&format!("{panic_info}\n"));
        }));

        let buf = nvim_api::nvim_create_buf(false, false).unwrap();
        let _win = nvim_api::nvim_open_win(
            buf,
            true,
            &KeysetFloatConfig {
                row: Object::Integer(0),
                col: Object::Integer(0),
                width: Object::Integer(40),
                height: Object::Integer(40),
                anchor: Object::Nil,
                relative: Object::String("win".to_string()),
                win: Object::Nil,
                bufpos: Object::Nil,
                external: Object::Nil,
                focusable: Object::Nil,
                zindex: Object::Nil,
                border: Object::Nil,
                title: Object::Nil,
                title_pos: Object::Nil,
                style: Object::Nil,
                noautocmd: Object::Nil,
            },
        )
        .unwrap();
        nvim_api::nvim_buf_set_lines(
            buf,
            0,
            0,
            false,
            &["Hello".to_string(), "From".to_string(), "WASM!".to_string()],
        )
        .unwrap();

        Object::Nil
    }
}

export_plugin!(MyPlugin);
