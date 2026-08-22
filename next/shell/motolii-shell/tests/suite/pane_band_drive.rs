//! pane 題帯 drive(2026-08-22 題帯レーン — 利用者実窓検分「リサイズはできるが
//! レイアウト変更ができない。ハンドルが無いせいか」への応答)。
//!
//! ## この柵が守る主張
//!
//! 1. **題帯の存在**(S6: affordance を隠さない): 全 pane の上端に pane 名入りの
//!    薄い題帯が常設される。旧実装(pane_grid 化 第1切片)の title_bar は匿名の
//!    8px `Space` で、見えないためつかめなかった。
//! 2. **帯全体が drag ハンドル**(S1: 大きい的): 題帯のラベル文字の外を press
//!    すると `DragEvent::Picked` が出る。**旧実装はここが構造的に死んでいた** —
//!    fork rev 73e686e の `TitleBar::is_over_pick_area` は title content の
//!    bounds を pick 対象から**除外**する(`title_bar.rs::is_over_pick_area`
//!    実測: `!title_layout.bounds().contains(cursor_position)`)ため、
//!    `Space::new().width(Fill)` を content にしていた旧 grip 帯は **pick 面積
//!    ゼロ = ドラッグが一切始まらない**(利用者の「レイアウト変更ができない」
//!    の直接原因)。このファイルの drag 試験は旧実装に当てると red になる —
//!    red→green の直接証拠。
//!
//! ラベル文字そのものの矩形は同じ実測理由で pick 対象外(構造的限界 — 題帯の
//! ラベル以外の全幅が的なので S1 は満たす)。

use motolii_shell::{Message, Shell};

fn shell() -> Shell {
    Shell::new().0
}

/// 既定状態(Browser 閉)の pane_grid 3枚 — Inspector/Stage/Timeline — の題帯
/// ラベルが view に実在する。Timeline/Stage は pane 本体が canvas/shader で
/// 文字を一切持たないため、このラベルは題帯だけが出せる(題帯が無ければ red)。
#[test]
fn every_default_pane_carries_a_named_title_band() {
    let shell = shell();

    for label in ["Inspector", "Stage", "Timeline"] {
        let mut ui = iced_test::simulator(shell.view());
        ui.find(label).unwrap_or_else(|error| {
            panic!("pane 題帯のラベル {label:?} が view に無い: {error:?}")
        });
    }
}

/// 題帯ラベルの正本は `pane_layout::title` — 4種すべてが名前を持つ(view の
/// closure が全 pane 一律にこの関数で帯を組むので、Stage の drag 試験
/// (下)と合わせて Browser 開状態の帯も同じコード経路で保証される —
/// `Simulator::find` は同名 text の2件目に届かない(header の "Browser"
/// トグルが先に拾われる)ため、Browser 帯の存在はこの正本関数で検分する)。
#[test]
fn every_pane_kind_has_a_title_label() {
    use motolii_shell::pane_layout::{title, PaneKind};
    assert_eq!(title(PaneKind::Browser), "Browser");
    assert_eq!(title(PaneKind::Inspector), "Inspector");
    assert_eq!(title(PaneKind::Stage), "Stage");
    assert_eq!(title(PaneKind::Timeline), "Timeline");
}

/// Settings パネル(pane_grid 外の全幅ストリップ)にも題帯が出る — pane 名の
/// 常設は5面すべて(発注書)。Settings は pane_grid の pane ではないので
/// drag ハンドルにはならない(題帯は名札のみ — grab カーソルも出ないため
/// 「掴めそうで掴めない」嘘はつかない)。
#[test]
fn the_settings_panel_carries_a_title_band_when_open() {
    let mut shell = shell();
    let _ = shell.update(Message::Settings(
        motolii_shell::settings_pane::Message::ToggleSettingsPanel,
    ));

    let mut ui = iced_test::simulator(shell.view());
    ui.find("Settings")
        .expect("Settings パネルの題帯ラベルが view に無い");
}

/// **本命(red→green)**: 題帯のラベルの右隣(帯の空き部分)を press すると
/// `PaneClicked` に加えて `PaneDragged(DragEvent::Picked)` が出る — 帯が実際に
/// drag ハンドルとして配線されている直接証拠。旧実装(匿名 Space 全幅)は
/// pick 面積ゼロでこの試験が red(モジュール冒頭 doc 参照)。
#[test]
fn pressing_the_stage_band_beside_its_label_picks_the_pane_for_dragging() {
    let shell = shell();
    let mut ui = iced_test::simulator(shell.view());

    let label = ui.find("Stage").expect("Stage 題帯のラベルが無い");
    let bounds = label.bounds();

    // ラベル矩形のすぐ右 — 題帯の中・ラベルの外(pick area)。
    let point = iced::Point::new(bounds.x + bounds.width + bounds.height, bounds.center_y());
    ui.point_at(point);
    let _ = ui.simulate([
        iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
        iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)),
    ]);

    let messages: Vec<Message> = ui.into_messages().collect();
    assert!(
        messages
            .iter()
            .any(|m| matches!(m, Message::PaneClicked(_))),
        "題帯 press で PaneClicked が出ていない: {messages:?}"
    );
    assert!(
        messages.iter().any(|m| matches!(
            m,
            Message::PaneDragged(iced::widget::pane_grid::DragEvent::Picked { .. })
        )),
        "題帯 press で DragEvent::Picked が出ていない — 帯が drag ハンドルに\
         なっていない(旧 grip 帯の pick 面積ゼロと同じ壊れ方): {messages:?}"
    );
}

/// 題帯の寸法は Rust 定数の直書きではなく tokens 経由(利用者裁定 2026-08-22
/// 「デザイン値の外出し徹底」): `pane_header_height` が dimensions.json 由来で
/// 存在し、ui_scale の唯一の乗算点(`Dimensions::scaled`)を通る。
#[test]
fn the_pane_header_height_token_exists_and_scales() {
    let dims = motolii_shell::tokens::Dimensions::default();
    let ratio = dims.micro_text / dims.pane_header_height;
    assert!(
        (0.37..=0.47).contains(&ratio),
        "題帯の文字/帯 比率が裁定168 の帯(0.42±0.05)を外れている: {ratio}"
    );

    let scaled = dims.scaled(1.5);
    assert!(
        (scaled.pane_header_height - dims.pane_header_height * 1.5).abs() < 0.01,
        "pane_header_height に ui_scale が掛かっていない"
    );
}
