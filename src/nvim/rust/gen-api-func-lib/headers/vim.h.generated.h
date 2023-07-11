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
DLLEXPORT Integer nvim_get_hl_id_by_name(String name) FUNC_API_SINCE(7);
DLLEXPORT Dictionary nvim_get_hl(Integer ns_id, Dict(get_highlight) *opts, Arena *arena, Error *err) FUNC_API_SINCE(11);
DLLEXPORT void nvim_set_hl(Integer ns_id, String name, Dict(highlight) *val, Error *err) FUNC_API_SINCE(7);
DLLEXPORT void nvim_set_hl_ns(Integer ns_id, Error *err) FUNC_API_SINCE(10);
DLLEXPORT void nvim_set_hl_ns_fast(Integer ns_id, Error *err) FUNC_API_SINCE(10) FUNC_API_FAST;
DLLEXPORT void nvim_feedkeys(String keys, String mode, Boolean escape_ks) FUNC_API_SINCE(1);
DLLEXPORT Integer nvim_input(String keys) FUNC_API_SINCE(1) FUNC_API_FAST;
DLLEXPORT void nvim_input_mouse(String button, String action, String modifier, Integer grid, Integer row, Integer col, Error *err) FUNC_API_SINCE(6) FUNC_API_FAST;
DLLEXPORT String nvim_replace_termcodes(String str, Boolean from_part, Boolean do_lt, Boolean special) FUNC_API_SINCE(1);
DLLEXPORT Object nvim_exec_lua(String code, Array args, Error *err) FUNC_API_SINCE(7) FUNC_API_REMOTE_ONLY;
DLLEXPORT Object nvim_notify(String msg, Integer log_level, Dictionary opts, Error *err) FUNC_API_SINCE(7);
DLLEXPORT Integer nvim_strwidth(String text, Error *err) FUNC_API_SINCE(1);
DLLEXPORT ArrayOf(String) nvim_list_runtime_paths(Error *err) FUNC_API_SINCE(1);
DLLEXPORT Array nvim__runtime_inspect(void);
DLLEXPORT ArrayOf(String) nvim_get_runtime_file(String name, Boolean all, Error *err) FUNC_API_SINCE(7) FUNC_API_FAST;
DLLEXPORT String nvim__get_lib_dir(void);
DLLEXPORT ArrayOf(String) nvim__get_runtime(Array pat, Boolean all, Dict(runtime) *opts, Error *err) FUNC_API_SINCE(8) FUNC_API_FAST;
DLLEXPORT void nvim_set_current_dir(String dir, Error *err) FUNC_API_SINCE(1);
DLLEXPORT String nvim_get_current_line(Error *err) FUNC_API_SINCE(1);
DLLEXPORT void nvim_set_current_line(String line, Error *err) FUNC_API_SINCE(1) FUNC_API_CHECK_TEXTLOCK;
DLLEXPORT void nvim_del_current_line(Error *err) FUNC_API_SINCE(1) FUNC_API_CHECK_TEXTLOCK;
DLLEXPORT Object nvim_get_var(String name, Error *err) FUNC_API_SINCE(1);
DLLEXPORT void nvim_set_var(String name, Object value, Error *err) FUNC_API_SINCE(1);
DLLEXPORT void nvim_del_var(String name, Error *err) FUNC_API_SINCE(1);
DLLEXPORT Object nvim_get_vvar(String name, Error *err) FUNC_API_SINCE(1);
DLLEXPORT void nvim_set_vvar(String name, Object value, Error *err) FUNC_API_SINCE(6);
DLLEXPORT void nvim_echo(Array chunks, Boolean history, Dict(echo_opts) *opts, Error *err) FUNC_API_SINCE(7);
DLLEXPORT void nvim_out_write(String str) FUNC_API_SINCE(1);
DLLEXPORT void nvim_err_write(String str) FUNC_API_SINCE(1);
DLLEXPORT void nvim_err_writeln(String str) FUNC_API_SINCE(1);
DLLEXPORT ArrayOf(Buffer) nvim_list_bufs(void) FUNC_API_SINCE(1);
DLLEXPORT Buffer nvim_get_current_buf(void) FUNC_API_SINCE(1);
DLLEXPORT void nvim_set_current_buf(Buffer buffer, Error *err) FUNC_API_SINCE(1) FUNC_API_CHECK_TEXTLOCK;
DLLEXPORT ArrayOf(Window) nvim_list_wins(void) FUNC_API_SINCE(1);
DLLEXPORT Window nvim_get_current_win(void) FUNC_API_SINCE(1);
DLLEXPORT void nvim_set_current_win(Window window, Error *err) FUNC_API_SINCE(1) FUNC_API_CHECK_TEXTLOCK;
DLLEXPORT Buffer nvim_create_buf(Boolean listed, Boolean scratch, Error *err) FUNC_API_SINCE(6);
DLLEXPORT Integer nvim_open_term(Buffer buffer, DictionaryOf(LuaRef) opts, Error *err) FUNC_API_SINCE(7);
DLLEXPORT void nvim_chan_send(Integer chan, String data, Error *err) FUNC_API_SINCE(7) FUNC_API_REMOTE_ONLY FUNC_API_LUA_ONLY;
DLLEXPORT ArrayOf(Tabpage) nvim_list_tabpages(void) FUNC_API_SINCE(1);
DLLEXPORT Tabpage nvim_get_current_tabpage(void) FUNC_API_SINCE(1);
DLLEXPORT void nvim_set_current_tabpage(Tabpage tabpage, Error *err) FUNC_API_SINCE(1) FUNC_API_CHECK_TEXTLOCK;
DLLEXPORT Boolean nvim_paste(String data, Boolean crlf, Integer phase, Error *err) FUNC_API_SINCE(6) FUNC_API_CHECK_TEXTLOCK;
DLLEXPORT void nvim_put(ArrayOf(String) lines, String type, Boolean after, Boolean follow, Error *err) FUNC_API_SINCE(6) FUNC_API_CHECK_TEXTLOCK;
DLLEXPORT void nvim_subscribe(uint64_t channel_id, String event) FUNC_API_SINCE(1) FUNC_API_REMOTE_ONLY;
DLLEXPORT void nvim_unsubscribe(uint64_t channel_id, String event) FUNC_API_SINCE(1) FUNC_API_REMOTE_ONLY;
DLLEXPORT Integer nvim_get_color_by_name(String name) FUNC_API_SINCE(1);
DLLEXPORT Dictionary nvim_get_color_map(void) FUNC_API_SINCE(1);
DLLEXPORT Dictionary nvim_get_context(Dict(context) *opts, Error *err) FUNC_API_SINCE(6);
DLLEXPORT Object nvim_load_context(Dictionary dict) FUNC_API_SINCE(6);
DLLEXPORT Dictionary nvim_get_mode(void) FUNC_API_SINCE(2) FUNC_API_FAST;
DLLEXPORT ArrayOf(Dictionary) nvim_get_keymap(String mode) FUNC_API_SINCE(3);
DLLEXPORT void nvim_set_keymap(uint64_t channel_id, String mode, String lhs, String rhs, Dict(keymap) *opts, Error *err) FUNC_API_SINCE(6);
DLLEXPORT void nvim_del_keymap(uint64_t channel_id, String mode, String lhs, Error *err) FUNC_API_SINCE(6);
DLLEXPORT Array nvim_get_api_info(uint64_t channel_id, Arena *arena) FUNC_API_SINCE(1) FUNC_API_FAST FUNC_API_REMOTE_ONLY;
DLLEXPORT void nvim_set_client_info(uint64_t channel_id, String name, Dictionary version, String type, Dictionary methods, Dictionary attributes, Error *err) FUNC_API_SINCE(4) FUNC_API_REMOTE_ONLY;
DLLEXPORT Dictionary nvim_get_chan_info(Integer chan, Error *err) FUNC_API_SINCE(4);
DLLEXPORT Array nvim_list_chans(void) FUNC_API_SINCE(4);
DLLEXPORT Array nvim_call_atomic(uint64_t channel_id, Array calls, Arena *arena, Error *err) FUNC_API_SINCE(1) FUNC_API_REMOTE_ONLY;
DLLEXPORT Object nvim__id(Object obj);
DLLEXPORT Array nvim__id_array(Array arr);
DLLEXPORT Dictionary nvim__id_dictionary(Dictionary dct);
DLLEXPORT Float nvim__id_float(Float flt);
DLLEXPORT Dictionary nvim__stats(void);
DLLEXPORT Array nvim_list_uis(void) FUNC_API_SINCE(4);
DLLEXPORT Array nvim_get_proc_children(Integer pid, Error *err) FUNC_API_SINCE(4);
DLLEXPORT Object nvim_get_proc(Integer pid, Error *err) FUNC_API_SINCE(4);
DLLEXPORT void nvim_select_popupmenu_item(Integer item, Boolean insert, Boolean finish, Dictionary opts, Error *err) FUNC_API_SINCE(6);
DLLEXPORT Array nvim__inspect_cell(Integer grid, Integer row, Integer col, Arena *arena, Error *err);
DLLEXPORT void nvim__screenshot(String path) FUNC_API_FAST;
DLLEXPORT Object nvim__unpack(String str, Error *err) FUNC_API_FAST;
DLLEXPORT Boolean nvim_del_mark(String name, Error *err) FUNC_API_SINCE(8);
DLLEXPORT Array nvim_get_mark(String name, Dictionary opts, Error *err) FUNC_API_SINCE(8);
DLLEXPORT Dictionary nvim_eval_statusline(String str, Dict(eval_statusline) *opts, Error *err) FUNC_API_SINCE(8) FUNC_API_FAST;
DLLEXPORT void nvim_error_event(uint64_t channel_id, Integer lvl, String data) FUNC_API_REMOTE_ONLY;
#include "nvim/func_attr.h"
