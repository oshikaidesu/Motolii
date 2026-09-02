// 家: rerun への接ぎ木。Document の意味を re_renderer の口へ写す所だけ。
// GPU・デコード・描画・色は上流の部品 — ここで作らない。
pub mod audio;
pub mod compositor;
pub mod engine;
pub mod export;
pub mod media;

pub use motolii_doc as doc;

/// 家の中の道は `crate::render::…` のまま。
pub mod render {
    pub use crate::{audio, compositor, engine, export, media};
}
