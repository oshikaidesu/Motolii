//! Slint 依存はこの private module に閉じ、公開 API へ型を出さない。

/// 骨格段階ではウィンドウを立てず、リンク解決だけを確認する。
pub(crate) fn slint_linked() -> bool {
    !slint::SharedString::from("motolii-ui").is_empty()
}
