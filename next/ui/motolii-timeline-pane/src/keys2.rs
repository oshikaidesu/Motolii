//! Timeline 第4切片(B15 キーフレーム束・B20 再生ヘッド移動束の残り)の
//! **意味の純関数**。`key_gesture.rs`(時刻ドラッグ/リタイム)・`nav.rs`
//! (playhead ナビゲーション)と対になる置き場 — この3ファイルはどれも
//! `motolii_store::Document`/`StoreView` を持たず、frame(`i64`)と
//! 呼び出し側が集めた投影(`RowProjection`/`PropertyRowProjection`)だけを
//! 受けて答えを返す。**結線(`Message`への解決・`Shell::update`)は次波** —
//! ここは部品のみ(発注書 EXACT TARGET)。
//!
//! ## 意味の出典
//!
//! `next/reference/normal-map.tsv` の `bundle` 列 `B15`(キーフレーム束)・
//! `B20`(再生ヘッド移動束)、`採用予定` 行から採った。**意味を発明しない** —
//! 各関数の doc に採用元 id を記す。見送った行とその理由は RETURN(発注書
//! 規律)で報告する(この地図自体は不触)。
//!
//! ## なぜ `motolii-eval` に触れないか
//!
//! この crate の依存は `iced`・`motolii-store`・`motolii-tokens-rs`・
//! `motolii-shell-state`・`motolii-icons` のみ(`Cargo.toml`)。ease/interpolation
//! の実際の型(Bezier/Hold/Linear)は `motolii-eval::track` 側にあり、
//! ここへは依存を足さない([`EaseSide`]/[`KeyInterpolation`] は
//! **pane-local の意味の顔**でしかない — `lib.rs` mod doc が言う
//! 「pane-local `Message`」と同じ「値は運ぶが機構は持たない」役割分担。
//! 実際の Bezier/Hold への変換は結線側=`motolii-shell` の仕事)。

use motolii_store::LayerId;

use crate::projection::{PropertyRowProjection, RowProjection};
use crate::state::KeySelector;

// ---------------------------------------------------------------------------
// キーの追加/トグル(map id 472-474 Add Keyframe・476 Add or remove keyframe
// at current time・477-478 Add Static Keyframe の位置決め)。
// ---------------------------------------------------------------------------

/// 「現在時刻にキーを追加/削除」(map 476)の判定結果。**位置決めだけ**を返す
/// — 実際に書き込む値(Static/Hold にするかどうかの id 477-478 の種別、
/// 補間前後の値そのもの)は `motolii-eval::KeyframeTrack` 側の仕事(呼び手が
/// 決める)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyframeToggle {
    /// この frame にはまだキーが無い → 追加する。
    Add,
    /// この frame に既にキーがある(`existing_frames` 内の添字)→ 削除する。
    Remove(usize),
}

/// map id 472(Add Keyframe)・473(同、公式 BMD 表記違い)・474(同、ショートカット
/// 表記違い)・476(Add or remove keyframe at current time)・477/478(Add Static
/// Keyframe — 挿入位置の判定は同じ、Hold 種別の指定は `motolii-eval` 側の
/// 領分なのでここでは扱わない)をまとめて1つの判定に落とす。
///
/// `existing_frames` は表示順(`property_rows` の `keys` 由来)である必要は
/// ない — ここは線形探索で足りる小さな集合(1 property あたりのキー数)。
pub fn toggle_keyframe_at(existing_frames: &[i64], frame: i64) -> KeyframeToggle {
    match existing_frames.iter().position(|&f| f == frame) {
        Some(index) => KeyframeToggle::Remove(index),
        None => KeyframeToggle::Add,
    }
}

// ---------------------------------------------------------------------------
// Easy Ease(map id 485-490)・Keyframe Velocity…(497)・Set velocity for
// selected keyframes(516)— 「どのキーの・どちら側に効かせるか」の対象解決。
// ---------------------------------------------------------------------------

/// Easy Ease の適用側(map 485「Easy Ease」=Both・486/489「…In」=In・
/// 487/490「…Out」=Out)。数値指定版(497 Keyframe Velocity…・516 Set velocity
/// for selected keyframes)も同じ対象解決を使う — 数値そのものの意味
/// (speed/influence)は `motolii-eval` 側のパラメータで、ここは運ばない。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EaseSide {
    In,
    Out,
    Both,
}

/// 選択キー集合の全員に同じ `side` を適用対象として割り当てる(map
/// 485-490・497・516)。**カーブの形自体は計算しない** — `motolii-eval` の
/// Bezier tangent 計算へ渡す「対象と側」の組だけがここの責務(mod doc の
/// 依存無し原則)。
pub fn ease_selection(selected: &[KeySelector], side: EaseSide) -> Vec<(KeySelector, EaseSide)> {
    selected.iter().cloned().map(|key| (key, side)).collect()
}

// ---------------------------------------------------------------------------
// 補間種別の設定(map id 495 Keyframe Interpolation…・512 Set interpolation
// for keyframes・513/514/515 Set interpolation to.../520 Toggle Hold Keyframe)。
// ---------------------------------------------------------------------------

/// pane-local の補間種別の顔。**`motolii-eval` の実際の enum の写しではない**
/// — 値を運ぶだけ(mod doc 参照)。実変換は結線側。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyInterpolation {
    Hold,
    Linear,
    Bezier,
}

/// map id 495(ダイアログ)・512(Set interpolation for keyframes)・
/// 513(linear/Auto Bezier)・514(linear/hold)・515(hold/Auto Bezier)を
/// まとめて1つの対象解決に落とす — [`ease_selection`] と同じ形
/// (選択集合全員に同じ種別を割り当てる)。
pub fn interpolation_targets(
    selected: &[KeySelector],
    kind: KeyInterpolation,
) -> Vec<(KeySelector, KeyInterpolation)> {
    selected.iter().cloned().map(|key| (key, kind)).collect()
}

/// map id 520(Toggle Hold Keyframe)。**現在値が要る**(トグルなので)—
/// 呼び手(`motolii-shell`)が該当キーの現在の補間種別を渡す
/// (`motolii-eval::KeyframeTrack` から読む、pane はここでも Document を
/// 持たない)。Hold ならそれ以外(Linear)へ、それ以外なら Hold へ。
pub fn toggle_hold(current: KeyInterpolation) -> KeyInterpolation {
    if current == KeyInterpolation::Hold {
        KeyInterpolation::Linear
    } else {
        KeyInterpolation::Hold
    }
}

// ---------------------------------------------------------------------------
// クリップボード(コピー/ペースト — map id 507 Reverse paste copied
// keyframes の前提となる型を新設。地図に「Copy/Paste keyframes」単体の行は
// 無い — 507 が要求する「逆順貼り付け」は普通の貼り付けが先に要るので、
// その土台としてここへ型ごと新設する、発注書「clipboard 型が要るなら型ごと
// 新設」)。
// ---------------------------------------------------------------------------

/// クリップボード内の1本のキー。`offset` はコピー時点の anchor(選択集合の
/// 最早 frame)からの相対値 — 絶対 frame を持たないので、どの時刻へ貼っても
/// 同じ相対配置を再現できる(`key_gesture::moved_key_group` が「掴んだ瞬間の
/// 値を基準に出し直す」のと同じ、delta を蓄積しない考え方)。
#[derive(Clone, Debug, PartialEq)]
pub struct ClipboardKey {
    pub layer: LayerId,
    pub property: motolii_store::PropertyId,
    pub offset: i64,
}

/// コピー済みキー一式。空(何もコピーしていない)を表せるよう `Default`。
#[derive(Clone, Debug, PartialEq, Default)]
pub struct KeyClipboard {
    pub keys: Vec<ClipboardKey>,
}

/// 選択キー集合をクリップボードへ写す。anchor は選択集合内の最早 frame —
/// `selected` が空なら空のクリップボードを返す(貼り付け側は no-op で吸収)。
pub fn copy_keys(selected: &[KeySelector]) -> KeyClipboard {
    let Some(anchor) = selected.iter().map(|key| key.frame).min() else {
        return KeyClipboard::default();
    };
    KeyClipboard {
        keys: selected
            .iter()
            .map(|key| ClipboardKey { layer: key.layer, property: key.property.clone(), offset: key.frame - anchor })
            .collect(),
    }
}

/// 通常貼り付け: `target` を新しい anchor として、コピー時の相対配置を
/// そのまま再現する。
pub fn paste_keys(clip: &KeyClipboard, target: i64) -> Vec<KeySelector> {
    clip.keys
        .iter()
        .map(|key| KeySelector { layer: key.layer, property: key.property.clone(), frame: target + key.offset })
        .collect()
}

/// map id 507(Reverse paste copied keyframes)。集合を時間反転してから
/// `target` を新しい anchor として貼る — 最大 offset を持つキーが `target`
/// (先頭)へ来て、offset 0 のキー(元の anchor)が末尾へ来る
/// ([`reversed_key_group`] と同じ「min+max-f」の鏡映と等価: ここでは
/// offset 空間で `max_offset - offset` を使う、両者は同型)。
pub fn paste_keys_reversed(clip: &KeyClipboard, target: i64) -> Vec<KeySelector> {
    let max_offset = clip.keys.iter().map(|key| key.offset).max().unwrap_or(0);
    clip.keys
        .iter()
        .map(|key| {
            KeySelector { layer: key.layer, property: key.property.clone(), frame: target + (max_offset - key.offset) }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// 選択(map id 509 Select all keyframes for property)。
// ---------------------------------------------------------------------------

/// 指定 property(`layer`+`property`)の全キーを選択集合として返す(map 509)。
/// 「表示中の全キー・全 property を選択」(map 510)は
/// `projection::key_order(property_rows)` が既に同じ意味を持つ
/// (行順→時刻順で全キーを1本の列にする)ので、ここでは重複させない
/// — 見送り理由は RETURN 参照。
pub fn keys_for_property(
    property_rows: &[PropertyRowProjection],
    layer: LayerId,
    property: &motolii_store::PropertyId,
) -> Vec<KeySelector> {
    property_rows
        .iter()
        .find(|row| row.layer == layer && &row.property == property)
        .map(|row| {
            row.keys
                .iter()
                .map(|key| KeySelector { layer, property: property.clone(), frame: key.frame })
                .collect()
        })
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Time-Reverse Keyframes(map id 518)。
// ---------------------------------------------------------------------------

/// 選択キー集合を、その集合自身の `[min, max]` の中心を軸に時間反転する
/// (map 518)。`key_gesture::retimed_key_frame` の anchor 固定伸縮とは別物
/// — ここは固定端が無く、集合全体が自分自身の範囲内で鏡映する。
pub fn reversed_key_group(frames: &[i64]) -> Vec<i64> {
    let (Some(&min), Some(&max)) = (frames.iter().min(), frames.iter().max()) else {
        return Vec::new();
    };
    frames.iter().map(|&f| min + max - f).collect()
}

// ---------------------------------------------------------------------------
// Next/Previous Clip/Edit(map id 1088・1089・1108・1109)。
// ---------------------------------------------------------------------------

/// 表示中の**全 clip**(選択中の1本だけではない)の start/end を集める(map
/// 1088「Next Clip/Edit」・1089「Next Edit」〈Premiere 表記違い〉・1108/1109
/// の Previous 側)。`nav::nearest_meaning_point`(既存、汎用「意味点」への
/// 前後移動)へそのまま渡せる形 — `nav::clip_edge_frame` は選択中の1 clip
/// 限定なので、複数 clip 横断の編集点ジャンプにはこちらが要る。
pub fn clip_edit_points(rows: &[RowProjection]) -> Vec<i64> {
    let mut out = Vec::with_capacity(rows.len() * 2);
    for row in rows {
        out.push(row.start);
        out.push(row.start + row.duration);
    }
    out
}

/// 表示中 property 行のキーフレーム時刻を、既存の意味点ナビゲータへ渡す
/// 列へ写す。property 行は選択 layer のキーを持つ行だけなので、ここで別の
/// Document 走査や「見えていない layer」の混入を行わない。
pub fn property_key_points(rows: &[PropertyRowProjection]) -> Vec<i64> {
    rows.iter().flat_map(|row| row.keys.iter().map(|key| key.frame)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn selector(layer: u64, property: &motolii_store::PropertyId, frame: i64) -> KeySelector {
        KeySelector { layer: LayerId(layer), property: property.clone(), frame }
    }

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

    // -- toggle_keyframe_at (map 472-474/476/477-478) -----------------------

    #[test]
    fn toggle_keyframe_at_adds_when_no_key_sits_on_the_frame() {
        assert_eq!(toggle_keyframe_at(&[10, 20], 15), KeyframeToggle::Add);
    }

    #[test]
    fn toggle_keyframe_at_removes_the_matching_index_when_a_key_already_sits_there() {
        assert_eq!(toggle_keyframe_at(&[10, 20, 30], 20), KeyframeToggle::Remove(1));
    }

    // -- ease_selection (map 485-490/497/516) --------------------------------

    #[test]
    fn ease_selection_tags_every_selected_key_with_the_same_side() {
        let property = motolii_store::PropertyId::new("opacity").expect("opacity は予約語ではない");
        let selected = vec![selector(1, &property, 10), selector(1, &property, 20)];
        let tagged = ease_selection(&selected, EaseSide::Out);
        assert_eq!(tagged.len(), 2);
        assert!(tagged.iter().all(|(_, side)| *side == EaseSide::Out));
    }

    #[test]
    fn ease_selection_of_an_empty_set_is_empty() {
        assert!(ease_selection(&[], EaseSide::Both).is_empty());
    }

    // -- interpolation_targets / toggle_hold (map 495/512-515/520) -----------

    #[test]
    fn interpolation_targets_tags_every_selected_key_with_the_same_kind() {
        let property = motolii_store::PropertyId::new("opacity").expect("opacity は予約語ではない");
        let selected = vec![selector(1, &property, 10)];
        let tagged = interpolation_targets(&selected, KeyInterpolation::Bezier);
        assert_eq!(tagged, vec![(selected[0].clone(), KeyInterpolation::Bezier)]);
    }

    #[test]
    fn toggle_hold_flips_between_hold_and_linear() {
        assert_eq!(toggle_hold(KeyInterpolation::Hold), KeyInterpolation::Linear);
        assert_eq!(toggle_hold(KeyInterpolation::Linear), KeyInterpolation::Hold);
        assert_eq!(toggle_hold(KeyInterpolation::Bezier), KeyInterpolation::Hold, "Hold以外は全てHoldへ倒す");
    }

    // -- clipboard (map 507、土台の copy/paste含む) --------------------------

    #[test]
    fn copy_keys_records_offsets_relative_to_the_earliest_selected_key() {
        let property = motolii_store::PropertyId::new("opacity").expect("opacity は予約語ではない");
        let selected = vec![selector(1, &property, 100), selector(1, &property, 130), selector(1, &property, 150)];
        let clip = copy_keys(&selected);
        let offsets: Vec<i64> = clip.keys.iter().map(|k| k.offset).collect();
        assert_eq!(offsets, vec![0, 30, 50]);
    }

    #[test]
    fn copy_keys_of_an_empty_selection_yields_an_empty_clipboard() {
        assert!(copy_keys(&[]).keys.is_empty());
    }

    #[test]
    fn paste_keys_reproduces_the_relative_layout_at_the_new_target() {
        let property = motolii_store::PropertyId::new("opacity").expect("opacity は予約語ではない");
        let selected = vec![selector(1, &property, 100), selector(1, &property, 130)];
        let clip = copy_keys(&selected);
        let pasted = paste_keys(&clip, 500);
        let frames: Vec<i64> = pasted.iter().map(|k| k.frame).collect();
        assert_eq!(frames, vec![500, 530], "相対間隔(30)が貼り付け先で保たれていない");
    }

    /// map 507: 逆順貼り付けは集合を時間反転してから target を先頭に置く。
    #[test]
    fn paste_keys_reversed_flips_the_group_before_placing_it_at_the_target() {
        let property = motolii_store::PropertyId::new("opacity").expect("opacity は予約語ではない");
        let selected = vec![selector(1, &property, 100), selector(1, &property, 110), selector(1, &property, 150)];
        let clip = copy_keys(&selected);
        let pasted = paste_keys_reversed(&clip, 200);
        let mut frames: Vec<i64> = pasted.iter().map(|k| k.frame).collect();
        frames.sort_unstable();
        // offsets [0, 10, 50] → reversed offsets [50, 40, 0] → +200 = [200, 240, 250]。
        assert_eq!(frames, vec![200, 240, 250]);
    }

    // -- keys_for_property (map 509) -----------------------------------------

    #[test]
    fn keys_for_property_returns_only_the_matching_property_row_keys() {
        let opacity = motolii_store::PropertyId::new("opacity").expect("opacity は予約語ではない");
        let position = motolii_store::PropertyId::new("position").expect("position は予約語ではない");
        let rows = vec![
            PropertyRowProjection {
                layer: LayerId(1),
                property: opacity.clone(),
                keys: vec![
                    crate::projection::PropertyKeyProjection { frame: 10, selected: false },
                    crate::projection::PropertyKeyProjection { frame: 20, selected: false },
                ],
            },
            PropertyRowProjection {
                layer: LayerId(1),
                property: position.clone(),
                keys: vec![crate::projection::PropertyKeyProjection { frame: 5, selected: false }],
            },
        ];
        let selected = keys_for_property(&rows, LayerId(1), &opacity);
        let frames: Vec<i64> = selected.iter().map(|k| k.frame).collect();
        assert_eq!(frames, vec![10, 20], "無関係な property (position) のキーまで混ざっている");
    }

    #[test]
    fn keys_for_property_returns_empty_when_the_property_has_no_row() {
        let opacity = motolii_store::PropertyId::new("opacity").expect("opacity は予約語ではない");
        assert!(keys_for_property(&[], LayerId(1), &opacity).is_empty());
    }

    // -- reversed_key_group (map 518) -----------------------------------------

    #[test]
    fn reversed_key_group_mirrors_around_the_midpoint_of_its_own_span() {
        assert_eq!(reversed_key_group(&[0, 25, 50, 100]), vec![100, 75, 50, 0]);
    }

    #[test]
    fn reversed_key_group_of_a_single_key_is_a_no_op() {
        assert_eq!(reversed_key_group(&[42]), vec![42]);
    }

    #[test]
    fn reversed_key_group_of_an_empty_set_is_empty() {
        assert!(reversed_key_group(&[]).is_empty());
    }

    // -- clip_edit_points (map 1088/1089/1108/1109) ---------------------------

    #[test]
    fn clip_edit_points_collects_start_and_end_across_all_rows() {
        let rows = vec![row(1, 0, 90), row(2, 200, 30)];
        let points = clip_edit_points(&rows);
        assert_eq!(points, vec![0, 90, 200, 230]);
    }

    #[test]
    fn clip_edit_points_of_no_rows_is_empty() {
        assert!(clip_edit_points(&[]).is_empty());
    }

    #[test]
    fn property_key_points_flattens_visible_property_rows() {
        let property = motolii_store::PropertyId::new("opacity").expect("opacity は予約語ではない");
        let rows = vec![PropertyRowProjection {
            layer: LayerId(1),
            property,
            keys: vec![
                PropertyKeyProjection { frame: 30, selected: false },
                PropertyKeyProjection { frame: 90, selected: true },
            ],
        }];
        assert_eq!(property_key_points(&rows), vec![30, 90]);
    }
}
