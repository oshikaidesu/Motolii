//! Timeline の投影の純関数(`rows`/`frame_to_x`/`frame_at_x`/
//! `time_band_segment_frames`/`property_rows`/`layer_row_top`)。**読むだけ** —
//! Document/Session を書き換えない。

use motolii_store::{Fps, LayerId, PropertyId, StoreView};

use crate::Session;

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
    pub start: i64,
    pub duration: i64,
    pub selected: bool,
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
            start: meta.timing.start,
            duration: meta.timing.duration,
            selected: session.selection == Some(id),
        });
    }
    out
}

/// comp フレーム → x px。`duration_frames <= 0` の空 comp では常に 0。
///
/// `pub(crate)`: screenshot 器具(`crate::screenshot`)が Timeline canvas と同じ
/// x 座標計算を使うため(マーカー・bar の位置を2箇所で別の式にしない)。
pub(crate) fn frame_to_x(frame: i64, width: f32, duration_frames: i64) -> f32 {
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

/// ルーラー目盛りの分割数。fps が引けない(comp 無し)時の時間方向リズム
/// ([`time_band_segment_frames`]、`super::canvas::draw_ruler_ticks` の実描画)の
/// フォールバックも同じ分割を使う — 「ルーラーと違う区間の刻み方」という
/// 新しい規則を増やさない。
/// `pub(crate)`: `super::canvas`(同じ `draw_ruler_ticks`)と `screenshot.rs`
/// 器具が同じ区間の刻み方を再現するのにも使う(`frame_to_x` と同じ理由)。
pub(crate) const RULER_TICK_DIVISIONS: i64 = 8;

/// 時間方向の明暗リズム(裁定148(1))の区間幅(フレーム数)。fps が引ければ
/// 1秒、引けなければ [`RULER_TICK_DIVISIONS`] 等分へ落ちる。`draw_time_bands`
/// と screenshot 器具の両方がこの1つの式から区間境界を出す(2箇所で別の
/// フォールバックを持たない)。
///
/// `pub(crate)`: `crate::screenshot` が Timeline canvas と同じ区間の刻み方を
/// 再現するため(`frame_to_x` と同じ理由)。
pub(crate) fn time_band_segment_frames(fps: Option<Fps>, duration_frames: i64) -> i64 {
    fps.map(|fps| fps.as_f64().round().max(1.0) as i64)
        .unwrap_or_else(|| (duration_frames / RULER_TICK_DIVISIONS).max(1))
}

// ---------------------------------------------------------------------------
// property 行(キー行) — 第2波 T3(裁定148/151・正典 §1.5/§3)。
// ---------------------------------------------------------------------------

/// Timeline のキー選択の識別子。**Document ではなく [`Session`] が持つ**
/// (EXACT TARGET 3: 選択状態は Session、undo の対象でも Document の写しでもない)。
/// `frame` は同一 property 内で一意(`KeyframeTrack::insert` が同時刻キーを
/// 上書きする — `motolii-eval` 側の保証)なので、この3つ組で1本のキーを指せる。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KeySelector {
    pub layer: LayerId,
    pub property: PropertyId,
    pub frame: i64,
}

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
/// **write-set 外 finding**(KNOWN 節: 本レーンは `input.rs`/`hit.rs` を編集
/// しない): `super::hit::hit_test`(クリップ面の bar 当たり判定)と
/// `super::lane_bar::hit_test`(レーンバー行/glyph の当たり判定)はどちらも
/// この関数を経由しない旧来の一様な `ruler_height + row_height * index` の
/// ままで、この拡張を知らない。選択 layer にキー付き property があり(=
/// property 行が展開している)、かつ他の層がその**下に**並んでいる間、それらの
/// 層の bar/M・S・L クリックは実際の描画位置(`super::canvas::draw`/
/// `super::lane_bar::draw` はどちらも本関数で正しい位置に描く)から
/// `property_row_count * param_row_height` ぶんズレる。解消には
/// `hit.rs`/`lane_bar.rs::hit_test` の呼び出し元(`input.rs`)がこの関数(相当の
/// 投影)を受け取れるよう署名を広げる必要があるが、それは本レーンの write-set
/// (`mod.rs`/`canvas.rs`/`lane_bar.rs` の draw のみ/`projection.rs`/
/// `key_rows.rs`)の外なので、ここでは直さず記録するだけに留める(並走レーン
/// lane-shell が `input.rs`/`hit.rs` を触っている — 統合はそちら側の仕事)。
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

/// property 行のキー選択操作(正典 §3・§4 と同じ文法: 単独/Cmd トグル/Shift 範囲)。
/// **選択の確定はここでは行わない** — `Session::selected_keys`/`key_anchor` の
/// 読み取りが要るので、唯一の書き口である `Shell::update` 側が確定する
/// (`super::key_rows` は「どのキーを・どの操作で」の判定だけを自己完結で持つ)。
#[derive(Debug, Clone, PartialEq)]
pub enum KeySelectionOp {
    /// クリック=単独。
    Single(KeySelector),
    /// Cmd=トグル(足し引き)。
    Toggle(KeySelector),
    /// Shift=`Session::key_anchor` から `key_order` 上の範囲。基点が無ければ
    /// 単独選択へ安全側で倒す(`Shell::apply_key_selection` 参照)。
    Range(KeySelector),
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
