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
DLLEXPORT void remote_ui_disconnect(uint64_t channel_id);
DLLEXPORT void remote_ui_wait_for_attach(void);
DLLEXPORT void nvim_ui_attach(uint64_t channel_id, Integer width, Integer height, Dictionary options, Error *err) FUNC_API_SINCE(1) FUNC_API_REMOTE_ONLY;
DLLEXPORT void ui_attach(uint64_t channel_id, Integer width, Integer height, Boolean enable_rgb, Error *err) FUNC_API_DEPRECATED_SINCE(1);
DLLEXPORT void nvim_ui_set_focus(uint64_t channel_id, Boolean gained, Error *error) FUNC_API_SINCE(11) FUNC_API_REMOTE_ONLY;
DLLEXPORT void nvim_ui_detach(uint64_t channel_id, Error *err) FUNC_API_SINCE(1) FUNC_API_REMOTE_ONLY;
DLLEXPORT void remote_ui_stop(UI *ui);
DLLEXPORT void nvim_ui_try_resize(uint64_t channel_id, Integer width, Integer height, Error *err) FUNC_API_SINCE(1) FUNC_API_REMOTE_ONLY;
DLLEXPORT void nvim_ui_set_option(uint64_t channel_id, String name, Object value, Error *error) FUNC_API_SINCE(1) FUNC_API_REMOTE_ONLY;
DLLEXPORT void nvim_ui_try_resize_grid(uint64_t channel_id, Integer grid, Integer width, Integer height, Error *err) FUNC_API_SINCE(6) FUNC_API_REMOTE_ONLY;
DLLEXPORT void nvim_ui_pum_set_height(uint64_t channel_id, Integer height, Error *err) FUNC_API_SINCE(6) FUNC_API_REMOTE_ONLY;
DLLEXPORT void nvim_ui_pum_set_bounds(uint64_t channel_id, Float width, Float height, Float row, Float col, Error *err) FUNC_API_SINCE(7) FUNC_API_REMOTE_ONLY;
DLLEXPORT void remote_ui_grid_clear(UI *ui, Integer grid);
DLLEXPORT void remote_ui_grid_resize(UI *ui, Integer grid, Integer width, Integer height);
DLLEXPORT void remote_ui_grid_scroll(UI *ui, Integer grid, Integer top, Integer bot, Integer left, Integer right, Integer rows, Integer cols);
DLLEXPORT void remote_ui_default_colors_set(UI *ui, Integer rgb_fg, Integer rgb_bg, Integer rgb_sp, Integer cterm_fg, Integer cterm_bg);
DLLEXPORT void remote_ui_hl_attr_define(UI *ui, Integer id, HlAttrs rgb_attrs, HlAttrs cterm_attrs, Array info);
DLLEXPORT void remote_ui_highlight_set(UI *ui, int id);
DLLEXPORT void remote_ui_grid_cursor_goto(UI *ui, Integer grid, Integer row, Integer col);
DLLEXPORT void remote_ui_cursor_goto(UI *ui, Integer row, Integer col);
DLLEXPORT void remote_ui_put(UI *ui, const char *cell);
DLLEXPORT void remote_ui_raw_line(UI *ui, Integer grid, Integer row, Integer startcol, Integer endcol, Integer clearcol, Integer clearattr, LineFlags flags, const schar_T *chunk, const sattr_T *attrs);
DLLEXPORT void remote_ui_flush_buf(UI *ui);
DLLEXPORT void remote_ui_flush(UI *ui);
DLLEXPORT void remote_ui_event(UI *ui, char *name, Array args);
DLLEXPORT void remote_ui_inspect(UI *ui, Dictionary *info);
#include "nvim/func_attr.h"
