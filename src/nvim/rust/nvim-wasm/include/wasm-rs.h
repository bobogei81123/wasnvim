#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include "nvim/api/private/defs.h"
#include "nvim/types.h"

/**
 * Initializes the Nvim WASM module.
 *
 * This function must be called before any other functions defined in this module.
 *
 * # Panics
 *
 * Panics when failing to create the wasm engine.
 */
void wasm_rs_init(void);

/**
 * Loads the WASM binary to the global store and returns the instance ID.
 *
 * # Safety
 *
 * The `file_path` pointer must be a valid UTF-8 CString.
 */
int32_t wasm_load_file(const char *file_path, const char **errmsg);

/**
 * Calls a function from a WASM instance
 */
Object wasm_call_func(int32_t instance_id, const char *func_name, Array args, const char **errmsg);

void wasm_call_wasmref(WasmRef wasmref);

void api_free_wasmref(WasmRef wasmref);
