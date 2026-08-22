//! LINK section(2026-08-22 発注「レイヤーを指す」文法 第3号): 型付き
//! `PropertySource::Link`(裁定206)は本日 store へ着地したが呼び手がゼロ
//! だった ── ここがその初めての呼び手。
//!
//! 落ちるテスト先行(発注の型)。**候補の絞り込み**(自分自身を選べない・
//! 循環しない)が本発注の核 ── store 側に書き込み時の循環拒否
//! (`validate_no_link_cycle`、private fn)は在るが、UI 側([`project`])は
//! それと**同じ判定を先回りして**候補から外すことで「拒否される選択肢を
//! 最初から出さない」を実現する。ここではその絞り込みが実際に効くこと・
//! store 側の最終防衛線も生きていることの両方を検証する。

use motolii_core::{Fps, RationalTime};
use motolii_inspector_pane::{
    clear_inspector_link, commit_inspector_link, project, LayerCandidate, LinkSourceCandidate,
    LinkTarget,
};
use motolii_shell_state::Session;
use motolii_store::{
    property, Composition, Document, Intent, LayerId, LayerMeta, LayerSource, LayerTiming,
    PropertyId, PropertySource, Value,
};

fn fps30() -> Fps {
    Fps::try_new(30, 1).expect("30fps は正値")
}

fn seed_composition(doc: &mut Document) {
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: fps30(),
        duration_frames: 300,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .expect("comp を置けるはず");
}

fn seed_layer(doc: &mut Document, layer: LayerId) {
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
}

fn session_selecting(layer: LayerId) -> Session {
    Session {
        selection: Some(layer),
        ..Session::default()
    }
}

fn source(layer: LayerId, property: LinkTarget) -> LinkSourceCandidate {
    LinkSourceCandidate {
        layer: LayerCandidate {
            id: layer,
            label: format!("layer {}", layer.0),
        },
        property,
    }
}

// ---------------------------------------------------------------------------
// 投影: 候補の絞り込み(発注書の核)
// ---------------------------------------------------------------------------

/// **自分自身を選べない**(発注書「参照先も同様」— 自分自身を選べてはいけない)。
#[test]
fn link_candidates_exclude_the_selected_layer_itself() {
    let mut doc = Document::new();
    seed_composition(&mut doc);
    let a = LayerId(1);
    let b = LayerId(2);
    seed_layer(&mut doc, a);
    seed_layer(&mut doc, b);

    let session = session_selecting(a);
    let projection = project(&doc.view(), &session)
        .expect("投影は組めるはず")
        .expect("選択ありなので Some のはず");

    let position_row = projection
        .links
        .iter()
        .find(|row| row.target == LinkTarget::Position)
        .expect("Position 行は常に在るはず");
    assert!(
        position_row
            .candidates
            .iter()
            .any(|c| c.layer.id == b),
        "他 layer は候補に居るはず"
    );
    assert!(
        !position_row.candidates.iter().any(|c| c.layer.id == a),
        "自分自身が候補に混ざっている"
    );
}

/// **循環を選べない**(発注書「循環してもいけない」)。B.Rotation が既に
/// A.Position を link 元にしている時、A.Position の候補から (B, Rotation) を
/// 除く(選ぶと A.Position→B.Rotation→A.Position の循環になる)。
#[test]
fn link_candidates_exclude_a_source_that_would_close_a_cycle() {
    let mut doc = Document::new();
    seed_composition(&mut doc);
    let a = LayerId(1);
    let b = LayerId(2);
    seed_layer(&mut doc, a);
    seed_layer(&mut doc, b);

    // B.Rotation ← A.Position(片方向の link)。
    commit_inspector_link(&mut doc, Some(b), LinkTarget::Rotation, source(a, LinkTarget::Position))
        .expect("link を書けるはず");

    let session = session_selecting(a);
    let projection = project(&doc.view(), &session)
        .expect("投影は組めるはず")
        .expect("選択ありなので Some のはず");
    let position_row = projection
        .links
        .iter()
        .find(|row| row.target == LinkTarget::Position)
        .expect("Position 行は常に在るはず");
    assert!(
        !position_row
            .candidates
            .iter()
            .any(|c| c.layer.id == b && c.property == LinkTarget::Rotation),
        "循環になる候補が絞り込まれていない"
    );
    // 無関係な組(B.Scale 等)は絞り込まれずに残る。
    assert!(
        position_row
            .candidates
            .iter()
            .any(|c| c.layer.id == b && c.property == LinkTarget::Scale),
        "無関係な候補まで絞り込まれている"
    );
}

// ---------------------------------------------------------------------------
// 書き口: link の設定・解除
// ---------------------------------------------------------------------------

/// link を選ぶと `Intent::SetPropertyLink` が1回出て、1 undo で戻る。
#[test]
fn commit_inspector_link_writes_a_property_link_and_undoes_in_one_step() {
    let mut doc = Document::new();
    seed_composition(&mut doc);
    let a = LayerId(1);
    let b = LayerId(2);
    seed_layer(&mut doc, a);
    seed_layer(&mut doc, b);

    commit_inspector_link(&mut doc, Some(a), LinkTarget::Opacity, source(b, LinkTarget::Position))
        .expect("link を書けるはず");

    let property = PropertyId::new(property::OPACITY).unwrap();
    match doc
        .view()
        .property_source(a, &property)
        .expect("読めるはず")
        .expect("link のはず")
    {
        PropertySource::Link(link) => {
            assert_eq!(link.source_layer, b);
            assert_eq!(link.source_property.name(), property::POSITION);
            assert_eq!(link.plugin_id, "motolii.link.identity");
        }
        other => panic!("Link のはずが {other:?}"),
    }

    doc.undo();
    assert!(
        matches!(
            doc.view().property_source(a, &property).expect("読めるはず"),
            None | Some(PropertySource::Track(_))
        ),
        "1 undo で戻らない(1操作=1 undo 違反)"
    );
}

/// Clear は `Intent::SetTrack` を再び投げて、t=0 の評価値を保った static
/// track へ戻す(`crate::slot` doc「専用の解除 variant は要らない」の実装)。
#[test]
fn clearing_a_link_reverts_to_a_static_track_holding_the_current_value() {
    let mut doc = Document::new();
    seed_composition(&mut doc);
    let a = LayerId(1);
    let b = LayerId(2);
    seed_layer(&mut doc, a);
    seed_layer(&mut doc, b);

    // B.Scale は既定 [1.0, 1.0](track 未着手)。
    commit_inspector_link(&mut doc, Some(a), LinkTarget::Scale, source(b, LinkTarget::Scale))
        .expect("link を書けるはず");

    clear_inspector_link(&mut doc, Some(a), LinkTarget::Scale).expect("clear は成功するはず");

    let property = PropertyId::new(property::SCALE).unwrap();
    match doc
        .view()
        .property_source(a, &property)
        .expect("読めるはず")
        .expect("clear 後も値は在るはず")
    {
        PropertySource::Track(track) => {
            assert_eq!(
                track.eval(RationalTime::ZERO),
                Value::Vec2([1.0, 1.0]),
                "unlink 後の値が evaluated link の見た目と食い違う"
            );
        }
        other => panic!("clear 後は Track のはずが {other:?}"),
    }
}

/// link でない property を clear しても no-op(決定7 と同じ判断)。
#[test]
fn clearing_a_non_link_property_is_a_no_op() {
    let mut doc = Document::new();
    seed_composition(&mut doc);
    let a = LayerId(1);
    seed_layer(&mut doc, a);

    clear_inspector_link(&mut doc, Some(a), LinkTarget::Rotation).expect("no-op のはず");
    let property = PropertyId::new(property::ROTATION).unwrap();
    assert!(
        doc.view()
            .property_source(a, &property)
            .expect("読めるはず")
            .is_none(),
        "no-op のはずが property が新設されている"
    );
}

/// 選択なしは黙って no-op(`commit_inspector_field` と同じ安全側)。
#[test]
fn link_edits_without_a_selection_are_silent_no_ops() {
    let mut doc = Document::new();
    seed_composition(&mut doc);
    let a = LayerId(1);
    let b = LayerId(2);
    seed_layer(&mut doc, a);
    seed_layer(&mut doc, b);

    commit_inspector_link(&mut doc, None, LinkTarget::Position, source(b, LinkTarget::Position))
        .expect("選択なしは no-op のはず");
    clear_inspector_link(&mut doc, None, LinkTarget::Position).expect("選択なしは no-op のはず");

    let property = PropertyId::new(property::POSITION).unwrap();
    assert!(doc
        .view()
        .property_source(a, &property)
        .expect("読めるはず")
        .is_none());
}
