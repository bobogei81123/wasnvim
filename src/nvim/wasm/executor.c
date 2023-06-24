#include "nvim/wasm/executor.h"

#include "nvim/api/private/defs.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/func_attr.h"
#include "nvim/message.h"
#include "nvim/rust/nvim-wasm/include/wasm-rs.h"

void wasm_init(void)
{
  wasm_rs_init();
}

void ex_wasm(exarg_T *const eap) FUNC_ATTR_NONNULL_ALL
{
  char *file_path = eap->arg;
  const char *errmsg = NULL;
  int32_t instance_id = wasm_load_file(file_path, &errmsg);
  if (instance_id < 0) {
    emsg_multiline(errmsg, /*multiline=*/true);
    return;
  }
  Array empty_arg = ARRAY_DICT_INIT;
  Object result = wasm_call_func(instance_id, "run", empty_arg, &errmsg);
  if (errmsg != NULL) {
    emsg_multiline(errmsg, /*multiline=*/true);
  }
  (void)result;
}
