//! 発注の落ちるテスト (e): `iced_test::Simulator` で、TaffyBox が並べた子 button へ
//! click が**座標どおり**届く — layout が本物である(絵合わせでなく入力の当たり判定
//! まで一致している)ことの証明。器具は motolii-menubar の oracle と同じ流儀
//! (text selector で find/click し `into_messages` で照合)。

use iced_core::{Element, Size};
use iced_test::Simulator;
use iced_widget::{button, text};
use motolii_taffy::{style_from_css_decl, TaffyBox};

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Left,
    Right,
}

const WINDOW: Size = Size {
    width: 800.0,
    height: 600.0,
};

/// 左右両端へ space-between で振り分けた 2 button。
fn view<'a>() -> Element<'a, Message, iced::Theme, iced::Renderer> {
    let root = style_from_css_decl(
        "display:flex; justify-content:space-between; align-items:flex-start; \
         padding:10px; width:800px; height:600px",
    )
    .expect("root css");
    let cell = style_from_css_decl("").expect("子は intrinsic サイズ(button まかせ)");

    TaffyBox::new(root)
        .push(cell.clone(), button(text("Left")).on_press(Message::Left))
        .push(cell, button(text("Right")).on_press(Message::Right))
        .into()
}

fn simulator<'a>() -> Simulator<'a, Message> {
    Simulator::with_size(iced_core::Settings::default(), WINDOW, view())
}

/// click が taffy の解いた座標へ届き、期待した message が発火する。
#[test]
fn clicks_reach_children_laid_out_by_taffy() {
    let mut ui = simulator();

    ui.click("Right").expect("Right button が click できない");
    ui.click("Left").expect("Left button が click できない");

    let messages: Vec<_> = ui.into_messages().collect();
    assert_eq!(
        messages,
        vec![Message::Right, Message::Left],
        "click の座標が taffy layout とずれている(当たり判定が違う所にある)"
    );
}

/// 位置の裏取り: space-between なら Right のラベルは窓の右半分・Left は左半分に
/// 居るはず(selector の bounds は絵ではなく layout 木から来る)。
#[test]
fn space_between_places_labels_on_opposite_halves() {
    let mut ui = simulator();

    let left = ui.find("Left").expect("Left が見つからない");
    let right = ui.find("Right").expect("Right が見つからない");

    let half = WINDOW.width / 2.0;
    assert!(
        left.bounds().x < half,
        "Left が左半分に居ない: {:?}",
        left.bounds()
    );
    assert!(
        right.bounds().x > half,
        "Right が右半分に居ない: {:?}",
        right.bounds()
    );
    // padding:10px — Left の button 枠は左端 10px から始まる(text はその内側)。
    assert!(
        left.bounds().x >= 10.0,
        "padding が効いていない: {:?}",
        left.bounds()
    );
}
