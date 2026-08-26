//! Browser pane(M-4a + browser-revise レーン)— red 先行。
//!
//! oracle は `docs/ui-quality-bar.md` の **Q0**(触れそうな物は必ず機能する /
//! 死に chrome 禁止)・**Q1**(click=選択、double-click=生成/適用)・
//! **Q3**(全操作に報酬。拒否は理由つき)と、`docs/ui-interaction-language.md`
//! Browser 節(Media 統合 shell・source rail・All Media の dedupe)。
//!
//! 移植元は egui 版 `motolii_ui::browser_panel` の運転席テスト
//! (`double_clicking_a_card_asks_for_that_file_to_be_placed` /
//! `a_single_click_only_selects_and_asks_for_nothing`)。駆動器が
//! kittest → `iced_test::Simulator` に替わっても、**問いは同じ**である。
//!
//! browser-revise レーン(2026-08-19)で検索欄・kind filter chip・表示モードが
//! 実装され、Q0 の判定が変わった行がある: `"Search files and tags"` は
//! かつて「機能しない死に chrome」の実例だったが、いまは実機能の
//! placeholder 文言なので dead-chrome リストから外し、逆に機能を検証する
//! 側のテストへ移した。

mod common;

use std::path::PathBuf;

use common::{
    double_click, drain, feed, file_dropped, file_hovered, files_hovered_left, press, redraw,
    starter_media_dir,
};
use motolii_shell_iced::{
    browser_pane, view, BrowserRail, BrowserViewMode, Message, ScriptedPrompts, Shell,
};
use motolii_ui::blitz_shell::UiIntent;

/// 新規 project へ座った殻(運転席の型)。
fn seated_shell(tag: &str) -> Shell {
    let dir = motolii_testkit::tmp_dir(tag);
    let project = dir.join("fresh.json");
    let mut shell = Shell::new(ScriptedPrompts {
        new_project_path: Some(project),
        ..ScriptedPrompts::default()
    });
    let pressed = press(iced_test::simulator(view(&shell)), view::NEW_PROJECT);
    drain(&mut shell, pressed);
    assert!(shell.is_seated(), "前提: 新規 project へ座れている");
    shell
}

/// journal の中の `AdmitPaths` だけを順に。
fn admissions(shell: &Shell) -> Vec<Vec<PathBuf>> {
    shell
        .intents()
        .into_iter()
        .filter_map(|event| match event.intent {
            UiIntent::AdmitPaths { paths } => Some(paths),
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// source rail(Q0: 触れる物だけが立つ)
// ---------------------------------------------------------------------------

/// 座った窓には `All media / Project / Recent` の3席が立ち、**機能しない rail は
/// 1つも立たない**(COLLECTIONS・Add folder・複数登録 folder は product に
/// 対応する機能が無いので置かない — `browser_pane` module doc の Q0 節)。
/// スタート画面には Browser そのものが無い(座席が無ければ見せる物が無い)。
#[test]
fn the_seated_window_offers_exactly_the_three_working_rails() {
    let dir = motolii_testkit::tmp_dir("iced_browser_rails");
    let project = dir.join("fresh.json");
    let mut shell = Shell::new(ScriptedPrompts {
        new_project_path: Some(project),
        ..ScriptedPrompts::default()
    });

    // スタート画面: Browser は無い。
    let mut before = iced_test::simulator(view(&shell));
    assert!(
        before.find(browser_pane::RAIL_ALL).is_err(),
        "座る前に Browser の rail が立っている"
    );
    drop(before);

    let pressed = press(iced_test::simulator(view(&shell)), view::NEW_PROJECT);
    drain(&mut shell, pressed);

    let mut ui = iced_test::simulator(view(&shell));
    for rail in [
        browser_pane::RAIL_ALL,
        browser_pane::RAIL_PROJECT,
        browser_pane::RAIL_RECENT,
    ] {
        assert!(
            ui.find(rail).is_ok(),
            "rail {rail:?} が立っていない(3席とも機能する実装で立つ)"
        );
    }
    // 死に chrome 禁止(Q0)。html mock に居るが product に対応機能が無い物
    // (COLLECTIONS/Add folder/複数登録 folder/tag 編集)を持ち込まない。
    // "Search files and tags" はここには**居ない** — browser-revise レーンで
    // 実装された今は機能する chrome で、検証は他のテストへ移した。
    for dead in [
        "Collections",
        "COLLECTIONS",
        "Add folder",
        "Edit tags",
        "PLACES",
    ] {
        assert!(
            ui.find(dead).is_err(),
            "{dead:?} は機能しないのに立っている(Q0 違反)"
        );
    }
    assert!(
        ui.find(browser_pane::SEARCH_PLACEHOLDER).is_ok(),
        "検索欄が立っていない(browser-revise レーンで機能する chrome になった)"
    );
}

/// rail を押すと表示が切り替わる(pane 内の状態。intent は増えない)。
#[test]
fn choosing_a_rail_switches_the_pane_without_touching_the_journal() {
    let mut shell = seated_shell("iced_browser_rail_switch");
    let before = shell.intent_count();

    let pressed = press(
        iced_test::simulator(view(&shell)),
        browser_pane::RAIL_RECENT,
    );
    assert_eq!(
        pressed,
        vec![Message::BrowserRailChosen(BrowserRail::Recent)],
        "rail のクリックは rail 切替の Message になる"
    );
    drain(&mut shell, pressed);

    assert_eq!(shell.browser().rail(), BrowserRail::Recent);
    assert_eq!(
        shell.intent_count(),
        before,
        "rail 切替は Document に触れない = journal に載らない(view 系 intent は将来枠)"
    );
}

// ---------------------------------------------------------------------------
// click = 選択 / double-click = 配置(Q1)
// ---------------------------------------------------------------------------

/// 単クリックは**選択だけ**。要求は出ず、Document も動かない。
/// 報酬(Q3)は selection tray が選んだ card を名指しすること。
#[test]
fn a_card_click_selects_and_asks_for_nothing() {
    let mut shell = seated_shell("iced_browser_click");
    let admissions_before = admissions(&shell).len();

    let clicked = press(iced_test::simulator(view(&shell)), "starter-clip.mp4");
    drain(&mut shell, clicked);

    let selected = shell
        .browser()
        .selected_id()
        .map(str::to_owned)
        .expect("単クリックは選択する");
    assert!(
        selected.ends_with("starter-clip.mp4"),
        "選ばれたのは触ったカード。実際: {selected:?}"
    );
    assert_eq!(
        admissions(&shell).len(),
        admissions_before,
        "単クリックは配置要求を出さない(選ぶだけ)"
    );
    assert_eq!(
        shell.track_item_count(),
        0,
        "選択で Document が動いてはならない"
    );

    // 選択の報酬: selection tray が名指しする(触ったのに何も変わらない、を作らない)。
    let mut after = iced_test::simulator(view(&shell));
    let tray = browser_pane::tray_label("starter-clip.mp4");
    assert!(
        after.find(tray.as_str()).is_ok(),
        "selection tray が選んだ card を名指ししていない: {tray:?}"
    );
}

/// ダブルクリックは admit + **playhead へ配置**。経路は OS ドロップと同じ
/// `UiIntent::AdmitPaths` 1本で、帯が成立を名指しで言う。
#[test]
fn a_double_clicked_card_lands_at_the_playhead() {
    if !motolii_testkit::ffmpeg_or_skip() {
        return;
    }
    let mut shell = seated_shell("iced_browser_place");
    let revision_before = shell.revision();

    let messages = double_click(iced_test::simulator(view(&shell)), "starter-clip.mp4");
    drain(&mut shell, messages);

    let clip = starter_media_dir().join("starter-clip.mp4");
    assert_eq!(
        admissions(&shell),
        vec![vec![clip]],
        "ダブルクリックは AdmitPaths 1件になる(OS ドロップと同じ合流点)"
    );
    assert_eq!(
        shell.track_item_count(),
        1,
        "帯が言うだけで Document が動かないのは不合格"
    );
    assert!(
        shell.revision() > revision_before,
        "配置は writer を通る(revision が進む)"
    );
    let report = shell.latest_report().unwrap_or_default();
    assert!(
        report.contains("placed") && report.contains("starter-clip.mp4"),
        "配置の成立は帯が名指しで言う。実際: {report:?}"
    );
    assert!(
        shell
            .browser()
            .selected_id()
            .is_some_and(|id| id.ends_with("starter-clip.mp4")),
        "置いたカードは選ばれてもいる(egui 版と同じ)"
    );
}

/// 指した先が消えていたら**理由を帯で言い**、存在しない path を intent にしない
/// (Q3 拒否の報酬 / egui 版 `an_unknown_card_id_yields_no_request` の対応物)。
#[test]
fn a_card_whose_file_vanished_says_why_and_admits_nothing() {
    let dir = motolii_testkit::tmp_dir("iced_browser_gone");
    let library = dir.join("library");
    std::fs::create_dir_all(&library).expect("library dir");
    let doomed = library.join("gone.mp4");
    std::fs::write(&doomed, b"mp4").expect("doomed clip");

    let project = dir.join("fresh.json");
    let mut shell = Shell::with_browser_root(
        ScriptedPrompts {
            new_project_path: Some(project),
            ..ScriptedPrompts::default()
        },
        library,
    );
    let pressed = press(iced_test::simulator(view(&shell)), view::NEW_PROJECT);
    drain(&mut shell, pressed);

    // 走査後に消える(移動・削除)。
    std::fs::remove_file(&doomed).expect("消す");

    let messages = double_click(iced_test::simulator(view(&shell)), "gone.mp4");
    drain(&mut shell, messages);

    assert_eq!(
        admissions(&shell),
        Vec::<Vec<PathBuf>>::new(),
        "存在しない path を intent にしない"
    );
    assert_eq!(shell.track_item_count(), 0);
    let report = shell.latest_report().unwrap_or_default();
    assert!(
        report.contains("gone.mp4"),
        "置けなかった理由は帯が名指しで言う(黙って無視しない)。実際: {report:?}"
    );
}

// ---------------------------------------------------------------------------
// rail の中身(Project = Document の台帳 / All media = dedupe / Recent = 新しい順)
// ---------------------------------------------------------------------------

/// Project rail は **Document の asset 台帳そのもの**を出し、All media は
/// 同じ file を canonical identity で dedupe して登録状態を言う
/// (ui-interaction-language の Browser 節)。
#[test]
fn the_project_rail_shows_the_documents_assets_and_all_media_dedupes() {
    if !motolii_testkit::ffmpeg_or_skip() {
        return;
    }
    let mut shell = seated_shell("iced_browser_project_rail");

    // 座った直後の Project rail は正直に空(Q7: 幽霊 card 禁止)。
    let switched = press(
        iced_test::simulator(view(&shell)),
        browser_pane::RAIL_PROJECT,
    );
    drain(&mut shell, switched);
    assert!(
        shell.browser_cards().is_empty(),
        "空 project に card を発明しない"
    );
    let mut empty = iced_test::simulator(view(&shell));
    assert!(
        empty.find(browser_pane::EMPTY_PROJECT).is_ok(),
        "空状態は次の一手つきで言う(Q7)"
    );
    drop(empty);

    // All media から1本置く。
    let back = press(iced_test::simulator(view(&shell)), browser_pane::RAIL_ALL);
    drain(&mut shell, back);
    let placed = double_click(iced_test::simulator(view(&shell)), "starter-clip.mp4");
    drain(&mut shell, placed);
    assert_eq!(shell.track_item_count(), 1, "前提: 1本置けている");

    // Project rail に、置いた asset が実名で立つ。
    let switched = press(
        iced_test::simulator(view(&shell)),
        browser_pane::RAIL_PROJECT,
    );
    drain(&mut shell, switched);
    let names: Vec<String> = shell
        .browser_cards()
        .into_iter()
        .map(|card| card.name)
        .collect();
    assert_eq!(
        names,
        vec!["starter-clip.mp4".to_owned()],
        "Project rail は Document の asset 台帳を映す(それ以外を混ぜない)"
    );
    let mut ui = iced_test::simulator(view(&shell));
    assert!(
        ui.find("starter-clip.mp4").is_ok(),
        "Project rail の card が絵として立っていない"
    );
    drop(ui);

    // All media は同じ file を2枚にしない。登録状態は meta が言う。
    let back = press(iced_test::simulator(view(&shell)), browser_pane::RAIL_ALL);
    drain(&mut shell, back);
    let cards = shell.browser_cards();
    let clips: Vec<_> = cards
        .iter()
        .filter(|card| card.name == "starter-clip.mp4")
        .collect();
    assert_eq!(clips.len(), 1, "同じ file が dedupe されず2枚立っている");
    assert!(
        clips[0].in_project,
        "Project 登録済みであることが card に出ていない"
    );
    assert!(
        clips[0].meta.contains("in project"),
        "登録状態は結果上へ示す(ui-interaction-language)。実際: {:?}",
        clips[0].meta
    );
    assert!(
        cards
            .iter()
            .filter(|card| card.name == "starter-still.png")
            .all(|card| !card.in_project),
        "置いていない file まで登録済みを名乗ってはならない"
    );
}

/// Recent rail は取り込みの新しい順(最新が先頭)。
#[test]
fn the_recent_rail_lists_the_newest_admission_first() {
    if !motolii_testkit::ffmpeg_or_skip() {
        return;
    }
    let mut shell = seated_shell("iced_browser_recent");

    let placed = double_click(iced_test::simulator(view(&shell)), "starter-clip.mp4");
    drain(&mut shell, placed);
    let placed = double_click(iced_test::simulator(view(&shell)), "starter-still.png");
    drain(&mut shell, placed);

    let switched = press(
        iced_test::simulator(view(&shell)),
        browser_pane::RAIL_RECENT,
    );
    drain(&mut shell, switched);
    let names: Vec<String> = shell
        .browser_cards()
        .into_iter()
        .map(|card| card.name)
        .collect();
    assert_eq!(
        names,
        vec![
            "starter-still.png".to_owned(),
            "starter-clip.mp4".to_owned()
        ],
        "Recent は新しい順(AssetId の降順 = 取り込み順の逆)"
    );
}

/// 空の登録 folder でも幽霊 card を出さず、folder を名指しして空と言う(Q7)。
#[test]
fn an_empty_library_is_honestly_empty() {
    let dir = motolii_testkit::tmp_dir("iced_browser_empty_lib");
    let library = dir.join("empty-library");
    std::fs::create_dir_all(&library).expect("library dir");
    let project = dir.join("fresh.json");
    let mut shell = Shell::with_browser_root(
        ScriptedPrompts {
            new_project_path: Some(project),
            ..ScriptedPrompts::default()
        },
        library,
    );
    let pressed = press(iced_test::simulator(view(&shell)), view::NEW_PROJECT);
    drain(&mut shell, pressed);

    assert!(
        shell.browser_cards().is_empty(),
        "空 folder に card を発明しない"
    );
    let empty_line = browser_pane::empty_all(&shell.browser().library_root_name());
    let mut ui = iced_test::simulator(view(&shell));
    assert!(
        ui.find(empty_line.as_str()).is_ok(),
        "空状態が folder を名指しして言われていない: {empty_line:?}"
    );
}

// ---------------------------------------------------------------------------
// OS ドロップとの分担(panel は受け皿の**表示だけ**。取り込みは殻の AdmitPaths)
// ---------------------------------------------------------------------------

/// 掴んだファイルが窓の上に居るあいだ panel の受け皿表示が点き、離れると消える。
/// そのままドロップした場合も**取り込みは殻の `AdmitPaths` のまま**(panel が奪わない)。
#[test]
fn a_hovering_file_lights_the_drop_target_without_stealing_the_drop() {
    let mut shell = seated_shell("iced_browser_hover");
    let clip = starter_media_dir().join("starter-clip.mp4");

    // 来た: 表示が点く。
    let hovered = feed(iced_test::simulator(view(&shell)), [file_hovered(&clip)]);
    assert_eq!(
        hovered,
        vec![Message::BrowserDropHover(true)],
        "hover は受け皿表示の Message になる(1回だけ)"
    );
    drain(&mut shell, hovered);
    assert!(shell.browser().drop_hover());
    let mut lit = iced_test::simulator(view(&shell));
    assert!(
        lit.find(browser_pane::DROP_TARGET).is_ok(),
        "受け皿表示が点いていない"
    );
    drop(lit);

    // 離れた: 表示が消える。
    let left = feed(iced_test::simulator(view(&shell)), [files_hovered_left()]);
    assert_eq!(left, vec![Message::BrowserDropHover(false)]);
    drain(&mut shell, left);
    assert!(!shell.browser().drop_hover());
    let mut dark = iced_test::simulator(view(&shell));
    assert!(
        dark.find(browser_pane::DROP_TARGET).is_err(),
        "離れたのに受け皿表示が残っている"
    );
    drop(dark);

    // hover → drop の一続き: 表示は消え、取り込みは従来どおり殻へ落ちる。
    // (media でない file なら ffmpeg 無しでも intent と帯まで観察できる。)
    let dir = motolii_testkit::tmp_dir("iced_browser_hover_drop");
    let notes = dir.join("notes.txt");
    std::fs::write(&notes, b"not media").expect("メモ");
    let dropped = feed(
        iced_test::simulator(view(&shell)),
        [file_hovered(&notes), file_dropped(&notes), redraw()],
    );
    assert_eq!(
        dropped,
        vec![
            Message::BrowserDropHover(true),
            Message::BrowserDropHover(false),
            Message::FilesDropped(vec![notes.clone()]),
        ],
        "drop は受け皿表示を畳み、取り込みは窓ぜんたいの FilesDropped のまま"
    );
    drain(&mut shell, dropped);
    assert_eq!(
        admissions(&shell),
        vec![vec![notes]],
        "panel が居ても OS ドロップの経路(AdmitPaths)は奪われない"
    );
}

// ---------------------------------------------------------------------------
// 検索欄(browser-revise レーンの追加、html:25 の移植)
// ---------------------------------------------------------------------------

/// 検索欄に打つと結果が絞り込まれ、一致しなくなった card は消える。
/// 値は pane 内の表示切替に留まり Document には触れない
/// (rail 切替と同じ分担 — `choosing_a_rail_switches_the_pane_without_touching_the_journal`)。
#[test]
fn typing_in_the_search_field_narrows_the_results() {
    let mut shell = seated_shell("iced_browser_search");
    let before = shell.intent_count();

    let ui = iced_test::simulator(view(&shell));
    let typed = common::type_into(ui, browser_pane::SEARCH_PLACEHOLDER, "clip");
    assert!(
        typed.iter().any(|message| matches!(
            message,
            Message::BrowserQueryChanged(query) if query == "clip"
        )),
        "打鍵は BrowserQueryChanged になる(最終値が打った文字列)。実際: {typed:?}"
    );
    drain(&mut shell, typed);

    assert_eq!(shell.browser().query(), "clip");
    let names: Vec<String> = shell
        .browser_cards()
        .into_iter()
        .map(|card| card.name)
        .collect();
    assert_eq!(
        names,
        vec!["starter-clip.mp4".to_owned()],
        "'clip' に一致しない card(starter-mark.svg / starter-still.png / starter-tone.wav)は消える"
    );
    assert_eq!(
        shell.intent_count(),
        before,
        "検索は Document に触れない = journal に載らない"
    );

    // 絵の上でも、一致しない card は消えている(選択も報酬も裏で嘘をつかない)。
    let mut ui = iced_test::simulator(view(&shell));
    assert!(ui.find("starter-clip.mp4").is_ok());
    assert!(
        ui.find("starter-mark.svg").is_err(),
        "検索に一致しない card が絵に残っている"
    );
}

/// 何も一致しない検索は幽霊 card を出さず、正直な空状態を言う(Q7)。
#[test]
fn a_search_with_no_matches_is_honestly_empty() {
    let mut shell = seated_shell("iced_browser_search_empty");

    let ui = iced_test::simulator(view(&shell));
    let typed = common::type_into(ui, browser_pane::SEARCH_PLACEHOLDER, "no-such-clip");
    drain(&mut shell, typed);

    assert!(shell.browser_cards().is_empty());
    let mut ui = iced_test::simulator(view(&shell));
    assert!(
        ui.find(browser_pane::EMPTY_FILTERED).is_ok(),
        "0件になった検索の空状態が正直に言われていない"
    );
}

// ---------------------------------------------------------------------------
// filter chip(browser-revise レーンの追加、html:103 の移植)
// ---------------------------------------------------------------------------

/// kind chip(Video/Image/Audio)を押すとその kind だけが残り、Clear で戻る。
/// 選び直しはトグル(押し直すと解除、html:339-343 と同じ)。
#[test]
fn a_kind_chip_narrows_to_that_kind_and_clear_resets_it() {
    let mut shell = seated_shell("iced_browser_chip");

    let pressed = press(iced_test::simulator(view(&shell)), browser_pane::CHIP_IMAGE);
    assert_eq!(
        pressed,
        vec![Message::BrowserKindFilterChosen("image")],
        "chip のクリックは kind filter の Message になる"
    );
    drain(&mut shell, pressed);
    assert_eq!(shell.browser().kind_filter(), Some("image"));

    let kinds: Vec<&'static str> = shell
        .browser_cards()
        .into_iter()
        .map(|card| card.kind)
        .collect();
    assert!(
        kinds.iter().all(|kind| *kind == "image"),
        "Image chip の後に image 以外の card が残っている: {kinds:?}"
    );
    assert!(
        kinds.len() >= 2,
        "starter kit の image は2本(svg/png)残るはず"
    );

    // 押し直すと解除(トグル)。
    let unpressed = press(iced_test::simulator(view(&shell)), browser_pane::CHIP_IMAGE);
    drain(&mut shell, unpressed);
    assert_eq!(
        shell.browser().kind_filter(),
        None,
        "押し直しは解除になっていない"
    );

    // Clear は kind chip と検索語を両方戻す(html:344-350)。
    let ui = iced_test::simulator(view(&shell));
    let typed = common::type_into(ui, browser_pane::SEARCH_PLACEHOLDER, "clip");
    drain(&mut shell, typed);
    let pressed = press(iced_test::simulator(view(&shell)), browser_pane::CHIP_VIDEO);
    drain(&mut shell, pressed);
    assert_eq!(shell.browser().kind_filter(), Some("video"));
    assert_eq!(shell.browser().query(), "clip");

    let cleared = press(
        iced_test::simulator(view(&shell)),
        browser_pane::CLEAR_FILTER,
    );
    assert_eq!(cleared, vec![Message::BrowserFiltersCleared]);
    drain(&mut shell, cleared);
    assert_eq!(
        shell.browser().kind_filter(),
        None,
        "Clear が kind filter を戻していない"
    );
    assert_eq!(shell.browser().query(), "", "Clear が検索語を戻していない");
    assert_eq!(
        shell.browser_cards().len(),
        4,
        "Clear の後は starter kit の4本全部が戻る"
    );
}

/// filter shelf は Filters ボタンで開閉できる(既定は開)。
/// `docs/ui-interaction-language.md`:122 の
/// 「常時同時表示しない・利用者が開閉できる」を満たす。
#[test]
fn the_filters_toggle_opens_and_closes_the_shelf() {
    let mut shell = seated_shell("iced_browser_shelf_toggle");
    assert!(
        shell.browser().shelf_open(),
        "既定は開(html の初期状態と同じ)"
    );

    let mut ui = iced_test::simulator(view(&shell));
    assert!(
        ui.find(browser_pane::CHIP_VIDEO).is_ok(),
        "既定で chip が見えていない"
    );
    drop(ui);

    let toggled = press(
        iced_test::simulator(view(&shell)),
        browser_pane::FILTERS_TOGGLE,
    );
    assert_eq!(toggled, vec![Message::BrowserFilterShelfToggled]);
    drain(&mut shell, toggled);
    assert!(!shell.browser().shelf_open());

    let mut ui = iced_test::simulator(view(&shell));
    assert!(
        ui.find(browser_pane::CHIP_VIDEO).is_err(),
        "閉じたのに chip が絵に残っている"
    );
    drop(ui);

    let toggled = press(
        iced_test::simulator(view(&shell)),
        browser_pane::FILTERS_TOGGLE,
    );
    drain(&mut shell, toggled);
    assert!(shell.browser().shelf_open(), "もう一度押すと開き直る");
}

// ---------------------------------------------------------------------------
// 表示モード(browser-revise レーンの追加、html:94-97 の移植)
// ---------------------------------------------------------------------------

/// view mode ボタンで Thumbnails/Grid/List を切り替えられる。既定は Grid
/// (html:96 `aria-pressed="true"`)。列数(Q3 の見た目報酬)は
/// `browser_pane::dims::columns` と同じ数を返す前提。
#[test]
fn the_view_mode_buttons_switch_between_thumbnails_grid_and_list() {
    let mut shell = seated_shell("iced_browser_view_mode");
    assert_eq!(shell.browser().view(), BrowserViewMode::Grid, "既定は Grid");

    let pressed = press(iced_test::simulator(view(&shell)), browser_pane::VIEW_LIST);
    assert_eq!(
        pressed,
        vec![Message::BrowserViewChosen(BrowserViewMode::List)]
    );
    drain(&mut shell, pressed);
    assert_eq!(shell.browser().view(), BrowserViewMode::List);

    let pressed = press(
        iced_test::simulator(view(&shell)),
        browser_pane::VIEW_THUMBNAILS,
    );
    drain(&mut shell, pressed);
    assert_eq!(shell.browser().view(), BrowserViewMode::Thumbnails);
    // css:272 `[data-view="thumbnails"] .cardCopy { display: none }` — この
    // view では名前が絵に出ない(意図どおり)。model 側は壊れていない。
    assert_eq!(
        shell.browser_cards().len(),
        4,
        "切り替えても card の中身は壊れない"
    );

    // Grid(css の cardCopy が出る view)へ戻すと、絵の上でも名前が見える。
    let pressed = press(iced_test::simulator(view(&shell)), browser_pane::VIEW_GRID);
    drain(&mut shell, pressed);
    assert_eq!(shell.browser().view(), BrowserViewMode::Grid);
    let mut ui = iced_test::simulator(view(&shell));
    assert!(ui.find("starter-clip.mp4").is_ok());
}
