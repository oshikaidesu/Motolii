use motolii_core::RationalTime;
use motolii_store::{LayerId, TextDocument};

use super::enums::text_justify_to_int;
use super::{Ctx, LottieExportError, UnsupportedForLottie};

// ---------------------------------------------------------------------------
// text(縮小スコープ——`text-document`/`font`/`animated-text-document` のみ。
// アニメーター・複数スタイル行・variable font 軸は unsupported へ積む)
// ---------------------------------------------------------------------------

/// **D-1 修正(2026-08-23)**: 以前は `view.text_document(layer)`(静的値)を1回
/// 引き、その1つの値から content track の各キー時刻だけ Lottie の `s`/`h` 列へ
/// 展開していた——`text_style.{id}.*`/`text_justify` の track(A-1b 着地分)は
/// 一度も読まれず、書き出しに乗らなかった(GOALS M15/D1 違反)。
///
/// **裁定206 の link 焼き込みと同じ作法**(`bake_property` 参照): comp の
/// 全フレームを `StoreView::resolved_text_document`(engine の text 経路と
/// 同じ1本、`motolii-engine::collect_text_documents` が通るのと同じ関数)で
/// 再サンプリングし、書き出す JSON blob が変わった時だけ Hold キーフレームを
/// 打つ。track が無い(既定値のまま)layer では全フレームで同じ blob になり、
/// 1本しか残らないので `encode_scalar_track` 同様に `"a": 0` の静的形へ落ちる
/// (下の `samples.len() <= 1` 分岐)——**旧来の静的 text と出力が変わらない**。
pub(crate) fn build_text_data(
    ctx: &Ctx<'_, '_>,
    layer: LayerId,
    unsupported: &mut Vec<UnsupportedForLottie>,
) -> Result<serde_json::Value, LottieExportError> {
    let Some(static_document) = ctx.view.text_document(layer)? else {
        return Ok(serde_json::json!({ "d": { "a": 0, "k": default_text_document_json("") } }));
    };

    if static_document.styles.len() > 1
        || !static_document.runs.is_empty()
        || !static_document.ranges.is_empty()
        || static_document.styles.iter().any(|s| !s.axes.is_empty() || !s.features.is_empty())
    {
        unsupported.push(UnsupportedForLottie {
            layer: Some(layer),
            category: "text-styling",
            detail: "複数スタイル行(runs)/アニメーター(ranges)/可変フォント軸(axes)/\
                     OpenType feature は未実装 — styles[0] 相当の1行だけを書き出した"
                .to_owned(),
        });
    }

    if static_document.slot_id.is_some() {
        unsupported.push(UnsupportedForLottie {
            layer: Some(layer),
            category: "text-slot",
            detail: "`animated-text-document sid`(TextDocument::slot_id)は comp の \
                     slots 表で `d` プロパティ全体を差し替える口だが、この export は \
                     まだ text の `d` に `sid` を立てる経路を実装していない"
                .to_owned(),
        });
    }

    if let Some(style) = static_document.styles.first() {
        if !style.font.family.is_empty() {
            unsupported.push(UnsupportedForLottie {
                layer: Some(layer),
                category: "font-list",
                detail: format!(
                    "text-document.f = `{}` を参照するが、対応する comp 直下の \
                     `fonts`(font-list、地図の note「第二の素材台帳を作らない」により \
                     Document 側に表が無い)を書いていない — 厳密な Lottie player は \
                     フォント解決に失敗しうる",
                    style.font.family
                ),
            });
        }
    }

    // 焼き込み本体(`bake_property` と同型)。`resolved_text_document` が
    // Err を返したら(型不一致等)そのまま伝播する — 黙って静的値へ落とさない。
    let frame_count = ctx.duration_frames.max(1);
    let mut samples: Vec<(i64, serde_json::Value)> = Vec::new();
    let mut last: Option<serde_json::Value> = None;
    for frame in 0..frame_count {
        let t = RationalTime::try_from_frame(frame, ctx.fps)?;
        let resolved = ctx
            .view
            .resolved_text_document(layer, t)?
            .unwrap_or_else(|| static_document.clone());
        let content = resolved.content.eval(t).to_owned();
        let doc_json = text_document_json(&resolved, resolved.styles.first(), &content);
        if last.as_ref() != Some(&doc_json) {
            samples.push((frame, doc_json.clone()));
            last = Some(doc_json);
        }
    }

    if samples.len() <= 1 {
        let doc_json = samples
            .into_iter()
            .next()
            .map(|(_, json)| json)
            .unwrap_or_else(|| default_text_document_json(""));
        return Ok(serde_json::json!({ "d": { "a": 0, "k": doc_json } }));
    }

    let mut out = Vec::with_capacity(samples.len());
    for (i, (frame, doc_json)) in samples.iter().enumerate() {
        let mut obj = serde_json::json!({
            "t": *frame as f64,
            "s": [doc_json],
        });
        if i + 1 < samples.len() {
            obj["h"] = serde_json::json!(1);
        }
        out.push(obj);
    }
    Ok(serde_json::json!({ "d": { "a": 1, "k": out } }))
}

fn default_text_document_json(content: &str) -> serde_json::Value {
    serde_json::json!({
        "t": content,
        "f": "",
        "s": 12.0,
        "j": 0,
        "tr": 0.0,
        "lh": 14.0,
        "fc": [0.0, 0.0, 0.0],
    })
}

fn text_document_json(
    document: &TextDocument,
    style: Option<&motolii_store::TextDocumentStyle>,
    content: &str,
) -> serde_json::Value {
    let mut obj = default_text_document_json(content);
    obj["j"] = serde_json::json!(text_justify_to_int(document.justify));
    if let Some(sz) = document.wrap_size {
        obj["sz"] = serde_json::json!([sz[0], sz[1]]);
    }
    if let Some(style) = style {
        obj["f"] = serde_json::json!(style.font.family);
        obj["s"] = serde_json::json!(style.size);
        obj["fc"] = serde_json::json!([style.fill[0], style.fill[1], style.fill[2]]);
        obj["tr"] = serde_json::json!(style.tracking);
        if let Some(lh) = style.line_height {
            obj["lh"] = serde_json::json!(lh);
        }
        if let Some(sc) = style.stroke_color {
            obj["sc"] = serde_json::json!([sc[0], sc[1], sc[2]]);
            obj["sw"] = serde_json::json!(style.stroke_width);
            obj["of"] = serde_json::json!(style.stroke_over_fill);
        }
    }
    obj
}
