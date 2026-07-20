//! egui依存はこのprivate moduleに閉じ、公開APIへ型を出さない。

/// 骨格段階ではwindowを立てず、リンク解決だけを確認する。
pub(crate) fn toolkit_linked() -> bool {
    std::mem::size_of::<egui::Context>() > 0
}
