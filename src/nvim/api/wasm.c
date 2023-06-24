#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/rust/nvim-wasm/include/wasm-rs.h"

#ifdef INCLUDE_GENERATED_DECLARATIONS
# include "api/autocmd.c.generated.h"
#endif

Integer nvim_wasm_load(String file, Error *error) FUNC_API_SINCE(99)
{
  const char *errmsg = NULL;
  int32_t instance_id = wasm_load_file(file.data, &errmsg);
  if (instance_id < 0) {
    api_set_error(error, kErrorTypeException, "%s", errmsg);
  }
  return instance_id;
}

Object nvim_wasm_call_func(Integer instance_id, String func_name, Array args, Error *error)
  FUNC_API_SINCE(99)
{
  const char *errmsg = NULL;
  Object result = wasm_call_func(instance_id, func_name.data, args, &errmsg);
  if (errmsg != NULL) {
    api_set_error(error, kErrorTypeException, "%s", errmsg);
  }
  return result;
}
