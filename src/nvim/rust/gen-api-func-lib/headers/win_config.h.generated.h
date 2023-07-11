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
DLLEXPORT Window nvim_open_win(Buffer buffer, Boolean enter, Dict(float_config) *config, Error *err) FUNC_API_SINCE(6) FUNC_API_CHECK_TEXTLOCK;
DLLEXPORT void nvim_win_set_config(Window window, Dict(float_config) *config, Error *err) FUNC_API_SINCE(6);
DLLEXPORT Dictionary nvim_win_get_config(Window window, Error *err) FUNC_API_SINCE(6);
#include "nvim/func_attr.h"
