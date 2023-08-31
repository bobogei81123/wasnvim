#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include "nvim/api/private/defs.h"

/**
 * Initializes the Nvim WASM module.
 *
 * This function must be called before any other functions defined in this module.
 *
 * # Panics
 *
 * Panics when failing to create the wasm engine.
 */
void wasm_init(void);

/**
 * Loads the WASM binary to the global store and returns the instance ID.
 *
 * # Safety
 * The `file_path` pointer must be a valid UTF-8 CString.
 */
int32_t wasm_load_file(const char *file_path, const char **errmsg);

/**
 * Calls a function exported by a WASM instance.
 *
 * # Arguments
 * * `instance_id` - The instance ID returned by `wasm_load_file`.
 * * `func_name` - The function name.
 * * `args` - The arguments passed as a Neovim API array.
 * * `errmsg` - If errored, a string describing the error will be stored.
 *
 * # Safety
 * All the pointers argument should be non-null and `errmsg` should point to a valid `Error`
 * struct.
 */
Object wasm_call_func(int32_t instance_id, char *func_name, Array args, const char **errmsg);

/**
 * Calls a WASM callback given the ref.
 *
 * # Arguments
 * * `wasmref` - The WASM ref pointing to the callback. It contains the instance ID and the
 *   callback reference.
 * * `name` - The name of the function or the event triggering the callback. If not null, then an
 *   extra string of this name will be passed as the first argument to the callback.
 * * `args` - The arguments passed as a Neovim API array.
 *
 * # Safety
 * If `name` is not null, it should point to a valid C-string.
 */
void wasm_call_wasmref(WasmRef wasmref, const char *name, Array args);

void api_free_wasmref(WasmRef wasmref);
