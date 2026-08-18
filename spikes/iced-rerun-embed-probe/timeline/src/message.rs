//! タイムラインが外に出す **意図(intent)** の一覧。
//!
//! ここが spike の主題である。方針をひとつ決めて、それを最後まで守った:
//!
//! > canvas は「起きた入力」ではなく「利用者が何をしようとしたか」を出す。
//!
//! つまり `MousePressed { x, y }` のような生イベント中継は1つも置かない。
//! hit test は canvas 側(`Program::update` = イベントパス)で済ませ、
//! ここに来る頃には「clip 3 の右端を掴んだ」まで解決されている。
//!
//! この縛りを置いた理由: 生イベントを message にすると、`update()` の中で
//! 「今どのモードか」を分岐し直すことになり、egui の if 沼が場所を変えて
//! 再現するだけになる。intent まで解決してから渡せば、`update()` は
//! **モードごとに1本の腕**しか持たなくて済む。

use iced::keyboard::Modifiers;
use iced::time::Instant;

use crate::model::{ClipId, Grab};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimelineMsg {
    // ── ポインタ ───────────────────────────────────────────────────────
    /// clip を掴んだ。`grab` で移動か trim かが既に決まっている。
    /// `at_frame` は**スナップ前**の位置(掴んだ点と clip 頭のズレを保つのに要る)。
    ClipGrabbed {
        id: ClipId,
        grab: Grab,
        at_frame: f64,
        /// Cmd 併用 = 選択に足す。
        additive: bool,
    },

    /// 何も無い所を押した = 選択解除(Cmd 併用なら何もしない)。
    EmptyPressed { additive: bool },

    /// ruler を押した = スクラブ開始。
    ScrubStarted { frame: f64 },

    /// ドラッグ中の移動。canvas は**ジェスチャ中しかこれを出さない**
    /// (ただのホバー移動では出さない)。
    PointerMoved { frame: f64 },

    /// ボタンを離した = 進行中ジェスチャの確定。
    PointerReleased,

    // ── ホイール(面の割り当ては AE/Premiere 同型)─────────────────────
    /// Cmd+ホイール = 横 zoom。`anchor_x` の下のフレームが動かない。
    ZoomedAt { anchor_x: f32, notches: f32 },
    /// Shift+ホイール = 横パン。
    PannedX { notches: f32 },
    /// 素のホイール = 縦スクロール。
    ScrolledY { notches: f32 },

    // ── キーボード ─────────────────────────────────────────────────────
    /// ←→ = ±1、Shift+←→ = ±10。製品 F-04 と同じ意味。
    Nudged { frames: i64 },

    // ── ランタイムから拾うしかなかったもの ─────────────────────────────
    /// 修飾キーの状態。**`mouse::Event::WheelScrolled` が modifiers を運ばない**ので、
    /// これを別経路で追いかけて model に持たないと Cmd+ホイールが書けない。
    /// (詰まった箇所その1。詳細は README。)
    ModifiersChanged(Modifiers),

    /// 計測用。`window::frames()` が返す描画時刻。
    Rendered(Instant),
}
