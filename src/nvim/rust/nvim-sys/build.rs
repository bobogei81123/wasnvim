use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=");
    let _out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    // This is... ugly...
    // TODO: Figure out how to generate those headers when running build.
    let dirs = [
        crate_dir.join("headers"),
        crate_dir.join("../../../"),
        crate_dir.join("../../../../build/src/nvim/auto"),
        crate_dir.join("../../../../build/cmake.config"),
        crate_dir.join("../../../../build/include"),
    ];

    let mut builder = bindgen::Builder::default()
        .header("headers/nvim.h")
        .clang_arg("-DINCLUDE_GENERATED_DECLARATIONS".to_string())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .derive_default(true);

    for dir in dirs {
        builder = builder.clang_arg(format!("-I{}", dir.to_str().unwrap()));
    }

    const TYPE_ALLOWLIST: [&str; 4] = ["String", "Error", "ErrorType", "Array"];
    const FUNCTION_ALLOWLIST: [&str; 4] = [
        "nvim_command",
        "nvim_call_function",
        "msg",
        "emsg_multiline",
    ];

    for type_ in TYPE_ALLOWLIST {
        builder = builder.allowlist_type(type_);
    }

    for function in FUNCTION_ALLOWLIST {
        builder = builder.allowlist_function(function);
    }

    builder
        .no_copy("Array")
        .no_copy("Dictionary")
        .generate()?
        .write_to_file(PathBuf::from(env::var("OUT_DIR")?).join("bindings.rs"))?;

    Ok(())
}
