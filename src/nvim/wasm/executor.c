#include "nvim/ex_cmds_defs.h"
#include "nvim/func_attr.h"
#include "nvim/rust/nvim-wasm/include/wasm-rs.h"
#include "nvim/wasm/executor.h"

void wasm_init(void)
{
  wasm_rs_init();
}

void ex_wasm(exarg_T *const eap) FUNC_ATTR_NONNULL_ALL
{
  char *file_path = eap->arg;
  wasm_rs_run(file_path);
}
