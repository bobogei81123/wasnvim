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
DLLEXPORT Object nvim_get_option_value(String name, Dict(option) *opts, Error *err) FUNC_API_SINCE(9);
DLLEXPORT void nvim_set_option_value(uint64_t channel_id, String name, Object value, Dict(option) *opts, Error *err) FUNC_API_SINCE(9);
DLLEXPORT Dictionary nvim_get_all_options_info(Error *err) FUNC_API_SINCE(7);
DLLEXPORT Dictionary nvim_get_option_info2(String name, Dict(option) *opts, Error *err) FUNC_API_SINCE(11);
DLLEXPORT getoption_T access_option_value_for(char *key, long *numval, char **stringval, int opt_flags, int opt_type, void *from, _Bool get, Error *err);
#include "nvim/func_attr.h"
