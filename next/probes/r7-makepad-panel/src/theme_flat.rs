//! Ableton の identity は palette ではなく形の文法にある(18テーマ同梱 = 色を全部
//! 差し替えても Ableton に見える、が証拠)。テーマ横断で不変なのは: 矩形のみ・
//! 角丸ゼロ・ベベルゼロ・影ゼロ、分離は 1px の暗線と明度段。
//! makepad の既定(corner_radius 2.5 / beveling 0.75)はその全部に反するので、
//! widget が theme.* を読む**前**に根を書き換える。現場の数百箇所を触らない。
//!
//! `AppMain::script_mod` の中の `script_eval!` でやると main.rs の script_mod 数が
//! 実行時 2 / ファイル 1 になって hot reload の対応付けが壊れる(実測)。独立ファイル。
use makepad_widgets::*;

script_mod! {
    mod.themes.dark.corner_radius = 0.0
    mod.themes.dark.beveling = 0.0
    // 派生値は定義時に計算済みなので、直接使われる物は自分でも潰す
    mod.themes.dark.container_corner_radius = 0.0
    mod.themes.dark.textselection_corner_radius = 0.0
    mod.theme = mod.themes.dark
}
