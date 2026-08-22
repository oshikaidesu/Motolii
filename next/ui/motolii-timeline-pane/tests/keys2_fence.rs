//! Timeline 第4切片(B15 キーフレーム束・B20 再生ヘッド移動束の残り)の
//! 落ちるテスト先行。`src/keys2.rs` の単体試験(inline `#[cfg(test)]`)は
//! 各関数の入出力を1つずつ確かめる — ここは**crate 境界の外から**、
//! `keys2` が既存の `projection`/`nav` と噛み合うことを確かめる統合オラクル
//! (`transport_fence.rs`/`work_area_shuttle_fence.rs` と同じ「_fence」の役割)。
//!
//! 意味の出典は `next/reference/normal-map.tsv` の bundle 列 B15/B20
//! (発注書 2026-08-22「Timeline 第4切片」)。**この波は結線しない** —
//! 統合手順は発注 RETURN に書く。ここは部品同士の整合性だけを縛る。

use motolii_shell_state::KeySelector;
use motolii_store::{LayerId, PropertyId};
use motolii_timeline_pane::key_order;
use motolii_timeline_pane::keys2::{
    clip_edit_points, copy_keys, ease_selection, interpolation_targets, keys_for_property, paste_keys,
    paste_keys_reversed, reversed_key_group, toggle_hold, toggle_keyframe_at, EaseSide, KeyInterpolation,
    KeyframeToggle,
};
use motolii_timeline_pane::nav::{nearest_meaning_point, JumpDirection};
use motolii_timeline_pane::{PropertyKeyProjection, PropertyRowProjection, RowProjection};

fn opacity() -> PropertyId {
    PropertyId::new("opacity").expect("opacity は予約語ではない")
}

fn key(layer: u64, property: &PropertyId, frame: i64) -> KeySelector {
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

// ---------------------------------------------------------------------------
// (a) クリップボード往復(map 507 の土台) — 恒等貼り付け・二重反転
// ---------------------------------------------------------------------------

/// 「コピーしたその場に貼る」= 恒等写像(相対配置がそのまま絶対値に戻る)。
#[test]
fn pasting_at_the_original_anchor_reproduces_the_original_frames() {
    let property = opacity();
    let selected = vec![key(1, &property, 100), key(1, &property, 130), key(1, &property, 150)];
    let clip = copy_keys(&selected);
    let pasted = paste_keys(&clip, 100);
    let frames: Vec<i64> = pasted.iter().map(|k| k.frame).collect();
    assert_eq!(frames, vec![100, 130, 150]);
}

/// map 507: 逆順貼り付けを**2回**行うと、相対配置は元に戻る(反転は対合
/// 〈involution〉— 2回適用すれば恒等)。1回目の逆順貼り付け結果を再度コピー
/// して逆順貼り付けすれば、最初の絶対 target に戻ることを確かめる。
#[test]
fn reverse_pasting_twice_returns_to_the_original_relative_layout() {
    let property = opacity();
    let selected = vec![key(1, &property, 100), key(1, &property, 110), key(1, &property, 150)];
    let clip = copy_keys(&selected);

    let once = paste_keys_reversed(&clip, 500);
    let reclipped = copy_keys(&once);
    let twice = paste_keys_reversed(&reclipped, 500);

    let mut frames: Vec<i64> = twice.iter().map(|k| k.frame).collect();
    frames.sort_unstable();
    assert_eq!(frames, vec![500, 510, 550], "二重反転が恒等になっていない(相対配置がコピー元と一致しない)");
}

/// 空選択からのコピー→貼り付けは常に空(no-op) — 呼び手が selected を
/// 空で渡しても panic しない。
#[test]
fn copy_paste_round_trip_on_an_empty_selection_stays_empty() {
    let clip = copy_keys(&[]);
    assert!(paste_keys(&clip, 999).is_empty());
    assert!(paste_keys_reversed(&clip, 999).is_empty());
}

// ---------------------------------------------------------------------------
// (b) toggle_keyframe_at ⇔ reversed_key_group: 挿入位置判定と時間反転
//     (map 472-478・518)が同じ frame 集合の上で矛盾なく振る舞う。
// ---------------------------------------------------------------------------

/// 反転してできた新しい frame 集合に対しても、`toggle_keyframe_at` は
/// 「既存のどれかと一致すれば Remove、しなければ Add」という同じ規則で
/// answers を返す(反転が判定ロジックを壊さない)。
#[test]
fn toggle_keyframe_at_agrees_with_reversed_frames() {
    let original = vec![0, 25, 50, 100];
    let reversed = reversed_key_group(&original);
    assert_eq!(reversed, vec![100, 75, 50, 0]);
    // 反転後の集合に対して、反転後の値そのものはどれも Remove と判定される。
    for &f in &reversed {
        assert!(matches!(toggle_keyframe_at(&reversed, f), KeyframeToggle::Remove(_)));
    }
    // 反転後の集合に無い値は Add。
    assert_eq!(toggle_keyframe_at(&reversed, 999), KeyframeToggle::Add);
}

// ---------------------------------------------------------------------------
// (c) ease_selection / interpolation_targets: 対象解決が選択集合を
//     過不足なく写す(map 485-490・495・512-515・516・520)。
// ---------------------------------------------------------------------------

#[test]
fn ease_selection_and_interpolation_targets_preserve_the_selection_set_without_dropping_or_duplicating() {
    let property = opacity();
    let selected = vec![key(1, &property, 10), key(1, &property, 20), key(2, &property, 10)];

    let eased = ease_selection(&selected, EaseSide::In);
    assert_eq!(eased.len(), selected.len());
    for original in &selected {
        assert!(eased.iter().any(|(k, side)| k == original && *side == EaseSide::In));
    }

    let interpolated = interpolation_targets(&selected, KeyInterpolation::Bezier);
    assert_eq!(interpolated.len(), selected.len());
    for original in &selected {
        assert!(interpolated.iter().any(|(k, kind)| k == original && *kind == KeyInterpolation::Bezier));
    }
}

/// map 520: Toggle Hold は Hold と非Hold の2値往復(3値目 Bezier からも
/// Hold へ倒れる、単体試験と同じ規則をここでも再確認)。
#[test]
fn toggle_hold_is_its_own_inverse_between_hold_and_linear() {
    let started_hold = KeyInterpolation::Hold;
    let flipped = toggle_hold(started_hold);
    let flipped_back = toggle_hold(flipped);
    assert_eq!(flipped_back, started_hold);
}

// ---------------------------------------------------------------------------
// (d) keys_for_property ⊆ key_order: 1 property 分の選択は、全 property の
//     行順→時刻順一覧(既存 `projection::key_order`、map 510 が指すもの)の
//     部分集合になる(map 509 が510の重複にならないことの直接の証拠)。
// ---------------------------------------------------------------------------

#[test]
fn keys_for_property_is_a_subset_of_key_order_across_all_property_rows() {
    let opacity = opacity();
    let position = PropertyId::new("position").expect("position は予約語ではない");
    let rows = vec![
        PropertyRowProjection {
            layer: LayerId(1),
            property: opacity.clone(),
            keys: vec![
                PropertyKeyProjection { frame: 10, selected: false },
                PropertyKeyProjection { frame: 20, selected: false },
            ],
        },
        PropertyRowProjection {
            layer: LayerId(1),
            property: position.clone(),
            keys: vec![PropertyKeyProjection { frame: 5, selected: false }],
        },
    ];

    let everything = key_order(&rows);
    let just_opacity = keys_for_property(&rows, LayerId(1), &opacity);

    assert_eq!(just_opacity.len(), 2, "opacity 分のキー数が合っていない");
    for selector in &just_opacity {
        assert!(everything.contains(selector), "keys_for_property が key_order に無いキーを返している");
    }
    // position のキーは混ざらない。
    assert!(just_opacity.iter().all(|s| s.property == opacity));
}

// ---------------------------------------------------------------------------
// (e) clip_edit_points × nav::nearest_meaning_point: 複数 clip 横断の
//     Next/Previous Clip/Edit(map 1088/1089/1108/1109)が既存の汎用「意味点」
//     ジャンプへそのまま渡せる。
// ---------------------------------------------------------------------------

#[test]
fn clip_edit_points_feed_directly_into_the_existing_meaning_point_navigator() {
    let rows = vec![row(1, 0, 90), row(2, 200, 30), row(3, 500, 10)];
    let points = clip_edit_points(&rows);
    assert_eq!(points, vec![0, 90, 200, 230, 500, 510]);

    // playhead=150 から Next: 次の編集点は200(row2 の start)。
    assert_eq!(nearest_meaning_point(&points, 150, JumpDirection::Next), Some(200));
    // 同じ位置から Prev: 直前の編集点は90(row1 の end)。
    assert_eq!(nearest_meaning_point(&points, 150, JumpDirection::Prev), Some(90));
}

#[test]
fn clip_edit_points_of_a_single_clip_only_offers_its_own_two_edges() {
    let rows = vec![row(1, 10, 20)];
    assert_eq!(clip_edit_points(&rows), vec![10, 30]);
}
