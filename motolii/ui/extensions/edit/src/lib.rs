//! 作品を変える側 — Intent・Undo・保存。
//!
//! コアは「保存された作品を開いて、ある時刻の値を答える」所までで(2026-09-20 利用者裁定)、
//! 変えるのは編集機の仕事。Flash が player と authoring を分けていたのと同じ形で、
//! 読む側はこの家を知らない。
//!
//! 依存は一方向: edit → doc。逆は `cargo arc check` が落とす。

pub mod document;
pub mod persist;
pub mod stagger;
pub mod text_edit;

#[cfg(feature = "fixture")]
pub mod fixture;

pub use document::{Animate, Document, Intent};

/// この家から読む側の名前を、元の場所と同じ形で引けるようにする。
pub use motolii_doc as doc;



pub fn blank_project() -> Document {
    use motolii_doc::store::{Composition, Fps, };
    // 効果は 1 つも登録しない見本。効果を使う検査は自分で `with_programs` する。
    let mut doc = Document::new();
    let comp = Composition {
        width: 1920,
        height: 1080,
        fps: Fps::try_new(30, 1).expect("30fps"),
        duration_frames: 1800,
        background: [0.0, 0.0, 0.0, 1.0],
    };
    let _ = doc.apply(Intent::SetComposition(comp));
    doc
}
