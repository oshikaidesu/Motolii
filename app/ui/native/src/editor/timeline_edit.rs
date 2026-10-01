//! 層をいじる手。責任は 3 つ — 写して貼る・割る・キーを消す。
//! ここは口だけを並べる。

mod copy_paste;
mod key_delete;
mod split;

pub(crate) use copy_paste::{
    copy_layers, duplicate_layers, ghostable, paste_layers, remap_clipboard_intent, LayerClipboard,
    GHOST_DEFAULT_DELAY,
};
pub(crate) use key_delete::{delete_key_selection_intents, selected_properties, selects_content};
pub(crate) use split::split_layers;

#[cfg(test)]
mod hierarchy_clipboard_regressions;
