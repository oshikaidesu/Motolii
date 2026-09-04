//! 右クリック menu。文脈操作の逃げ道で、中身は打鍵で既に撃てる `Intent` の再掲
//! (`docs/ui-inherited-grammar-gap.md` Tier 0「全面に同じ献立を出す」)。
//!
//! 献立は Document を変えない純関数([`entries`])で、面(Timeline/Stage/層行)は
//! **どこで右クリックされたか**だけを [`MenuRequest`] に書く。実行は打鍵と同じ
//! [`crate::dispatch::run_intent`] を通るので、menu 専用の書き込み経路は無い。
//!
//! Blitz は DOM 要素へ `contextmenu` を配るが、custom widget(Timeline 帯・Stage)には
//! 右ボタンの `PointerDown` しか届かない。そのため widget 側は Signal(`menu`)へ
//! 要求を書き、chrome がそれを読んで描く。

use dioxus_native::prelude::*;
use motolii_store::LayerId;

use crate::dispatch::{run_intent, IntentCtx};
use crate::keymap::Intent;

/// 右クリックされた文脈。層の上なら `Layer`(選択は要求を出す側が済ませておく)。
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MenuTarget {
    Layer(LayerId),
    Timeline,
    Stage,
}

/// menu を開く要求。`x`/`y` は client 座標(`#app` は 100vh・overflow hidden なので
/// document 原点と一致し、`position:fixed` でそのまま置ける)。
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct MenuRequest {
    pub x: f64,
    pub y: f64,
    pub target: MenuTarget,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Entry {
    Item { label: &'static str, shortcut: &'static str, intent: Intent },
    Separator,
}

const fn item(label: &'static str, shortcut: &'static str, intent: Intent) -> Entry {
    Entry::Item { label, shortcut, intent }
}

/// 文脈ごとの献立。打鍵に無い verb はここにも置かない(menu だけの経路を作らない)。
pub fn entries(target: MenuTarget) -> Vec<Entry> {
    match target {
        MenuTarget::Layer(_) => vec![
            item("Split at Playhead", "⌘K", Intent::Split),
            item("Duplicate", "⌘D", Intent::Duplicate),
            item("Delete", "⌫", Intent::Delete),
            Entry::Separator,
            item("Select All", "⌘A", Intent::SelectAll),
            item("Deselect", "Esc", Intent::Deselect),
        ],
        MenuTarget::Timeline | MenuTarget::Stage => vec![
            item("Play / Pause", "Space", Intent::PlayPause),
            item("Go to Start", "Home", Intent::Home),
            item("Go to End", "End", Intent::End),
            Entry::Separator,
            item("Select All", "⌘A", Intent::SelectAll),
        ],
    }
}

/// 開いていれば backdrop と menu を描く。閉じるのは backdrop クリック・項目実行・Esc
/// (Esc は `app.rs` の keydown が `menu` を見て閉じる)。
pub fn context_menu(mut menu: Signal<Option<MenuRequest>>, ctx: IntentCtx) -> Element {
    let Some(req) = menu() else {
        return rsx!();
    };
    let items = entries(req.target).into_iter().enumerate().map(|(i, entry)| match entry {
        Entry::Separator => rsx!(div { key: "{i}", class: "msep" }),
        Entry::Item { label, shortcut, intent } => {
            let ctx = ctx.clone();
            rsx!(
                div {
                    key: "{i}",
                    class: "mi",
                    onclick: move |_| {
                        println!("PROBE room=input verdict=context-menu item={label:?}");
                        menu.set(None);
                        run_intent(&ctx, intent);
                    },
                    span { class: "ml", "{label}" }
                    span { class: "ms", "{shortcut}" }
                }
            )
        }
    });
    rsx!(
        div {
            id: "ctxbackdrop",
            onclick: move |_| menu.set(None),
            oncontextmenu: move |evt| {
                evt.prevent_default();
                menu.set(None);
            },
        }
        div {
            id: "ctxmenu",
            style: "left:{req.x}px;top:{req.y}px;",
            {items}
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn intents(target: MenuTarget) -> Vec<Intent> {
        entries(target)
            .into_iter()
            .filter_map(|e| match e {
                Entry::Item { intent, .. } => Some(intent),
                Entry::Separator => None,
            })
            .collect()
    }

    #[test]
    fn layer_menu_offers_split_duplicate_delete() {
        let got = intents(MenuTarget::Layer(LayerId(1)));
        assert!(got.contains(&Intent::Split));
        assert!(got.contains(&Intent::Duplicate));
        assert!(got.contains(&Intent::Delete));
    }

    #[test]
    fn empty_surface_never_offers_layer_verbs() {
        for target in [MenuTarget::Timeline, MenuTarget::Stage] {
            let got = intents(target);
            assert!(!got.contains(&Intent::Split));
            assert!(!got.contains(&Intent::Duplicate));
            assert!(!got.contains(&Intent::Delete));
            assert!(got.contains(&Intent::SelectAll));
        }
    }

    /// 献立は打鍵の再掲: menu の各項目は keymap に同じ Intent の binding を持つ。
    #[test]
    fn every_item_is_reachable_from_the_keymap() {
        for target in [MenuTarget::Layer(LayerId(1)), MenuTarget::Timeline, MenuTarget::Stage] {
            for intent in intents(target) {
                assert!(
                    crate::keymap::is_bound(intent),
                    "{intent:?} is in the menu but has no key binding"
                );
            }
        }
    }

    #[test]
    fn no_leading_trailing_or_doubled_separators() {
        for target in [MenuTarget::Layer(LayerId(1)), MenuTarget::Timeline, MenuTarget::Stage] {
            let es = entries(target);
            assert!(!matches!(es.first(), Some(Entry::Separator)));
            assert!(!matches!(es.last(), Some(Entry::Separator)));
            assert!(!es.windows(2).any(|w| w[0] == Entry::Separator && w[1] == Entry::Separator));
        }
    }
}
