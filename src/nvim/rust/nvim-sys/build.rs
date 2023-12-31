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

    const TYPE_ALLOWLIST: [&str; 7] = [
        "Arena",
        "Array",
        "Dictionary",
        "Error",
        "ErrorType",
        "KeyValuePair",
        "String",
    ];
    const NON_COPY_TYPE: [&str; 6] = ["Arena", "String", "Array", "Error", "Dictionary", "Object"];

    let api_functions = gen_api_func_lib::api_functions();
    let function_allowlist = [
        "api_clear_error",
        "api_free_array",
        "api_free_dictionary",
        "api_free_object",
        "api_free_string",
        "arena_finish",
        "arena_mem_free",
        "copy_array",
        "copy_dictionary",
        "copy_object",
        "copy_string",
        "emsg_multiline",
        "msg",
        "preserve_exit",
        "try_to_free_memory",
        "xcalloc",
        "xfree",
        "xmalloc",
        "xrealloc",
    ]
    .into_iter()
    .chain(api_functions.iter().map(|f| f.name.as_str()));
    const VAR_ALLOWLIST: [&str; 1] = ["e_outofmem"];

    for type_ in TYPE_ALLOWLIST {
        builder = builder.allowlist_type(type_);
    }
    for type_ in NON_COPY_TYPE {
        builder = builder.no_copy(type_);
    }
    for function in function_allowlist {
        eprintln!("{}", function);
        builder = builder.allowlist_function(function);
    }
    for var in VAR_ALLOWLIST {
        builder = builder.allowlist_var(var);
    }

    builder
        .generate()?
        .write_to_file(PathBuf::from(env::var("OUT_DIR")?).join("bindings.rs"))?;

    Ok(())
}
