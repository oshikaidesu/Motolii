//! AUDIO section(B42、裁定184 型別 section 第4号)。
//!
//! **持つ**: AUDIO section の view だけ([`audio_section`])。Level/Pan/Fade
//! In/Fade Out の4行は [`crate::transform::transform_row`] をそのまま4回
//! 呼ぶだけで組み立つ(**ident 行は無い** — mask/effect と違い per-id の
//! 一覧ではなく、layer につき常に高々1組の固定4行なので新しい編集文法を
//! 発明しない)。
//!
//! **持たない**: 値の意味・書き口は一切ここに無い ── `Level`/`Pan`/
//! `FadeIn`/`FadeOut` は [`crate::transform::TransformField`]/
//! [`crate::transform::KeyRow`] への拡張 variant として乗っているので、
//! 既存の値セル文法(`commit_inspector_field`/drag-to-scrub)がそのまま書く。
//! 新しい `Message` variant はゼロ ── shell 側の改修は要らない。投影
//! ([`crate::projection::AudioSectionProjection`])もここには無い(他の
//! `*Projection` と同じく `crate::projection` に同居する)。

use motolii_tokens_rs::{Colors, Dimensions};

use iced::widget::column;
use iced::Element;

use motolii_settings_pane::chrome::section_header;
use crate::projection::AudioSectionProjection;
use crate::transform::{transform_row, FieldDraft};
use crate::Message;

/// AUDIO section: 音声を持ち得る layer(`LayerSource::Media`)選択時のみ現れる
/// (裁定184 型別 section 第4号)。**ident 行は無い** — mask/effect と違い
/// per-id の一覧ではなく、layer につき常に高々1組の固定4行(Level/Pan/
/// Fade In/Fade Out)なので、[`transform_row`] を直接4回呼ぶだけで済む
/// (Position/Rotation/Opacity と同じ組み方 — 新しい編集文法を発明しない)。
pub(crate) fn audio_section(
    audio: &AudioSectionProjection,
    field_draft: Option<&FieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    column![
        section_header("AUDIO", dims, colors),
        transform_row(&audio.level, field_draft, dims, colors),
        transform_row(&audio.pan, field_draft, dims, colors),
        transform_row(&audio.fade_in, field_draft, dims, colors),
        transform_row(&audio.fade_out, field_draft, dims, colors),
    ]
    .into()
}
