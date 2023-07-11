/// Generates the WIT files describing the WASM interface.
///
/// Use `cargo run --bin gen-wit` to preview the outputs.
/// Use `cargo run --bin gen-wit <path>` to generate `keysets.wit` and `api.wit` under the
/// directory <path>.

use std::{fmt::Display, fs::File, io::Write, path::Path};

use anyhow::Result;
use gen_api_func_lib::{
    api_functions, api_keysets, ApiArrayType, ApiDictionaryType, ApiFunc, ApiKeyset, ApiType,
};
use indoc::indoc;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let keysets = api_keysets();
    let functions = api_functions();
    match args.as_slice() {
        [] => {
            println!("---------- keysets.wit ----------\n");
            write_keysets(&mut std::io::stdout(), &keysets);
            println!("\n---------- api.wit ----------\n");
            write_funcs(&mut std::io::stdout(), &functions, &keysets);
        }
        [path] => {
            let keyset_path = Path::new(path).join("keysets.wit");
            let mut keyset_file = File::create(&keyset_path).unwrap_or_else(|_| {
                panic!(
                    "Failed to open keyset file {} to write",
                    keyset_path.display()
                )
            });
            let func_path = Path::new(path).join("api.wit");
            let mut func_file = File::create(&func_path).unwrap_or_else(|_| {
                panic!(
                    "Failed to open api function file {} to write",
                    func_path.display()
                )
            });
            write_keysets(&mut keyset_file, &keysets);
            write_funcs(&mut func_file, &functions, &keysets)
        }
        _ => {
            panic!(
                "There should be exactly one argument which is the directory \
                   the wit files will be generated, or no argument, which the \
                   content will be printed to stdout"
            )
        }
    };
}

fn write_keysets<W: Write>(w: &mut W, keysets: &[ApiKeyset]) {
    write!(
        w,
        "{}",
        indoc! {"
            package nvim:api

            interface nvim-keysets {
              use nvim-types.{object}
        "}
    )
    .unwrap();
    for keyset in keysets {
        writeln!(w, "\n{}", WitKeysetItem(keyset)).unwrap();
    }
    writeln!(w, "}}").unwrap();
}

fn write_funcs<W: Write>(w: &mut W, functions: &[ApiFunc], keysets: &[ApiKeyset]) {
    write!(
        w,
        "{}",
        indoc! {"
            package nvim:api

            interface nvim-api {
              use nvim-types.{primitive, array, dictionary, object, buffer, window, tabpage}
        "}
    )
    .unwrap();

    let keysets_concat = keysets
        .iter()
        .map(|k| format!("%{}", k.wit_name()))
        .collect::<Vec<_>>()
        .join(", ");
    writeln!(
        w,
        "use nvim-keysets.{{{keysets_concat}}}",
        keysets_concat = keysets_concat,
    )
    .unwrap();

    for func in functions {
        if let Ok(item) = WitFuncItem::new(func) {
            writeln!(w, "\n{}", item).unwrap();
        }
    }
    writeln!(
        w,
        "{}",
        indoc! {"
            }

            world plugin {
              import nvim-api
            }
        "}
    )
    .unwrap();
}

struct WitKeysetItem<'a>(&'a ApiKeyset);

impl<'a> Display for WitKeysetItem<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  record %{name} {{", name = &self.0.wit_name()).unwrap();
        for field in &self.0.fields {
            writeln!(f, "    %{field}: object,", field = field.wit_name(),)?;
        }
        write!(f, "  }}").unwrap();
        Ok(())
    }
}

struct WitFuncItem<'a>(&'a ApiFunc);

impl<'a> WitFuncItem<'a> {
    fn new(func: &'a ApiFunc) -> Result<Self> {
        Ok(Self(func))
    }
}

impl<'a> Display for WitFuncItem<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  /// Corresponds to `{name}`.", name = self.0.name).unwrap();
        writeln!(f, "  ///").unwrap();
        writeln!(f, "  /// See `:help {name}`.", name = self.0.name).unwrap();
        write!(f, "  %{name}: func", name = self.0.wit_name()).unwrap();
        {
            write!(f, "(").unwrap();
            let args = &self.0.args;
            let mut first = true;
            for arg in &args.args {
                if first {
                    first = false;
                } else {
                    write!(f, ", ").unwrap();
                }
                write!(f, "%{name}: {type}", name = arg.wit_name(),
                    type = DisplayType(&arg.type_))?;
            }
            write!(f, ")").unwrap();
        }
        {
            let return_ = &self.0.return_;
            match (return_.type_.as_ref(), return_.has_error) {
                (None, false) => (),
                (None, true) => {
                    write!(f, " -> result<_, string>").unwrap();
                }
                (Some(type_), false) => {
                    write!(f, " -> {}", DisplayType(type_)).unwrap();
                }
                (Some(type_), true) => {
                    write!(f, " -> result<{}, string>", DisplayType(type_)).unwrap();
                }
            }
        }
        Ok(())
    }
}

struct DisplayType<'a>(&'a ApiType);

impl<'a> Display for DisplayType<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ApiType::*;
        match self.0 {
            Boolean => write!(f, "bool"),
            Integer => write!(f, "s64"),
            Float => write!(f, "float64"),
            String => write!(f, "string"),
            Array(ApiArrayType {
                inner_type: Some(inner_type),
            }) => write!(f, "list<{}>", DisplayType(inner_type)),
            Array(_) => write!(f, "list<object>"),
            Dictionary(ApiDictionaryType {
                inner_type: Some(inner_type),
            }) => write!(f, "list<tuple<string, {}>>", DisplayType(inner_type)),
            Dictionary(_) => write!(f, "list<tuple<string, object>>"),
            Keyset(keyset) => write!(f, "%{}", keyset.wit_name()),
            LuaRef => write!(f, "lua_ref"),
            Buffer => write!(f, "buffer"),
            Window => write!(f, "window"),
            Tabpage => write!(f, "tabpage"),
            Object => write!(f, "object"),
        }
    }
}
