//! The write side of a Motolii work: edits, undo and saving.
//!
//! `Document` owns a work; every change is an `Intent` passed to `Document::apply`
//! (`apply_all` / `apply_then` group several into one undo step), and `undo` / `redo`
//! move along the recorded steps. `persist` saves and reopens; `stagger` and
//! `text_edit` build Intents for multi-layer and text edits. Reading is `motolii-doc`,
//! which does not know this crate; the dependency runs edit -> doc only.

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
