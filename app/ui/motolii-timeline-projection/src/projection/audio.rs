//! 音声の有無 + ソース path の投影(SP-2 分割、`projection.rs` 209-336行を
//! 移設)。**中身は無改変**。

use super::*;

// ---------------------------------------------------------------------------
// 音声の有無 + ソース path(TL7 統合手順1、EXACT TARGET 1「RowProjection
// (または兄弟)に音声の有無とソース path を載せ」)。
// ---------------------------------------------------------------------------

/// `RowProjection` の**兄弟**(本体は変更していない)。`RowProjection` は
/// exhaustive な struct literal で複数箇所組まれている既存の構築子 — そこへ
/// 新フィールドを足すと write-set 外を壊す。独立の型で運ぶことでこれを回避した。
///
/// `may_have_audio` は**候補**であって断定ではない(旧名 `has_audio` はここを
/// 嘘にしていた)。probe(`motolii_media::probe_container`)は ffprobe サイドカーを
/// 叩く I/O なので、この投影の純関数(`rows()` と同じ「読むだけ」)からは呼べない。
/// `LayerSource::Media`(動画も静止画も音声も同じ variant)を「音声を持ち得る」の
/// 保守的な近似として使う — 静止画 Media には実際には音声が無い。
///
/// **断定するのは呼び手**: `motolii_media::waveform_peaks` は audio stream の無い
/// 素材へ `Err` を返すので、呼び手はその結果を1度だけキャッシュし、空なら波形を
/// 描かない。この投影が `true` を返した行に必ず波形が出るわけではない、という
/// 非対称はここに書いてある通りで、下流でそれを断定へ格上げしないこと。
///
/// Media 以外(Solid/Null/Shape/Text/Group)は音声を持ち得ないので常に `false`。
#[derive(Clone, Debug, PartialEq)]
pub struct AudioRowProjection {
    pub layer: LayerId,
    pub may_have_audio: bool,
    pub source_path: Option<String>,
    /// この行が素材のどこを見せているか。波形は**クリップと同じ窓**を映さねば
    /// ならない(トリムしても波形が動かないと、絵が嘘になる)ので、切り出しに
    /// 要る `source_in`/`duration`/`speed` をここへ載せる。呼び手が `meta` を
    /// もう一度読んで同じ値を組み直さずに済む。
    pub timing: LayerTiming,
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
            let timing = meta.timing;
            let (may_have_audio, source_path) = match meta.source {
                LayerSource::Media { path, .. } => (true, Some(path)),
                _ => (false, None),
            };
            Some(AudioRowProjection { layer: id, may_have_audio, source_path, timing })
        })
        .collect()
}

/// 波形を素材へ聞く時の解像度(バケット数)。
///
/// **1素材につき1回だけ**取り、ズームでは取り直さない — `waveform_peaks` は
/// ffmpeg を素材の端から端まで走らせる I/O で、ズームのたびに叩くと
/// 「プレイヘッドのカクつきは合否そのもの」(憲法3)を毎回踏む。だから解像度は
/// 表示の都合ではなく**一度きりのコストで足りる細かさ**として決め打つ:
/// 4K 幅(3840px)の pane に3分の曲を全部入れても 1px あたり複数バケットが乗る。
/// 描く側は必要なぶんだけ間引く(増やす方向には後から取り直せない、という
/// 非対称があるので、細かい側へ倒す)。
pub const WAVEFORM_BUCKETS: usize = 4096;

/// クリップが見せている素材の区間を、`peaks` のバケット添字の範囲へ写す。
///
/// `peaks` は**素材の全長**を `peaks_len` 個へ等分した物(`waveform_peaks` の
/// 契約)。クリップはそのうち `[source_in, source_in + duration × speed)` だけを
/// 見せているので、トリムやタイムストレッチをすると波形も一緒に動かねばならない
/// — 動かないと「クリップは切れたのに波形は元のまま」という、窓を叩くと見える嘘に
/// なる。
///
/// `source_total_frames` は素材の総尺を**comp の fps で数えたフレーム数**
/// (`Engine::media_duration` → `try_to_frame_round(comp.fps)`)。`LayerTiming` の
/// 各値と同じ物差しで割らないと比が合わない。
///
/// 逆再生(`speed.num() < 0`)は `None` — 波形を逆から見せるのが正しいのか、
/// それとも素材の順で見せるのかを決めた裁定がまだ無い。**推測で絵を出さない**
/// (Q0)。総尺が 0 以下・`peaks_len` が 0 の時も `None`。
pub fn waveform_bucket_range(
    timing: &LayerTiming,
    source_total_frames: i64,
    peaks_len: usize,
) -> Option<std::ops::Range<usize>> {
    if peaks_len == 0 || source_total_frames <= 0 || timing.duration <= 0 {
        return None;
    }
    if timing.speed.num() <= 0 {
        return None;
    }
    let span = i128::from(timing.duration) * i128::from(timing.speed.num())
        / i128::from(timing.speed.den());
    let first = i128::from(timing.source_in);
    let last = first + span;
    let total = i128::from(source_total_frames);
    let scale = |frame: i128| -> usize {
        let clamped = frame.clamp(0, total);
        (clamped * peaks_len as i128 / total).min(peaks_len as i128 - 1) as usize
    };
    let start = scale(first);
    // 終端は開区間。1バケットも無い範囲(極端なズームや極短クリップ)でも
    // 「何も無い」ではなく1バケットは見せる — クリップは実在するので。
    let end = scale(last.max(first + 1)).max(start) + 1;
    Some(start..end.min(peaks_len))
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

    /// **オラクル**: `LayerSource::Media` の layer は `may_have_audio == true` +
    /// 元の path をそのまま運ぶ。
    #[test]
    fn media_layers_report_has_audio_and_their_path() {
        let mut doc = doc_with_comp();
        let media = LayerId(1);
        place(&mut doc, media, LayerSource::Media { path: "clip.mov".into(), fingerprint: None });

        let out = audio_rows(&doc.view());
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].layer, media);
        assert!(out[0].may_have_audio, "Media layer は may_have_audio=true であるべき");
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
        assert!(!out[0].may_have_audio, "Solid layer が音声を持つ扱いになっている");
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
