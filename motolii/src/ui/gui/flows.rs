//! 利用者の道 — 「普通こうなる」を 1 本ずつ。器具は親(`gui.rs`)の物を使う。

use super::*;

fn type_chars(gui: &mut Gui, text: &str) {
    for ch in text.chars() {
        gui.key(
            keyboard_types::Key::Character(ch.to_string()),
            keyboard_types::Modifiers::empty(),
        );
    }
}

fn enter(gui: &mut Gui) {
    gui.key(keyboard_types::Key::Enter, keyboard_types::Modifiers::empty());
}

fn history_back(gui: &Gui) -> usize {
    gui.session.doc.lock().unwrap().history_depth().0
}

/// 名前を開いたら全選択。打てば置き換わり、そのまま Enter なら何も起きない。
#[test]
fn an_opened_name_is_selected_so_typing_replaces_it() {
    let mut gui = Gui::open();
    let (x, y) = gui.center_of(".lsurface", 1);
    let before = history_back(&gui);
    gui.click(x, y);
    gui.click(x, y);
    enter(&mut gui);
    assert_eq!(gui.count("input"), 0, "Enter did not close the untouched field");
    assert_eq!(history_back(&gui), before, "an unchanged name became an undo step");

    // blitz は 500ms の実時間と 2px で連打を数える。位置をずらして数え直させる。
    gui.click(x + 3.0, y);
    gui.click(x + 3.0, y);
    type_chars(&mut gui, "Q");
    enter(&mut gui);
    assert!(gui.texts(".lsurface").iter().any(|n| n == "Q"), "typing did not replace the selected name: {:?}", gui.texts(".lsurface"));
    assert_eq!(history_back(&gui), before + 1);
}

/// 空の名前は名前にならない。
#[test]
fn an_emptied_name_is_not_a_rename() {
    let mut gui = Gui::open();
    let (x, y) = gui.center_of(".lsurface", 1);
    let names = gui.texts(".lsurface");
    let before = history_back(&gui);
    gui.click(x, y);
    gui.click(x, y);
    gui.key(keyboard_types::Key::Backspace, keyboard_types::Modifiers::empty());
    enter(&mut gui);
    assert_eq!(gui.count("input"), 0);
    assert_eq!(gui.texts(".lsurface"), names);
    assert_eq!(history_back(&gui), before);
}

/// 窓を離れたら欄は確定して消える(§6b)。
#[test]
fn losing_window_focus_commits_the_open_field() {
    let mut gui = Gui::open();
    let (x, y) = gui.center_of(".lsurface", 1);
    gui.click(x, y);
    gui.click(x, y);
    type_chars(&mut gui, "W");
    gui.lose_focus();
    assert_eq!(gui.count("input"), 0, "focus loss left the field open");
    assert!(gui.session.field().is_none());
    assert!(gui.texts(".lsurface").iter().any(|n| n == "W"), "{:?}", gui.texts(".lsurface"));
}

/// 数字を開いたら今の値が入っていて全選択。打てば置き換わり、同じ値なら何も起きない。
#[test]
fn a_number_field_starts_from_the_current_value() {
    let mut gui = Gui::open();
    let (x, y) = gui.center_of(".lsurface", 1);
    gui.click(x, y);
    let cells = gui.texts(".prow .v");
    let idx = cells
        .iter()
        .position(|c| c.trim().parse::<f64>().is_ok())
        .expect("a numeric cell in the inspector");
    let shown = cells[idx].clone();
    let (cx, cy) = gui.center_of(".prow .v", idx);
    let before = history_back(&gui);
    gui.click(cx, cy);
    gui.click(cx, cy);
    assert_eq!(gui.count("input.typing"), 1, "double-click did not open the number");
    assert_eq!(gui.h.attr("input.typing", "value").unwrap_or_default().trim(), shown.trim());
    enter(&mut gui);
    assert_eq!(history_back(&gui), before, "an unchanged number became an undo step");

    gui.click(cx + 3.0, cy);
    gui.click(cx + 3.0, cy);
    type_chars(&mut gui, "42");
    enter(&mut gui);
    assert_eq!(gui.count("input.typing"), 0);
    let after = gui.texts(".prow .v");
    assert!(after[idx].trim().parse::<f64>().is_ok_and(|v| (v - 42.0).abs() < 1e-6), "typed number did not replace the value: {:?}", after[idx]);
    assert_eq!(history_back(&gui), before + 1);
}

/// 落とした物は落とした先に出る。棚へ落とせば棚に、机へ落とせば机に、押さなくても。
#[test]
fn a_dropped_file_appears_where_it_landed_without_another_click() {
    let mut gui = Gui::open();
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.png");
    let b = dir.path().join("b.png");
    image::RgbaImage::from_pixel(4, 4, image::Rgba([20, 200, 40, 255])).save(&a).unwrap();
    image::RgbaImage::from_pixel(4, 4, image::Rgba([20, 40, 200, 255])).save(&b).unwrap();

    let cards = gui.count(".tcard");
    let stage = gui.center_of("#stage", 0);
    let summary = gui.drop_files(&[a], stage.0, stage.1);
    assert_eq!(summary.admitted, 1, "{}", summary.notice());
    assert_eq!(gui.count(".tcard"), cards + 1, "a dropped file did not appear on the shelf");

    let desk = gui.center_of("#desk", 0);
    let summary = gui.drop_files(&[b], desk.0, desk.1);
    assert_eq!(summary.admitted, 1, "{}", summary.notice());
    assert_eq!(gui.count(".desk-refs .ref"), 1, "a file dropped on the desk did not appear there");
    assert_eq!(gui.count(".tcard"), cards + 1, "a reference leaked onto the shelf");
}

/// 窓の中の余白(menubar)へ落としても、tab は別窓へ飛ばない。外へ出した時だけ。
#[test]
fn a_tab_released_over_chrome_stays_put() {
    let mut gui = Gui::open();
    let before = gui.texts(".ptab");
    let (x, y) = gui.center_of("#dock-tab-Create", 0);
    gui.press(x, y);
    gui.motion(x, y + 40.0);
    let file = gui.center_of("#menu-file", 0);
    gui.motion(file.0, file.1);
    gui.release(file.0, file.1);
    gui.settle();
    assert_eq!(gui.texts(".ptab"), before, "a release over the menubar moved a tab");
    assert_eq!(gui.count(".dock-ghost"), 0);
}

/// 掴んだまま引き出しの外へ出ても、色は置き去りにならない(そこまでの色で確定)。
#[test]
fn dragging_out_of_the_color_wheel_commits_the_pick() {
    let mut gui = Gui::open();
    let rows = gui.count(".lsurface");
    for i in 1..rows {
        let (x, y) = gui.center_of(".lsurface", i);
        gui.click(x, y);
        if gui.count(".prow.color") > 0 {
            break;
        }
    }
    let (x, y) = gui.center_of(".prow.color", 0);
    gui.click(x, y);
    let colors = gui.center_of("#dock-tab-Colors", 0);
    gui.click(colors.0, colors.1);
    let (sx, sy) = gui.size_of_nth(".sv-square", 0);
    let (cx, cy) = gui.center_of(".sv-square", 0);
    gui.press(cx + sx / 2.0 - 4.0, cy - sy / 2.0 + 4.0);
    let stage = gui.center_of("#stage", 0);
    gui.motion(stage.0, stage.1);
    gui.settle();
    let slot = match gui.session.live_focus() {
        Some(crate::ui::session::Focus::Color(slot)) => slot,
        other => panic!("focus drifted: {other:?}"),
    };
    let after = crate::ui::color::read_color(&gui.session.doc, &slot).unwrap();
    let (_, s, v) = crate::ui::color::rgb_to_hsv([after[0], after[1], after[2]]);
    assert!(s > 0.9 && v > 0.9, "leaving the drawer lost the pick: {after:?}");
    gui.release(stage.0, stage.1);
}

fn first_numeric_cell(gui: &mut Gui) -> (usize, String) {
    let cells = gui.texts(".prow .v");
    let idx = cells
        .iter()
        .position(|c| c.trim().parse::<f64>().is_ok())
        .expect("a numeric cell in the inspector");
    (idx, cells[idx].clone())
}

/// 数値を擦っている最中の Escape は取り消し。値は掴む前へ戻り、Undo には残らない。
#[test]
fn escape_while_scrubbing_restores_the_value() {
    let mut gui = Gui::open();
    let (x, y) = gui.center_of(".lsurface", 1);
    gui.click(x, y);
    let (idx, shown) = first_numeric_cell(&mut gui);
    let (cx, cy) = gui.center_of(".prow .v", idx);
    let before = history_back(&gui);
    gui.press(cx, cy);
    gui.motion(cx + 60.0, cy);
    gui.settle();
    assert_ne!(gui.texts(".prow .v")[idx], shown, "scrubbing did not move the value");
    gui.key(keyboard_types::Key::Escape, keyboard_types::Modifiers::empty());
    gui.release(cx + 60.0, cy);
    gui.settle();
    assert_eq!(gui.texts(".prow .v")[idx], shown, "Escape did not restore the value");
    assert_eq!(history_back(&gui), before, "a cancelled scrub left an undo step");
    assert!(gui.session.selection.get().is_some(), "Escape dropped the selection instead of the scrub");
}

/// 擦ったまま窓の外で放しても、そこで確定する(Undo 1 手)。
#[test]
fn releasing_a_scrub_outside_the_window_commits_it() {
    let mut gui = Gui::open();
    let (x, y) = gui.center_of(".lsurface", 1);
    gui.click(x, y);
    let (idx, shown) = first_numeric_cell(&mut gui);
    let (cx, cy) = gui.center_of(".prow .v", idx);
    let before = history_back(&gui);
    gui.press(cx, cy);
    gui.motion(cx + 60.0, cy);
    gui.release(-20.0, -20.0);
    gui.settle();
    assert_ne!(gui.texts(".prow .v")[idx], shown, "the scrub was lost on an outside release");
    assert_eq!(history_back(&gui), before + 1);
    assert!(gui.session.scrub.lock().unwrap().is_none());
}

/// 文字の本文は複数行で、時間を開けていない限り 1 つの文字を差し替える。
#[test]
fn text_content_is_multiline_and_replaces_the_only_keyframe() {
    let mut gui = Gui::open();
    let rows = gui.count(".lsurface");
    let mut layer = None;
    for i in 1..rows {
        let (x, y) = gui.center_of(".lsurface", i);
        gui.click(x, y);
        if gui.count(".prow.content-row") > 0 {
            layer = gui.session.selection.get();
            break;
        }
    }
    let layer = layer.expect("a text layer in the fixture");
    let keys_before = gui.session.doc.lock().unwrap().view().text_document(layer).unwrap().unwrap().content.keys().len();
    let (x, y) = gui.center_of(".prow.content-row .v.content", 0);
    gui.click(x, y);
    gui.click(x, y);
    assert_eq!(gui.count("textarea.content"), 1, "the content field is not multiline");
    type_chars(&mut gui, "ab");
    gui.key(keyboard_types::Key::Enter, keyboard_types::Modifiers::empty());
    type_chars(&mut gui, "cd");
    gui.key(keyboard_types::Key::Enter, keyboard_types::Modifiers::SUPER);
    assert_eq!(gui.count("textarea"), 0);
    let text = gui.session.doc.lock().unwrap().view().text_document(layer).unwrap().unwrap();
    let keys = text.content.keys();
    assert!(keys.iter().any(|k| k.content.contains("ab\ncd")), "the line break was lost: {:?}", keys.iter().map(|k| k.content.clone()).collect::<Vec<_>>());
    if keys_before <= 1 {
        assert_eq!(keys.len(), 1, "typing opened time without asking");
    }
}

/// COLOR の行を押したら、輪の居る Colors が前に出る。
#[test]
fn focusing_a_color_row_brings_the_colors_panel_forward() {
    let mut gui = Gui::open();
    let rows = gui.count(".lsurface");
    for i in 1..rows {
        let (x, y) = gui.center_of(".lsurface", i);
        gui.click(x, y);
        if gui.count(".prow.color") > 0 {
            break;
        }
    }
    assert!(!gui.classes("#dock-tab-Colors").iter().any(|c| c.contains("on")));
    let (x, y) = gui.center_of(".prow.color", 0);
    gui.click(x, y);
    assert!(gui.classes("#dock-tab-Colors").iter().any(|c| c.contains("on")), "Colors did not come forward");
    assert_eq!(gui.count(".color-pick"), 1);
}

/// Enter で選んだ層の名前が開く。M で印が生まれ、本文を書く場所が開いている。Edit menu に Undo / Redo。
#[test]
fn enter_renames_m_marks_and_edit_menu_has_undo() {
    let mut gui = Gui::open();
    let (x, y) = gui.center_of(".lsurface", 1);
    gui.click(x, y);
    gui.key(keyboard_types::Key::Enter, keyboard_types::Modifiers::empty());
    assert_eq!(gui.count("input.lsurface"), 1, "Enter did not open the name");
    gui.key(keyboard_types::Key::Escape, keyboard_types::Modifiers::empty());
    assert_eq!(gui.count("input"), 0);

    let before = gui.session.doc.lock().unwrap().view().markers().unwrap().len();
    gui.key(keyboard_types::Key::Character("m".into()), keyboard_types::Modifiers::empty());
    assert_eq!(gui.session.doc.lock().unwrap().view().markers().unwrap().len(), before + 1, "M did not mark");
    assert_eq!(gui.count(".desk-note .mbody"), 1, "the note did not open with the marker");

    let edit = gui.center_of("#menu-edit", 0);
    gui.click(edit.0, edit.1);
    assert_eq!(gui.count("#menu-edit-list"), 1);
    assert!(gui.texts("#menu-edit-list .vitem").iter().any(|t| t == "Undo"));
    let undo = gui.center_of_text("#menu-edit-list .vitem", "Undo");
    gui.click(undo.0, undo.1);
    assert_eq!(gui.session.doc.lock().unwrap().view().markers().unwrap().len(), before, "Edit▸Undo did not undo");
    assert_eq!(gui.count("#menu-edit-list"), 0, "picking Undo left the menu open");
}

/// 複数選択: Inspector は Transform の共通行を出し、擦れば全部が同じ差分で動き、打てば全部が同じ値になる。
#[test]
fn a_multi_selection_shows_common_rows_and_writes_to_every_layer() {
    let mut gui = Gui::open();
    let (x1, y1) = gui.center_of(".lsurface", 1);
    let (x2, y2) = gui.center_of(".lsurface", 2);
    gui.click(x1, y1);
    gui.click_super(x2, y2);
    let chosen = gui.session.selection.all();
    assert_eq!(chosen.len(), 2, "Cmd-click did not add to the selection");
    assert!(gui.texts(".ident b").iter().any(|t| t == "2 layers"), "{:?}", gui.texts(".ident b"));

    let pos = crate::doc::store::PropertyId::new(crate::doc::store::property::POSITION).unwrap();
    let t = gui.session.clock.current_time();
    let x_of = |gui: &Gui, l: crate::doc::store::LayerId| -> f64 {
        let d = gui.session.doc.lock().unwrap();
        match crate::ui::inspector::value_with_default(&d.view(), l, &pos, crate::doc::store::property::POSITION, t) {
            Some(crate::doc::eval::Value::Vec2([x, _])) => x,
            other => panic!("position is not a vec2: {other:?}"),
        }
    };
    let before: Vec<f64> = chosen.iter().map(|l| x_of(&gui, *l)).collect();

    let labels = gui.texts(".prow .n");
    let row = labels.iter().position(|l| l == "Position").expect("Position row");
    let (cx, cy) = gui.center_of(".prow .v", row * 3);
    gui.press(cx, cy);
    gui.motion(cx + 40.0, cy);
    gui.release(cx + 40.0, cy);
    gui.settle();
    let after: Vec<f64> = chosen.iter().map(|l| x_of(&gui, *l)).collect();
    for i in 0..2 {
        assert!((after[i] - before[i] - 40.0).abs() < 1e-6, "layer {i} moved by {} not 40", after[i] - before[i]);
    }

    gui.click(cx + 3.0, cy);
    gui.click(cx + 3.0, cy);
    assert_eq!(gui.count("input.typing"), 1);
    type_chars(&mut gui, "10");
    enter(&mut gui);
    for l in &chosen {
        assert!((x_of(&gui, *l) - 10.0).abs() < 1e-6, "typed value did not reach every layer");
    }
}

/// 文字の級数は Inspector の数の行。擦れば変わる。
#[test]
fn a_text_layer_has_a_scrubbable_size_row() {
    let mut gui = Gui::open();
    let rows = gui.count(".lsurface");
    let mut layer = None;
    for i in 1..rows {
        let (x, y) = gui.center_of(".lsurface", i);
        gui.click(x, y);
        if gui.count(".prow.content-row") > 0 {
            layer = gui.session.selection.get();
            break;
        }
    }
    let layer = layer.expect("a text layer");
    let labels = gui.texts(".prow .n");
    assert!(labels.iter().any(|l| l == "Size"), "{labels:?}");
    // Size の欄は文字の行の後ろ(Content の次)。数字の入った `.v` を Content 以降で探す。
    let cells = gui.texts(".prow .v");
    let content_at = cells.iter().position(|c| c == &gui.texts(".prow.content-row .v")[0]).unwrap_or(0);
    let cell = (content_at..cells.len())
        .find(|i| cells[*i].trim().parse::<f64>().is_ok())
        .expect("a numeric Size cell");
    let shown: f64 = cells[cell].trim().parse().unwrap();
    let (cx, cy) = gui.center_of(".prow .v", cell);
    // 右へ引きすぎると放す先が ◇ に乗る(blitz は up の先へ click を配る)。欄の中で放す。
    gui.press(cx, cy);
    gui.motion(cx + 12.0, cy);
    gui.release(cx + 12.0, cy);
    gui.settle();
    let style = gui.session.doc.lock().unwrap().view().text_document(layer).unwrap().unwrap().styles[0].id;
    let prop = crate::doc::store::PropertyId::text_style_size(style);
    let t = gui.session.clock.current_time();
    let after = gui.session.doc.lock().unwrap().view().value_at(layer, &prop, t).unwrap();
    assert!(matches!(after, Some(crate::doc::eval::Value::F64(v)) if v > shown), "size did not grow: {after:?} from {shown}");
}

/// Blend の格子は hover するだけで Stage が変わり、離れれば戻る。Undo には残らない(§4)。
#[test]
fn hovering_a_blend_cell_previews_it_on_the_stage() {
    let mut gui = Gui::open();
    let (x, y) = gui.center_of(".lsurface", 1);
    gui.click(x, y);
    let layer = gui.session.selection.get().unwrap();
    let blend = gui.center_of_text(".desk-foot .chip", "Blend");
    gui.click(blend.0, blend.1);
    assert!(gui.count(".blend-cell") > 2);
    let before = history_back(&gui);
    let prop = crate::doc::store::PropertyId::blend_mode();
    let t = gui.session.clock.current_time();
    let resolved = |gui: &Gui| gui.session.doc.lock().unwrap().view().value_at(layer, &prop, t).unwrap();
    assert_eq!(resolved(&gui), None);

    let (cx, cy) = gui.center_of(".blend-cell", 2);
    gui.motion(cx, cy);
    gui.settle();
    assert!(matches!(resolved(&gui), Some(crate::doc::eval::Value::Enum(2))), "hover did not preview: {:?}", resolved(&gui));

    let stage = gui.center_of("#stage", 0);
    gui.motion(stage.0, stage.1);
    gui.settle();
    assert_eq!(resolved(&gui), None, "leaving did not restore the blend");
    assert_eq!(history_back(&gui), before);
}

/// 文字層の Inspector には縁取り・行間・字送りの行が最初から在る。
#[test]
fn a_text_layer_exposes_stroke_line_height_and_tracking() {
    let mut gui = Gui::open();
    let rows = gui.count(".lsurface");
    for i in 1..rows {
        let (x, y) = gui.center_of(".lsurface", i);
        gui.click(x, y);
        if gui.count(".prow.content-row") > 0 {
            break;
        }
    }
    let labels = gui.texts(".prow .n");
    for want in ["Stroke", "Line height", "Tracking"] {
        assert!(labels.iter().any(|l| l == want), "missing {want}: {labels:?}");
    }
}

/// 机の書き置きは、選んでいる文字層の本文へ 1 押しで送れる。
#[test]
fn a_marker_note_can_be_sent_to_the_selected_text_layer() {
    let mut gui = Gui::open();
    {
        use crate::doc::store::{Intent, Marker, RationalTime};
        let marker = Marker { name: "verse".into(), time: RationalTime::ZERO, duration: RationalTime::ZERO, body: "la la".into() };
        gui.session.doc.lock().unwrap().apply(Intent::SetMarkers { markers: vec![marker] }).unwrap();
    }
    let rows = gui.count(".lsurface");
    let mut layer = None;
    for i in 1..rows {
        let (x, y) = gui.center_of(".lsurface", i);
        gui.click(x, y);
        if gui.count(".prow.content-row") > 0 {
            layer = gui.session.selection.get();
            break;
        }
    }
    let layer = layer.expect("a text layer");
    let text = gui.center_of_text(".desk-foot .chip", "Text");
    gui.click(text.0, text.1);
    let send = gui.center_of(".desk-note .tolayer", 0);
    gui.click(send.0, send.1);
    let content = gui.session.doc.lock().unwrap().view().text_document(layer).unwrap().unwrap().content.eval(gui.session.clock.current_time()).to_string();
    assert_eq!(content, "la la");
}
