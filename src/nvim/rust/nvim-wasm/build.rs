use gen_api_func_lib::{
    api_functions, ApiArg, ApiArrayType, ApiDictionaryType, ApiField, ApiFunc, ApiFuncArgs,
    ApiFuncReturn, ApiKeyset, ApiType,
};

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::{env, fs::File, io::Write, path::Path};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let mut out_file = File::create(Path::new(&out_dir).join("api_impl.rs")).unwrap();

    let api_funcs = api_functions();
    let funcs_impls = api_funcs.iter().map(gen_fn_impl);

    let api_impl = quote! {
        // The channel IDjused by all WASM calls.
        const WASM_INTERNAL_CALL: u64 = (1u64 << 63) + 2;

        #[allow(non_snake_case, clippy::let_unit_value)]
        impl nvim_api::Host for NvimHost {
            #(#funcs_impls)*
        }
    };

    writeln!(out_file, "{api_impl}").unwrap();
}

/// Generates a single function implementation.
///
/// # Example
///
/// For API function
///
/// %nvim-create-augroup: func(%name: string, %opts: %keyset-create-augroup) -> result<s64, string>
///
/// The following definition is generated
///
/// ```rust
///     fn nvim_create_augroup(
///     &mut self,
///     name: String,
///     opts: nvim_keysets::KeysetCreateAugroup,
/// ) -> wasmtime::Result<Result<i64, String>> {
///     let name = <nvim_rs::NvimString>::from_wasm_type(name);
///     let mut opts__clear = <nvim_rs::NvimObject>::from_wasm_type(opts.clear);
///     let mut opts = nvim_sys::KeyDict_create_augroup {
///         clear: opts__clear.as_mut_borrowed_ffi(),
///     };
///     let mut __api_func_error_var = nvim_rs::NvimResult::new_ok();
///     let __api_func_result_var = unsafe {
///         nvim_sys::nvim_create_augroup(
///             WASM_INTERNAL_CALL,
///             name.as_borrowed_ffi(),
///             &mut opts,
///             __api_func_error_var.as_ffi_mut(),
///         )
///     };
///     if let Err(err) = __api_func_error_var.into_result() {
///         return Ok(Err(err.to_string()));
///     }
///     Ok(Ok((__api_func_result_var).try_into_wasm_type()?))
/// }
/// ```
fn gen_fn_impl(func: &ApiFunc) -> TokenStream {
    let func_name = format_ident!("r#{}", wit_name_to_snake(&func.wit_name()));
    let func_args = gen_fn_args(func);
    let func_return_type = gen_fn_return_type(&func.return_);
    let func_body = gen_fn_body(func);
    quote! {
        fn #func_name(#func_args) -> #func_return_type {
            #func_body
        }
    }
}

fn gen_fn_args(func: &ApiFunc) -> TokenStream {
    let fn_arg = func.args.args.iter().map(gen_fn_arg);
    quote! {
        &mut self,
        #(#fn_arg,)*
    }
}

fn gen_fn_arg(arg: &ApiArg) -> TokenStream {
    let arg_name = format_ident!("r#{}", wit_name_to_snake(&arg.wit_name()));
    let arg_type = wasm_type(&arg.type_);
    quote! {#arg_name: #arg_type}
}

fn gen_fn_return_type(return_: &ApiFuncReturn) -> TokenStream {
    let mut return_type = if let Some(ty) = &return_.type_ {
        wasm_type(ty)
    } else {
        quote!(())
    };
    if return_.has_error {
        return_type = quote!(Result<#return_type, String>)
    }

    quote!(wasmtime::Result<#return_type>)
}

fn gen_fn_body(func: &ApiFunc) -> TokenStream {
    let args_conversion = gen_args_conversion(&func.args);
    let extra_vars_definition = gen_extra_vars_definition(func);
    let call_api_function = gen_call_api_function(func);
    let mut ts = quote! {
        #args_conversion
        #extra_vars_definition
        #call_api_function
    };
    if func.return_.has_error {
        ts.extend([gen_error_return()]);
    }
    ts.extend([gen_return_val_conversion(
        &func.return_,
        func.args.has_arena,
    )]);

    ts
}

fn gen_args_conversion(args: &ApiFuncArgs) -> TokenStream {
    let args_conversion = args.args.iter().map(gen_arg_conversion);
    quote!(#(#args_conversion)*)
}

fn gen_arg_conversion(arg: &ApiArg) -> TokenStream {
    let arg_wit_name = arg.wit_name();
    let arg_name = format_ident!("r#{}", wit_name_to_snake(&arg_wit_name));
    let arg_host_type = host_type(&arg.type_);
    match &arg.type_ {
        ApiType::Keyset(keyset) => {
            let keyset_fields_conversion = gen_keyset_fields_conversion(&arg_wit_name, keyset);
            let keyset_fields_assignment = gen_keyset_fields_assignment(&arg_wit_name, keyset);
            quote! {
                #keyset_fields_conversion
                let mut #arg_name = #arg_host_type {
                    #keyset_fields_assignment
                };
            }
        }
        _ => {
            quote! {
                let #arg_name = <#arg_host_type>::from_wasm_type(#arg_name);
            }
        }
    }
}

fn gen_keyset_fields_conversion(arg_name: &str, keyset: &ApiKeyset) -> TokenStream {
    let conversions = keyset.fields.iter().map(|field| {
        let field_name = format_ident!("r#{}", wit_name_to_snake(&field.wit_name()));
        let field_type = host_type(&ApiType::Object);
        let arg_var_name = format_ident!("r#{}", wit_name_to_snake(arg_name));
        let field_var_name = keyset_field_var_name(arg_name, field);

        quote! {
            let mut #field_var_name = <#field_type>::from_wasm_type(#arg_var_name.#field_name);
        }
    });
    quote! {
        #(#conversions)*
    }
}

fn gen_keyset_fields_assignment(arg_name: &str, keyset: &ApiKeyset) -> TokenStream {
    let assginments = keyset.fields.iter().map(|field| {
        let origin_field_name = format_ident!("r#{}", field.name);
        let field_var_name = keyset_field_var_name(arg_name, field);
        quote! {
            #origin_field_name: #field_var_name.as_mut_borrowed_ffi()
        }
    });
    quote! {
        #(#assginments,)*
    }
}

fn keyset_field_var_name(arg_name: &str, field: &ApiField) -> Ident {
    format_ident!(
        "r#{arg_name}__{field_name}",
        arg_name = wit_name_to_snake(arg_name),
        field_name = wit_name_to_snake(&field.wit_name())
    )
}

fn gen_extra_vars_definition(func: &ApiFunc) -> TokenStream {
    let mut ts = TokenStream::new();
    if func.args.has_arena {
        let arena_var = arena_var_name();
        ts.extend([quote! {
            let mut #arena_var = nvim_rs::arena::NvimArena::new();
        }]);
    }
    if func.return_.has_error {
        let result_var = error_var_name();
        ts.extend([quote! {
            let mut #result_var = nvim_rs::NvimResult::new_ok();
        }]);
    }

    ts
}

fn gen_call_api_function(func: &ApiFunc) -> TokenStream {
    let result_var_name = result_var_name();
    let host_func_name = format_ident!("r#{}", func.name);
    let mut host_args_expression = gen_host_args_expression(&func.args);
    if func.return_.has_error {
        let result_var = error_var_name();
        host_args_expression.extend([quote! {
            #result_var.as_ffi_mut(),
        }])
    }
    quote! {
        let #result_var_name = unsafe {
            nvim_sys::#host_func_name(
                #host_args_expression
            )
        };
    }
}

fn gen_host_args_expression(args: &ApiFuncArgs) -> TokenStream {
    let mut ts = TokenStream::new();
    if args.has_channel_id {
        ts.extend([quote! {
            WASM_INTERNAL_CALL,
        }])
    }
    let args_expression = args.args.iter().map(|arg| {
        let arg_name = format_ident!("r#{}", wit_name_to_snake(&arg.wit_name()));
        use ApiType::*;
        match &arg.type_ {
            Boolean | Integer | Float => {
                quote!(#arg_name,)
            }
            Buffer | Window | Tabpage => {
                quote!(#arg_name.handle() as i32,)
            }
            String | Array(_) | Dictionary(_) | Object => {
                quote!(#arg_name.as_borrowed_ffi(),)
            }
            Keyset(_) => {
                quote!(&mut #arg_name,)
            }
            _ => {
                quote!(todo!(),)
            }
        }
    });
    ts.extend(args_expression);

    if args.has_arena {
        let arena_var = arena_var_name();
        ts.extend([quote! {
            #arena_var.as_ffi_mut(),
        }])
    }

    ts
}

fn gen_error_return() -> TokenStream {
    let error_var = error_var_name();
    quote! {
        if let Err(err) = #error_var.into_result() {
            return Ok(Err(err.to_string()));
        }
    }
}

fn gen_return_val_conversion(return_: &ApiFuncReturn, has_arena: bool) -> TokenStream {
    let Some(rtype) = &return_.type_ else {
        return if return_.has_error { quote!{ Ok(Ok(())) } } else { quote!{ Ok(()) } }
    };

    let return_var = result_var_name();
    let host_type_name = host_type(rtype);
    use ApiType::*;
    let expr = match &rtype {
        Boolean | Integer | Float => {
            quote!(#return_var)
        }
        Buffer | Window | Tabpage => {
            quote!(
                <#host_type_name>::from_handle(#return_var as i64)
            )
        }
        String | Array(_) | Dictionary(_) | Object => {
            if has_arena {
                let arena_var = arena_var_name();
                quote!(
                    unsafe { <#host_type_name>::from_ffi_ref_with_arena(&#return_var, &#arena_var) }
                )
            } else {
                quote! {
                    unsafe { <#host_type_name>::from_ffi(#return_var) }
                }
            }
        }
        _ => {
            quote!(todo!())
        }
    };

    let mut final_expr = quote! { Ok((#expr).try_into_wasm_type()?) };
    if return_.has_error {
        final_expr = quote! { Ok(#final_expr) };
    }

    final_expr
}

fn arena_var_name() -> Ident {
    format_ident!("__api_func_arena_var")
}

fn error_var_name() -> Ident {
    format_ident!("__api_func_error_var")
}

fn result_var_name() -> Ident {
    format_ident!("__api_func_result_var")
}

fn wasm_type(arg: &ApiType) -> TokenStream {
    use ApiType::*;
    match arg {
        Boolean => quote!(bool),
        Integer => quote!(i64),
        Float => quote!(f64),
        String => quote!(String),
        Array(ApiArrayType { inner_type }) => {
            let inner_wasm_type = wasm_type(
                inner_type
                    .as_ref()
                    .map(|x| &**x)
                    .unwrap_or(&ApiType::Object),
            );
            quote!(Vec<#inner_wasm_type>)
        }
        Dictionary(ApiDictionaryType { inner_type }) => {
            let inner_wasm_type = wasm_type(
                inner_type
                    .as_ref()
                    .map(|x| &**x)
                    .unwrap_or(&ApiType::Object),
            );
            quote!(Vec<(String, #inner_wasm_type)>)
        }
        Keyset(keyset) => {
            let keyset_name = format_ident!("r#{}", wit_name_to_camel(&keyset.wit_name()));
            quote!(nvim_keysets::#keyset_name)
        }
        LuaRef => quote!(nvim_types::LuaRef),
        Buffer => quote!(nvim_types::Buffer),
        Window => quote!(nvim_types::Window),
        Tabpage => quote!(nvim_types::Tabpage),
        Object => quote!(nvim_types::Object),
    }
}

fn host_type(arg: &ApiType) -> TokenStream {
    use ApiType::*;
    match arg {
        Boolean => quote!(bool),
        Integer => quote!(i64),
        Float => quote!(f64),
        String => quote!(nvim_rs::NvimString),
        Array(_) => quote!(nvim_rs::NvimArray),
        Dictionary(_) => quote!(nvim_rs::NvimDictionary),
        Keyset(keyset) => {
            let keyset_name = format_ident!("KeyDict_{}", keyset.name);
            quote!(nvim_sys::#keyset_name)
        }
        LuaRef => quote!(nvim_types::LuaRef),
        Buffer => quote!(nvim_rs::NvimBuffer),
        Window => quote!(nvim_rs::NvimWindow),
        Tabpage => quote!(nvim_rs::NvimTabpage),
        Object => quote!(nvim_rs::NvimObject),
    }
}

fn wit_name_to_snake(wit_name: &str) -> String {
    wit_name.replace('-', "_")
}

fn wit_name_to_camel(wit_name: &str) -> String {
    wit_name
        .split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}
