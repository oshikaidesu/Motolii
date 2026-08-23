//! 動詞 id。**`motolii-verbs` crate は現時点で存在しない**(発注書が読むべき物と
//! して挙げていたが、リポジトリ全体を検索しても実体が無かった — RETURN の
//! 逸脱台帳参照)。したがって `Verb.id` へ直接繋ぐことはできず、この crate
//! 自身が正本の動詞 id を持つ。将来 `motolii-verbs` が実在した時点で、そちらの
//! id 型との対応表(または型そのものの差し替え)を1箇所足せばよい形にしてある
//! ([`VerbId`] は `motolii_store`/`motolii_shell` のどちらにも依存しない孤立
//! enum なので、対応表を足しても循環にならない)。
//!
//! **列挙は現行 shell に実装済みの物だけ**(`resolve_navigation_key`・
//! `inspector_pointer_event` の到達可能な腕、`defaults.rs` 冒頭 doc 参照) —
//! 発注書「新しい割当を発明しない」に対応する側。まだキーが無い動詞
//! (`AddLayer`/`SaveACopyRequested`/`ToggleCheckerboard` 等、`menu.rs` で
//! `shortcut: None` になっている物)はここに含めない。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerbId {
    // ---- Global scope(テキスト入力中でも常に発火、`inspector_pointer_event`
    // の早期の match 腕) ----
    EscapeCancel,
    DeleteSelectedKeys,
    NudgeKeyframeBack,
    NudgeKeyframeBackFast,
    NudgeKeyframeForward,
    NudgeKeyframeForwardFast,
    ResetToRenderCamera,

    // ---- U2 playhead ナビゲーション動詞束(`resolve_navigation_key`、
    // テキスト入力中は無効) ----
    StepPlayheadBack,
    StepPlayheadBackFast,
    StepPlayheadForward,
    StepPlayheadForwardFast,
    JumpPlayheadToStart,
    JumpPlayheadToEnd,
    // 作業範囲の先頭/末尾(map 1064、B18 第5波結線): Shift+Home/End。
    // `resolve_navigation_key` は Shift 修飾の有無で JumpPlayheadTo{Start,End}
    // と分岐する(`defaults.rs::nav_bundle_bindings` 冒頭コメント参照)。
    JumpToWorkAreaStart,
    JumpToWorkAreaEnd,
    JumpMeaningPointPrev,
    JumpMeaningPointPrevLayerOnly,
    JumpMeaningPointNext,
    JumpMeaningPointNextLayerOnly,
    JumpClipEdgeIn,
    JumpClipEdgeOut,
    // JKL シャトル(B21 第5波結線)。旧割当の bare j/k(意味点ジャンプ)は
    // `,`/`.` へ移設された(`defaults.rs::nav_bundle_bindings` 冒頭コメント・
    // `motolii_shell::resolve_navigation_key` 実測 2026-08-22 参照、裁定208 の
    // 無主レーンで追随)。**bare `l`(ShuttleForward)・Cmd+L(ToggleLoop)は
    // 未転写**(この試験が候補キーに `l` を含まないため未検出だっただけ —
    // RETURN の逸脱台帳に報告、この crate では追わない)。
    ShuttleReverse,
    ShuttleStop,
    // Mark Out(作業範囲の Out を playhead へ、B18 第5波結線)。**bare
    // `b`(Mark In/SetWorkAreaIn)は同じ理由で未転写**(RETURN 参照)。
    SetWorkAreaOut,
    Undo,
    Redo,
    CopyLayer,
    PasteLayer,
    CutLayer,
    DuplicateLayer,
    SelectAllLayers,
    DeselectAllLayers,
    NewProjectRequested,
    SaveRequested,
    SaveAsRequested,
    QuitRequested,
    GroupLayers,
    UngroupLayers,
    TogglePlayback,
}

impl VerbId {
    /// 全 variant(`defaults.rs` の網羅性試験・oracle が使う)。増やす時は
    /// ここへも足すことを `all()` の doc 経由で強制する形にしたいが、
    /// enum の variant を機械列挙する外部 crate は引かない(依存ゼロ方針)
    /// ので、oracle 側が `default_bindings()` の verb 集合との突き合わせで
    /// 検分する(`tests/keymap_oracle.rs` 参照)。
    pub const ALL: &'static [VerbId] = &[
        VerbId::EscapeCancel,
        VerbId::DeleteSelectedKeys,
        VerbId::NudgeKeyframeBack,
        VerbId::NudgeKeyframeBackFast,
        VerbId::NudgeKeyframeForward,
        VerbId::NudgeKeyframeForwardFast,
        VerbId::ResetToRenderCamera,
        VerbId::StepPlayheadBack,
        VerbId::StepPlayheadBackFast,
        VerbId::StepPlayheadForward,
        VerbId::StepPlayheadForwardFast,
        VerbId::JumpPlayheadToStart,
        VerbId::JumpPlayheadToEnd,
        VerbId::JumpToWorkAreaStart,
        VerbId::JumpToWorkAreaEnd,
        VerbId::JumpMeaningPointPrev,
        VerbId::JumpMeaningPointPrevLayerOnly,
        VerbId::JumpMeaningPointNext,
        VerbId::JumpMeaningPointNextLayerOnly,
        VerbId::JumpClipEdgeIn,
        VerbId::JumpClipEdgeOut,
        VerbId::ShuttleReverse,
        VerbId::ShuttleStop,
        VerbId::SetWorkAreaOut,
        VerbId::Undo,
        VerbId::Redo,
        VerbId::CopyLayer,
        VerbId::PasteLayer,
        VerbId::CutLayer,
        VerbId::DuplicateLayer,
        VerbId::SelectAllLayers,
        VerbId::DeselectAllLayers,
        VerbId::NewProjectRequested,
        VerbId::SaveRequested,
        VerbId::SaveAsRequested,
        VerbId::QuitRequested,
        VerbId::GroupLayers,
        VerbId::UngroupLayers,
        VerbId::TogglePlayback,
    ];
}
