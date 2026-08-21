//! Timeline の投影の純関数(`rows`/`frame_to_x`/`frame_at_x`/
//! `time_band_segment_frames`/`property_rows`/`layer_row_top`)。**読むだけ** —
//! Document/Session を書き換えない。

use motolii_store::{Fps, LayerId, LayerTiming, PropertyId, StoreView};

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
}

/// `store`/`session` から Timeline の行を組み立てる。**読むだけ**。
///
/// `store.layers()` は「present な layer」しか返さない(削除は墓標なので既に除外
/// 済み — `view.rs`)。ここでは並び順を `LayerId` 昇順のまま使う。bar の重ね順
/// (`meta.order`)は Stage 側の合成順であって、Timeline の縦位置の所有者にしない
/// (`ui-score-model.md` 4層構成: 縦位置は packing 結果にすぎない)。
pub fn rows(store: &StoreView<'_>, session: &Session) -> Vec<RowProjection> {
    let mut out = Vec::new();
    for id in store.layers() {
        let Ok(Some(meta)) = store.meta(id) else {
            continue;
        };
        let attrs = store.attrs(id).ok().flatten().unwrap_or_default();
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
        });
    }
    out
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

/// 小目盛の px 間隔下限(利用者裁定 2026-08-21 夜)。[`tick_steps`] が
/// [`step_ladder_frames`] の中からこれを下回らない最小のステップを選ぶ —
/// これより詰まった刻みは「読めない目盛り」なので次のステップへ切り上げる。
/// token ではなく画面上の可読性下限(pane-local な定数、`RULER_TICK_DIVISIONS`
/// と同格 — 発注書 EXACT TARGET 1)。
pub(crate) const MIN_MINOR_TICK_PX: f32 = 10.0;

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
    let mut out: Vec<i64> = vec![1, 5, 10];
    if let Some(fps) = fps {
        let fps = fps.as_f64();
        for seconds in [1.0, 2.0, 5.0, 10.0, 30.0, 60.0, 300.0] {
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
/// 小目盛 = [`step_ladder_frames`] の中で px 間隔が [`MIN_MINOR_TICK_PX`]
/// 以上になる最小のステップ。全ステップが下限に届かない極端な尺(巨大
/// `duration_frames`)では、退化させずラダー最大値へ落ちる。
///
/// 大目盛 = ラダー上で小目盛のちょうど5倍か10倍になっている直近上位のステップ
/// (`draw_ruler_ticks` doc 参照 — 秒/分混在ラダーは全区間が等比ではないため、
/// 2倍/3倍しか離れていない隣接ステップは飛ばす)。ラダー上に見つからなければ
/// 小目盛の10倍を直接計算し(ラダー外でも構わない)、「大目盛は常に小目盛の
/// 整数倍」だけは常に守る。
///
/// `pub`: `super::canvas::draw_ruler_ticks`/`draw_time_bands` と
/// `motolii_shell::screenshot` 器具が同じ刻みを再現するため(`frame_to_x` と
/// 同じ理由)。
pub fn tick_steps(fps: Option<Fps>, duration_frames: i64, clip_width: f32) -> (i64, i64) {
    let ladder = step_ladder_frames(fps);
    if duration_frames <= 0 || clip_width <= 0.0 {
        let minor = ladder.first().copied().unwrap_or(1);
        return (minor, minor.saturating_mul(5));
    }
    let px_per_frame = clip_width / duration_frames as f32;
    let minor = ladder
        .iter()
        .copied()
        .find(|&step| step as f32 * px_per_frame >= MIN_MINOR_TICK_PX)
        .unwrap_or_else(|| ladder.last().copied().unwrap_or(1));
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
/// `clip_width` を引数に足した(裁定160 切片7時点は無かった) — 大目盛は
/// px 密度({[`MIN_MINOR_TICK_PX`]})依存なので、区間幅も同じ入力を要る。
///
/// `pub`: `motolii_shell::screenshot` が Timeline canvas と同じ区間の刻み方を
/// 再現するため(`frame_to_x` と同じ理由、裁定160 切片7で緩めた)。
pub fn time_band_segment_frames(fps: Option<Fps>, duration_frames: i64, clip_width: f32) -> i64 {
    tick_steps(fps, duration_frames, clip_width).1
}

#[cfg(test)]
mod tick_tests {
    use super::*;

    fn fps30() -> Fps {
        Fps::try_new(30, 1).expect("30/1 は正の既約 fps")
    }

    /// **オラクル(a)**: 30fps・尺1800f(60s)・幅1349px → 小目盛=1s級(30f)、
    /// 大目盛は小目盛のちょうど5倍か10倍(150f=5s か 300f=10s のどちらか)。
    #[test]
    fn tick_steps_typical_case_lands_on_second_class_minor() {
        let (minor, major) = tick_steps(Some(fps30()), 1800, 1349.0);
        assert_eq!(minor, 30, "小目盛が1s(30f)級になっていない");
        assert!(
            major == 150 || major == 300,
            "大目盛が5s(150f)/10s(300f)のどちらでもない: {major}"
        );
        assert_eq!(major % minor, 0, "大目盛が小目盛の整数倍でない");
    }

    /// **オラクル(a)**: 大目盛は常に小目盛の整数倍(5倍か10倍)— どんな尺でも。
    #[test]
    fn major_is_always_an_integer_multiple_of_minor() {
        for duration in [1, 2, 10, 37, 100, 1_800, 12_345, 100_000, 5_000_000] {
            let (minor, major) = tick_steps(Some(fps30()), duration, 1349.0);
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
        let (minor, major) = tick_steps(Some(fps30()), 10, 1349.0);
        assert!(minor >= 1);
        assert!(major >= minor);
    }

    /// **オラクル(a)**: 極端に長い尺(100000f)でも同じラダーから退化なく
    /// 値が出る(巨大 duration で minor/major が0や負にならない)。
    #[test]
    fn tick_steps_does_not_degenerate_on_a_huge_duration() {
        let (minor, major) = tick_steps(Some(fps30()), 100_000, 1349.0);
        assert!(minor >= 1);
        assert!(major > 0 && major % minor == 0);
    }

    /// 小目盛の px 間隔は [`MIN_MINOR_TICK_PX`] 以上(ラダーが尽きて最大値へ
    /// 落ちる極端値を除く、通常域での契約)。
    #[test]
    fn minor_step_respects_the_min_px_floor_in_the_normal_range() {
        let (minor, _major) = tick_steps(Some(fps30()), 1800, 1349.0);
        let px_per_frame = 1349.0 / 1800.0;
        assert!(
            minor as f32 * px_per_frame >= MIN_MINOR_TICK_PX,
            "小目盛の px 間隔が下限を下回っている"
        );
    }

    /// fps が引けない(comp 無し)時も 0 割り/パニックせず、最小ラダー
    /// (1,5,10)から値を返す。
    #[test]
    fn tick_steps_without_fps_falls_back_to_the_short_ladder() {
        let (minor, major) = tick_steps(None, 100, 1349.0);
        assert!(minor >= 1);
        assert!(major >= minor && major % minor == 0);
    }

    /// `duration_frames <= 0`/`clip_width <= 0.0` は空 comp と同じ安全側
    /// (パニックしない、`minor <= major`)。
    #[test]
    fn tick_steps_guards_non_positive_inputs() {
        assert_eq!(tick_steps(Some(fps30()), 0, 1349.0).0, 1);
        assert_eq!(tick_steps(Some(fps30()), 1800, 0.0).0, 1);
        assert_eq!(tick_steps(Some(fps30()), -5, 1349.0).0, 1);
    }

    /// **オラクル(b)**: 明暗帯の区間幅(旧 `time_band_segment_frames`)は
    /// `tick_steps` の大目盛と常に一致する(2箇所で別の刻み方を持たない)。
    #[test]
    fn time_band_segment_matches_tick_steps_major() {
        let (_minor, major) = tick_steps(Some(fps30()), 1800, 1349.0);
        assert_eq!(time_band_segment_frames(Some(fps30()), 1800, 1349.0), major);
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
