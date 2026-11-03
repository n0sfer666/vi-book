#![forbid(unsafe_code)]

pub mod actions;
pub mod app;
mod autocmd;
mod base_keymap;
pub mod buffer;
pub mod buffer_list;
pub mod checktime;
pub mod clip_sync;
pub mod clipboard;
pub mod command_line;
pub mod config_path;
mod core_exec;
pub mod cursor;
pub mod disk_mark;
pub mod disk_message;
pub mod event_map;
pub mod external;
pub mod frame;
pub mod fuzzy;
#[cfg(feature = "highlight")]
pub mod grammar_cli;
pub mod help;
pub mod hscroll;
pub mod input;
mod input_target;
pub mod key_feed;
pub mod launch;
pub mod layout;
pub mod link_follow;
pub mod list_nav;
pub mod list_panel;
#[cfg(feature = "lua")]
pub mod lua_bridge;
#[cfg(feature = "lua")]
mod lua_keymaps;
pub mod map_action;
mod map_exec;
mod map_listing;
pub mod markdown_bridge;
pub mod menu;
mod message_draw;
mod message_input;
mod message_wrap;
pub mod messages;
mod number;
mod options;
pub mod overlay;
pub mod pager;
pub mod pane;
pub mod panel;
pub mod picker;
pub mod read_draw;
pub mod read_motion;
pub mod read_select;
pub mod read_state;
pub mod read_view;
pub mod render;
pub mod requests;
pub mod router;
mod set_exec;
pub mod split;
pub mod status;
pub mod term;
pub mod term_paint;
mod text_draw;
pub mod theme;
pub mod ui_state;
#[cfg(feature = "vault")]
pub mod vault_bridge;
pub mod view;

pub use app::run;
