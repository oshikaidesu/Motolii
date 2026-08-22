//! B38 第3切片(裁定184 型別 section 第2号): Inspector EFFECTS section —
//! effect を持つ layer での**のみ** section が現れ、remove / reorder(上下)/
//! bypass / Glow param 値編集がすべて既存文法(`Intent::SetEffects` /
//! `Intent::SetTrack`)経由で書けること。
//!
//! 落ちるテスト先行(発注の型)。view の存在照合は `mask_section.rs` と同じ
//! iced_test 手口(`Target::Text` 列挙)。**未実行**(supervisor が波末一括 —
//! 裁定189 の運転規律)。

use iced_test::selector::{Candidate, Target};

use motolii_core::{Fps, RationalTime};
use motolii_inspector_pane::{
    commit_inspector_field, effects_with_moved_down, effects_with_moved_up, effects_with_removed,
    effects_with_toggled_enabled, move_inspector_effect_down, move_inspector_effect_up, project,
    property_id, remove_inspector_effect, toggle_inspector_effect_bypass, view, FieldDraft,
    GlowParam, KeyCellState, KeyRow, Message, RowValue, TransformField, GLOW_PLUGIN_ID,
};
use motolii_shell_state::Session;
use motolii_store::{
    Composition, Document, EffectId, EffectInstance, Intent, LayerId, LayerMeta, LayerSource,
    LayerTiming, PropertyId, Value,
};
use motolii_tokens_rs::{Colors, Dimensions};

fn fps30() -> Fps {
    Fps::try_new(30, 1).expect("30fps は正値")
}

/// comp と layer を1つ置いた Document(`mask_section.rs` と同じ形)。
fn doc_with_layer() -> (Document, LayerId) {
    let mut doc = Document::new();
    let layer = LayerId(1);
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: fps30(),
        duration_frames: 300,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .expect("comp を置けるはず");
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::Solid {
                    rgba: [255, 0, 0, 255],
                    width: 64,
                    height: 64,
                },
                order: 0,
                timing: LayerTiming::place(0, None, 300),
            },
        },
    ])
    .expect("layer を置けるはず");
    (doc, layer)
}

fn session_selecting(layer: LayerId) -> Session {
    Session {
        selection: Some(layer),
        ..Session::default()
    }
}

/// Glow(既知 plugin・enabled)+ 未知 plugin(bypass 中)の2本 stack。
/// bypass 中(enabled=false)の effect も `StoreView::effects` の生の列には
/// 現れる(弾くのは resolve 側)— Inspector は生の stack を編集する側なので、
/// bypass 中の行も出ることをこの fixture で固定する。
fn two_effects() -> Vec<EffectInstance> {
    vec![
        EffectInstance {
            id: EffectId(1),
            plugin_id: GLOW_PLUGIN_ID.to_owned(),
            enabled: true,
        },
        EffectInstance {
            id: EffectId(2),
            plugin_id: "third-party.sparkle".to_owned(),
            enabled: false,
        },
    ]
}

fn doc_with_two_effects() -> (Document, LayerId) {
    let (mut doc, layer) = doc_with_layer();
    doc.apply(Intent::SetEffects {
        layer,
        effects: two_effects(),
    })
    .expect("effect を書けるはず");
    (doc, layer)
}

/// `mask_section.rs` と同じ「`find` を尽きるまで繰り返す」手口。
fn collect_targets(element: iced::Element<'_, Message>) -> Vec<Target> {
    let mut ui = iced_test::simulator(element);
    let mut found: Vec<Target> = Vec::new();
    loop {
        let already = found.clone();
        let selector = move |candidate: Candidate<'_>| -> Option<Target> {
            let target = Target::from(candidate);
            if already.contains(&target) {
                None
            } else {
                Some(target)
            }
        };
        match ui.find(selector) {
            Ok(target) => found.push(target),
            Err(_) => break,
        }
        assert!(found.len() <= 5_000, "candidate 列挙が終わらない");
    }
    found
}

fn has_text(targets: &[Target], content: &str) -> bool {
    targets
        .iter()
        .any(|t| matches!(t, Target::Text { content: c, .. } if c == content))
}

// ---------------------------------------------------------------------------
// 投影: effect を持つ layer でのみ section の中身が生える
// ---------------------------------------------------------------------------

/// effect の無い layer の投影は `effects` が空 — view は EFFECTS section を
/// 出さない(Q0: 効かない chrome を並べない、MASK section と同じ判断)。
#[test]
fn a_layer_without_effects_projects_no_effect_rows_and_shows_no_effects_section() {
    let (doc, layer) = doc_with_layer();
    let session = session_selecting(layer);
    let projection = project(&doc.view(), &session)
        .expect("投影は組めるはず")
        .expect("選択ありなので Some のはず");
    assert!(
        projection.effects.is_empty(),
        "effect の無い layer に effect 行が生えている"
    );

    let targets = collect_targets(view(
        Some(&projection),
        None,
        None,
        Dimensions::default(),
        Colors::default(),
    ));
    assert!(
        !has_text(&targets, "EFFECTS"),
        "effect の無い layer に EFFECTS section header が出ている"
    );
}

/// effect 持ち layer は store の並び(= 適用順)どおりに行が生え、表示名は
/// 既知 plugin だけ人間可読名(`Glow`)・未知は plugin_id そのまま(M13)。
/// bypass 中(enabled=false)の effect も行として現れる(消えてはいない)。
#[test]
fn an_effect_bearing_layer_projects_one_row_per_effect_in_stack_order() {
    let (doc, layer) = doc_with_two_effects();
    let session = session_selecting(layer);
    let projection = project(&doc.view(), &session)
        .expect("投影は組めるはず")
        .expect("選択ありなので Some のはず");

    assert_eq!(projection.effects.len(), 2);
    assert_eq!(projection.effects[0].id, EffectId(1));
    assert_eq!(projection.effects[0].name, "Glow", "既知 plugin は人間可読名のはず");
    assert!(projection.effects[0].enabled);
    assert_eq!(projection.effects[1].id, EffectId(2));
    assert_eq!(
        projection.effects[1].name, "third-party.sparkle",
        "未知 plugin は plugin_id をそのまま出すはず(捏造しない)"
    );
    assert!(!projection.effects[1].enabled, "bypass 中も行として現れるはず");

    let targets = collect_targets(view(
        Some(&projection),
        None,
        None,
        Dimensions::default(),
        Colors::default(),
    ));
    assert!(has_text(&targets, "EFFECTS"), "EFFECTS section header が出ない");
    assert!(has_text(&targets, "Glow"), "Glow の行が出ない");
    assert!(
        has_text(&targets, "third-party.sparkle"),
        "未知 plugin の行が出ない"
    );
    assert!(has_text(&targets, "Bypass"), "Bypass トグルが出ない");
    assert!(has_text(&targets, "Remove"), "Remove ボタンが出ない");
    assert!(has_text(&targets, "↑"), "上へ移動ボタンが出ない");
    assert!(has_text(&targets, "↓"), "下へ移動ボタンが出ない");
}

/// Glow の param 行は engine カタログの3本(threshold/intensity/radius)が
/// カタログ順に生え、track 未打ちは engine 既定の写し(1.0/0.75/1.0、換算
/// なしの store 単位)。未知 plugin は param 行ゼロ(catalog を知らないので
/// 捏造しない)。
#[test]
fn glow_param_rows_follow_the_catalog_and_unknown_plugins_get_none() {
    let (doc, layer) = doc_with_two_effects();
    let session = session_selecting(layer);
    let projection = project(&doc.view(), &session)
        .expect("投影は組めるはず")
        .expect("選択ありなので Some のはず");

    let glow = &projection.effects[0];
    assert_eq!(glow.params.len(), 3, "Glow の param カタログは3本のはず");
    let labels: Vec<&str> = glow.params.iter().map(|row| row.label).collect();
    assert_eq!(labels, vec!["Threshold", "Intensity", "Radius"]);

    for (row, (param, default)) in glow.params.iter().zip([
        (GlowParam::Threshold, 1.0),
        (GlowParam::Intensity, 0.75),
        (GlowParam::Radius, 1.0),
    ]) {
        assert_eq!(row.decimals, 2, "param 行の精度は2桁のはず");
        assert_eq!(row.key.row, KeyRow::EffectParam(EffectId(1), param));
        assert_eq!(
            row.key.state,
            KeyCellState::Static,
            "track 未打ちは Static のはず"
        );
        match &row.value {
            RowValue::Scalar(slot) => {
                assert!(slot.present);
                assert!(slot.editable);
                assert!(!slot.keyed);
                assert_eq!(
                    slot.value, default,
                    "track 未打ちの表示は engine 既定の写しのはず"
                );
                assert_eq!(
                    slot.field,
                    Some(TransformField::EffectParam(EffectId(1), param))
                );
            }
            other => panic!("param 行は Scalar のはず: {other:?}"),
        }
    }

    assert!(
        projection.effects[1].params.is_empty(),
        "未知 plugin に param 行を捏造している"
    );
}

// ---------------------------------------------------------------------------
// 編集: remove / reorder / bypass は `Intent::SetEffects` 1回 = 1 undo
// ---------------------------------------------------------------------------

/// remove は対象だけを列から外し、1 undo で戻る。
#[test]
fn removing_an_effect_drops_only_that_effect_and_undoes_in_one_step() {
    let (mut doc, layer) = doc_with_two_effects();

    remove_inspector_effect(&mut doc, Some(layer), EffectId(1)).expect("remove は成功するはず");
    let effects = doc.view().effects(layer).expect("effect を読めるはず");
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].id, EffectId(2), "対象外の effect が消えている");

    doc.undo();
    let effects = doc.view().effects(layer).expect("effect を読めるはず");
    assert_eq!(effects.len(), 2, "1 undo で戻らない(1操作=1 undo 違反)");
    assert_eq!(effects[0].id, EffectId(1));
}

/// remove しても param track は残る(inert — 1 click = 1 undo を保つための
/// 決定、`Message::RemoveEffect` doc)。undo で effect が戻れば param もその
/// まま効く。
#[test]
fn removing_an_effect_keeps_its_param_tracks_inert_in_the_document() {
    let (mut doc, layer) = doc_with_two_effects();
    let property = PropertyId::effect_param(EffectId(1), "radius").expect("param 名は非予約語");
    let field = TransformField::EffectParam(EffectId(1), GlowParam::Radius);
    let mut draft = Some(FieldDraft {
        field,
        text: "2".to_owned(),
    });
    commit_inspector_field(&mut doc, &mut draft, Some(layer), 0, fps30(), field)
        .expect("param の確定は成功するはず");

    remove_inspector_effect(&mut doc, Some(layer), EffectId(1)).expect("remove は成功するはず");
    let value = doc
        .view()
        .value_at(layer, &property, RationalTime::ZERO)
        .expect("track を読めるはず");
    assert_eq!(
        value,
        Some(Value::F64(2.0)),
        "param track まで消えている(残す決定 — undo 復元とセット)"
    );
}

/// reorder(上下)は隣と入れ替えるだけ・1 undo で戻る。端では Intent を
/// 出さない(空 undo 段を作らない)。
#[test]
fn reordering_swaps_neighbours_and_edge_moves_emit_no_intent() {
    let (mut doc, layer) = doc_with_two_effects();

    move_inspector_effect_down(&mut doc, Some(layer), EffectId(1))
        .expect("下へ移動は成功するはず");
    let effects = doc.view().effects(layer).expect("effect を読めるはず");
    assert_eq!(
        (effects[0].id, effects[1].id),
        (EffectId(2), EffectId(1)),
        "隣と入れ替わっていない"
    );

    doc.undo();
    let effects = doc.view().effects(layer).expect("effect を読めるはず");
    assert_eq!(
        (effects[0].id, effects[1].id),
        (EffectId(1), EffectId(2)),
        "1 undo で戻らない(1操作=1 undo 違反)"
    );

    // 端の no-op: 先頭を上へ・末尾を下へ — Intent を出していなければ、次の
    // undo は直前の実編集(初期 SetEffects)まで一気に戻る。
    move_inspector_effect_up(&mut doc, Some(layer), EffectId(1)).expect("端は no-op のはず");
    move_inspector_effect_down(&mut doc, Some(layer), EffectId(2)).expect("端は no-op のはず");
    let effects = doc.view().effects(layer).expect("effect を読めるはず");
    assert_eq!((effects[0].id, effects[1].id), (EffectId(1), EffectId(2)));

    doc.undo();
    assert!(
        doc.view()
            .effects(layer)
            .expect("effect を読めるはず")
            .is_empty(),
        "端 reorder が空の undo 段を積んでいる(undo が初期 SetEffects まで戻らない)"
    );
}

/// bypass は対象の enabled だけを裏返し、1 undo で戻る。列からは消えない。
#[test]
fn toggling_bypass_flips_only_that_effect_and_undoes_in_one_step() {
    let (mut doc, layer) = doc_with_two_effects();

    toggle_inspector_effect_bypass(&mut doc, Some(layer), EffectId(1))
        .expect("bypass トグルは成功するはず");
    let effects = doc.view().effects(layer).expect("effect を読めるはず");
    assert_eq!(effects.len(), 2, "bypass が effect を消している(削除とは別物)");
    assert!(!effects[0].enabled, "対象の enabled が裏返っていない");
    assert!(!effects[1].enabled, "対象外の effect が動いている");
    assert_eq!(
        effects[0].plugin_id, GLOW_PLUGIN_ID,
        "bypass が plugin_id を巻き込んでいる"
    );

    doc.undo();
    let effects = doc.view().effects(layer).expect("effect を読めるはず");
    assert!(effects[0].enabled, "1 undo で戻らない(1操作=1 undo 違反)");
}

/// 選択なし・存在しない effect id は黙って no-op(mask 系と同じ安全側)。
#[test]
fn effect_edits_without_a_selection_or_with_a_stale_id_are_silent_no_ops() {
    let (mut doc, layer) = doc_with_two_effects();
    let before = doc.view().effects(layer).expect("effect を読めるはず");

    remove_inspector_effect(&mut doc, None, EffectId(1)).expect("選択なしは no-op のはず");
    toggle_inspector_effect_bypass(&mut doc, Some(layer), EffectId(99))
        .expect("居ない effect id は no-op のはず");
    move_inspector_effect_up(&mut doc, Some(layer), EffectId(99))
        .expect("居ない effect id は no-op のはず");

    assert_eq!(
        doc.view().effects(layer).expect("effect を読めるはず"),
        before,
        "no-op のはずが Document が動いている"
    );
}

// ---------------------------------------------------------------------------
// 編集: param は既存の値セル文法(`TransformField` → `Intent::SetTrack`)
// ---------------------------------------------------------------------------

/// param の値セル Enter は `effect.{id}.param.{name}` track への
/// `Intent::SetTrack`(既存 `commit_inspector_field` がそのまま書く —
/// **換算なし**の store 単位、opacity 系の % 換算とは違う)。
#[test]
fn committing_a_glow_param_draft_writes_the_effect_param_track_raw() {
    let (mut doc, layer) = doc_with_two_effects();

    let field = TransformField::EffectParam(EffectId(1), GlowParam::Threshold);
    let mut draft = Some(FieldDraft {
        field,
        text: "0.6".to_owned(),
    });
    commit_inspector_field(&mut doc, &mut draft, Some(layer), 0, fps30(), field)
        .expect("param の確定は成功するはず");

    let property = PropertyId::effect_param(EffectId(1), "threshold").expect("param 名は非予約語");
    let value = doc
        .view()
        .value_at(layer, &property, RationalTime::ZERO)
        .expect("track を読めるはず")
        .expect("確定後は値が有るはず");
    assert_eq!(value, Value::F64(0.6), "表示 0.6 は store でもそのまま 0.6 のはず");
    assert_eq!(
        property_id(field).expect("effect param の property は作れるはず"),
        property,
        "field → property の対応が effect.{{id}}.param.{{name}} になっていない"
    );
}

// ---------------------------------------------------------------------------
// 純関数(oracle (b) — Document を通さない最小の意味固定)
// ---------------------------------------------------------------------------

/// 純関数の境界: stale id は `None`、端の move も `None`(Intent を出させない)。
#[test]
fn pure_list_edits_return_none_for_stale_ids_and_edge_moves() {
    let effects = two_effects();
    assert!(effects_with_removed(&effects, EffectId(9)).is_none());
    assert!(effects_with_toggled_enabled(&effects, EffectId(9)).is_none());
    assert!(effects_with_moved_up(&effects, EffectId(1)).is_none(), "先頭の上移動");
    assert!(effects_with_moved_down(&effects, EffectId(2)).is_none(), "末尾の下移動");

    let down = effects_with_moved_down(&effects, EffectId(1)).expect("中の移動は Some");
    assert_eq!((down[0].id, down[1].id), (EffectId(2), EffectId(1)));
    let up = effects_with_moved_up(&effects, EffectId(2)).expect("中の移動は Some");
    assert_eq!((up[0].id, up[1].id), (EffectId(2), EffectId(1)));
}
