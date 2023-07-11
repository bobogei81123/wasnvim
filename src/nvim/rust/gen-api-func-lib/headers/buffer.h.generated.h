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
DLLEXPORT Integer nvim_buf_line_count(Buffer buffer, Error *err) FUNC_API_SINCE(1);
DLLEXPORT Boolean nvim_buf_attach(uint64_t channel_id, Buffer buffer, Boolean send_buffer, DictionaryOf(LuaRef) opts, Error *err) FUNC_API_SINCE(4);
DLLEXPORT Boolean nvim_buf_detach(uint64_t channel_id, Buffer buffer, Error *err) FUNC_API_SINCE(4) FUNC_API_REMOTE_ONLY;
DLLEXPORT void nvim__buf_redraw_range(Buffer buffer, Integer first, Integer last, Error *err);
DLLEXPORT ArrayOf(String) nvim_buf_get_lines(uint64_t channel_id, Buffer buffer, Integer start, Integer end, Boolean strict_indexing, lua_State *lstate, Error *err) FUNC_API_SINCE(1);
DLLEXPORT void nvim_buf_set_lines(uint64_t channel_id, Buffer buffer, Integer start, Integer end, Boolean strict_indexing, ArrayOf(String) replacement, Error *err) FUNC_API_SINCE(1) FUNC_API_CHECK_TEXTLOCK;
DLLEXPORT void nvim_buf_set_text(uint64_t channel_id, Buffer buffer, Integer start_row, Integer start_col, Integer end_row, Integer end_col, ArrayOf(String) replacement, Error *err) FUNC_API_SINCE(7);
DLLEXPORT ArrayOf(String) nvim_buf_get_text(uint64_t channel_id, Buffer buffer, Integer start_row, Integer start_col, Integer end_row, Integer end_col, Dictionary opts, lua_State *lstate, Error *err) FUNC_API_SINCE(9);
DLLEXPORT Integer nvim_buf_get_offset(Buffer buffer, Integer index, Error *err) FUNC_API_SINCE(5);
DLLEXPORT Object nvim_buf_get_var(Buffer buffer, String name, Error *err) FUNC_API_SINCE(1);
DLLEXPORT Integer nvim_buf_get_changedtick(Buffer buffer, Error *err) FUNC_API_SINCE(2);
DLLEXPORT ArrayOf(Dictionary) nvim_buf_get_keymap(Buffer buffer, String mode, Error *err) FUNC_API_SINCE(3);
DLLEXPORT void nvim_buf_set_keymap(uint64_t channel_id, Buffer buffer, String mode, String lhs, String rhs, Dict(keymap) *opts, Error *err) FUNC_API_SINCE(6);
DLLEXPORT void nvim_buf_del_keymap(uint64_t channel_id, Buffer buffer, String mode, String lhs, Error *err) FUNC_API_SINCE(6);
DLLEXPORT void nvim_buf_set_var(Buffer buffer, String name, Object value, Error *err) FUNC_API_SINCE(1);
DLLEXPORT void nvim_buf_del_var(Buffer buffer, String name, Error *err) FUNC_API_SINCE(1);
DLLEXPORT String nvim_buf_get_name(Buffer buffer, Arena *arena, Error *err) FUNC_API_SINCE(1);
DLLEXPORT void nvim_buf_set_name(Buffer buffer, String name, Error *err) FUNC_API_SINCE(1);
DLLEXPORT Boolean nvim_buf_is_loaded(Buffer buffer) FUNC_API_SINCE(5);
DLLEXPORT void nvim_buf_delete(Buffer buffer, Dictionary opts, Error *err) FUNC_API_SINCE(7) FUNC_API_CHECK_TEXTLOCK;
DLLEXPORT Boolean nvim_buf_is_valid(Buffer buffer) FUNC_API_SINCE(1);
DLLEXPORT Boolean nvim_buf_del_mark(Buffer buffer, String name, Error *err) FUNC_API_SINCE(8);
DLLEXPORT Boolean nvim_buf_set_mark(Buffer buffer, String name, Integer line, Integer col, Dictionary opts, Error *err) FUNC_API_SINCE(8);
DLLEXPORT ArrayOf(Integer, 2) nvim_buf_get_mark(Buffer buffer, String name, Error *err) FUNC_API_SINCE(1);
DLLEXPORT Object nvim_buf_call(Buffer buffer, LuaRef fun, Error *err) FUNC_API_SINCE(7) FUNC_API_LUA_ONLY;
DLLEXPORT Dictionary nvim__buf_stats(Buffer buffer, Error *err);
DLLEXPORT _Bool buf_collect_lines(buf_T *buf, size_t n, linenr_T start, int start_idx, _Bool replace_nl, Array *l, lua_State *lstate, Error *err);
#include "nvim/func_attr.h"
