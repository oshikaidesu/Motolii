//! Stage 描画の計測器具。**debug ビルドのみ**(release は呼び出し側ごと
//! `#[cfg(debug_assertions)]` で消える — 製品経路に一切残らない)。
//!
//! 実機チラつき調査(2026-08-20)の柵として追加。数える対象は3つだけ:
//! - `HANDLE_CREATIONS`: `image::Handle::from_rgba` を呼んだ回数。iced の
//!   `Handle::from_rgba` は呼ぶたび `Id::unique()` で新しい id を割る
//!   (`iced_core-0.14.0/src/image.rs` 実測 — `next/` の実際の依存は
//!   crates.io の `iced 0.14.0` であって fork ではない)ので、ここが増えた
//!   回数がそのまま「新しいテクスチャとして扱われた回数」になる。
//! - `LAST_HANDLE_BYTES`: 直近で Handle を作った RGBA の byte 数。iced の
//!   wgpu backend は `MAX_SYNC_SIZE = 2 * 1024 * 1024` を境に、それ以上は
//!   バックグラウンドスレッドへ非同期アップロードへ回す
//!   (`iced_wgpu-0.14.0/src/image/cache.rs::upload_raster` 実測)。非同期の間、
//!   `draw_image` はその frame 何も描かない(同 crate `iced_core-0.14.0` の
//!   `Allocation` doc comment: 「アニメさせると望ましくないチラつきが起きる」と
//!   明記)— これが実測できたチラつきの一次原因。
//! - `RENDER_FRAME_CALLS` / `RENDER_FRAME_NANOS`: `Engine::render_frame` の
//!   呼び出し回数と合計時間。CPU 合成がボトルネックかどうかを実測で切り分ける。

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

static HANDLE_CREATIONS: AtomicU64 = AtomicU64::new(0);
static LAST_HANDLE_BYTES: AtomicUsize = AtomicUsize::new(0);
static RENDER_FRAME_CALLS: AtomicU64 = AtomicU64::new(0);
static RENDER_FRAME_NANOS: AtomicU64 = AtomicU64::new(0);
static TOKENS_RELOADS: AtomicU64 = AtomicU64::new(0);

pub fn record_handle_creation(bytes: usize) {
    HANDLE_CREATIONS.fetch_add(1, Ordering::Relaxed);
    LAST_HANDLE_BYTES.store(bytes, Ordering::Relaxed);
}

pub fn record_render_frame(elapsed: std::time::Duration) {
    RENDER_FRAME_CALLS.fetch_add(1, Ordering::Relaxed);
    RENDER_FRAME_NANOS.fetch_add(elapsed.as_nanos() as u64, Ordering::Relaxed);
}

pub fn record_tokens_reload() {
    TOKENS_RELOADS.fetch_add(1, Ordering::Relaxed);
}

pub fn handle_creations() -> u64 {
    HANDLE_CREATIONS.load(Ordering::Relaxed)
}

pub fn last_handle_bytes() -> usize {
    LAST_HANDLE_BYTES.load(Ordering::Relaxed)
}

pub fn render_frame_calls() -> u64 {
    RENDER_FRAME_CALLS.load(Ordering::Relaxed)
}

pub fn render_frame_nanos() -> u64 {
    RENDER_FRAME_NANOS.load(Ordering::Relaxed)
}

pub fn tokens_reloads() -> u64 {
    TOKENS_RELOADS.load(Ordering::Relaxed)
}

/// テストの前で計測をゼロへ戻す。**プロセス全体で共有**(static)なので、
/// 同一プロセス内の他テストの計測と混ざらないよう、計る側が必ず先に呼ぶこと。
pub fn reset() {
    HANDLE_CREATIONS.store(0, Ordering::Relaxed);
    LAST_HANDLE_BYTES.store(0, Ordering::Relaxed);
    RENDER_FRAME_CALLS.store(0, Ordering::Relaxed);
    RENDER_FRAME_NANOS.store(0, Ordering::Relaxed);
    TOKENS_RELOADS.store(0, Ordering::Relaxed);
}
