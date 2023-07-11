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
DLLEXPORT void api_extmark_free_all_mem(void);
DLLEXPORT Integer nvim_create_namespace(String name) FUNC_API_SINCE(5);
DLLEXPORT Dictionary nvim_get_namespaces(void) FUNC_API_SINCE(5);
DLLEXPORT const char *describe_ns(NS ns_id);
DLLEXPORT _Bool ns_initialized(uint32_t ns);
DLLEXPORT ArrayOf(Integer) nvim_buf_get_extmark_by_id(Buffer buffer, Integer ns_id, Integer id, Dictionary opts, Error *err) FUNC_API_SINCE(7);
DLLEXPORT Array nvim_buf_get_extmarks(Buffer buffer, Integer ns_id, Object start, Object end, Dictionary opts, Error *err) FUNC_API_SINCE(7);
DLLEXPORT Integer nvim_buf_set_extmark(Buffer buffer, Integer ns_id, Integer line, Integer col, Dict(set_extmark) *opts, Error *err) FUNC_API_SINCE(7);
DLLEXPORT Boolean nvim_buf_del_extmark(Buffer buffer, Integer ns_id, Integer id, Error *err) FUNC_API_SINCE(7);
DLLEXPORT uint32_t src2ns(Integer *src_id);
DLLEXPORT Integer nvim_buf_add_highlight(Buffer buffer, Integer ns_id, String hl_group, Integer line, Integer col_start, Integer col_end, Error *err) FUNC_API_SINCE(1);
DLLEXPORT void nvim_buf_clear_namespace(Buffer buffer, Integer ns_id, Integer line_start, Integer line_end, Error *err) FUNC_API_SINCE(5);
DLLEXPORT void nvim_set_decoration_provider(Integer ns_id, Dict(set_decoration_provider) *opts, Error *err) FUNC_API_SINCE(7) FUNC_API_LUA_ONLY;
DLLEXPORT VirtText parse_virt_text(Array chunks, Error *err, int *width);
#include "nvim/func_attr.h"
