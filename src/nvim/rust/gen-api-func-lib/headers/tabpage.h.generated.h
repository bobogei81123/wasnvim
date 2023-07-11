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
DLLEXPORT ArrayOf(Window) nvim_tabpage_list_wins(Tabpage tabpage, Error *err) FUNC_API_SINCE(1);
DLLEXPORT Object nvim_tabpage_get_var(Tabpage tabpage, String name, Error *err) FUNC_API_SINCE(1);
DLLEXPORT void nvim_tabpage_set_var(Tabpage tabpage, String name, Object value, Error *err) FUNC_API_SINCE(1);
DLLEXPORT void nvim_tabpage_del_var(Tabpage tabpage, String name, Error *err) FUNC_API_SINCE(1);
DLLEXPORT Window nvim_tabpage_get_win(Tabpage tabpage, Error *err) FUNC_API_SINCE(1);
DLLEXPORT Integer nvim_tabpage_get_number(Tabpage tabpage, Error *err) FUNC_API_SINCE(1);
DLLEXPORT Boolean nvim_tabpage_is_valid(Tabpage tabpage) FUNC_API_SINCE(1);
#include "nvim/func_attr.h"
