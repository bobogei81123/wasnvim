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
DLLEXPORT Dictionary nvim_parse_cmd(String str, Dictionary opts, Error *err) FUNC_API_SINCE(10) FUNC_API_FAST;
DLLEXPORT String nvim_cmd(uint64_t channel_id, Dict(cmd) *cmd, Dict(cmd_opts) *opts, Error *err) FUNC_API_SINCE(10);
DLLEXPORT void nvim_create_user_command(uint64_t channel_id, String name, Object command, Dict(user_command) *opts, Error *err) FUNC_API_SINCE(9);
DLLEXPORT void nvim_del_user_command(String name, Error *err) FUNC_API_SINCE(9);
DLLEXPORT void nvim_buf_create_user_command(uint64_t channel_id, Buffer buffer, String name, Object command, Dict(user_command) *opts, Error *err) FUNC_API_SINCE(9);
DLLEXPORT void nvim_buf_del_user_command(Buffer buffer, String name, Error *err) FUNC_API_SINCE(9);
DLLEXPORT void create_user_command(uint64_t channel_id, String name, Object command, Dict(user_command) *opts, int flags, Error *err);
DLLEXPORT Dictionary nvim_get_commands(Dict(get_commands) *opts, Error *err) FUNC_API_SINCE(4);
DLLEXPORT Dictionary nvim_buf_get_commands(Buffer buffer, Dict(get_commands) *opts, Error *err) FUNC_API_SINCE(4);
#include "nvim/func_attr.h"
