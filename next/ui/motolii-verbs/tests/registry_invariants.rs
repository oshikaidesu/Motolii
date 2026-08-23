//! レジストリ全体の性質を検査する(発注書「S6 監査がレジストリの性質として
//! 通ること」「map 行id の重複が無いこと」)。
//!
//! S6 そのものは `Verb` 定義時点で `s6_checked`(const 評価)がすでに強制して
//! いる — このテストは (a) その事実をレジストリ全体に対して再確認する
//! 回帰テスト(将来 `s6_checked` の実装が緩んだ時に検出できるように)、
//! (b) const 評価では表現できない「複数 `Verb` にまたがる」性質(id の一意性・
//! map 行id の重複)を検査する。

use motolii_verbs::registry::ALL_VERBS;
use motolii_verbs::s6_compliant;

#[test]
fn all_verbs_are_s6_compliant() {
    for verb in ALL_VERBS {
        assert!(
            s6_compliant(verb.entries),
            "S6 違反(構築時の s6_checked を通過したはずなのに再検査で落ちた): {}",
            verb.id
        );
    }
}

#[test]
fn every_verb_has_at_least_one_entry() {
    for verb in ALL_VERBS {
        assert!(
            !verb.entries.is_empty(),
            "動詞に入口が1つも無い: {}",
            verb.id
        );
    }
}

#[test]
fn verb_ids_are_unique() {
    let mut ids: Vec<&str> = ALL_VERBS.iter().map(|v| v.id).collect();
    ids.sort_unstable();
    let mut deduped = ids.clone();
    deduped.dedup();
    assert_eq!(ids, deduped, "id が重複している動詞がある");
}

#[test]
fn normal_map_row_ids_are_claimed_by_at_most_one_verb() {
    // 発注書「map 行id の重複が無いこと」。正当な相互参照(例: normal-map 467
    // 行が Undo/Redo 両方の shortcut を1行で確認している)は `map_ids` の
    // 「主出典id」には含めていない(`registry.rs` の doc コメント参照 —
    // 437/435 のみを主出典として `map_ids` に入れ、467 は転記コメントのみ)。
    // このテストが green であることは、その除外規律が守られている証拠。
    let mut seen: Vec<(u32, &str)> = Vec::new();
    for verb in ALL_VERBS {
        for &id in verb.map_ids {
            if let Some((_, other)) = seen.iter().find(|(seen_id, _)| *seen_id == id) {
                panic!(
                    "normal-map 行id {id} が複数の動詞に重複請求されている: {other} と {}",
                    verb.id
                );
            }
            seen.push((id, verb.id));
        }
    }
}

#[test]
fn menu_and_context_slices_only_reference_registered_verbs() {
    use motolii_verbs::registry::{
        CANVAS_CONTEXT, CLIP_CONTEXT, EDIT_MENU, HELP_MENU, KEYFRAME_CONTEXT, LAYER_MENU,
        LAYER_ROW_CONTEXT, WINDOW_MENU,
    };
    let all_ids: Vec<&str> = ALL_VERBS.iter().map(|v| v.id).collect();
    let slices: Vec<(&str, &[&motolii_verbs::Verb])> = vec![
        ("EDIT_MENU", EDIT_MENU),
        ("LAYER_MENU", LAYER_MENU),
        ("WINDOW_MENU", WINDOW_MENU),
        ("HELP_MENU", HELP_MENU),
        ("CLIP_CONTEXT", CLIP_CONTEXT),
        ("LAYER_ROW_CONTEXT", LAYER_ROW_CONTEXT),
        ("CANVAS_CONTEXT", CANVAS_CONTEXT),
        ("KEYFRAME_CONTEXT", KEYFRAME_CONTEXT),
    ];
    for (name, slice) in slices {
        for verb in slice {
            assert!(
                all_ids.contains(&verb.id),
                "{name} が ALL_VERBS に無い動詞を参照している: {}",
                verb.id
            );
        }
    }
}

/// レジストリの規模を固定する回帰テスト(発注書の対象範囲: Edit8・Layer9・
/// Window6・Help3・4文脈の右クリック項目)。数が変わったら意図的な変更か
/// 確認すること。
#[test]
fn all_verbs_has_the_expected_count() {
    // Edit8 + Layer9 + Window6 + Help3 = 26(メニュー4本、重複無し)
    // + LayerRowContext 固有5(Rename + restack4。Hide/Solo/Lock は Layer
    //   menu の再入口なので Layer9 側で既カウント、二重に数えない)
    // + KeyframeContext 固有6(Interpolation5 + Delete。CLIP_CONTEXT/
    //   CANVAS_CONTEXT は全項目が既存動詞の再入口なので新規動詞ゼロ)
    // = 26 + 5 + 6 = 37。
    // + CLIP_CONTEXT 固有1(Split — E-1 で新設。GOALS M6 の最後の1件で、
    //   既存メニューの再入口ではない初の clip 右クリック固有動詞)= 38。
    assert_eq!(ALL_VERBS.len(), 38);
}
