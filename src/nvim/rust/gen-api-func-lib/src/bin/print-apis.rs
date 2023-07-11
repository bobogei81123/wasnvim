/// Prints the metadata of all the API functions

use gen_api_func_lib::api_functions;

fn main() {
    let funcs = api_functions();
    for func in funcs {
        println!("{:#?}", func);
    }
}
