//! property 行(キー行)の投影・行の縦位置(SP-2 分割、`projection.rs`
//! 620-771行を移設)。**中身は無改変**。

use super::*;

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
