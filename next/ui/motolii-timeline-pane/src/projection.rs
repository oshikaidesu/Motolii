//! Timeline の投影の純関数(`rows`/`frame_to_x`/`frame_at_x`/
//! `time_band_segment_frames`/`property_rows`/`layer_row_top`)。**読むだけ** —
//! Document/Session を書き換えない。

use std::collections::{HashMap, HashSet};

use motolii_store::{Fps, LayerId, LayerSource, LayerTiming, PropertyId, StoreView};

use crate::state::Session;

/// `KeySelector`/`KeySelectionOp` は裁定160 切片6 で `crate::state` へ移設済み
/// (pane split survey §2.3: `Session ⇄ timeline` の型循環解消 — `state` は
/// leaf、`timeline` はそこへ依存する片方向)。`pub use` は `timeline::mod` の
/// `pub use projection::{..., KeySelectionOp, KeySelector, ...}` を無改修で
/// 保つための re-export(型 alias で外部参照を壊さない手口)。
pub use crate::state::{KeySelectionOp, KeySelector};

/// 1層分の読み取り投影。**Document の写しではなく、1度描くための使い捨て値**。
#[derive(Clone, Debug, PartialEq)]
pub struct RowProjection {
    pub id: LayerId,
    pub name: String,
    pub hidden: bool,
    /// solo(`LayerAttrs.solo`)。レーンバーの S トグルが読む(裁定147)。
    pub solo: bool,
    /// locked(`LayerAttrs.locked`)。レーンバーの L トグルが読む(裁定147)。
    pub locked: bool,
    /// レイヤー差し色の index(`LayerAttrs.label_color`)。`None` = 未割当 —
    /// bar は既定色(`way_timeline`)のまま(`canvas::draw` 参照)。
    pub label_color: Option<u8>,
    pub start: i64,
    pub duration: i64,
    pub selected: bool,
    /// クリップ drag のプレビュー中(第2波T5、正典 §2「ドラッグ中の bar は
    /// ACCENT」)。`rows()` は常に `false` — [`apply_clip_preview`] だけが
    /// 掴んでいる1行にだけ立てる。`selected` とは別ロール(trim は選択を
    /// 変えないので、選択と drag 中は独立に真偽が分かれ得る)。
    pub dragging: bool,
    /// **裁定173 H2**: `attrs.parent` を辺として読んだ木の深さ。0 = 最上位
    /// (parent が無い、または parent が存在しない/循環している孤立行)。
    /// rail のインデントの出典(`rail::layer_row` 参照)。
    pub depth: u16,
    /// この行が子(`attrs.parent` がこの layer を指す present な layer)を
    /// 1つでも持つか。fold 三角を出すかどうかの出典 — 子を持たない行は
    /// 矢印を出さない(旧世界 `timeline_rows.rs` と同じ規則)。
    pub has_children: bool,
    /// 子が展開されているか(`!session.timeline_fold.is_folded(id)`)。
    /// `has_children == false` の行では意味を持たない(矢印自体が無い)。
    pub children_open: bool,
}

/// `store`/`session` から Timeline の行を組み立てる。**読むだけ**。
///
/// `store.layers()` は「present な layer」しか返さない(削除は墓標なので既に除外
/// 済み — `view.rs`)。bar の重ね順(`meta.order`)は Stage 側の合成順であって、
/// Timeline の縦位置の所有者にしない(`ui-score-model.md` 4層構成: 縦位置は
/// packing 結果にすぎない)。
///
/// **裁定173 H2**(旧世界 `crates/motolii-ui/src/timeline_rows.rs::rows` の
/// fold 軸独立 flatten-per-frame アルゴリズムの概念移植): `attrs.parent` を
/// 表示の木の辺として読み、親→子の順に深さ優先で平坦化する。**H1(変換合成)
/// はこの関数の範囲外** — ここは表示上の親子だけを見る、`meta.timing`/
/// `attrs.*` の読み方自体は導入前と無改造。**木を持たない**: 呼ばれるたびに
/// `store.layers()` から作り直す使い捨ての `Vec`(旧世界と同じ思想、mod doc
/// 「木を持たない」節参照)。
///
/// 兄弟の並びは常に `LayerId` 昇順(`store.layers()` の既存順をそのまま踏襲 —
/// 導入前の並びを変えない、oracle「既存 suite 全緑」の根拠)。fold が閉じている
/// layer の子孫は出ない(行自身は残る、旧世界と同じ規則)。**既定 = 全展開**
/// (`TimelineFoldState` doc 参照)なので、parent が1つも設定されていない
/// Document(導入前の全ドキュメント)は今までと完全に同じ並び・同じ行数を返す。
///
/// **循環 parent は投影段でも安全**(H-survey §2.2 の推奨どおり、書き込み時
/// ガード[`validate_no_parent_cycle`]とは独立な第二の柵): 壊れた Document
/// (バグ・旧ファイルの直接編集)を読んでも無限再帰せず、どの layer もちょうど
/// 1回だけ行になる — 根( parent=None)へ辿り着けない循環メンバーは、深さ0の
/// 孤立行として出す(消えない)。
pub fn rows(store: &StoreView<'_>, session: &Session) -> Vec<RowProjection> {
    let layers = store.layers();
    let present: HashSet<LayerId> = layers.iter().copied().collect();

    // 親 → 子(`store.layers()` の昇順のまま集める)のマップ。parent が
    // present set の外(削除済み/存在しない LayerId)を指していたら根として
    // 扱う — 壊れた参照で行を失わない(§2.2 と同じ防衛の精神)。
    let mut children_of: HashMap<Option<LayerId>, Vec<LayerId>> = HashMap::new();
    for &id in &layers {
        let attrs = store.attrs(id).ok().flatten().unwrap_or_default();
        let parent = attrs.parent.filter(|p| present.contains(p));
        children_of.entry(parent).or_default().push(id);
    }

    // **fold を無視した構造的到達性**を先に求める(1巡目)。ここで「`None` から
    // 辿り着けない」と判定された layer だけが、本当に壊れた(循環)参照 —
    // fold で畳まれているだけの子は、fold を見ないこの巡回では普通に辿り
    // 着けるので、ここでは「壊れていない」と正しく判定される(旧設計の
    // バグ: 畳んだ子を「未出力」のまま防衛2巡目に回してしまい、fold を
    // 無視して孤立行として出してしまっていた — 到達性判定と fold 適用を
    // 同じ巡回に混ぜていたのが原因。ここで2つの巡回に分離して切り離す)。
    let mut reachable: HashSet<LayerId> = HashSet::new();
    {
        let mut visiting = HashSet::new();
        if let Some(roots) = children_of.get(&None) {
            for &root in roots {
                mark_reachable(root, &children_of, &mut visiting, &mut reachable);
            }
        }
    }

    // 根の集合 = 本来の根(`parent == None`)+ `None` から辿り着けない孤立
    // (循環)メンバー。後者は本来ならここに来ない(書き込み時ガード
    // `validate_no_parent_cycle` が防ぐ)が、壊れた Document を読んだ時の
    // 第二の柵として、深さ0の孤立行にする(oracle「循環 parent は投影段でも
    // 安全」— 消さない・panic しない)。
    let mut roots: Vec<LayerId> = children_of.get(&None).cloned().unwrap_or_default();
    for &id in &layers {
        if !reachable.contains(&id) {
            roots.push(id);
        }
    }

    // 2巡目(実際の行組み立て)。ここで初めて fold を見る。
    let mut out = Vec::with_capacity(layers.len());
    let mut emitted: HashSet<LayerId> = HashSet::new();
    let mut visiting: HashSet<LayerId> = HashSet::new();
    for root in roots {
        push_layer(store, session, root, 0, &children_of, &mut visiting, &mut emitted, &mut out);
    }
    out
}

/// `rows()` 1巡目: fold を一切見ず、`None`(根)から `children_of` を辿って
/// 構造的に到達できる layer 集合を求める。`push_layer` と同じ
/// `visiting`(同一枝内の循環ガード)+ `reachable`(二重挿入防止、木なので
/// 通常は1回しか呼ばれないが循環時の保険)の形。
fn mark_reachable(
    id: LayerId,
    children_of: &HashMap<Option<LayerId>, Vec<LayerId>>,
    visiting: &mut HashSet<LayerId>,
    reachable: &mut HashSet<LayerId>,
) {
    if !reachable.insert(id) {
        return;
    }
    if !visiting.insert(id) {
        return;
    }
    if let Some(children) = children_of.get(&Some(id)) {
        for &child in children {
            mark_reachable(child, children_of, visiting, reachable);
        }
    }
    visiting.remove(&id);
}

/// 1 layer を、開いていればその子孫まで再帰的に `out` へ積む(旧世界
/// `push_item` の概念移植)。`visiting` は同一枝内の再訪(循環)を止める防御的
/// ガード、`emitted` は「もう出力したか」を覚える(枝をまたいだ二重計上・
/// `rows()` の防衛2巡目での再計上を防ぐ)。
fn push_layer(
    store: &StoreView<'_>,
    session: &Session,
    id: LayerId,
    depth: u16,
    children_of: &HashMap<Option<LayerId>, Vec<LayerId>>,
    visiting: &mut HashSet<LayerId>,
    emitted: &mut HashSet<LayerId>,
    out: &mut Vec<RowProjection>,
) {
    if !emitted.insert(id) {
        return; // 既に出力済み(通常は起きない — 循環防衛の保険)。
    }
    if !visiting.insert(id) {
        return; // 同一枝内で自分自身に戻ってきた(循環) — ここで止める。
    }

    let Ok(Some(meta)) = store.meta(id) else {
        visiting.remove(&id);
        return; // present だが meta が引けない(起こらないはずだが安全側、旧来と同じ)。
    };
    let attrs = store.attrs(id).ok().flatten().unwrap_or_default();
    let children = children_of.get(&Some(id)).cloned().unwrap_or_default();
    let has_children = !children.is_empty();
    let children_open = !session.timeline_fold.is_folded(id);

    out.push(RowProjection {
        id,
        name: attrs.name,
        hidden: attrs.hidden,
        solo: attrs.solo,
        locked: attrs.locked,
        label_color: attrs.label_color,
        start: meta.timing.start,
        duration: meta.timing.duration,
        selected: row_selected(session, id),
        dragging: false,
        depth,
        has_children,
        children_open,
    });

    if children_open {
        for child in children {
            push_layer(store, session, child, depth + 1, children_of, visiting, emitted, out);
        }
    }
    visiting.remove(&id);
}

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

/// 行ハイライトの選択判定。`selection`(単一 focus)と `selected_layers`
/// (U1 の複数選択集合)は身分が別(`Session` の doc 参照)だが、**行の見た目は
/// どちらも同じ選択**(AE 同型: 複数選択の各 layer 行は同一ハイライト。primary の
/// 区別は property 行の展開(`selected_row_index` = `selection` のみ)が担う)。
pub fn row_selected(session: &Session, id: LayerId) -> bool {
    session.selection == Some(id) || session.selected_layers.contains(&id)
}

/// comp フレーム → x px。`duration_frames <= 0` の空 comp では常に 0。
///
/// `pub`: screenshot 器具(`motolii_shell::screenshot`)が Timeline canvas と
/// 同じ x 座標計算を使うため(マーカー・bar の位置を2箇所で別の式にしない)。
/// **裁定160 切片7で `pub(crate)` → `pub` に緩めた** — screenshot.rs は
/// crate 分割後 `motolii-shell` 側に残るので、`pub(crate)`(同一クレート限定)
/// のままでは呼べなくなる。
pub fn frame_to_x(frame: i64, width: f32, duration_frames: i64) -> f32 {
    if duration_frames <= 0 || width <= 0.0 {
        return 0.0;
    }
    let ratio = frame as f32 / duration_frames as f32;
    (ratio * width).clamp(0.0, width)
}

/// x px → comp フレーム。**scrub の core**(canvas の click/drag と単体 test の両方が
/// これを呼ぶ)。範囲外は端へ丸める。
pub fn frame_at_x(x: f32, width: f32, duration_frames: i64) -> i64 {
    if duration_frames <= 0 || width <= 0.0 {
        return 0;
    }
    let ratio = (x / width).clamp(0.0, 1.0);
    let frame = (ratio * duration_frames as f32).round() as i64;
    frame.clamp(0, (duration_frames - 1).max(0))
}

/// 目盛りの目標セル比率(小目盛間隔 px / 行高 px)。利用者裁定 2026-08-21 夜
/// 「比率の原則」(`docs/ui-spatial-score.md` S4)— **形は絶対 px でなく比率で
/// 定数化する**(x/y の2変数が1つの数へ畳まれスケール不変になり、モックと
/// 実装を同じ定数で機械検査できる)。合格モック
/// (`next/reference/mocks/timeline-semantics.html`)実測 13.5px/26px ≈ 0.52 が
/// 出典。[`tick_steps`] は [`step_ladder_frames`] の中からこの比率に**最も
/// 近い**ステップを選ぶ — σ 初回実装は `MIN_MINOR_TICK_PX`(px 絶対下限)で
/// 梯子を選び 0.92 になり「モック通りでない」を利用者が検出した実例の修理
/// (発注書 EXACT TARGET 1)。絶対 px が許されるのは物理由来の下限のみ
/// (ヒット寸 12px 等 — ここは該当しない)。
pub(crate) const TARGET_CELL_RATIO: f32 = 0.52;

/// 目盛りの候補ステップ(フレーム数、昇順・重複無し)。**時刻へ絶対整列**
/// (0, step, 2*step, ... — 全尺等分と違い端数のフレームが出ない)。
///
/// fps が引ければ「1f, 5f, 10f, 1s, 2s, 5s, 10s, 30s, 1m, 5m」という
/// 人間に読みやすい混合ラダー(発注書 EXACT TARGET 1 の候補列そのもの)。
/// fps が引けない(comp 無し)時は秒/分を frame へ直せないので、先頭の
/// 「1, 5, 10」十進ラダーだけへ落ちる(`duration_frames` も同時に 0 になる
/// 経路がほとんどなので、この短いラダーで実害は無い — [`TimelinePane::new`]
/// 参照)。
fn step_ladder_frames(fps: Option<Fps>) -> Vec<i64> {
    // 2f と半秒(fps/2)を含む(σ2 検収 FINDING の処置 2026-08-21: 梯子が粗いと
    // 比率最近傍でも目標 0.52 に届かない — 30fps・幅1426px で 10f=0.305/30f=0.914 の
    // 二択だった穴を 15f=0.457 が埋める。半秒は時間整列の正当な段)。
    let mut out: Vec<i64> = vec![1, 2, 5, 10];
    if let Some(fps) = fps {
        let fps = fps.as_f64();
        for seconds in [0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0, 300.0] {
            out.push((fps * seconds).round().max(1.0) as i64);
        }
        out.sort_unstable();
        out.dedup();
    }
    out
}

/// ルーラー目盛りの小目盛/大目盛のステップ(フレーム数)。利用者裁定
/// 2026-08-21 夜: 全尺等分(旧 `RULER_TICK_DIVISIONS`)を撤去し、小目盛/大目盛の
/// 階層を導入する。明暗帯([`time_band_segment_frames`])はこの大目盛の周期に
/// 揃える(単一のラダーが目盛りと明暗帯の両方の出典 — 2箇所で別の刻み方を
/// 持たない)。
///
/// 小目盛 = [`step_ladder_frames`] の中で「セル比率(小目盛の px 間隔 /
/// `row_height`)が [`TARGET_CELL_RATIO`] に最も近い」ステップ(比率の原則
/// — 発注書 EXACT TARGET 1・2)。同点(浮動小数の完全一致)はラダー順で先に
/// 見つかった方(`Iterator::min_by` の安定順)を採る。
///
/// 大目盛 = ラダー上で小目盛のちょうど5倍か10倍になっている直近上位のステップ
/// (`draw_ruler_ticks` doc 参照 — 秒/分混在ラダーは全区間が等比ではないため、
/// 2倍/3倍しか離れていない隣接ステップは飛ばす)。ラダー上に見つからなければ
/// 小目盛の10倍を直接計算し(ラダー外でも構わない)、「大目盛は常に小目盛の
/// 整数倍」だけは常に守る。時間整列(0, step, 2*step, ...)は不変。
///
/// `pub`: `super::canvas::draw_ruler_ticks`/`draw_time_bands` と
/// `motolii_shell::screenshot` 器具が同じ刻みを再現するため(`frame_to_x` と
/// 同じ理由)。`row_height` は呼び手の `dims.row_height`(pane 側)/
/// `dims.row_height`(screenshot 側)をそのまま渡す — 比率の分母は常にこの1つ
/// (2箇所で別の行高を使わない)。
pub fn tick_steps(
    fps: Option<Fps>,
    duration_frames: i64,
    clip_width: f32,
    row_height: f32,
) -> (i64, i64) {
    let ladder = step_ladder_frames(fps);
    if duration_frames <= 0 || clip_width <= 0.0 || row_height <= 0.0 {
        let minor = ladder.first().copied().unwrap_or(1);
        return (minor, minor.saturating_mul(5));
    }
    let px_per_frame = clip_width / duration_frames as f32;
    let cell_ratio_gap = |step: i64| -> f32 {
        (step as f32 * px_per_frame / row_height - TARGET_CELL_RATIO).abs()
    };
    let minor = ladder
        .iter()
        .copied()
        .min_by(|&a, &b| {
            cell_ratio_gap(a)
                .partial_cmp(&cell_ratio_gap(b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or(1);
    let major = ladder
        .iter()
        .copied()
        .find(|&candidate| {
            candidate > minor
                && candidate % minor == 0
                && matches!(candidate / minor, 5 | 10)
        })
        .unwrap_or_else(|| minor.saturating_mul(10));
    (minor, major)
}

/// 時間方向の明暗リズム(裁定148(1))の区間幅(フレーム数)。**大目盛の周期に
/// 揃える**([`tick_steps`] の第2要素、利用者裁定 2026-08-21 夜 — 旧・固定1秒/
/// `RULER_TICK_DIVISIONS` 等分は撤去)。`draw_time_bands` と screenshot 器具の
/// 両方がこの1つの式から区間境界を出す(2箇所で別のフォールバックを持たない)。
///
/// `clip_width`/`row_height` を引数に足した(裁定160 切片7時点は `clip_width`
/// すら無かった) — 大目盛は [`TARGET_CELL_RATIO`] のセル比率(`clip_width`と
/// `row_height` の両方に依存)から出るので、区間幅も同じ入力を要る。
///
/// `pub`: `motolii_shell::screenshot` が Timeline canvas と同じ区間の刻み方を
/// 再現するため(`frame_to_x` と同じ理由、裁定160 切片7で緩めた)。
pub fn time_band_segment_frames(
    fps: Option<Fps>,
    duration_frames: i64,
    clip_width: f32,
    row_height: f32,
) -> i64 {
    tick_steps(fps, duration_frames, clip_width, row_height).1
}

#[cfg(test)]
mod tick_tests {
    use super::*;

    fn fps30() -> Fps {
        Fps::try_new(30, 1).expect("30/1 は正の既約 fps")
    }

    /// 合格モックの実測 `row_height`(`next/reference/mocks/timeline-semantics.html`
    /// の行高 26px)— 発注書の EXACT TARGET/ORACLE (a) が指す値。
    const MOCK_ROW_HEIGHT: f32 = 26.0;

    /// ラダーの中で [`TARGET_CELL_RATIO`] に最も近いステップを、テスト側でも
    /// 独立に計算する(発注書オラクル(a)「期待値はテスト内で梯子から計算」)。
    /// `tick_steps` 本体と同じ式を **意図して重複実装**する — 実装のバグを
    /// 実装自身の式で覆い隠さないため(oracle は独立検算)。
    fn nearest_minor_by_ratio(duration_frames: i64, clip_width: f32, row_height: f32) -> i64 {
        let ladder = step_ladder_frames(Some(fps30()));
        let px_per_frame = clip_width / duration_frames as f32;
        ladder
            .into_iter()
            .min_by(|&a, &b| {
                let gap = |step: i64| (step as f32 * px_per_frame / row_height - TARGET_CELL_RATIO).abs();
                gap(a).partial_cmp(&gap(b)).unwrap_or(std::cmp::Ordering::Equal)
            })
            .expect("ladder は常に非空(step_ladder_frames が最低 [1,5,10] を返す)")
    }

    /// **オラクル(a)**: 30fps・尺1800f・幅1426px・行高26px(発注書 EXACT TARGET)
    /// → 選ばれる小目盛が、全ラダー候補の中でセル比率(spacing_px/row_height)を
    /// [`TARGET_CELL_RATIO`](0.52)に最も近づけるステップと一致する。具体値
    /// (10f)も assert する — 期待値はテスト内の独立計算([`nearest_minor_by_ratio`])
    /// から出しており、実装のマジックナンバーをそのまま複製していない。
    #[test]
    fn tick_steps_picks_the_ladder_step_nearest_the_target_cell_ratio() {
        // 期待値 15f = 半秒(σ2 検収 FINDING の処置で梯子へ 2f・fps/2 を追加した後の
        // 最近傍: 15f→比率0.457。旧梯子では 10f→0.305 が最善だった — 梯子の粗さが
        // 比率原則の到達度を制約していた実例)。
        let expected_minor = nearest_minor_by_ratio(1800, 1426.0, MOCK_ROW_HEIGHT);
        assert_eq!(expected_minor, 15, "テスト側の独立計算自体が想定値からずれている");

        let (minor, major) = tick_steps(Some(fps30()), 1800, 1426.0, MOCK_ROW_HEIGHT);
        assert_eq!(minor, expected_minor, "小目盛が比率最近傍のステップになっていない");
        assert_eq!(minor, 15, "小目盛の具体値が想定(15f=半秒)とずれている");
        assert_eq!(major % minor, 0, "大目盛が小目盛の整数倍でない");
    }

    /// **オラクル(a)**: 幅810px(モック寸そのもの)でも同じ比率最近傍の原則が
    /// 成立する — このケースは実際にモック実測(13.5px/26px≈0.519)に極めて近い
    /// step(30f=1s級)を選ぶ、比率原則の直接の実例。
    #[test]
    fn tick_steps_picks_the_nearest_ratio_step_at_mock_width_too() {
        let expected_minor = nearest_minor_by_ratio(1800, 810.0, MOCK_ROW_HEIGHT);
        assert_eq!(expected_minor, 30, "テスト側の独立計算自体が想定値からずれている");

        let (minor, _major) = tick_steps(Some(fps30()), 1800, 810.0, MOCK_ROW_HEIGHT);
        assert_eq!(minor, expected_minor, "幅810pxで比率最近傍のステップになっていない");

        // 実際にモック実測比率(0.52)へ極めて近いことも確認する(比率の原則の
        // 目的そのもの — 「モック通りでない」の再発防止)。
        let px_per_frame = 810.0 / 1800.0;
        let ratio = minor as f32 * px_per_frame / MOCK_ROW_HEIGHT;
        assert!(
            (ratio - TARGET_CELL_RATIO).abs() < 0.01,
            "選ばれた小目盛のセル比率がモック実測(0.52)から遠すぎる: {ratio}"
        );
    }

    /// **オラクル(a)**: 大目盛は常に小目盛の整数倍(5倍か10倍)— どんな尺でも。
    #[test]
    fn major_is_always_an_integer_multiple_of_minor() {
        for duration in [1, 2, 10, 37, 100, 1_800, 12_345, 100_000, 5_000_000] {
            let (minor, major) = tick_steps(Some(fps30()), duration, 1349.0, MOCK_ROW_HEIGHT);
            assert!(minor >= 1, "小目盛が1未満に退化した(duration={duration})");
            assert!(
                major >= minor && major % minor == 0,
                "大目盛が小目盛の整数倍でない(duration={duration}, minor={minor}, major={major})"
            );
        }
    }

    /// **オラクル(a)**: 極端に短い尺(10f)でも最小1f まで退化するだけで
    /// パニックしない・0にならない。
    #[test]
    fn tick_steps_does_not_degenerate_on_a_tiny_duration() {
        let (minor, major) = tick_steps(Some(fps30()), 10, 1349.0, MOCK_ROW_HEIGHT);
        assert!(minor >= 1);
        assert!(major >= minor);
    }

    /// **オラクル(a)**: 極端に長い尺(100000f)でも同じラダーから退化なく
    /// 値が出る(巨大 duration で minor/major が0や負にならない)。
    #[test]
    fn tick_steps_does_not_degenerate_on_a_huge_duration() {
        let (minor, major) = tick_steps(Some(fps30()), 100_000, 1349.0, MOCK_ROW_HEIGHT);
        assert!(minor >= 1);
        assert!(major > 0 && major % minor == 0);
    }

    /// fps が引けない(comp 無し)時も 0 割り/パニックせず、最小ラダー
    /// (1,5,10)から値を返す。
    #[test]
    fn tick_steps_without_fps_falls_back_to_the_short_ladder() {
        let (minor, major) = tick_steps(None, 100, 1349.0, MOCK_ROW_HEIGHT);
        assert!(minor >= 1);
        assert!(major >= minor && major % minor == 0);
    }

    /// `duration_frames <= 0`/`clip_width <= 0.0`/`row_height <= 0.0` は空 comp
    /// と同じ安全側(パニックしない、`minor <= major`)。`row_height` の退化
    /// ガードは比率原則導入で新設した経路(発注書 EXACT TARGET 2 の「退化ガード
    /// は維持」— 0除算の分母が増えたので、ここも同格で守る)。
    #[test]
    fn tick_steps_guards_non_positive_inputs() {
        assert_eq!(tick_steps(Some(fps30()), 0, 1349.0, MOCK_ROW_HEIGHT).0, 1);
        assert_eq!(tick_steps(Some(fps30()), 1800, 0.0, MOCK_ROW_HEIGHT).0, 1);
        assert_eq!(tick_steps(Some(fps30()), -5, 1349.0, MOCK_ROW_HEIGHT).0, 1);
        assert_eq!(tick_steps(Some(fps30()), 1800, 1349.0, 0.0).0, 1);
        assert_eq!(tick_steps(Some(fps30()), 1800, 1349.0, -1.0).0, 1);
    }

    /// **オラクル(b)**: 明暗帯の区間幅(旧 `time_band_segment_frames`)は
    /// `tick_steps` の大目盛と常に一致する(2箇所で別の刻み方を持たない)。
    #[test]
    fn time_band_segment_matches_tick_steps_major() {
        let (_minor, major) = tick_steps(Some(fps30()), 1800, 1349.0, MOCK_ROW_HEIGHT);
        assert_eq!(
            time_band_segment_frames(Some(fps30()), 1800, 1349.0, MOCK_ROW_HEIGHT),
            major
        );
    }
}

// ---------------------------------------------------------------------------
// property 行(キー行) — 第2波 T3(裁定148/151・正典 §1.5/§3)。
// ---------------------------------------------------------------------------

/// property 行の1キー(comp フレーム位置 + 選択状態)。
#[derive(Clone, Debug, PartialEq)]
pub struct PropertyKeyProjection {
    pub frame: i64,
    pub selected: bool,
}

/// 1つの property 行。**選択 layer の下にだけ挿入される**(EXACT TARGET 1 —
/// 全 layer 分ではなく `session.selection` の1層分だけ、正典 §5 候補
/// 「Show Only Animated」を選択 layer 限定で先取りする形)。
#[derive(Clone, Debug, PartialEq)]
pub struct PropertyRowProjection {
    pub layer: LayerId,
    pub property: PropertyId,
    pub keys: Vec<PropertyKeyProjection>,
}

/// `session.selection` の layer が持つ、**キーを持つ property だけ**の行
/// (裁定151「既定=キーを持つ property のみ」)。track 列挙は
/// [`StoreView::properties`]/[`StoreView::track`] という既存の読み口をそのまま
/// 使う(新しい走査を発明しない、Inspector の property 行組み立てと同じ経路)。
///
/// **読むだけ**。layer が選ばれていない、または comp が無く fps が引けない時は
/// 空(黙って誤った位置に描くより描かない — [`super::TimelinePane::marker_frame`]
/// と同じ理由)。
pub fn property_rows(
    store: &StoreView<'_>,
    session: &Session,
    fps: Option<Fps>,
) -> Vec<PropertyRowProjection> {
    let Some(layer) = session.selection else {
        return Vec::new();
    };
    let Some(fps) = fps else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for property in store.properties(layer) {
        let Ok(Some(track)) = store.track(layer, &property) else {
            continue;
        };
        if track.keys().is_empty() {
            continue;
        }
        let keys: Vec<PropertyKeyProjection> = track
            .keys()
            .iter()
            .filter_map(|key| {
                let frame = key.t.try_to_frame_round(fps).ok()?;
                let selected = session.selected_keys.iter().any(|selector| {
                    selector.layer == layer && selector.property == property && selector.frame == frame
                });
                Some(PropertyKeyProjection { frame, selected })
            })
            .collect();
        if keys.is_empty() {
            continue;
        }
        out.push(PropertyRowProjection { layer, property, keys });
    }
    out
}

/// `rows` 内で `session.selection` が指す行の添字。property 行は必ずこの行の
/// すぐ下に挿入される(EXACT TARGET 1)。
pub fn selected_row_index(rows: &[RowProjection], session: &Session) -> Option<usize> {
    let layer = session.selection?;
    rows.iter().position(|row| row.id == layer)
}

/// レイヤー行 `index` の描画 top(ルーラー下相対、`row_height` の倍数からの
/// **押し下げぶん**だけを返す — 呼び出し側が `ruler_height` を足す、
/// `frame_to_x`/`frame_at_x` と同じ「呼び出し側が足す」約束)。選択 layer の下に
/// 挿入された property 行ぶんだけ、それより後ろの層行を押し下げる
/// (EXACT TARGET 1)。
///
/// 行 y 計算の唯一の正本(T3b・EXACT TARGET 3)。draw 側(`super::canvas::draw`/
/// `super::lane_bar::draw`)はこの関数を直接呼び、hit 側(`super::hit::hit_test`/
/// `super::lane_bar::hit_test`)はこの関数の逆写像である [`layer_row_at_y`] を
/// 経由する — どちらも同じ式から縦位置を導くので、絵と当たりが常に一致する
/// (旧 finding: 以前は hit 側だけがこの押し下げを知らず、展開行より下の
/// layer への bar/M・S・L クリックが縦にズレていた。T3 が記録し T3b で解消)。
pub fn layer_row_top(
    row_height: f32,
    param_row_height: f32,
    property_row_count: usize,
    selected_index: Option<usize>,
    index: usize,
) -> f32 {
    let base = row_height * index as f32;
    match selected_index {
        Some(selected) if index > selected => base + param_row_height * property_row_count as f32,
        _ => base,
    }
}

/// [`layer_row_top`] の逆写像 — y(ルーラー下相対)からレイヤー行の添字を返す。
/// `super::hit::hit_test`(クリップ面の bar 当たり判定)と
/// `super::lane_bar::hit_test`(レーンバー行/glyph の当たり判定)が共有する
/// 唯一の逆算(T3b EXACT TARGET 3 — 2箇所で別の式を持たない)。
///
/// 選択 layer の下に挿入された property 行の帯(押し下げの隙間)の内側は
/// どの層行にも属さないので `None` を返す(その y 範囲は `key_rows.rs` の
/// 帯が `input.rs`/`hit.rs` より先に自己完結で吸収する — mod doc 参照。ここで
/// `None` を返すのは、万一 key_rows の吸収を経ずに呼ばれても隣接する層へ
/// 誤って割り当てない安全側の振る舞い)。
pub(crate) fn layer_row_at_y(
    y: f32,
    row_height: f32,
    param_row_height: f32,
    property_row_count: usize,
    selected_index: Option<usize>,
) -> Option<usize> {
    if y < 0.0 || row_height <= 0.0 {
        return None;
    }
    let has_band = property_row_count > 0 && selected_index.is_some();
    if !has_band {
        return Some((y / row_height).floor() as usize);
    }
    let selected = selected_index.expect("has_band guards selected_index.is_some()");
    let boundary = row_height * (selected as f32 + 1.0);
    if y < boundary {
        return Some((y / row_height).floor() as usize);
    }
    let band_height = param_row_height * property_row_count as f32;
    if y < boundary + band_height {
        return None; // property 行の帯の内側 — 層行ではない。
    }
    let shifted = y - band_height;
    Some((shifted / row_height).floor() as usize)
}

/// 行順→時刻順(`key_order`、正典 §3・§4 と同じ文法)で並んだキー全体。
/// Shift 範囲選択が基準にする1本の列 — `key_rows.rs`(描画・当たり判定)と
/// `Shell::apply_key_selection`(範囲の確定)の両方がこの1つの関数を使う。
pub fn key_order(property_rows: &[PropertyRowProjection]) -> Vec<KeySelector> {
    property_rows
        .iter()
        .flat_map(|row| {
            row.keys.iter().map(move |key| KeySelector {
                layer: row.layer,
                property: row.property.clone(),
                frame: key.frame,
            })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// ドラッグ中のライブプレビュー(第2波T5、正典 §5.5「プレビューは毎フレーム」)。
// ---------------------------------------------------------------------------

/// クリップ drag のプレビューを行の投影へ焼き込む(`TimelinePane::
/// with_clip_preview` から呼ばれる純関数)。**`layer` が一致する1行だけ**
/// `start`/`duration` を置き換え、[`RowProjection::dragging`] を立てる —
/// 一致する行が無ければ黙って素通り(発明しない)。`preview` が `None` なら
/// `rows` をそのまま返す(通常描画、呼び出し側で分岐を増やさない)。
///
/// `TimelineDragState`(`crate::lib` 側の pane-local transient)を直接は
/// 知らない — 呼び出し側(`Shell::build_timeline_pane`)が `(layer,
/// drag.preview)` へ薄く写して渡す。EXACT TARGET 1 の「プレビュー後timing」。
pub(super) fn apply_clip_preview(
    rows: Vec<RowProjection>,
    preview: Option<(LayerId, LayerTiming)>,
) -> Vec<RowProjection> {
    let Some((layer, timing)) = preview else {
        return rows;
    };
    rows.into_iter()
        .map(|mut row| {
            if row.id == layer {
                row.start = timing.start;
                row.duration = timing.duration;
                row.dragging = true;
            }
            row
        })
        .collect()
}

/// キー drag/リタイムのプレビューを property 行へ焼き込む(`TimelinePane::
/// with_key_preview` から呼ばれる純関数)。`preview` は「掴んだ瞬間の
/// selector(layer/property/**旧**frame) → 新 frame」のペア列
/// (`TimelineKeyDragState::origins` と `preview` を呼び出し側が index で
/// ゆわえて渡す — この関数自体は `TimelineKeyDragState` を知らない)。
/// 一致する `(layer, property, frame)` の key だけ frame を置き換える —
/// 一致しなければ黙って素通り。`preview` が `None`(非ドラッグ中)なら
/// `rows` をそのまま返す。
///
/// リタイム中は選択キー全部が `origins`/`preview` に並ぶので、この1関数で
/// move/retime どちらのプレビューも同じ経路を通る(EXACT TARGET 4)。
pub(super) fn apply_key_preview(
    rows: Vec<PropertyRowProjection>,
    preview: Option<&[(KeySelector, i64)]>,
) -> Vec<PropertyRowProjection> {
    let Some(preview) = preview else {
        return rows;
    };
    rows.into_iter()
        .map(|mut row| {
            for key in &mut row.keys {
                if let Some(&(_, new_frame)) = preview.iter().find(|(selector, _)| {
                    selector.layer == row.layer
                        && selector.property == row.property
                        && selector.frame == key.frame
                }) {
                    key.frame = new_frame;
                }
            }
            row
        })
        .collect()
}

#[cfg(test)]
mod preview_tests {
    use super::*;
    use motolii_store::Speed;

    fn row(id: u64, start: i64, duration: i64) -> RowProjection {
        RowProjection {
            id: LayerId(id),
            name: String::new(),
            hidden: false,
            solo: false,
            locked: false,
            label_color: None,
            start,
            duration,
            selected: false,
            dragging: false,
            depth: 0,
            has_children: false,
            children_open: true,
        }
    }

    fn timing(start: i64, duration: i64) -> LayerTiming {
        LayerTiming { start, duration, source_in: 0, speed: Speed::NORMAL }
    }

    /// **オラクル(赤→緑)**: `preview` の layer に一致する行だけ start/duration が
    /// 置き換わり `dragging` が立つ。他の行は無傷。
    #[test]
    fn apply_clip_preview_replaces_only_the_matching_layer() {
        let rows = vec![row(1, 0, 50), row(2, 60, 20)];
        let out = apply_clip_preview(rows, Some((LayerId(2), timing(40, 10))));

        assert_eq!(out[0], row(1, 0, 50), "掴んでいない行が動いている");
        assert_eq!(out[1].start, 40);
        assert_eq!(out[1].duration, 10);
        assert!(out[1].dragging, "掴んでいる行の dragging が立っていない");
        assert!(!out[0].dragging);
    }

    /// `preview == None`(非ドラッグ中)は素通り — 呼び出しが増えても通常描画を
    /// 汚さない。
    #[test]
    fn apply_clip_preview_none_is_a_passthrough() {
        let rows = vec![row(1, 0, 50)];
        let out = apply_clip_preview(rows.clone(), None);
        assert_eq!(out, rows);
    }

    fn key_row(layer: LayerId, property: PropertyId, frames: &[i64]) -> PropertyRowProjection {
        PropertyRowProjection {
            layer,
            property,
            keys: frames
                .iter()
                .map(|&frame| PropertyKeyProjection { frame, selected: true })
                .collect(),
        }
    }

    /// **オラクル(赤→緑)**: 一致する selector(旧 frame)の key だけ新 frame へ
    /// 置き換わる。同じ property の他 key は無傷。
    #[test]
    fn apply_key_preview_replaces_only_the_matching_selector() {
        let layer = LayerId(1);
        let property = PropertyId::new("opacity").expect("opacity は予約語ではない");
        let rows = vec![key_row(layer, property.clone(), &[10, 20])];
        let pairs = [(KeySelector { layer, property: property.clone(), frame: 10 }, 15)];

        let out = apply_key_preview(rows, Some(&pairs));

        assert_eq!(out[0].keys[0].frame, 15, "一致した key の frame が置き換わっていない");
        assert_eq!(out[0].keys[1].frame, 20, "一致していない key まで動いている");
    }

    /// リタイムのように複数 key を同時にプレビューする形(EXACT TARGET 4)。
    #[test]
    fn apply_key_preview_moves_every_paired_key_in_one_pass() {
        let layer = LayerId(1);
        let property = PropertyId::new("opacity").expect("opacity は予約語ではない");
        let rows = vec![key_row(layer, property.clone(), &[0, 20, 70, 90])];
        let pairs = [
            (KeySelector { layer, property: property.clone(), frame: 20 }, 20),
            (KeySelector { layer, property: property.clone(), frame: 70 }, 41),
            (KeySelector { layer, property: property.clone(), frame: 90 }, 50),
        ];

        let out = apply_key_preview(rows, Some(&pairs));

        assert_eq!(
            out[0].keys.iter().map(|k| k.frame).collect::<Vec<_>>(),
            vec![0, 20, 41, 50],
            "選択キー全部が比例位置でプレビューされていない"
        );
    }

    /// 一致しない selector(別 layer/property/frame)は黙って無視 — 発明しない。
    #[test]
    fn apply_key_preview_ignores_a_selector_that_does_not_match_any_key() {
        let layer = LayerId(1);
        let other_layer = LayerId(9);
        let property = PropertyId::new("opacity").expect("opacity は予約語ではない");
        let rows = vec![key_row(layer, property.clone(), &[10])];
        let pairs = [(KeySelector { layer: other_layer, property: property.clone(), frame: 10 }, 99)];

        let out = apply_key_preview(rows, Some(&pairs));

        assert_eq!(out[0].keys[0].frame, 10, "一致しない selector で動いてしまった");
    }

    /// `preview == None`(非ドラッグ中)は素通り。
    #[test]
    fn apply_key_preview_none_is_a_passthrough() {
        let layer = LayerId(1);
        let property = PropertyId::new("opacity").expect("opacity は予約語ではない");
        let rows = vec![key_row(layer, property, &[10, 20])];
        let out = apply_key_preview(rows.clone(), None);
        assert_eq!(out, rows);
    }

    /// **オラクル(U1 finding「multi-select のハイライト未配線」の根治)**:
    /// `selected_layers` の一員は focus(`selection`)でなくても行が選択扱いに
    /// なる。focus 単独・非選択も従来どおり。
    #[test]
    fn row_selected_includes_multi_selection_members() {
        let mut session = Session::default();
        session.selection = Some(LayerId(1));
        session.selected_layers = vec![LayerId(1), LayerId(2)];

        assert!(row_selected(&session, LayerId(1)), "focus 行が選択扱いでない");
        assert!(
            row_selected(&session, LayerId(2)),
            "selected_layers の一員(非 focus)がハイライトされない — U1 finding の未配線"
        );
        assert!(!row_selected(&session, LayerId(3)), "非選択行まで選択扱いになっている");
    }
}

// ---------------------------------------------------------------------------
// 裁定173 H2 — ツリー行(旧世界 `timeline_rows.rs::rows` テストの概念移植)。
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tree_tests {
    use super::*;
    use motolii_store::{
        Composition, Document, Intent, LayerAttrsPatch, LayerMeta, LayerSource, LayerTiming,
    };

    fn doc_with_comp() -> Document {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 64,
            height: 64,
            fps: Fps::try_new(30, 1).expect("30/1 は正の既約 fps"),
            duration_frames: 300,
            background: [0.0, 0.0, 0.0, 1.0],
        }))
        .expect("comp 設定");
        doc
    }

    fn solid() -> LayerSource {
        LayerSource::Solid { rgba: [255, 0, 0, 255], width: 64, height: 64 }
    }

    /// `AddLayer`+`SetMeta` を1回で済ませる fixture ヘルパー(`layer_meta.rs`
    /// の `place` と同じ形)。
    fn place(doc: &mut Document, layer: LayerId, start: i64, duration: i64) {
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: solid(),
                    order: layer.0 as i16,
                    timing: LayerTiming { start, duration, source_in: 0, ..Default::default() },
                },
            },
        ])
        .expect("layer 配置");
    }

    fn set_parent(doc: &mut Document, layer: LayerId, parent: LayerId) {
        doc.apply(Intent::SetAttrs {
            layer,
            patch: LayerAttrsPatch { parent: Some(Some(parent)), ..Default::default() },
        })
        .expect("parent 設定");
    }

    /// 親(`Title scene`) > [子A(`Shared left`), 子B(`Reference text`)]、
    /// および parent を持たない兄弟(`Background`)。旧世界の fixture
    /// (`Group(キーあり) > [子A, 子B]`+兄弟の平clip)と同型 — Group は今回の
    /// スコープ外(H1)なので、代わりに「親を持つ普通の layer」で同じ形を作る。
    fn fixture() -> (Document, LayerId, LayerId, LayerId, LayerId) {
        let mut doc = doc_with_comp();
        let parent = LayerId(1);
        let child_a = LayerId(2);
        let child_b = LayerId(3);
        let sibling = LayerId(4);
        place(&mut doc, parent, 0, 100);
        place(&mut doc, child_a, 0, 100);
        place(&mut doc, child_b, 0, 100);
        place(&mut doc, sibling, 0, 100);
        set_parent(&mut doc, child_a, parent);
        set_parent(&mut doc, child_b, parent);
        (doc, parent, child_a, child_b, sibling)
    }

    /// **オラクル(赤→緑)**: fold 既定 = 全展開。parent を持つ子は、親を畳んで
    /// いなければ最初から見える(旧世界は既定畳みだったが、H2 は「導入前の
    /// 見た目を壊さない」ため既定を反転している — `TimelineFoldState` doc 参照)。
    #[test]
    fn default_fold_state_shows_the_whole_tree_expanded() {
        let (doc, parent, child_a, child_b, sibling) = fixture();
        let session = Session::default();
        let out = rows(&doc.view(), &session);

        let ids: Vec<LayerId> = out.iter().map(|r| r.id).collect();
        assert_eq!(
            ids,
            vec![parent, child_a, child_b, sibling],
            "既定(fold なし)で子が最初から展開されていない"
        );
        assert_eq!(out[0].depth, 0);
        assert_eq!(out[1].depth, 1, "子の深さが+1になっていない");
        assert_eq!(out[2].depth, 1);
        assert_eq!(out[3].depth, 0, "parentを持たない兄弟は最上位のまま");
        assert!(out[0].has_children, "子を持つ行が矢印を出さない");
        assert!(out[0].children_open);
        assert!(!out[3].has_children, "子を持たない行が矢印を出している");
    }

    /// **オラクル(赤→緑)**: 親を畳むと、子は消えるが親行自身は残る
    /// (旧世界 `group_closed_hides_children_but_keeps_the_group_row` の移植)。
    #[test]
    fn folding_the_parent_hides_children_but_keeps_the_parent_row() {
        let (doc, parent, child_a, child_b, sibling) = fixture();
        let mut session = Session::default();
        session.timeline_fold.fold(parent);
        let out = rows(&doc.view(), &session);

        let ids: Vec<LayerId> = out.iter().map(|r| r.id).collect();
        assert_eq!(ids, vec![parent, sibling], "畳んだ子が出てしまっている。親行は残るべき");
        assert!(!ids.contains(&child_a));
        assert!(!ids.contains(&child_b));
        assert!(out[0].has_children, "畳んでいても子を持つことは変わらない");
        assert!(!out[0].children_open);
    }

    /// **オラクル(赤→緑)**: 畳んで開き直すと同じ画面へ復元される(旧世界
    /// `reopening_a_group_restores_each_child_fold_state` の移植 — H2 は
    /// children_open の1軸のみ移植、旧世界の params_open 軸は別玉)。
    #[test]
    fn refolding_and_reopening_restores_the_same_screen() {
        let (doc, parent, ..) = fixture();
        let mut session = Session::default();
        let before = rows(&doc.view(), &session);

        session.timeline_fold.fold(parent);
        session.timeline_fold.unfold(parent);
        let after = rows(&doc.view(), &session);

        assert_eq!(before, after, "畳んで戻すと同じ画面へ復元されない");
    }

    /// **オラクル**: 兄弟の並びは常に `LayerId` 昇順(Document の書き込み順や
    /// parent 設定順に依存しない — `store.layers()` の既存順を踏襲)。
    #[test]
    fn sibling_order_follows_layer_id_ascending_not_write_order() {
        let mut doc = doc_with_comp();
        let parent = LayerId(10);
        let child_hi = LayerId(30);
        let child_lo = LayerId(20);
        place(&mut doc, parent, 0, 100);
        // 書き込み順は hi → lo(昇順の逆)。
        place(&mut doc, child_hi, 0, 100);
        place(&mut doc, child_lo, 0, 100);
        set_parent(&mut doc, child_hi, parent);
        set_parent(&mut doc, child_lo, parent);

        let out = rows(&doc.view(), &Session::default());
        let ids: Vec<LayerId> = out.iter().map(|r| r.id).collect();
        assert_eq!(ids, vec![parent, child_lo, child_hi], "兄弟の並びがLayerId昇順になっていない");
    }

    /// **オラクル(既存の柵の回帰確認)**: `validate_no_parent_cycle`(書き込み
    /// 時)は今も循環を拒む — H2 はこの既存ガードには触れていない。
    #[test]
    fn writing_a_cyclic_parent_is_still_rejected_at_write_time() {
        let mut doc = doc_with_comp();
        let a = LayerId(1);
        let b = LayerId(2);
        place(&mut doc, a, 0, 100);
        place(&mut doc, b, 0, 100);
        set_parent(&mut doc, a, b);

        let cyclic = doc.apply(Intent::SetAttrs {
            layer: b,
            patch: LayerAttrsPatch { parent: Some(Some(a)), ..Default::default() },
        });
        assert!(cyclic.is_err(), "書き込み時ガードが循環を通してしまっている");
    }

    /// **オラクル(H-survey §2.2 防衛)**: `validate_no_parent_cycle` は書き込み
    /// 時にしか働かない — 壊れた Document(バグ・旧ファイルの直接編集)を
    /// 読んだ時のために、投影段(`push_layer`)にも独立な第二の柵がある。
    /// ここでは書き口を経由せず `children_of` マップへ直接 A↔B の循環を
    /// 仕込み、`push_layer` を直接駆動して無限ループにならず・行を1つも
    /// 失わないことを確かめる(`rows()` は書き口の柵がある限り本物の循環
    /// Document を作れないので、内部関数を直接叩くのがこの柵を見る唯一の道)。
    #[test]
    fn push_layer_survives_a_cyclic_children_map_without_hanging_or_dropping_rows() {
        let mut doc = doc_with_comp();
        let a = LayerId(1);
        let b = LayerId(2);
        place(&mut doc, a, 0, 100); // parent は設定しない(attrs.parent = None のまま)。
        place(&mut doc, b, 0, 100);

        let mut children_of: HashMap<Option<LayerId>, Vec<LayerId>> = HashMap::new();
        children_of.insert(Some(a), vec![b]); // 人為的な循環: a の子が b、
        children_of.insert(Some(b), vec![a]); // b の子が a。

        let mut out = Vec::new();
        let mut visiting = HashSet::new();
        let mut emitted = HashSet::new();
        push_layer(&doc.view(), &Session::default(), a, 0, &children_of, &mut visiting, &mut emitted, &mut out);

        let ids: Vec<LayerId> = out.iter().map(|r| r.id).collect();
        assert_eq!(ids, vec![a, b], "循環マップで行が欠けた、または多重計上された");
        assert_eq!(out[0].depth, 0);
        assert_eq!(out[1].depth, 1, "循環の2番目のノードが子として1段深くならない");
        assert!(visiting.is_empty(), "呼び出し後にvisitingガードが掃除されていない(将来の再帰が誤爆する)");
    }

    /// Document に存在しない LayerId が fold 状態に残っていても壊れない
    /// (旧世界 `fold_state_for_a_missing_layer_is_ignored` の移植)。
    #[test]
    fn fold_state_for_a_missing_layer_is_ignored() {
        let (doc, parent, ..) = fixture();
        let mut session = Session::default();
        session.timeline_fold.fold(parent);

        let ghost = LayerId(999_999);
        session.timeline_fold.fold(ghost);
        session.timeline_fold.unfold(ghost);

        let out = rows(&doc.view(), &session);
        assert!(!out.iter().any(|r| r.id == ghost), "消えたLayerIdの行が出てしまっている");
    }

    /// 裁定174 G1(グループ化動詞)の実証: `fixture()` は「Group は今回のスコープ
    /// 外(H1)」と当時は注記していたが、G1 着地により `LayerSource::Group` を
    /// 実際の親として使える。`Document::group_layers`(既存 Intent の合成)で
    /// 組んだ Group が、H2 のツリー行へ**改造なしで**そのまま乗ることを見る
    /// — projection.rs 自身は G1 のために1行も変わっていない(裁定174 doc の
    /// 「H2 のツリー行が自動反映するはず」を直接確かめる)。
    #[test]
    fn a_group_layers_group_shows_up_as_a_tree_parent_with_no_projection_changes() {
        let mut doc = doc_with_comp();
        let (a, b, sibling) = (LayerId(1), LayerId(2), LayerId(3));
        place(&mut doc, a, 0, 100);
        place(&mut doc, b, 0, 100);
        place(&mut doc, sibling, 0, 100);

        let group = doc
            .group_layers(&[a, b])
            .expect("グループ化できる")
            .expect("非空選択なので Group が必ず生まれる");
        assert_eq!(doc.view().meta(group).unwrap().unwrap().source, LayerSource::Group);

        let session = Session::default();
        let out = rows(&doc.view(), &session);
        let ids: Vec<LayerId> = out.iter().map(|r| r.id).collect();
        // 兄弟は常に LayerId 昇順(`rows()` の doc 参照)。sibling(3) は
        // group(4)より id が若いので Group より先に並ぶ — Group は最後に
        // 生まれた layer なので `LayerId` が一番大きい(`next_layer_id` は
        // 常に最大+1)。
        assert_eq!(
            ids,
            vec![sibling, group, a, b],
            "Group とその子がツリー行として順どおりに並んでいない"
        );
        let group_row = &out[1];
        assert_eq!(group_row.id, group);
        assert_eq!(group_row.depth, 0);
        assert!(group_row.has_children, "Group 行が子持ち矢印を出していない");
        assert_eq!(out[0].depth, 0, "Group に属さない兄弟が最上位のまま");
        assert_eq!(out[2].depth, 1, "Group の子が深さ+1になっていない");
        assert_eq!(out[3].depth, 1);
    }
}
