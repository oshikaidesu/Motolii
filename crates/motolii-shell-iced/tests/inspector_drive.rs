//! Inspector pane の運転席(M-4b)— red 先行。
//!
//! 審判は 2026-08-13 の key add UX 決定の Oracle と、品質バー Q0 / Q2 / Q3 / Q5 / Q7:
//!
//! - Positive: unkeyed の ◇ を押す → 既存 typed intent が layer / time / property を
//!   1回運ぶ → accepted snapshot が同じ property の current key を返す →
//!   値行は1つのまま、ボタンだけが filled current へ変わる
//! - Negative: reject 応答では状態を進めず、理由が帯に出る。current-key 専用の
//!   重複値行を作らない。局所 button state だけを先行させない(optimistic 禁止)
//!
//! 状態の導出元は **accepted snapshot だけ**なので、審判も `Shell::inspector()`
//! (投影)と `Shell::document()`(snapshot)だけを読む。

mod common;

use common::{drain, feed, file_dropped, press, redraw, starter_media_dir};
use motolii_shell_iced::inspector_model::InspectorSeat;
use motolii_shell_iced::inspector_pane::{self, InspectorEvent};
use motolii_shell_iced::widgets::{KeyState, ScrubEvent};
use motolii_shell_iced::{view, Message, ScriptedPrompts, Shell};
use motolii_ui::blitz_shell::{UiEditParam, UiIntent, UiItemFlag};

/// New project を1つ作って、starter still を1本落として、その layer を選ぶ。
/// 返り値は選ばれた layer id。
fn seat_with_selected_layer(shell: &mut Shell) -> u64 {
    let still = starter_media_dir().join("starter-still.png");
    let pressed = press(
        iced_test::simulator(view(shell)),
        motolii_shell_iced::view::NEW_PROJECT,
    );
    drain(shell, pressed);
    let dropped = feed(
        iced_test::simulator(view(shell)),
        [file_dropped(&still), redraw()],
    );
    drain(shell, dropped);
    let document = shell.document().expect("座席が立っている");
    let layer = document
        .tracks
        .iter()
        .flat_map(|track| &track.items)
        .map(|item| match item {
            motolii_doc::TrackItem::Clip(clip) => clip.envelope.layer_id.get(),
            motolii_doc::TrackItem::Group(group) => group.envelope.layer_id.get(),
        })
        .next()
        .expect("落とした still が clip として立っている");
    drain(shell, vec![Message::LayerSelected(layer)]);
    layer
}

fn ready_model(shell: &Shell) -> motolii_shell_iced::InspectorModel {
    match shell.inspector() {
        InspectorSeat::Ready(model) => model,
        other => panic!("Inspector が座っていない: {other:?}"),
    }
}

/// **空状態の正直さ(Q7)。** 座っただけ(選択なし)の Inspector は
/// 「No selection」を面に出し、fixture にも幽霊行にも落ちない。
#[test]
fn an_unselected_inspector_says_no_selection() {
    let dir = motolii_testkit::tmp_dir("iced_inspector_empty");
    let mut shell = Shell::new(ScriptedPrompts {
        new_project_path: Some(dir.join("fresh.json")),
        ..ScriptedPrompts::default()
    });
    let pressed = press(
        iced_test::simulator(view(&shell)),
        motolii_shell_iced::view::NEW_PROJECT,
    );
    drain(&mut shell, pressed);

    assert_eq!(shell.inspector(), InspectorSeat::NoSelection);
    // 絵にも同じ一言が立っている(黙って空の面にしない)。
    let mut ui = iced_test::simulator(view(&shell));
    ui.find(inspector_pane::NO_SELECTION)
        .expect("空状態の理由が面に出ていない");
}

/// **APPEARANCE section と Opacity 行は選択さえあれば必ず存在する(潰れ回帰
/// レーン、2026-08-19)。** 存在の pin であって、可視性/クリップの pin では
/// **ない** — 実測したところ `iced_test::Simulator::find` はレイアウト木を
/// 文字列一致で辿るだけで、`scrollable`/`container` の viewport clip を
/// 考慮しない(意図的に `scroller` を高さ固定+`clip(true)` の潰れた箱へ
/// 差し替えて確認済み。それでもこの `find` は変わらず成功した)。「潰れて
/// 見えない」という視覚回帰を実際に検出するのは、クリック位置がクリップ域
/// 外だと失敗する [`common::scroll_then_click`] 経由の押下テスト
/// (`an_effect_toggle_writes_the_shared_definition` が EFFECTS 側で既に
/// 実践している型)。この test はそれより弱い保証 — 「APPEARANCE section
/// 自体を丸ごと落とす」「Opacity 行を出し忘れる」といった構造欠落だけを拾う。
#[test]
fn the_appearance_section_and_opacity_row_are_reachable() {
    let dir = motolii_testkit::tmp_dir("iced_inspector_opacity_reachable");
    let mut shell = Shell::new(ScriptedPrompts {
        new_project_path: Some(dir.join("fresh.json")),
        ..ScriptedPrompts::default()
    });
    seat_with_selected_layer(&mut shell);

    let mut ui = iced_test::simulator(view(&shell));
    ui.find(inspector_pane::APPEARANCE)
        .expect("APPEARANCE 見出しが面に無い(scrollable が欠けると潰れて消える)");
    ui.find("Opacity")
        .expect("Opacity 行が面に無い(scrollable が欠けると潰れて消える)");
}

/// **2026-08-13 Oracle の Positive 列。** unkeyed の ◇ を1回押すと:
/// 1. `KeyParamAtPlayhead` が layer / property / 画面の成分値を**1回だけ**運び、
/// 2. accepted snapshot からの再投影で同じ行が current(◆)になり、
/// 3. 行は増えず、成分値も変わらない。
#[test]
fn pressing_an_unkeyed_diamond_becomes_an_intent_and_a_current_key() {
    if !motolii_testkit::ffmpeg_or_skip() {
        return;
    }
    let dir = motolii_testkit::tmp_dir("iced_inspector_key_add");
    let mut shell = Shell::new(ScriptedPrompts {
        new_project_path: Some(dir.join("fresh.json")),
        ..ScriptedPrompts::default()
    });
    let layer = seat_with_selected_layer(&mut shell);

    let before = ready_model(&shell);
    let position = before
        .transform_row(UiEditParam::Position)
        .expect("Position 行がある")
        .clone();
    assert_eq!(
        position.key_state,
        KeyState::Unkeyed,
        "前提: まだキーが無い"
    );
    assert_eq!(position.components.len(), 2, "Vec2 は X/Y の2成分");
    let rows_before = before.transform.len();
    let intents_before = shell.intent_count();

    // 絵の ◇ を押す(depth-first の先頭 = Position の header の1枚)。
    let pressed = press(iced_test::simulator(view(&shell)), "\u{25c7}");
    drain(&mut shell, pressed);

    // 1. intent が1回、layer / property / 画面の成分値ごと運ばれた。
    let intents = shell.intents();
    assert_eq!(
        intents.len(),
        intents_before + 1,
        "◇ 1押しは intent 1件である"
    );
    assert_eq!(
        intents.last().map(|event| event.intent.clone()),
        Some(UiIntent::KeyParamAtPlayhead {
            layer,
            param: UiEditParam::Position,
            components: position.components.clone(),
        }),
        "画面に出ている成分値がそのままキーになる(2026-08-13裁定)"
    );

    // 2.-3. accepted snapshot からの再投影: ボタンだけが current になり、
    //        行は増えず、値も変わらない。
    let after = ready_model(&shell);
    let keyed = after
        .transform_row(UiEditParam::Position)
        .expect("同じ Position 行");
    assert_eq!(keyed.key_state, KeyState::Current, "◆(current)になる");
    assert_eq!(
        keyed.components, position.components,
        "キー化しても値行の数は変わらない"
    );
    assert_eq!(
        after.transform.len(),
        rows_before,
        "current-key 専用の重複値行を作らない"
    );
}

/// **M / S は Document を通る(F-03 の再現禁止)。** 押下状態は投影
/// (= `ItemEnvelope`)から返り、局所 bool はどこにも無い。
#[test]
fn mute_and_solo_toggle_through_the_document() {
    if !motolii_testkit::ffmpeg_or_skip() {
        return;
    }
    let dir = motolii_testkit::tmp_dir("iced_inspector_flags");
    let mut shell = Shell::new(ScriptedPrompts {
        new_project_path: Some(dir.join("fresh.json")),
        ..ScriptedPrompts::default()
    });
    let layer = seat_with_selected_layer(&mut shell);
    assert!(!ready_model(&shell).muted, "既定は聞こえて見える");

    let pressed = press(iced_test::simulator(view(&shell)), "M");
    drain(&mut shell, pressed);
    assert!(
        ready_model(&shell).muted,
        "M は Document の visible を書く(見た目だけの反転ではない)"
    );
    assert_eq!(
        shell.intents().last().map(|event| event.intent.clone()),
        Some(UiIntent::ToggleItemFlag {
            layer,
            flag: UiItemFlag::Mute,
        }),
    );
    // snapshot 側でも同じ真実(Q5)。
    let document = shell.document().expect("seated");
    let visible = document
        .tracks
        .iter()
        .flat_map(|track| &track.items)
        .find_map(|item| match item {
            motolii_doc::TrackItem::Clip(clip) if clip.envelope.layer_id.get() == layer => {
                Some(clip.envelope.visible)
            }
            _ => None,
        })
        .expect("clip がある");
    assert!(!visible, "M = !visible(Timeline 行と同じ約束)");

    let pressed = press(iced_test::simulator(view(&shell)), "S");
    drain(&mut shell, pressed);
    assert!(ready_model(&shell).solo, "S も同じ1手で Document を書く");

    // もう一度 M → 戻る(反転要求であって set ではない)。
    let pressed = press(iced_test::simulator(view(&shell)), "M");
    drain(&mut shell, pressed);
    assert!(!ready_model(&shell).muted);
}

/// **スクラブの確定は1回の Undo 単位で Document へ入る。** 値は accepted
/// snapshot からの再投影にだけ現れる(先行する局所値が無い)。
#[test]
fn a_scrub_commit_writes_the_document_once() {
    if !motolii_testkit::ffmpeg_or_skip() {
        return;
    }
    let dir = motolii_testkit::tmp_dir("iced_inspector_scrub");
    let mut shell = Shell::new(ScriptedPrompts {
        new_project_path: Some(dir.join("fresh.json")),
        ..ScriptedPrompts::default()
    });
    let layer = seat_with_selected_layer(&mut shell);
    let revision = shell.revision();

    drain(
        &mut shell,
        vec![Message::Inspector(InspectorEvent::Scrub {
            param: UiEditParam::Opacity,
            component: 0,
            event: ScrubEvent::Committed(0.5),
        })],
    );

    assert!(
        shell.revision() > revision,
        "確定が Document へ入っていない"
    );
    let opacity = ready_model(&shell)
        .transform_row(UiEditParam::Opacity)
        .expect("Opacity 行")
        .components
        .clone();
    assert_eq!(opacity, vec![0.5], "投影が accepted の値を返す");
    // 台帳には「成分を書いた」+「掴みを離した」が同じ順で載る(replay 可能)。
    let intents: Vec<UiIntent> = shell
        .intents()
        .into_iter()
        .map(|event| event.intent)
        .collect();
    let tail = &intents[intents.len() - 2..];
    assert_eq!(
        tail,
        [
            UiIntent::SetParamComponent {
                layer,
                param: UiEditParam::Opacity,
                component: 0,
                value: 0.5,
            },
            UiIntent::EndParamEdit,
        ],
    );
}

/// **reject は状態を進めず、理由が帯に出る(Q3 の拒否の報酬 / 2026-08-13 Negative)。**
/// 存在しない FX definition への ON/OFF は断られ、revision も投影も動かない。
#[test]
fn a_rejected_edit_reports_a_reason_and_does_not_advance() {
    if !motolii_testkit::ffmpeg_or_skip() {
        return;
    }
    let dir = motolii_testkit::tmp_dir("iced_inspector_reject");
    let mut shell = Shell::new(ScriptedPrompts {
        new_project_path: Some(dir.join("fresh.json")),
        ..ScriptedPrompts::default()
    });
    seat_with_selected_layer(&mut shell);
    let revision = shell.revision();
    let before = ready_model(&shell);

    drain(
        &mut shell,
        vec![Message::Inspector(InspectorEvent::SetEffectEnabled {
            definition_id: 999,
            enabled: false,
        })],
    );

    assert_eq!(shell.revision(), revision, "reject が状態を進めている");
    assert_eq!(
        ready_model(&shell),
        before,
        "reject 後の投影は1 bit も変わらない(optimistic 禁止)"
    );
    let said = shell.latest_report().unwrap_or_default();
    assert!(
        said.contains("rejected"),
        "断った理由が帯に出ていない: {said:?}"
    );
}

/// **共有 FX の ON/OFF は実 intent で recipe を書く。** 押下状態は
/// `EffectDefinition.enabled`(Document)からだけ導出される。
/// param 行は閉じた語彙(tint = Color 1行)だけが出る。
#[test]
fn an_effect_toggle_writes_the_shared_definition() {
    if !motolii_testkit::ffmpeg_or_skip() {
        return;
    }
    let dir = motolii_testkit::tmp_dir("iced_inspector_fx");
    let path = dir.join("fx.json");
    // まず殻で「project を作って still を1本置いて保存」まで進め、座席を返す。
    let mut first = Shell::new(ScriptedPrompts {
        new_project_path: Some(path.clone()),
        ..ScriptedPrompts::default()
    });
    seat_with_selected_layer(&mut first);
    let saved = common::command_key('s');
    let typed = feed(iced_test::simulator(view(&first)), saved);
    drain(&mut first, typed);
    drop(first); // OS lock を返す(次の殻が同じ project を開けるように)

    // 次に**永続層で** tint recipe と、置いた clip からの EffectUse を仕立てる。
    // FX の追加 UI は非目標(2026-08-13)なので、テスト fixture はファイルを
    // 直接組む — 開いた後の書き道は相変わらず intent だけである。
    let mut document = motolii_doc::load_document(&path).expect("load");
    let definition_id = document.next_stable_id.allocate().expect("stable id");
    let use_id = document.next_stable_id.allocate().expect("stable id");
    document
        .effect_definitions
        .push(motolii_doc::EffectDefinition::new(
            motolii_doc::EffectDefinitionId::from_raw(definition_id),
            "core.filter.tint",
            1,
            true,
            std::collections::BTreeMap::new(),
            serde_json::Map::new(),
        ));
    {
        let item = document
            .tracks
            .iter_mut()
            .flat_map(|track| &mut track.items)
            .next()
            .expect("置いた clip");
        let envelope = match item {
            motolii_doc::TrackItem::Clip(clip) => &mut clip.envelope,
            motolii_doc::TrackItem::Group(group) => &mut group.envelope,
        };
        envelope.effects.push(motolii_doc::EffectUse {
            id: motolii_doc::EffectId::from_raw(use_id),
            definition_id: motolii_doc::EffectDefinitionId::from_raw(definition_id),
        });
    }
    let mut session =
        motolii_doc::ProjectSession::acquire(&path, &motolii_doc::ResourceLimits::production())
            .expect("acquire");
    session
        .save_document(&document, &motolii_doc::SaveOptions::default())
        .expect("save");
    drop(session);

    let mut shell = Shell::new(ScriptedPrompts {
        open_project_path: Some(path),
        ..ScriptedPrompts::default()
    });
    let pressed = press(
        iced_test::simulator(view(&shell)),
        motolii_shell_iced::view::OPEN_PROJECT,
    );
    drain(&mut shell, pressed);
    let layer = shell
        .document()
        .expect("seated")
        .tracks
        .iter()
        .flat_map(|track| &track.items)
        .map(|item| match item {
            motolii_doc::TrackItem::Clip(clip) => clip.envelope.layer_id.get(),
            motolii_doc::TrackItem::Group(group) => group.envelope.layer_id.get(),
        })
        .next()
        .expect("clip");
    drain(&mut shell, vec![Message::LayerSelected(layer)]);

    let model = ready_model(&shell);
    assert_eq!(model.effects.len(), 1);
    assert!(model.effects[0].enabled, "既定は ON(Document の値)");
    assert_eq!(
        model.effects[0].params.len(),
        1,
        "tint の param は Color 1行だけ(List / AssetRef は行を出さない)"
    );

    // ON を押す → OFF になる(実 intent。見た目だけの反転ではない)。
    // EFFECTS section は scrollable の中(2026-08-18 視覚再現レーンで TRANSFORM
    // が伸びたので、既定ビューポートには入らないことがある) — 先に送ってから押す。
    let pressed = common::scroll_then_click(
        iced_test::simulator(view(&shell)),
        iced::Point::new(900.0, 300.0),
        "ON",
    );
    drain(&mut shell, pressed);
    assert_eq!(
        shell.intents().last().map(|event| event.intent.clone()),
        Some(UiIntent::SetEffectEnabled {
            definition_id,
            enabled: false,
        }),
    );
    assert!(
        !ready_model(&shell).effects[0].enabled,
        "OFF は Document(recipe)からの再投影で返る"
    );
    // 絵も OFF を名乗る(帯だけ揃って絵が違うのは不合格)。
    let mut ui = iced_test::simulator(view(&shell));
    ui.find("OFF").expect("OFF ボタンが立っていない");
}
