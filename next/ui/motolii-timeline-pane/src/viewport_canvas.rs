//! **D-5(Timeline 縦カリング)の wrapper**(裁定「迂回よりwrapper」
//! 2026-08-18)。
//!
//! ## 探索範囲(RETURN 用の一次記録)
//!
//! iced fork(`~/.cargo/git/checkouts/iced-*/widget/src/canvas.rs`、pin rev
//! `73e686ee05efd7d1b61cfea2647186b336d9ab9c`、`next/Cargo.toml:88`)を実測。
//! - `widget/src/canvas.rs` の `impl Widget<...> for Canvas<...>`:
//!   `draw`/`update` は `_viewport`/`viewport: &Rectangle` を引数に**持って
//!   いる**(iced 本体の `Widget` トレイト自体は viewport を運んでいる)。
//!   しかし `Canvas::draw` は `_viewport` を(先頭アンダースコアの通り)
//!   一切使わず、`self.program.draw(state, renderer, theme, bounds, cursor)`
//!   へは渡していない。`Canvas::update` も同様(`viewport` 未使用)。
//!   `canvas::Program` トレイト(`widget/src/canvas/program.rs`)自体の
//!   `draw`/`update` シグネチャに `viewport` パラメータが無い —
//!   Program 経由では原理的に viewport へ到達できない。
//! - `widget/src/scrollable.rs`: `on_scroll(Viewport) -> Message` で
//!   `scrollable::Viewport::bounds()`(可視矩形)/`content_bounds()` が
//!   **取れる**ことを確認したが、`notify_scroll`/`notify_viewport` は
//!   実際に scroll 操作(wheel/scrollbar drag/auto-scroll)が起きた時にしか
//!   呼ばれない(`scrollable.rs:599,631,661,693,808,866,953,975` を実測 —
//!   すべて `Event::Mouse`/`Event::Touch` の分岐内)。ウィンドウ resize や
//!   初回 layout だけでは発火しない = 「スクロール前・resize 直後」に
//!   viewport 高さが stale なまま残る窓がある。絵を変えない(裁定218の
//!   OUTCOME)ことを最優先するなら、古い/欠けた viewport で行を落として
//!   しまう経路を持つのは危険 — 採用しない
//! - `core/src/widget.rs` の `Widget` トレイト定義そのもの: `draw`/`update`
//!   は `viewport: &Rectangle` を受け取るのが標準の形(`Canvas` が自分で
//!   握りつぶしているだけ)。つまり iced 自体に「別の到達手段」は無い —
//!   `canvas()` を経由する限り Program は viewport を知り得ない、という
//!   のが構造的事実(A-5 が残した finding どおり)。`Widget` を自分で実装
//!   すれば標準機能の範囲内で viewport が取れる、というのが今回の結論
//!
//! ## 形
//!
//! `iced::widget::canvas::Program` トレイト自体は**触らない**(iced の
//! API を再実装しない、発注書の禁止事項)。widget 層だけ薄く足す:
//! - [`ViewportProgram`]: `canvas::Program` のスーパートレイト。既定実装は
//!   `Program::draw` へそのまま委譲する(= viewport を使わない Program は
//!   このトレイトへ切り替えても**挙動が一切変わらない**、最小差分)。
//!   `TimelinePane` だけが `draw_viewport` を上書きしてカリングする
//!   (`canvas.rs::draw` へ `viewport` を1本追加で渡すだけ)。
//! - [`ViewportCanvas`]: `iced::widget::canvas::Canvas` と同形の `Widget`
//!   実装。フィールド・`tag`/`state`/`size`/`layout`/`update`/
//!   `mouse_interaction`/`From<...> for Element` は fork の `Canvas` の
//!   実装をそのまま写した(複製した箇所はどれも数行の定型 — `Widget`
//!   トレイト自体を再実装しているわけではなく、**この1つの具象型**が
//!   トレイトを実装しているだけ)。変えたのは `draw` の中で `_viewport`
//!   を握りつぶさず `program.draw_viewport(..., *viewport)` へ渡す1点のみ。
//!
//! `TimelinePane::view()` の呼び出し側は `iced::widget::canvas(self)...` を
//! `ViewportCanvas::new(self)...` に差し替えるだけ(`.width()`/`.height()`/
//! `.into()` の形は同じ、`super::TimelinePane::view` 参照)。

use iced::advanced::graphics::geometry;
use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{tree, Tree};
use iced::advanced::{mouse, renderer, Widget};
use iced::widget::canvas;
use iced::advanced::Shell;
use iced::{Element, Event, Length, Rectangle, Size, Vector};

use std::marker::PhantomData;

/// [`canvas::Program`] に viewport-aware な draw を1本足すだけの拡張トレイト。
/// 既定実装は `Program::draw` へそのまま委譲する — viewport を使わない
/// `Program` をこのトレイトへ差し替えても挙動は不変。
pub(crate) trait ViewportProgram<Message, Theme = iced::Theme, Renderer = iced::Renderer>:
    canvas::Program<Message, Theme, Renderer>
where
    Renderer: geometry::Renderer,
{
    fn draw_viewport(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        viewport: Rectangle,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let _ = viewport;
        canvas::Program::draw(self, state, renderer, theme, bounds, cursor)
    }
}

/// `iced::widget::canvas::Canvas` と同形のラッパ widget(モジュール doc 参照)。
pub(crate) struct ViewportCanvas<P, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Renderer: geometry::Renderer,
    P: ViewportProgram<Message, Theme, Renderer>,
{
    width: Length,
    height: Length,
    program: P,
    message_: PhantomData<Message>,
    theme_: PhantomData<Theme>,
    renderer_: PhantomData<Renderer>,
}

impl<P, Message, Theme, Renderer> ViewportCanvas<P, Message, Theme, Renderer>
where
    P: ViewportProgram<Message, Theme, Renderer>,
    Renderer: geometry::Renderer,
{
    const DEFAULT_SIZE: f32 = 100.0;

    pub(crate) fn new(program: P) -> Self {
        Self {
            width: Length::Fixed(Self::DEFAULT_SIZE),
            height: Length::Fixed(Self::DEFAULT_SIZE),
            program,
            message_: PhantomData,
            theme_: PhantomData,
            renderer_: PhantomData,
        }
    }

    pub(crate) fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub(crate) fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}

impl<P, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for ViewportCanvas<P, Message, Theme, Renderer>
where
    Renderer: geometry::Renderer,
    P: ViewportProgram<Message, Theme, Renderer>,
{
    fn tag(&self) -> tree::Tag {
        struct Tag<T>(T);
        tree::Tag::of::<Tag<P::State>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(P::State::default())
    }

    fn size(&self) -> Size<Length> {
        Size { width: self.width, height: self.height }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width, self.height)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        // upstream `Canvas::update` の意味を保つ部分だけを写した(hit test は
        // 絵の縦カリングと無関係 — viewport は draw 側だけで使う)。upstream
        // が持つ `last_mouse_interaction` キャッシュ(冗長 redraw を1回減らす
        // だけの最適化、`Canvas` 構造体の `&mut self` フィールド)はここでは
        // 複製しない — 無くても正しさに影響しない(悪くても余分な redraw
        // request が増えるだけ、絵は変わらない)、D-5 の目的(縦カリング)には
        // 不要な複製なので削った。
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<P::State>();

        if let Some(action) =
            canvas::Program::update(&self.program, state, event, bounds, cursor)
        {
            let (message, redraw_request, event_status) = action.into_inner();

            shell.request_redraw_at(redraw_request);

            if let Some(message) = message {
                shell.publish(message);
            }

            if event_status == iced::event::Status::Captured {
                shell.capture_event();
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<P::State>();

        canvas::Program::mouse_interaction(&self.program, state, bounds, cursor)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        if bounds.width < 1.0 || bounds.height < 1.0 {
            return;
        }

        let state = tree.state.downcast_ref::<P::State>();

        renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
            let layers = self.program.draw_viewport(state, renderer, theme, bounds, cursor, *viewport);

            for layer in layers {
                renderer.draw_geometry(layer);
            }
        });
    }
}

impl<'a, P, Message, Theme, Renderer> From<ViewportCanvas<P, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: 'a + geometry::Renderer,
    P: 'a + ViewportProgram<Message, Theme, Renderer>,
{
    fn from(value: ViewportCanvas<P, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}
