#define DEFINE_FUNC_ATTRIBUTES
#include "nvim/func_attr.h"
#undef DEFINE_FUNC_ATTRIBUTES
#ifndef DLLEXPORT
#  ifdef MSWIN
#    define DLLEXPORT __declspec(dllexport)
#  else
#    define DLLEXPORT
#  endif
#endif
DLLEXPORT Integer nvim_wasm_load(String file, Error *error) FUNC_API_SINCE(99);
DLLEXPORT Object nvim_wasm_call_func(Integer instance_id, String func_name, Array args, Error *error) FUNC_API_SINCE(99);
#include "nvim/func_attr.h"
