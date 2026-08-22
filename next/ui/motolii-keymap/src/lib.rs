//! owns: keymap 層 — (キー+修飾キー+文脈)→動詞id の対応表・解決器・構築時
//! 衝突検出。上流に無い(iced はキーイベントを配るだけで、割当を意味づける層は
//! アプリ側の責務)。発注書「キーは仮置き(keymap 層未実装)」—
//! `next/reference/timeline-grammar.md` 拘束6の実体化。
//!
//! ## 何を建てたか
//!
//! これまで **ショートカットは `next/shell/motolii-shell/src/lib.rs` の match
//! 文へ直書き**されており(`resolve_navigation_key`・`inspector_pointer_event`)、
//! (a) map(`reference/timeline-grammar.md` §8)のショートカット行が消化できない
//! (b) 利用者がキーを変更できない (c) shortcut 併記が `menu.rs` の手書き文字列
//! だけで実態と同期する保証が無い、という欠陥が生まれていた(発注書「背景」)。
//!
//! この crate は (キー+修飾キー+文脈) → **動詞 id**([`VerbId`])の対応を
//! **データ**([`Binding`]・[`Keymap`])として持つ。**この波では crate を建てる
//! だけ**——`motolii-shell` の実際の match 文を書き換えて配線するのは次切片
//! (発注書 EXACT TARGET)。
//!
//! ## 動詞 id について(重要な逸脱)
//!
//! 発注書は「`motolii-verbs`(37動詞のレジストリが既にある)と繋ぐのが本筋」と
//! 指示していたが、**`next/ui/motolii-verbs/` はこの回収の時点でリポジトリの
//! どこにも存在しない**(`Verb`/`SHORTCUT_ONLY_REGISTRY` を全文検索してもヒット
//! ゼロ)。同様に `next/ui/motolii-menubar/src/menus.rs`/`context.rs` も存在せず、
//! 実際にメニューの意味定義を持つのは `next/shell/motolii-shell/src/menu.rs`
//! (`motolii_menubar::{Item, Menu}` を使う側)だった。**存在しない物を読めと
//! 指示された**——これ自体がこの発注が問題視する「実態が無いまま周りだけ
//! 書かれる」パターンの再発なので、RETURN で正直に報告する。
//!
//! 対処として、この crate 自身が孤立した [`VerbId`] enum を正本として持つ
//! (`motolii_store`/`motolii_shell` のどちらにも依存しない)。将来
//! `motolii-verbs` が実在した時点で対応表を1つ足すだけで済む形にしてある
//! ([`verb`] モジュール冒頭 doc 参照)。
//!
//! ## 構成
//!
//! - [`key`]: `Key`/`Modifiers`/`ModifierSpec` — iced に依存しない中立表現
//! - [`verb`]: [`VerbId`] — 動詞 id(現行 shell に実装済みの物だけ)
//! - [`binding`]: [`Binding`]/[`Scope`]/[`Keymap`] — 対応表・解決器・衝突検出
//! - [`defaults`]: [`default_bindings`]/[`default_keymap`] — 現行 shell の
//!   割当をそのまま写した既定表

mod binding;
mod defaults;
mod key;
mod verb;

pub use binding::{Binding, Conflict, Keymap, Scope};
pub use defaults::{default_bindings, default_keymap, global_bindings, nav_bundle_bindings, nav_bundle_keymap};
pub use key::{Key, ModifierSpec, Modifiers, NamedKey, Requirement};
pub use verb::VerbId;
