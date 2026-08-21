//! owns: TL-arch(`docs/reviews/2026-08-22-timeline-canvas-widget-survey.md`)
//! §5「性能の実測値が無い」(EVIDENCE_GAP #1)を埋める器具。widget タイムライン
//! (§2.3 `pin` + `Stack` + 自作 translate container)が Phase 2 の判定線を
//! 満たすかを、4条件(静止/パン=カメラ/パン=素朴再構築/zoom=x-only再配置)×
//! 3規模(100/500/1000 bar)で測る。
//!
//! **測るのは CPU 側(diff・layout・tree構築・`draw()` の primitive 記録)だけ**。
//! headless renderer の `draw()` は primitive を記録するところまでで、実際の
//! GPU submit/present は測れない(`iced_test` の既知の限界)。詳細は
//! `tests/r4.rs` 冒頭のコメントと RETURN の EVIDENCE_GAP を参照。

use std::cell::Cell;
use std::rc::Rc;

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self as advanced_renderer, Renderer as _};
use iced::advanced::widget::{self, Widget};
use iced::advanced::{mouse, Shell};
use iced::widget::{container, mouse_area, pin, text, Stack};
use iced::{Background, Color, Element, Event, Length, Point, Rectangle, Size, Vector};

// ---------------------------------------------------------------------------
// 規模・寸法の定数(条件(a)〜(d)で共有)
// ---------------------------------------------------------------------------

/// 3規模。100/500/1000 bar(≒layer)— タスク仕様どおり。
pub const SCALES: [usize; 3] = [100, 500, 1000];

/// 120Hz 予算(supervisor 判定線、条件(b)専用)。マイクロ秒。
pub const FRAME_BUDGET_US: u128 = 8_300;

/// 1 layer = 1 行。行の高さ(canvas.rs の row_height 相当のプレースホルダ値、
/// 比率の正本ではなく probe 専用の合成値)。
pub const ROW_HEIGHT: f32 = 24.0;
pub const BAR_HEIGHT: f32 = 18.0;

/// clip 1本の長さ(フレーム単位、全 bar 共通の合成値)。
pub const BAR_DURATION_FRAMES: f32 = 300.0;

/// timeline 全体の長さ(フレーム単位)。bar は `TOTAL_FRAMES / n` 間隔で
/// 時間軸上に均等配置する — n が増えるほど間隔は詰まるが、常に同じ
/// 時間レンジへ広がる(合成データ、実 Document の値とは無関係)。
pub const TOTAL_FRAMES: f32 = 30_000.0;

/// zoom=1.0 における px/frame の基準値。
const BASE_PX_PER_FRAME: f32 = 40.0 / BAR_DURATION_FRAMES;

/// headless simulator の既定サイズ(1024x768)より広く取り、1000 bar 規模でも
/// 複数行が同時に画面内へ収まる程度の合成 viewport。
pub const VIEWPORT: Size = Size::new(1600.0, 900.0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message {
    Noop,
}

fn px_per_frame(zoom: f32) -> f32 {
    BASE_PX_PER_FRAME * zoom
}

/// bar `index`(0..n)の開始フレーム。時間軸上に均等分散させる合成配置。
fn start_frame(index: usize, n: usize) -> f32 {
    index as f32 * (TOTAL_FRAMES / n as f32)
}

fn bar_color(index: usize) -> Color {
    // 視覚精度は無用(非対話 probe)。RGB を回してテクスチャキャッシュに
    // 頼らせない(r1 の `document_with` と同じ狙い)だけの合成色。
    Color::from_rgb8(
        (80 + (index * 7) % 160) as u8,
        (60 + (index * 13) % 180) as u8,
        200,
    )
}

/// bar 1本ぶんの widget 構成。`mouse_area(container(text))` — TL-arch §3 が
/// 離散候補(強)と分類した bar 本体の最小構成(絵1枚+当たり判定1枚+ラベル1枚)
/// を模す。
pub fn bar(index: usize, width: f32) -> Element<'static, Message> {
    mouse_area(
        container(text(format!("c{index}")).size(10))
            .width(Length::Fixed(width.max(1.0)))
            .height(Length::Fixed(BAR_HEIGHT))
            .style(move |_theme| {
                container::Style::default()
                    .background(Background::Color(bar_color(index)))
            }),
    )
    .on_press(Message::Noop)
    .into()
}

/// (a)静止・(c)パン=素朴再構築・(d)zoom=x-only再配置で使う純関数。
/// `n` 本の bar を `pin` で絶対配置した `Stack` を**毎回組み立て直す**。
/// `pan_px` は x から素朴に引くだけ(view 再構築でパンを表現する条件(c)用)。
/// `zoom` は x・幅の両方に乗算する(y=row位置は触らない — 条件(d)の定義)。
pub fn stacked_bars(n: usize, zoom: f32, pan_px: f32) -> Element<'static, Message> {
    let ppf = px_per_frame(zoom);
    let bar_width = BAR_DURATION_FRAMES * ppf;

    let mut layer = Stack::with_capacity(n);
    for i in 0..n {
        let x = start_frame(i, n) * ppf - pan_px;
        let y = i as f32 * ROW_HEIGHT;
        layer = layer.push(pin(bar(i, bar_width)).x(x).y(y));
    }
    layer.into()
}

/// 条件(b)の下地: pan=0 で固定 layout した Stack。パンは `TranslateLane` が
/// draw 時の translation でだけ表現する(layout はここで固まったまま動かない)。
pub fn stacked_bars_fixed(n: usize, zoom: f32) -> Element<'static, Message> {
    stacked_bars(n, zoom, 0.0)
}

// ---------------------------------------------------------------------------
// TranslateLane — scrollable(§2.6)と同じ手口の自作 translate container
// ---------------------------------------------------------------------------

/// `scrollable` の draw 実測(`widget/src/scrollable.rs` 行1067-1069
/// `renderer.with_translation(...)`)と同じ手口を最小構成で切り出した
/// widget。**layout はオフセットを一切参照しない**(content の layout は
/// 一度きり、オフセットは `Rc<Cell<f32>>` に外部から書き込まれ、
/// `draw()`/`update()`/`mouse_interaction()` がその都度読むだけ)。
///
/// `layout_calls` は「パン中に layout が再度走っていないこと」を機械的に
/// 証明するための計測フック(本番コードには不要、probe 専用)。
pub struct TranslateLane<'a, Message> {
    content: Element<'a, Message, iced::Theme, iced::Renderer>,
    offset_x: Rc<Cell<f32>>,
    layout_calls: Rc<Cell<u32>>,
}

impl<'a, Message> TranslateLane<'a, Message> {
    pub fn new(
        content: impl Into<Element<'a, Message, iced::Theme, iced::Renderer>>,
        offset_x: Rc<Cell<f32>>,
        layout_calls: Rc<Cell<u32>>,
    ) -> Self {
        Self {
            content: content.into(),
            offset_x,
            layout_calls,
        }
    }
}

/// 画面座標→content座標への写像。draw は content を `-offset` だけ動かして
/// 描くので、カーソルは content 座標系に合わせて `+offset` する
/// (`scrollable.rs` 行1046 `cursor_position + translation` と同じ向き)。
fn shift_cursor(cursor: mouse::Cursor, offset_x: f32) -> mouse::Cursor {
    match cursor {
        mouse::Cursor::Available(p) => mouse::Cursor::Available(Point::new(p.x + offset_x, p.y)),
        other => other,
    }
}

impl<'a, Message> Widget<Message, iced::Theme, iced::Renderer> for TranslateLane<'a, Message> {
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn tag(&self) -> widget::tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> widget::tree::State {
        self.content.as_widget().state()
    }

    fn diff(&mut self, tree: &mut widget::Tree) {
        self.content.as_widget_mut().diff(tree);
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.layout_calls.set(self.layout_calls.get() + 1);
        // 重要: `offset_x` はここで一切読まない — layout は content の
        // 「地の座標」を一度だけ確定する。パンは draw 側の translation。
        self.content.as_widget_mut().layout(tree, renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(tree, layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let shifted = shift_cursor(cursor, self.offset_x.get());
        self.content
            .as_widget_mut()
            .update(tree, event, layout, shifted, renderer, shell, viewport);
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let shifted = shift_cursor(cursor, self.offset_x.get());
        self.content
            .as_widget()
            .mouse_interaction(tree, layout, shifted, viewport, renderer)
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &advanced_renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let offset_x = self.offset_x.get();
        let shifted_cursor = shift_cursor(cursor, offset_x);

        renderer.with_translation(Vector::new(-offset_x, 0.0), |renderer| {
            self.content
                .as_widget()
                .draw(tree, renderer, theme, style, layout, shifted_cursor, viewport);
        });
    }
}

impl<'a, Message: 'a> From<TranslateLane<'a, Message>>
    for Element<'a, Message, iced::Theme, iced::Renderer>
{
    fn from(widget: TranslateLane<'a, Message>) -> Self {
        Element::new(widget)
    }
}
