//! 運転席 — Timeline 第2波 T3(パラメータ行+キー菱形、裁定148/151)の落ちるテスト
//! 先行(EXACT TARGET のオラクル a/b/c)。
//!
//! - (a) キーを持つ layer 選択で property 行が rows に現れる・持たない property は
//!   現れない・layer 未選択なら空
//! - (b) 菱形クリックで単独選択、Cmd でトグル、Shift で範囲(`key_order` =
//!   行順→時刻順)
//! - (c) キー選択中の Delete がキーを消し、Undo 1回で戻る

use motolii_shell::timeline_pane::{frame_at_x, BarPart, KeySelectionOp, KeySelector, TimelinePane};
use motolii_shell::tokens::Tokens;
use motolii_shell::{Message, Session, Shell};

use motolii_store::{
    property, Composition, Document, Fps, Intent, Interp, Keyframe, KeyframeTrack, LayerId,
    LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value,
};

fn shell() -> Shell {
    Shell::new().0
}

// ---------------------------------------------------------------------------
// (a) 投影: 既定=キーを持つ property のみ、選択 layer の下にだけ挿入される
// ---------------------------------------------------------------------------

/// fixture の既定選択(「サビ歌詞」)は position に2キー持つ — `fixture.rs` の
/// 「数層に position/opacity キー」受入条件そのもの。
#[test]
fn a_keyed_layer_selection_surfaces_its_property_rows() {
    let shell = Shell::new_fixture().0;
    let rows = shell.timeline_property_rows();
    assert!(
        !rows.is_empty(),
        "サビ歌詞(fixture の既定選択)は position キーを持つのに property 行が空"
    );
    let position_row = rows
        .iter()
        .find(|row| row.property.name() == property::POSITION)
        .unwrap_or_else(|| panic!("position の property 行が無い: {rows:?}"));
    assert_eq!(
        position_row.keys.iter().map(|k| k.frame).collect::<Vec<_>>(),
        vec![510, 570],
        "fixture が打った2キー(t(510)/t(570))と一致しない"
    );
}

/// キーを持たない property(fixture のサビ歌詞は opacity にキーを打っていない)は
/// 行に現れない。
#[test]
fn properties_without_any_key_do_not_get_a_row() {
    let shell = Shell::new_fixture().0;
    let rows = shell.timeline_property_rows();
    assert!(
        rows.iter().all(|row| row.property.name() != property::OPACITY),
        "キーを持たない opacity の行が現れている: {rows:?}"
    );
}

/// layer 未選択なら property 行は無い(選択 layer の下にしか挿入しない、
/// EXACT TARGET 1)。
#[test]
fn no_selection_means_no_property_rows() {
    let shell = shell(); // Shell::new() — 選択が無い空 Document
    assert!(shell.session().selection.is_none());
    assert!(shell.timeline_property_rows().is_empty());
}

/// キーを持たない layer を選ぶと property 行は空になる(「Bロール_街並み」は
/// fixture でキーを打っていない層)。
#[test]
fn selecting_an_unkeyed_layer_yields_no_property_rows() {
    let mut shell = Shell::new_fixture().0;
    let unkeyed = LayerId(3); // 「Bロール_街並み」(fixture.rs の並び3番目)
    let _ = shell.update(Message::Select(unkeyed));
    assert_eq!(shell.session().selection, Some(unkeyed));
    assert!(
        shell.timeline_property_rows().is_empty(),
        "キーを持たない層なのに property 行が出ている: {:?}",
        shell.timeline_property_rows()
    );
}

// ---------------------------------------------------------------------------
// (b) キー選択: 単独 / Cmd トグル / Shift 範囲
// ---------------------------------------------------------------------------

/// comp 全体(0..300, fps=30)を占有する1層、opacity に3キー(0/100/250)を
/// 持たせて選択済みにする。
fn one_layer_with_opacity_keys() -> (Document, Session, LayerId, PropertyId) {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 640,
        height: 360,
        fps: Fps::try_new(30, 1).expect("30fps"),
        duration_frames: 300,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .expect("comp を置ける");
    doc.mark_undo_floor();

    let id = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(id),
        Intent::SetMeta {
            layer: id,
            meta: LayerMeta {
                source: LayerSource::Solid {
                    rgba: [80, 160, 220, 255],
                    width: 240,
                    height: 135,
                },
                order: 0,
                timing: LayerTiming::place(0, None, 300),
            },
        },
    ])
    .expect("layer を置ける");

    let property = PropertyId::new(property::OPACITY).expect("opacity は予約語ではない");
    let mut track = KeyframeTrack::new();
    for frame in [0_i64, 100, 250] {
        track.insert(Keyframe {
            t: RationalTime::try_new(frame, 30).expect("frame は収まる"),
            value: Value::F64(1.0),
            interp: Interp::Linear,
            spatial: None,
        });
    }
    doc.apply(Intent::SetTrack { layer: id, property: property.clone(), track })
        .expect("track を書ける");

    let mut session = Session::default();
    session.selection = Some(id);
    (doc, session, id, property)
}

/// `Simulator::new` の既定サイズ(upstream doc comment: 1024x768、
/// `iced_test_spike.rs::DEFAULT_WIDTH` と同じ前提)。
const DEFAULT_WIDTH: f32 = 1024.0;

/// キー1本分の画面座標。`frame_to_x` 自体は `pub(crate)` で外から呼べないので、
/// 同じ式(`ratio = frame/duration; x = ratio*width` — `projection.rs` の doc
/// どおり丸めなし)をここでも素直に組む。当たり判定の許容(12px)に対して
/// 十分小さい誤差にしかならない。
fn key_screen_point(dims: &motolii_shell::tokens::Dimensions, frame: i64) -> iced::Point {
    let rail_width = dims.timeline_lane_bar_width;
    let clip_width = DEFAULT_WIDTH - rail_width;
    let ratio = frame as f32 / 300.0;
    let x = rail_width + (ratio * clip_width).clamp(0.0, clip_width);
    // 選択 layer は行0の唯一の層 → property 行(opacity 1本)は layer 行の
    // すぐ下、`ruler_height(=row_height) + row_height*1` から始まる
    // (`projection::layer_row_top` と同じ計算)。
    let band_top = dims.row_height * 2.0;
    let y = band_top + dims.timeline_param_row_height / 2.0;
    iced::Point::new(x, y)
}

fn click_at(pane: TimelinePane, point: iced::Point) -> Vec<Message> {
    let mut ui = iced_test::simulator(pane.view());
    ui.point_at(point);
    let _ = ui.simulate([
        iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
        iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)),
    ]);
    ui.into_messages().collect()
}

/// 第2波T4(正典 §3): 修飾キー無しの press は「単独選択への差し替え」と
/// 「時刻ドラッグの開始」を1つの `Message::TimelineKeyGrabbed` が兼ねる
/// (T3 時点の `TimelineKeySelect(Single)` から進化 — 選択の実際の確定は
/// `Shell::start_timeline_key_drag` 側、`timeline_key_grab_drive.rs` 参照)。
#[test]
fn clicking_a_key_diamond_with_no_modifier_publishes_a_grab() {
    let (doc, session, layer, property) = one_layer_with_opacity_keys();
    let tokens = Tokens::default();
    let store = doc.view();
    let pane = TimelinePane::new(&store, &session, tokens.dims, tokens.colors, iced::keyboard::Modifiers::default());

    let point = key_screen_point(&tokens.dims, 100);
    let messages = click_at(pane, point);

    let expected = KeySelector { layer, property, frame: 100 };
    assert!(
        matches!(
            messages.as_slice(),
            [Message::TimelineKeyGrabbed { key, at_frame: 100, retime: false }] if *key == expected
        ),
        "菱形クリックが grab(選択差し替え+drag開始)を出していない: {messages:?}"
    );
}

#[test]
fn cmd_clicking_a_key_diamond_publishes_a_toggle() {
    let (doc, session, layer, property) = one_layer_with_opacity_keys();
    let tokens = Tokens::default();
    let store = doc.view();
    let pane = TimelinePane::new(&store, &session, tokens.dims, tokens.colors, iced::keyboard::Modifiers::COMMAND);

    let point = key_screen_point(&tokens.dims, 0);
    let messages = click_at(pane, point);

    let expected = KeySelector { layer, property, frame: 0 };
    assert!(
        matches!(
            messages.as_slice(),
            [Message::TimelineKeySelect(KeySelectionOp::Toggle(k))] if *k == expected
        ),
        "Cmd+クリックがトグルを出していない: {messages:?}"
    );
}

#[test]
fn shift_clicking_a_key_diamond_publishes_a_range_op() {
    let (doc, session, layer, property) = one_layer_with_opacity_keys();
    let tokens = Tokens::default();
    let store = doc.view();
    let pane = TimelinePane::new(&store, &session, tokens.dims, tokens.colors, iced::keyboard::Modifiers::SHIFT);

    let point = key_screen_point(&tokens.dims, 250);
    let messages = click_at(pane, point);

    let expected = KeySelector { layer, property, frame: 250 };
    assert!(
        matches!(
            messages.as_slice(),
            [Message::TimelineKeySelect(KeySelectionOp::Range(k))] if *k == expected
        ),
        "Shift+クリックが範囲操作を出していない: {messages:?}"
    );
}

/// 空白(菱形の外、property 行の帯の内側)クリックは吸収されるだけ —
/// `input.rs`/`hit.rs` の一様行モデルへ誤って落ちない(`projection::
/// layer_row_top` の write-set 外 finding を読める形で確認する)。
#[test]
fn clicking_the_blank_area_of_a_property_row_is_absorbed_and_publishes_nothing() {
    let (doc, session, _layer, _property) = one_layer_with_opacity_keys();
    let tokens = Tokens::default();
    let store = doc.view();
    let pane = TimelinePane::new(&store, &session, tokens.dims, tokens.colors, iced::keyboard::Modifiers::default());

    // frame 300(comp 末尾ちょうど)には opacity のキーが無い — 空白のはず。
    let point = key_screen_point(&tokens.dims, 300);
    let messages = click_at(pane, point);
    assert!(messages.is_empty(), "property 行の空白クリックが何か publish している: {messages:?}");
}

// ---------------------------------------------------------------------------
// Shift 範囲の確定(`Shell::apply_key_selection`) — `key_order` = 行順→時刻順
// ---------------------------------------------------------------------------

/// `Shell::apply_key_selection` の範囲確定(`key_order` = 行順→時刻順)を
/// **fixture 経由**で確認する: fixture の「サビ歌詞」は position に2キー
/// (510/570)を持つ — 単独クリックで anchor を立て、Shift クリック相当の
/// `Range` を送って、その閉区間(この場合は2キーとも)が選ばれることを見る。
/// `Shell` は Document を外から直接触らせない(背骨1)ので、任意本数のキーを
/// 持つ独自 Document を Shell 経由で駆動する近道は無い — 2キーの fixture でも
/// 「anchor から他方まで」という範囲確定の核(単独区間ではなく複数キーへ
/// 広がること)は十分に検分できる。
#[test]
fn range_selection_on_the_fixture_selects_the_closed_interval_in_key_order() {
    let mut shell = Shell::new_fixture().0;
    let layer = shell.session().selection.expect("fixture は既定選択を持つ");
    let property = PropertyId::new(property::POSITION).expect("position は予約語ではない");

    let first = KeySelector { layer, property: property.clone(), frame: 510 };
    let second = KeySelector { layer, property: property.clone(), frame: 570 };

    // 単独クリック相当 — anchor が first になる。
    let _ = shell.update(Message::TimelineKeySelect(KeySelectionOp::Single(first.clone())));
    // Shift クリック相当 — anchor(first)から second までの閉区間。
    let _ = shell.update(Message::TimelineKeySelect(KeySelectionOp::Range(second.clone())));

    let rows = shell.timeline_property_rows();
    let position_row = rows
        .iter()
        .find(|row| row.property.name() == property::POSITION)
        .expect("position 行がある");
    let selected: Vec<i64> = position_row
        .keys
        .iter()
        .filter(|k| k.selected)
        .map(|k| k.frame)
        .collect();
    assert_eq!(selected, vec![510, 570], "Shift 範囲が anchor〜clicked の閉区間を選んでいない");
}

// ---------------------------------------------------------------------------
// (c) Delete: キー選択が層選択より優先。1操作 = 1 undo で戻る。
// ---------------------------------------------------------------------------

#[test]
fn deleting_a_selected_key_removes_only_that_key_and_undo_restores_it_in_one_step() {
    let mut shell = Shell::new_fixture().0;
    let layer = shell.session().selection.expect("fixture は既定選択を持つ");
    let property = PropertyId::new(property::POSITION).expect("position は予約語ではない");

    let before = shell.timeline_property_rows();
    let position_before = before
        .iter()
        .find(|row| row.property.name() == property::POSITION)
        .expect("position 行がある");
    assert_eq!(
        position_before.keys.iter().map(|k| k.frame).collect::<Vec<_>>(),
        vec![510, 570]
    );

    let target = KeySelector { layer, property: property.clone(), frame: 510 };
    let _ = shell.update(Message::TimelineKeySelect(KeySelectionOp::Single(target)));
    let _ = shell.update(Message::TimelineDeleteSelectedKeys);

    let after_delete = shell.timeline_property_rows();
    let position_after = after_delete
        .iter()
        .find(|row| row.property.name() == property::POSITION)
        .expect("position 行はまだ残る(もう1本キーが残っているため)");
    assert_eq!(
        position_after.keys.iter().map(|k| k.frame).collect::<Vec<_>>(),
        vec![570],
        "選択した510フレームのキーだけが消えるべき"
    );
    assert!(
        shell.timeline_property_rows().iter().all(|row| row.keys.iter().all(|k| !k.selected)),
        "削除後は選択も消えているべき(Session::selected_keys を空にする)"
    );

    assert!(shell.can_undo(), "削除は undo できる1操作のはず");
    let _ = shell.update(Message::Undo);

    let after_undo = shell.timeline_property_rows();
    let position_restored = after_undo
        .iter()
        .find(|row| row.property.name() == property::POSITION)
        .expect("undo 1回で position 行が戻るはず");
    assert_eq!(
        position_restored.keys.iter().map(|k| k.frame).collect::<Vec<_>>(),
        vec![510, 570],
        "Undo 1回で削除前の2キーへ戻らない = 1操作が複数 undo になっている"
    );
}

/// 選択キーが無ければ no-op(Delete が誤って他の状態を巻き込まない)。
/// fixture は `mark_undo_floor` 済みなので、delete 前は `can_undo() == false`
/// — no-op ならそのまま `false` のはず。
#[test]
fn deleting_with_no_key_selection_is_a_no_op() {
    let mut shell = Shell::new_fixture().0;
    assert!(!shell.can_undo(), "fixture は undo floor 直後で戻れないはず(前提の確認)");
    let _ = shell.update(Message::TimelineDeleteSelectedKeys);
    assert!(
        !shell.can_undo(),
        "選択キーが無いのに Delete が何か undo 履歴を作っている"
    );
}

// ---------------------------------------------------------------------------
// 純関数(hit.rs/input.rs には触れない自己完結の証明) — 稼働中の座標だけ確認。
// ---------------------------------------------------------------------------

/// `frame_at_x`(公開の純関数)自体はこのレーンの write-set(`projection.rs`)
/// に手を入れていないことの回帰チェック — 変えていなければ既存 iced_test_spike
/// のルーラー試験と同じ式で一致する。
#[test]
fn frame_at_x_is_still_the_untouched_pure_function() {
    let width = 874.0;
    assert_eq!(frame_at_x(0.0, width, 300), 0);
    assert_eq!(frame_at_x(width, width, 300), 299);
}

// ---------------------------------------------------------------------------
// T3b: property 行の展開でズレていた hit 経路(行の縦位置)の統合。
//
// layer0(選択・opacity に2キー → property 行1本が展開)の下に layer1(標的)・
// layer2(おとり)を並べる。**layer1 と layer2 は同じフレーム区間の bar を持つ**
// ように仕組む — 修正前の `hit.rs::hit_test`/`lane_bar.rs::hit_test` は
// `layer_row_top` の押し下げを知らず旧来の一様 `row_height * index` で行を
// 割り出すので、layer1 の bar/M glyph の実描画位置(押し下げ後)をクリックすると
// 誤って layer2(1行分ズレた添字)に当たる。x 座標(フレーム区間)が同じなので、
// 「当たった layer が違う」という最も分かりやすい形で赤が出る。
// ---------------------------------------------------------------------------

/// layer0(選択・opacity に2キー、押し下げ1行ぶん=property_rows.len()==1)の下に
/// layer1(標的、[150,230))・layer2(おとり、同じ [150,230))を持つ Document。
fn three_layers_first_selected_and_keyed() -> (Document, Session, LayerId, LayerId, LayerId) {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 640,
        height: 360,
        fps: Fps::try_new(30, 1).expect("30fps"),
        duration_frames: 300,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .expect("comp を置ける");
    doc.mark_undo_floor();

    let layer0 = LayerId(1);
    let layer1 = LayerId(2); // 標的 — 押し下げ後にクリックする層。
    let layer2 = LayerId(3); // おとり — 修正前のバグが誤って当てる層(1行下)。
    for (id, start, source_frames) in [
        (layer0, 0_i64, 60_i64),
        (layer1, 150, 80),
        (layer2, 150, 80), // layer1 と全く同じ区間 — x だけでは区別が付かない。
    ] {
        doc.apply_all([
            Intent::AddLayer(id),
            Intent::SetMeta {
                layer: id,
                meta: LayerMeta {
                    source: LayerSource::Solid { rgba: [80, 160, 220, 255], width: 240, height: 135 },
                    order: 0,
                    timing: LayerTiming::place(start, Some(source_frames), 300),
                },
            },
        ])
        .expect("layer を置ける");
    }

    let property = PropertyId::new(property::OPACITY).expect("opacity は予約語ではない");
    let mut track = KeyframeTrack::new();
    for frame in [0_i64, 50] {
        track.insert(Keyframe {
            t: RationalTime::try_new(frame, 30).expect("frame は収まる"),
            value: Value::F64(1.0),
            interp: Interp::Linear,
            spatial: None,
        });
    }
    doc.apply(Intent::SetTrack { layer: layer0, property, track })
        .expect("track を書ける");

    let mut session = Session::default();
    session.selection = Some(layer0);
    (doc, session, layer0, layer1, layer2)
}

/// `super::TimelinePane::layer_row_top`(pub(crate) につき外から直接呼べない)と
/// **同じ式**をここでも組む(`key_screen_point` が `frame_to_x` にする扱いと同じ
/// 理由)。押し下げは選択行(`selected_index`)より後ろの行にだけ掛かる。
fn layer_row_top_mirror(
    row_height: f32,
    param_row_height: f32,
    property_row_count: usize,
    selected_index: usize,
    index: usize,
) -> f32 {
    let base = row_height * index as f32;
    if index > selected_index {
        base + param_row_height * property_row_count as f32
    } else {
        base
    }
}

/// layer1(押し下げ後の実描画位置、`rows` 内添字1)の bar 中心の画面座標。
fn pushed_layer1_bar_point(dims: &motolii_shell::tokens::Dimensions) -> iced::Point {
    let rail_width = dims.timeline_lane_bar_width;
    let clip_width = DEFAULT_WIDTH - rail_width;
    let ratio = 190.0 / 300.0; // [150,230) の中ほど。
    let x = rail_width + (ratio * clip_width).clamp(0.0, clip_width);

    let row_top =
        dims.row_height + layer_row_top_mirror(dims.row_height, dims.timeline_param_row_height, 1, 0, 1);
    let y = row_top + dims.row_height / 2.0;
    iced::Point::new(x, y)
}

/// レーンバー内、layer1(押し下げ後)の M glyph 中心の画面座標。`lane_bar.rs::
/// glyph_slots` と同じ式(rail 右端から `spacing_s` 空けて3個右詰め、M が先頭)。
fn pushed_layer1_mute_glyph_point(dims: &motolii_shell::tokens::Dimensions) -> iced::Point {
    let rail_width = dims.timeline_lane_bar_width;
    let glyph_w = dims.inspector_glyph_width;
    let gap = dims.spacing_xs;
    let block_w = glyph_w * 3.0 + gap * 2.0;
    let mute_x0 = rail_width - dims.spacing_s - block_w;

    let row_top =
        dims.row_height + layer_row_top_mirror(dims.row_height, dims.timeline_param_row_height, 1, 0, 1);
    let glyph_h = (dims.row_height - dims.spacing_xs).max(1.0);
    let glyph_y0 = row_top + (dims.row_height - glyph_h) / 2.0;

    iced::Point::new(mute_x0 + glyph_w / 2.0, glyph_y0 + glyph_h / 2.0)
}

/// **ORACLE (1/2)**: property 行が展開されている間、その下の layer1 の bar を
/// クリックすると layer1 が掴まる(`TimelineBarGrabbed{layer: layer1, part:
/// Body, ..}`)— 修正前は1行ぶんズレて layer2(おとり)を掴んでいた。
#[test]
fn clicking_a_bar_below_the_expanded_property_band_grabs_the_right_layer() {
    let (doc, session, _layer0, layer1, layer2) = three_layers_first_selected_and_keyed();
    let tokens = Tokens::default();
    let store = doc.view();
    let pane = TimelinePane::new(&store, &session, tokens.dims, tokens.colors, iced::keyboard::Modifiers::default());

    let point = pushed_layer1_bar_point(&tokens.dims);
    let messages = click_at(pane, point);

    // press+release を1回ずつ注入するので `TimelineBarGrabbed`(press)に続けて
    // `TimelineDragReleased`(release、`input.rs::update` の `DragKind::Clip`
    // 腕)が出る — ここで見るのは最初の1件(押した瞬間にどの layer を掴んだか)。
    match messages.first() {
        Some(Message::TimelineBarGrabbed { layer, part: BarPart::Body, .. }) => {
            assert_eq!(
                *layer, layer1,
                "property 行の押し下げ後、layer1 の bar クリックが layer2(1行下)を \
                 掴んでいる — hit.rs::hit_test がまだ layer_row_top を知らない: {messages:?}"
            );
        }
        other => panic!(
            "layer1 の bar クリックが TimelineBarGrabbed(Body) を出していない \
             (layer2={layer2:?} は同じ x 区間のおとり): {other:?}"
        ),
    }
}

/// **ORACLE (2/2)**: 同状態で layer1 の M glyph をクリックすると
/// `LaneBarToggleMute(layer1)` が出る — 修正前は1行ぶんズレて layer2 の
/// mute を切り替えていた。
#[test]
fn clicking_mute_below_the_expanded_property_band_toggles_the_right_layer() {
    let (doc, session, _layer0, layer1, layer2) = three_layers_first_selected_and_keyed();
    let tokens = Tokens::default();
    let store = doc.view();
    let pane = TimelinePane::new(&store, &session, tokens.dims, tokens.colors, iced::keyboard::Modifiers::default());

    let point = pushed_layer1_mute_glyph_point(&tokens.dims);
    let messages = click_at(pane, point);

    assert!(
        matches!(messages.as_slice(), [Message::LaneBarToggleMute(id)] if *id == layer1),
        "property 行の押し下げ後、layer1 の M glyph クリックが layer2({layer2:?})を \
         切り替えている — lane_bar.rs::hit_test がまだ layer_row_top を知らない: {messages:?}"
    );
}
