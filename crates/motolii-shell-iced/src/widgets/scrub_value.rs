//! 数値スクラブ — AE / AM 文法の主食。**ドラッグで増減・ダブルクリックでテキスト入力**。
//!
//! egui shell では `egui::DragValue`(`inspector_panel/mod.rs` の
//! `draw_value_field`)が担っていた席。イベント語彙は発注 capsule で固定:
//! gesture は必ず [`ScrubEvent::Started`] で開き、[`ScrubEvent::Committed`]
//! (release / Enter = 確定)か [`ScrubEvent::Cancelled`](Esc = 復元)で閉じる。
//! 途中の値は [`ScrubEvent::Changed`] が同 tick で運ぶ(Q3 / B3)。
//!
//! ## 値の所有(Q5)
//!
//! 値の正本は呼び出し側で、widget は [`ScrubSpec::value`] を映すだけ。ドラッグ中の
//! 表示だけは gesture の手元値(=直近に `Changed` で言った値)を使う — 言った値と
//! 見えている値が同 tick でずれない(Q4: preview = 結果)。
//!
//! ## クリック1回は gesture ではない
//!
//! 押して動かさず離しても何も言わない(egui の `DragValue` と同じ)。`Started` は
//! **最初に動いた時**か**テキスト編集に入った時**にだけ出るので、消費側は
//! `Started` = 「復元値を控える合図」とだけ覚えればよい。

use iced::advanced::widget::{tree, Tree};
use iced::advanced::{layout, mouse, renderer, text, Layout, Shell, Widget};
use iced::{keyboard, Element, Event, Length, Pixels, Point, Rectangle, Size};

use crate::theme::{self, Tokens};

/// 欄の高さ。
pub const SCRUB_H: f32 = 20.0;
/// 字の左右 padding。
const PAD_X: f32 = 6.0;
/// 字の大きさ(mono)。
const FONT_SIZE: f32 = 11.0;
/// テキスト編集中の caret の高さ。
const CARET_H: f32 = 12.0;

/// scrub gesture の語彙。**この enum が公開契約**(消費側 capsule と同文)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrubEvent {
    /// gesture が開いた(ドラッグの初動、またはテキスト編集の開始)。
    /// 消費側はここで復元用の値を控える(egui 版の `BeginEdit` に当たる)。
    Started,
    /// ドラッグ中の値(clamp / integer snap 済み)。同じ値は二度言わない。
    Changed(f64),
    /// 確定(release、または Enter)。egui 版の `EndEdit` に当たる。
    Committed(f64),
    /// 取り消し(Esc)。値の復元は `Started` で控えた側の仕事。
    Cancelled,
}

/// 1つの scrub 欄の仕様。値の正本は呼び出し側(Q5 — widget は値を所有しない)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrubSpec {
    /// いま表示する値。
    pub value: f64,
    /// 表示桁(固定)。
    pub decimals: usize,
    /// 下限(あれば clamp)。
    pub min: Option<f64>,
    /// 上限(あれば clamp)。
    pub max: Option<f64>,
    /// ドラッグ 1px あたりの増分。
    pub step: f64,
    /// true なら整数へ snap する。
    pub integer: bool,
    /// この欄の行が持つ `--property-color`。inspector-library.css:391
    /// `.valueCell:focus-within, .valueCell.dragging { box-shadow: inset 0 0 0
    /// 1px var(--property-color); }` — 枠色は行ごとに違う(全欄で
    /// action-active 固定ではない)。呼び出し側が行の帯色をそのまま渡す。
    pub accent: iced::Color,
}

impl ScrubSpec {
    /// integer snap → clamp の順で通す(上限 12 の欄で 12.4 が「12を越えて」
    /// 見えることがない)。
    fn constrain(&self, value: f64) -> f64 {
        let mut value = if self.integer { value.round() } else { value };
        if let Some(min) = self.min {
            value = value.max(min);
        }
        if let Some(max) = self.max {
            value = value.min(max);
        }
        value
    }

    fn format(&self, value: f64) -> String {
        format_value(value, self.decimals)
    }
}

/// 表示は `decimals` 桁固定(発注 capsule)。
pub fn format_value(value: f64, decimals: usize) -> String {
    format!("{value:.decimals$}")
}

/// scrub 欄を1つ組む。ドラッグで増減、ダブルクリックでテキスト入力。
pub fn scrub_value<'a, M>(
    spec: ScrubSpec,
    on_event: impl Fn(ScrubEvent) -> M + 'a,
) -> iced::Element<'a, M>
where
    M: 'a,
{
    Element::new(ScrubValue {
        spec,
        on_event: Box::new(on_event),
    })
}

struct ScrubValue<'a, M> {
    spec: ScrubSpec,
    on_event: Box<dyn Fn(ScrubEvent) -> M + 'a>,
}

/// gesture の段階。`Pressed` は「押したがまだ動いていない」— ここで離せば
/// ただのクリックで、何も言わない。
enum Phase {
    Idle,
    /// 左ボタンが落ちた。動けば `Dragging`、動かず離せば何もなし。
    Pressed {
        start: Point,
        start_value: f64,
    },
    /// ドラッグ中。`last` = 直近に `Changed` で言った(=表示している)値。
    Dragging {
        start: Point,
        start_value: f64,
        last: f64,
    },
    /// ダブルクリックで開いたテキスト入力。`pristine` の間は最初の1字が
    /// 前の値を置き換える(ダブルクリック=全選択の慣行の写し)。
    Editing {
        buffer: String,
        pristine: bool,
    },
}

struct State {
    phase: Phase,
    last_click: Option<mouse::Click>,
    hovered: bool,
}

impl Default for State {
    fn default() -> Self {
        State {
            phase: Phase::Idle,
            last_click: None,
            hovered: false,
        }
    }
}

impl<M> ScrubValue<'_, M> {
    fn publish(&self, shell: &mut Shell<'_, M>, event: ScrubEvent) {
        shell.publish((self.on_event)(event));
    }

    /// テキスト入力を閉じる。読めれば確定、読めなければ取り消し(黙って
    /// 変な値にしない — 復元は Q3 の「拒否の報酬」として見える)。
    fn commit_buffer(&self, shell: &mut Shell<'_, M>, buffer: &str) {
        match buffer.trim().parse::<f64>() {
            Ok(value) => self.publish(shell, ScrubEvent::Committed(self.spec.constrain(value))),
            Err(_) => self.publish(shell, ScrubEvent::Cancelled),
        }
    }
}

impl<M> Widget<M, iced::Theme, iced::Renderer> for ScrubValue<'_, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fixed(SCRUB_H))
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, Length::Fill, Length::Fixed(SCRUB_H))
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        shell: &mut Shell<'_, M>,
        _viewport: &Rectangle,
    ) {
        let state: &mut State = tree.state.downcast_mut();
        let bounds = layout.bounds();

        match event {
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                // 接触の報酬(Q3): hover の縁で絵を変える。
                let over = bounds.contains(*position);
                if over != state.hovered {
                    state.hovered = over;
                    shell.request_redraw();
                }

                match &mut state.phase {
                    Phase::Pressed { start, start_value } => {
                        let (start, start_value) = (*start, *start_value);
                        // 初動 — gesture はここで開く。
                        self.publish(shell, ScrubEvent::Started);
                        let value = self.spec.constrain(
                            start_value + f64::from(position.x - start.x) * self.spec.step,
                        );
                        let mut last = start_value;
                        if value != start_value {
                            self.publish(shell, ScrubEvent::Changed(value));
                            last = value;
                        }
                        state.phase = Phase::Dragging {
                            start,
                            start_value,
                            last,
                        };
                        shell.capture_event();
                        shell.request_redraw();
                    }
                    Phase::Dragging {
                        start,
                        start_value,
                        last,
                    } => {
                        let value = self.spec.constrain(
                            *start_value + f64::from(position.x - start.x) * self.spec.step,
                        );
                        if value != *last {
                            *last = value;
                            self.publish(shell, ScrubEvent::Changed(value));
                            shell.request_redraw();
                        }
                        shell.capture_event();
                    }
                    _ => {}
                }
            }
            Event::Mouse(mouse::Event::CursorLeft) => {
                if state.hovered {
                    state.hovered = false;
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(position) = cursor.position() else {
                    return;
                };
                let over = bounds.contains(position);
                match &mut state.phase {
                    Phase::Editing { buffer, .. } if !over => {
                        // 外を押した = 確定して欄を出る。クリック自体は下の面へ通す。
                        let buffer = buffer.clone();
                        self.commit_buffer(shell, &buffer);
                        state.phase = Phase::Idle;
                        shell.request_redraw();
                    }
                    Phase::Editing { .. } => {}
                    Phase::Idle if over => {
                        let click =
                            mouse::Click::new(position, mouse::Button::Left, state.last_click);
                        state.last_click = Some(click);
                        if click.kind() == mouse::click::Kind::Double {
                            state.phase = Phase::Editing {
                                buffer: self.spec.format(self.spec.value),
                                pristine: true,
                            };
                            self.publish(shell, ScrubEvent::Started);
                        } else {
                            state.phase = Phase::Pressed {
                                start: position,
                                start_value: self.spec.value,
                            };
                        }
                        shell.capture_event();
                        shell.request_redraw();
                    }
                    _ => {}
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                match &state.phase {
                    Phase::Pressed { .. } => {
                        // 動かないクリックは gesture ではない — 何も言わない。
                        state.phase = Phase::Idle;
                        shell.capture_event();
                        shell.request_redraw();
                    }
                    Phase::Dragging { last, .. } => {
                        // release = 確定(Q1)。
                        self.publish(shell, ScrubEvent::Committed(*last));
                        state.phase = Phase::Idle;
                        shell.capture_event();
                        shell.request_redraw();
                    }
                    _ => {}
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, text, .. }) => {
                match &mut state.phase {
                    Phase::Pressed { .. } => {
                        if matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) {
                            // まだ gesture が開いていない(Started 前)ので黙って畳む。
                            state.phase = Phase::Idle;
                            shell.capture_event();
                            shell.request_redraw();
                        }
                    }
                    Phase::Dragging { .. } => {
                        if matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) {
                            // Esc = cancel 復元(Q1)。復元値は Started で控えた側が持つ。
                            self.publish(shell, ScrubEvent::Cancelled);
                            state.phase = Phase::Idle;
                            shell.capture_event();
                            shell.request_redraw();
                        }
                    }
                    Phase::Editing { buffer, pristine } => {
                        match key {
                            keyboard::Key::Named(keyboard::key::Named::Enter) => {
                                let buffer = buffer.clone();
                                self.commit_buffer(shell, &buffer);
                                state.phase = Phase::Idle;
                            }
                            keyboard::Key::Named(keyboard::key::Named::Escape) => {
                                self.publish(shell, ScrubEvent::Cancelled);
                                state.phase = Phase::Idle;
                            }
                            keyboard::Key::Named(keyboard::key::Named::Backspace) => {
                                if *pristine {
                                    // 全選択相当 — まとめて消える。
                                    buffer.clear();
                                    *pristine = false;
                                } else {
                                    let _ = buffer.pop();
                                }
                            }
                            _ => {
                                let Some(input) = text else {
                                    return;
                                };
                                let printable: String =
                                    input.chars().filter(|c| !c.is_control()).collect();
                                if printable.is_empty() {
                                    return;
                                }
                                if *pristine {
                                    // 最初の1字が前の値を置き換える(全選択の慣行)。
                                    buffer.clear();
                                    *pristine = false;
                                }
                                buffer.push_str(&printable);
                            }
                        }
                        shell.capture_event();
                        shell.request_redraw();
                    }
                    Phase::Idle => {}
                }
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let state: &State = tree.state.downcast_ref();
        match &state.phase {
            Phase::Editing { .. } => mouse::Interaction::Text,
            Phase::Pressed { .. } | Phase::Dragging { .. } => {
                mouse::Interaction::ResizingHorizontally
            }
            Phase::Idle if cursor.is_over(layout.bounds()) => {
                // 掴んで横に動かす面だと cursor が先に言う(Q3 接触の報酬)。
                mouse::Interaction::ResizingHorizontally
            }
            Phase::Idle => mouse::Interaction::None,
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme_in: &iced::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        use iced::advanced::text::Renderer as _;
        use iced::advanced::Renderer as _;

        let t = Tokens::resolve(theme_in);
        let state: &State = tree.state.downcast_ref();
        let bounds = layout.bounds();

        // 地と枠。inspector-library.css:369-392 `.valueCell` の写し:
        // 既定は枠なしの沈んだ地(surface_app 側へ寄る)、hover は
        // `--property-color` を薄く混ぜた地、drag/編集だけ inset 1px の枠が立つ
        // (地色そのものは変わらない — box-shadow は塗りを置き換えない)。
        // 角丸は css 側にも無い(Ableton 風フラット、2026-07-14 裁定)ので
        // 0 にする — 旧 radius 3.0 は正本に無い値だった。
        let accent = self.spec.accent;
        let sunken = theme::mix(t.surface_app, 74.0, t.surface_panel);
        let tinted = theme::mix(accent, 9.0, t.surface_app);
        let (fill, border) = match &state.phase {
            Phase::Editing { .. } | Phase::Pressed { .. } | Phase::Dragging { .. } => {
                (tinted, Some(accent))
            }
            Phase::Idle if state.hovered => (tinted, None),
            Phase::Idle => (sunken, None),
        };
        let (border_color, border_width) = match border {
            Some(color) => (color, 1.0),
            None => (iced::Color::TRANSPARENT, 0.0),
        };
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: iced::Border {
                    color: border_color,
                    width: border_width,
                    radius: 0.0.into(),
                },
                ..renderer::Quad::default()
            },
            fill,
        );

        // 数。ドラッグ中は直近に言った値(preview = 結果、Q4)。
        let content = match &state.phase {
            Phase::Editing { buffer, .. } => buffer.clone(),
            Phase::Dragging { last, .. } => self.spec.format(*last),
            _ => self.spec.format(self.spec.value),
        };
        renderer.fill_text(
            text::Text {
                content,
                bounds: Size::new(bounds.width - PAD_X * 2.0, bounds.height),
                size: Pixels(FONT_SIZE),
                line_height: text::LineHeight::default(),
                font: iced::Font::MONOSPACE,
                align_x: text::Alignment::Right,
                align_y: iced::alignment::Vertical::Center,
                shaping: text::Shaping::Basic,
                wrapping: text::Wrapping::None,
                ellipsis: text::Ellipsis::None,
                hint_factor: None,
            },
            Point::new(bounds.x + bounds.width - PAD_X, bounds.center_y()),
            t.text_primary,
            bounds,
        );

        // テキスト編集中は caret を出す(編集面である報酬)。
        if let Phase::Editing { .. } = &state.phase {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: bounds.x + bounds.width - PAD_X + 1.0,
                        y: bounds.center_y() - CARET_H / 2.0,
                        width: 1.0,
                        height: CARET_H,
                    },
                    ..renderer::Quad::default()
                },
                t.action_active,
            );
        }
    }
}
