//! 裁定168 施工(利用者実窓較正で発覚した違反(B)の根治): Inspector の値セル
//! (X/Y/Z)が隣接セルへ文字ごと重ならないこと。実窓実測「960.000540.000」
//! (Position 行の X/Y セルの数字がゼロ gap で密着して読める)の再現・回帰柵。
//!
//! ## この柵の限界(正直に書く — `inspector_pixel_fence.rs` 冒頭の「対象外」
//! 節と同じ姿勢)
//!
//! `iced_test`/`iced_selector` の `Target::Text` は widget **layout** 木を
//! 歩いて得る(`iced_core::widget::Operation::text` 経由)。`text(..)` に
//! 明示 `.width()` が無い(`Length::Shrink`)場合、`iced_core::widget::text::
//! layout` は `layout::sized` で **layout 上のサイズだけ**を親の `Limits::max`
//! (= このコードでは値セルの箱幅 38px)へ丸め込む(実測: `characterization_
//! tests::a_shrink_text_widget_inside_a_fixed_width_container_reports_layout_
//! bounds_clamped_to_the_container_width` 参照)。**しかし実描画(`draw()`)は
//! `paragraph.min_bounds()` という自然サイズを基準に文字を配置する** ——
//! layout 上は重ならなくても実際のペイントは箱の外へ滲み得る、というのが
//! 「箱の gap は0でないのに文字は重なる」の実体。
//!
//! さらに `Container::clip(true)`(この施工で追加した根治策)は
//! `iced_widget::container::Container::draw` の `clipped_viewport` 計算にしか
//! 効かない(実測 — `iced_widget-0.14.2/src/container.rs` 351/365行)。
//! `Container::operate`(`iced_test`/`iced_selector` が読む経路)は
//! `self.clip` を一切参照せず、`viewport`/`visible_bounds` は
//! `Scrollable` の traverse でしか更新されない(実測 —
//! `iced_selector-0.14.0/src/find.rs` の `container`/`scrollable`/`text` 各
//! callback を比較)。**つまり `clip(true)` の効果は `bounds()` にも
//! `visible_bounds()` にも一切現れない** — この柵ではペイント時の越境防止を
//! 直接は検証できない(is/ought のギャップ、iced_test の構造的限界として
//! 正直に記録するだけに留める)。
//!
//! 検証できるのは (1) layout 上の箱の gap が意図どおりであること
//! ([`adjacent_value_cell_boxes_are_separated_by_the_expected_gap`])と、
//! (2) 自然サイズ(clip される前の真の文字幅)が箱よりどれだけ広いか
//! (`characterization_tests` — clip(true) が実際に何を切る必要があるかの
//! 実測記録)の2点。`.clip(true)` そのものの正しさは
//! `iced_widget::container::Container::draw` のソース読解(このファイル冒頭の
//! doc)で裏付ける。

use iced_test::selector::{Candidate, Target};

use motolii_inspector_pane::{
    view, AttrsProjection, ComponentSlot, RowValue, SelectionProjection, TransformRowProjection,
};
use motolii_store::LayerId;
use motolii_tokens_rs::{Colors, Dimensions};

/// `inspector_pixel_fence.rs`/`q0_fence.rs` と同じ「`find` を尽きるまで繰り返す」
/// 手口(`Simulator` に `find_all` が無い、iced_test 0.14.0 実測)。
fn collect_targets(element: iced::Element<'_, motolii_inspector_pane::Message>) -> Vec<Target> {
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

fn present_editable(axis: &'static str, value: f64) -> ComponentSlot {
    ComponentSlot {
        axis,
        present: true,
        value,
        editable: true,
        field: None, // この柵は表示 bounds だけを見るので field は使わない。
    }
}

/// 実窓較正の再現値(Position X=960 / Y=540 / Z=0)— 3桁小数(decimals=3)で
/// `format_number` を通すと `"960.000"`/`"540.000"`(いずれも7文字)になる、
/// 実測で報告された文字列そのもの。
fn position_row_matching_the_field_report() -> TransformRowProjection {
    TransformRowProjection {
        label: "Position",
        value: RowValue::Vector([
            present_editable("X", 960.0),
            present_editable("Y", 540.0),
            present_editable("Z", 0.0),
        ]),
        decimals: 3,
    }
}

fn selection_with_position_row() -> SelectionProjection {
    SelectionProjection {
        layer: LayerId(0),
        kind: "solid",
        transform: vec![position_row_matching_the_field_report()],
        attrs: AttrsProjection {
            name: "Layer".to_owned(),
            hidden: false,
            blend_mode: "Normal".to_owned(),
            speed_percent: 100.0,
        },
    }
}

fn number_text_bounds(targets: &[Target], content: &str) -> iced::Rectangle {
    targets
        .iter()
        .filter(|t| matches!(t, Target::Text { content: c, .. } if c == content))
        .map(|t| t.bounds())
        .next()
        .unwrap_or_else(|| panic!("値セルの文字 {content:?} が Target::Text として見つからない"))
}

/// 裁定167 下段(容器側の兄弟 gap)= `0.075 × 行高` の px 最近傍丸め。
/// `lib.rs::sibling_gap_px` と同じ式 — テスト側は正本の値をそのまま検算する
/// のではなく、「柵として要求する下限」を独立に計算する(実装がこの式から
/// ズレたらここが red になる)。
fn expected_min_gap_px(row_height: f32) -> f32 {
    (row_height * 0.075).round()
}

/// **layout レベルの回帰柵**(ファイル冒頭「この柵の限界」参照 — paint 越境
/// そのものは検証できない、layout 上の箱の gap が意図どおりであることだけを
/// 見る)。値セル(X/Y)の `Target::Text` bounds(= 箱幅へ丸め込まれた
/// layout サイズ)が水平方向に重ならず、隣接セル間の gap が裁定167 下段の
/// 丸め後 px 以上であること。
#[test]
fn adjacent_value_cell_boxes_are_separated_by_the_expected_gap() {
    let dims = Dimensions::default();
    let colors = Colors::default();
    let selection = selection_with_position_row();

    let element = view(Some(&selection), None, None, dims, colors);
    let targets = collect_targets(element);

    let x = number_text_bounds(&targets, "960.000");
    let y = number_text_bounds(&targets, "540.000");

    // Position 行は X が先(左)・Y が後(右)に並ぶ(`transform_row` の
    // `RowValue::Vector` 順序どおり)。
    assert!(x.x < y.x, "X/Y の描画順が想定と違う(X.x={} Y.x={})", x.x, y.x);

    let gap = y.x - (x.x + x.width);
    let min_gap = expected_min_gap_px(dims.inspector_row_height);

    assert!(
        gap >= min_gap - 0.5, // 実フォント測定の丸め誤差ぶんだけ緩める(±0.5px)
        "X/Y の値セルの箱(layout bounds)が近すぎる — \
         X bounds={x:?}, Y bounds={y:?}, 実測 gap={gap}px, 下限={min_gap}px"
    );
}

/// **特性計測**(oracle ではなく実測記録): `.clip(true)` が実際に何を切る
/// 必要があるかを数値で残す。「960.000」(値セルの実測フォント・サイズ)の
/// 自然幅(制約なし)は既定 dims の箱幅(38px)より広い — これが「箱の gap は
/// 0でないのに文字は重なる」の直接証拠。`iced_widget::text::layout` が
/// `Length::Shrink` な `text(..)` を `Length::Fixed(38)` の `container` へ
/// 収めると、報告される `Target::Text` の layout bounds は箱幅ちょうどへ
/// 丸め込まれる(自然幅とは別物) — この2つの値の差が、`clip(true)` 無しでは
/// 実描画時に隣へ滲む量。
#[test]
fn a_shrink_text_widget_inside_a_fixed_width_container_reports_layout_bounds_clamped_to_the_container_width(
) {
    use iced::widget::{container, text};
    use iced::Length;

    let dims = Dimensions::default();
    let content = "960.000";

    let natural: iced::Element<'static, motolii_inspector_pane::Message> =
        text(content.to_owned()).size(dims.body_text).into();
    let natural_targets = collect_targets(natural);
    let natural_width = natural_targets
        .iter()
        .find(|t| matches!(t, Target::Text { .. }))
        .expect("自然幅の Text candidate が見つからない")
        .bounds()
        .width;

    let boxed: iced::Element<'static, motolii_inspector_pane::Message> = container(
        text(content.to_owned())
            .size(dims.body_text)
            .align_x(iced::alignment::Horizontal::Center),
    )
    .width(Length::Fixed(dims.inspector_value_width))
    .into();
    let boxed_targets = collect_targets(boxed);
    let boxed_width = boxed_targets
        .iter()
        .find(|t| matches!(t, Target::Text { .. }))
        .expect("箱詰め後の Text candidate が見つからない")
        .bounds()
        .width;

    assert_eq!(
        boxed_width, dims.inspector_value_width,
        "箱詰め後の layout bounds が箱幅そのものに丸め込まれていない(想定どおりの \
         layout::sized クランプが起きていない — 上流 iced の挙動が変わった疑い)"
    );
    assert!(
        natural_width > boxed_width,
        "自然幅({natural_width}px)が箱幅({boxed_width}px)を超えていない — \
         この特性計測の前提(clip が要る場面がある)が崩れている。fixture を \
         見直すこと"
    );
}
