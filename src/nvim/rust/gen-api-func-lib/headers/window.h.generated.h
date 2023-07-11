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
DLLEXPORT Buffer nvim_win_get_buf(Window window, Error *err) FUNC_API_SINCE(1);
DLLEXPORT void nvim_win_set_buf(Window window, Buffer buffer, Error *err) FUNC_API_SINCE(5) FUNC_API_CHECK_TEXTLOCK;
DLLEXPORT ArrayOf(Integer, 2) nvim_win_get_cursor(Window window, Error *err) FUNC_API_SINCE(1);
DLLEXPORT void nvim_win_set_cursor(Window window, ArrayOf(Integer, 2) pos, Error *err) FUNC_API_SINCE(1);
DLLEXPORT Integer nvim_win_get_height(Window window, Error *err) FUNC_API_SINCE(1);
DLLEXPORT void nvim_win_set_height(Window window, Integer height, Error *err) FUNC_API_SINCE(1);
DLLEXPORT Integer nvim_win_get_width(Window window, Error *err) FUNC_API_SINCE(1);
DLLEXPORT void nvim_win_set_width(Window window, Integer width, Error *err) FUNC_API_SINCE(1);
DLLEXPORT Object nvim_win_get_var(Window window, String name, Error *err) FUNC_API_SINCE(1);
DLLEXPORT void nvim_win_set_var(Window window, String name, Object value, Error *err) FUNC_API_SINCE(1);
DLLEXPORT void nvim_win_del_var(Window window, String name, Error *err) FUNC_API_SINCE(1);
DLLEXPORT ArrayOf(Integer, 2) nvim_win_get_position(Window window, Error *err) FUNC_API_SINCE(1);
DLLEXPORT Tabpage nvim_win_get_tabpage(Window window, Error *err) FUNC_API_SINCE(1);
DLLEXPORT Integer nvim_win_get_number(Window window, Error *err) FUNC_API_SINCE(1);
DLLEXPORT Boolean nvim_win_is_valid(Window window) FUNC_API_SINCE(1);
DLLEXPORT void nvim_win_hide(Window window, Error *err) FUNC_API_SINCE(7) FUNC_API_CHECK_TEXTLOCK;
DLLEXPORT void nvim_win_close(Window window, Boolean force, Error *err) FUNC_API_SINCE(6) FUNC_API_CHECK_TEXTLOCK;
DLLEXPORT Object nvim_win_call(Window window, LuaRef fun, Error *err) FUNC_API_SINCE(7) FUNC_API_LUA_ONLY;
DLLEXPORT void nvim_win_set_hl_ns(Window window, Integer ns_id, Error *err) FUNC_API_SINCE(10);
#include "nvim/func_attr.h"
