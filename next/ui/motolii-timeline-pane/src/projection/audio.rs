//! 音声の有無 + ソース path の投影(SP-2 分割、`projection.rs` 209-336行を
//! 移設)。**中身は無改変**。

use super::*;

// ---------------------------------------------------------------------------
// 音声の有無 + ソース path(TL7 統合手順1、EXACT TARGET 1「RowProjection
// (または兄弟)に音声の有無とソース path を載せ」)。
// ---------------------------------------------------------------------------

/// `RowProjection` の**兄弟**(本体は変更していない)。`RowProjection` は
/// `next/shell/motolii-shell/tests/suite/drive.rs`(EXACT TARGET 外)が
/// exhaustive な struct literal で複数箇所組んでいる既存の構築子 — そこへ
/// 新フィールドを足すと、この発注の write-set 外のテストを壊す。発注書の
/// 「RowProjection(または兄弟)」という言葉どおり、独立の型で運ぶことでこれを
/// 回避した。
///
/// `has_audio` は実際に音声トラックを持つかの厳密な probe ではない —
/// probe(`motolii_media::probe_container`)は ffprobe サイドカーを叩く I/O
/// なので、この投影の純関数(`rows()` と同じ「読むだけ」)からは呼べない
/// (`waveform_view` モジュール doc 「このモジュールは motolii-media を一切
/// 叩かない」と同じ理由)。`LayerSource::Media`(動画も静止画も同じ variant)を
/// 「音声を持ち得る」の保守的な近似として使う — 静止画 Media には実際には
/// 音声が無いが、`motolii_media::waveform_peaks` が audio stream 無しで `Err`
/// を返すだけで実害は無い(`PaneState::plan_waveforms` の doc 参照 —
/// `Message::WaveformFetchFailed` が空 peaks の `Ready` へ落として無限リトライを
/// 防ぐ)。Media 以外(Solid/Null/Shape/Text/Group)は音声を持ち得ないので
/// 常に `false`。
#[derive(Clone, Debug, PartialEq)]
pub struct AudioRowProjection {
    pub layer: LayerId,
    pub has_audio: bool,
    pub source_path: Option<String>,
}

/// `store` の全 present layer から [`AudioRowProjection`] を組む。**読むだけ**
/// (`rows()` と同じ「Document/Session を書き換えない」原則)。fold/選択等の
/// 表示状態には依存しない — 波形取得の要否そのもの(`plan_waveforms`)は
/// 呼び出し側が「今どのクリップが画面に見えているか」を別途フィルタしてから
/// 呼ぶ想定で、この関数自体は候補を並べるだけ。
pub fn audio_rows(store: &StoreView<'_>) -> Vec<AudioRowProjection> {
    store
        .layers()
        .into_iter()
        .filter_map(|id| {
            let meta = store.meta(id).ok().flatten()?;
            let (has_audio, source_path) = match meta.source {
                LayerSource::Media { path, .. } => (true, Some(path)),
                _ => (false, None),
            };
            Some(AudioRowProjection { layer: id, has_audio, source_path })
        })
        .collect()
}

/// [`audio_rows`] の落ちるテスト先行(TL7 統合手順1)。**未実行**(裁定189 —
/// supervisor が波末一括で回す)。
#[cfg(test)]
mod audio_row_tests {
    use super::*;
    use motolii_store::{Composition, Document, Fps as StoreFps, Intent, LayerMeta};

    fn doc_with_comp() -> Document {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 64,
            height: 64,
            fps: StoreFps::try_new(30, 1).expect("30/1 は正の既約 fps"),
            duration_frames: 300,
            background: [0.0, 0.0, 0.0, 1.0],
        }))
        .expect("comp 設定");
        doc
    }

    fn place(doc: &mut Document, id: LayerId, source: LayerSource) {
        doc.apply_all([
            Intent::AddLayer(id),
            Intent::SetMeta {
                layer: id,
                meta: LayerMeta {
                    source,
                    order: 0,
                    timing: LayerTiming { start: 0, duration: 100, source_in: 0, ..Default::default() },
                },
            },
        ])
        .expect("layer 配置");
    }

    /// **オラクル**: `LayerSource::Media` の layer は `has_audio == true` +
    /// 元の path をそのまま運ぶ。
    #[test]
    fn media_layers_report_has_audio_and_their_path() {
        let mut doc = doc_with_comp();
        let media = LayerId(1);
        place(&mut doc, media, LayerSource::Media { path: "clip.mov".into(), fingerprint: None });

        let out = audio_rows(&doc.view());
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].layer, media);
        assert!(out[0].has_audio, "Media layer は has_audio=true であるべき");
        assert_eq!(out[0].source_path.as_deref(), Some("clip.mov"));
    }

    /// **オラクル**: Media 以外(ここでは Solid)は常に音声を持たない扱い —
    /// `source_path` も運ばない。
    #[test]
    fn non_media_layers_never_report_audio() {
        let mut doc = doc_with_comp();
        let solid = LayerId(1);
        place(&mut doc, solid, LayerSource::Solid { rgba: [255, 0, 0, 255], width: 64, height: 64 });

        let out = audio_rows(&doc.view());
        assert_eq!(out.len(), 1);
        assert!(!out[0].has_audio, "Solid layer が音声を持つ扱いになっている");
        assert!(out[0].source_path.is_none());
    }

    /// **オラクル**: present な layer を1つも取りこぼさない(`rows()` と同じ
    /// 「読むだけ・全部並べる」原則)。
    #[test]
    fn audio_rows_covers_every_present_layer() {
        let mut doc = doc_with_comp();
        place(&mut doc, LayerId(1), LayerSource::Media { path: "a.mp4".into(), fingerprint: None });
        place(&mut doc, LayerId(2), LayerSource::Null);

        let out = audio_rows(&doc.view());
        let mut layers: Vec<LayerId> = out.iter().map(|row| row.layer).collect();
        layers.sort();
        assert_eq!(layers, vec![LayerId(1), LayerId(2)], "present layer を1つも取りこぼしてはいけない");
    }
}
