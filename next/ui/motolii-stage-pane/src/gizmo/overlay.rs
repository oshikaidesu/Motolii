use glam::Vec2;
use iced::widget::canvas;
use iced::{keyboard, mouse, Point, Rectangle};

use motolii_core::{CompSpec, ResolvedCamera};
use motolii_engine::ObservationCamera;
use motolii_tokens_rs::{Colors, Dimensions};

use super::*;

/// ギズモの canvas 一式。`Shell::view` が選択レイヤーの [`GizmoTarget`] から
/// 毎フレーム組み直し、`stack!` で `StageOverlay` の**上**に重ねる(結線は
/// supervisor)。掴んでいない場所のイベントは capture しない —
/// ホイール/中ボタン(観測カメラ)や空クリックは下の層へ素通しする。
#[derive(Clone, Copy)]
pub struct GizmoOverlay {
    comp: CompSpec,
    render_camera: ResolvedCamera,
    observation: Option<ObservationCamera>,
    target: GizmoTarget,
    dims: Dimensions,
    colors: Colors,
}

impl GizmoOverlay {
    pub fn new(
        comp: CompSpec,
        render_camera: ResolvedCamera,
        observation: Option<ObservationCamera>,
        target: GizmoTarget,
        dims: Dimensions,
        colors: Colors,
    ) -> Self {
        Self {
            comp,
            render_camera,
            observation,
            target,
            dims,
            colors,
        }
    }

    pub fn view<'a>(self) -> iced::Element<'a, GizmoDrag> {
        canvas(self)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }

    fn layout(&self, bounds: Rectangle) -> Option<GizmoLayout> {
        gizmo_layout(
            bounds,
            self.comp,
            self.render_camera,
            self.observation,
            &self.target,
            self.dims,
        )
    }

    fn message(&self, phase: GizmoPhase) -> GizmoDrag {
        GizmoDrag {
            layer: self.target.layer,
            phase,
        }
    }
}

/// canvas の一時状態(drag と Shift の追跡)。
#[derive(Default)]
pub struct GizmoInteraction {
    drag: Option<GizmoDragState>,
    shift: bool,
}

impl canvas::Program<GizmoDrag> for GizmoOverlay {
    type State = GizmoInteraction;

    fn update(
        &self,
        state: &mut GizmoInteraction,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<GizmoDrag>> {
        match event {
            canvas::Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                state.shift = modifiers.shift();
                // drag 中なら Shift の途中切り替えを即時反映(capture はしない —
                // 修飾キーは他 widget も見る)。
                let drag = state.drag.as_mut()?;
                let value = drag.refresh(state.shift)?;
                Some(canvas::Action::publish(
                    self.message(GizmoPhase::Move { value }),
                ))
            }
            canvas::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            }) => {
                // Esc = キャンセル(drag 中だけ食う — それ以外は素通し)。
                state.drag.take()?;
                Some(canvas::Action::publish(self.message(GizmoPhase::Cancel)).and_capture())
            }
            canvas::Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    let position = cursor.position_in(bounds)?;
                    let layout = self.layout(bounds)?;
                    let handle = gizmo_hit_test(&layout, position)?;
                    let drag = GizmoDragState::begin(self.target, &layout, handle, position)?;
                    state.drag = Some(drag);
                    Some(
                        canvas::Action::publish(self.message(GizmoPhase::Start {
                            property: handle.property(),
                        }))
                        .and_capture(),
                    )
                }
                mouse::Event::CursorMoved { .. } => {
                    let drag = state.drag.as_mut()?;
                    // drag 中は bounds の外へ出ても続く(絶対座標からローカルへ
                    // 自前で戻す — `position_in` は外に出た瞬間 `None` になる)。
                    let absolute = cursor.position()?;
                    let position = Point::new(absolute.x - bounds.x, absolute.y - bounds.y);
                    let value = drag.update(position, state.shift);
                    Some(
                        canvas::Action::publish(self.message(GizmoPhase::Move { value }))
                            .and_capture(),
                    )
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    let drag = state.drag.take()?;
                    let phase = match drag.last_value() {
                        // 契約: Start は必ず Commit か Cancel で閉じる(空クリック=
                        // 動いていない= Cancel、値を書かない)。
                        Some(value) if drag.moved() => GizmoPhase::Commit { value },
                        _ => GizmoPhase::Cancel,
                    };
                    Some(canvas::Action::publish(self.message(phase)).and_capture())
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// bbox(accent hairline)+8ハンドル+回転ハンドル(stem+円)+anchor ⊕。
    /// 全部 [`GizmoLayout`] の座標をそのまま描く — hit-test と同じ正本(Q0)。
    fn draw(
        &self,
        _state: &GizmoInteraction,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let Some(layout) = self.layout(bounds) else {
            return Vec::new();
        };
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let accent = self.colors.action_active;
        let handle_fill = self.colors.surface_raised;
        let hairline = canvas::Stroke::default()
            .with_color(accent)
            .with_width(self.dims.border_width);

        // 選択の器(bbox)。
        let bbox = canvas::Path::new(|builder| {
            builder.move_to(layout.corners[0]);
            for corner in &layout.corners[1..] {
                builder.line_to(*corner);
            }
            builder.close();
        });
        frame.stroke(&bbox, hairline.clone());

        // 回転ハンドル(stem を先に描き、ハンドル群を上へ)。
        if let Some(rotate) = layout.rotate_handle {
            let stem = canvas::Path::line(layout.top_center, rotate);
            frame.stroke(&stem, hairline.clone());
            let knob = canvas::Path::circle(rotate, self.dims.gizmo_handle_size * 0.5);
            frame.fill(&knob, handle_fill);
            frame.stroke(&knob, hairline.clone());
        }

        // 8ハンドル(正方形、中心合わせ)。
        let handle_size = self.dims.gizmo_handle_size;
        for point in layout.scale_handles {
            let square = canvas::Path::rectangle(
                Point::new(point.x - handle_size * 0.5, point.y - handle_size * 0.5),
                iced::Size::new(handle_size, handle_size),
            );
            frame.fill(&square, handle_fill);
            frame.stroke(&square, hairline.clone());
        }

        // anchor ⊕(AE の慣習形。第2切片から drag 対象 — 命中は hit_radius、
        // 見た目の寸はトークンのまま: 変形の不動点はハンドルより一段軽い視覚重量)。
        let radius = self.dims.gizmo_anchor_radius;
        let ring = canvas::Path::circle(layout.anchor, radius);
        frame.stroke(&ring, hairline.clone());
        let cross_horizontal = canvas::Path::line(
            Point::new(layout.anchor.x - radius * 2.0, layout.anchor.y),
            Point::new(layout.anchor.x + radius * 2.0, layout.anchor.y),
        );
        let cross_vertical = canvas::Path::line(
            Point::new(layout.anchor.x, layout.anchor.y - radius * 2.0),
            Point::new(layout.anchor.x, layout.anchor.y + radius * 2.0),
        );
        frame.stroke(&cross_horizontal, hairline.clone());
        frame.stroke(&cross_vertical, hairline);

        vec![frame.into_geometry()]
    }

    /// hover/drag のカーソル形状(Q0「触れそう」の正直な合図)。ハンドルの
    /// リサイズ向きは回転済みの screen 位置から出す([`resize_interaction`])。
    fn mouse_interaction(
        &self,
        state: &GizmoInteraction,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        let interaction_for = |handle: GizmoHandle, layout: &GizmoLayout, dragging: bool| {
            match handle {
                GizmoHandle::Body => {
                    if dragging {
                        mouse::Interaction::Grabbing
                    } else {
                        mouse::Interaction::Move
                    }
                }
                GizmoHandle::Rotate => {
                    if dragging {
                        mouse::Interaction::Grabbing
                    } else {
                        mouse::Interaction::Grab
                    }
                }
                // ⊕ グリフと同形の Crosshair(hover)— 「精密に置く物」の合図。
                // 第1切片の「表示のみ」からの繰り上がりを、カーソルが最初に語る
                // (Q0: 触れる物は触れそうに見せる)。
                GizmoHandle::Anchor => {
                    if dragging {
                        mouse::Interaction::Grabbing
                    } else {
                        mouse::Interaction::Crosshair
                    }
                }
                GizmoHandle::Scale(scale_handle) => {
                    let index = SCALE_HANDLES
                        .iter()
                        .position(|h| *h == scale_handle)
                        .unwrap_or(0);
                    let point = layout.scale_handles[index];
                    let center_x =
                        layout.corners.iter().map(|c| c.x).sum::<f32>() / layout.corners.len() as f32;
                    let center_y =
                        layout.corners.iter().map(|c| c.y).sum::<f32>() / layout.corners.len() as f32;
                    resize_interaction(Vec2::new(point.x - center_x, point.y - center_y))
                }
            }
        };

        if let Some(drag) = &state.drag {
            if let Some(layout) = self.layout(bounds) {
                return interaction_for(drag.handle(), &layout, true);
            }
            return mouse::Interaction::Grabbing;
        }
        let Some(position) = cursor.position_in(bounds) else {
            return mouse::Interaction::default();
        };
        let Some(layout) = self.layout(bounds) else {
            return mouse::Interaction::default();
        };
        match gizmo_hit_test(&layout, position) {
            Some(handle) => interaction_for(handle, &layout, false),
            None => mouse::Interaction::default(),
        }
    }
}

/// bbox 中心→ハンドルの screen 方向から、リサイズカーソルの4向きを選ぶ
/// (45° 扇形で最寄りの軸へ丸める — 回転したレイヤーでも視覚的に正しい向き)。
/// `ResizingDiagonallyDown` = NW–SE 軸(↘)、`ResizingDiagonallyUp` = NE–SW 軸(↗)
/// (iced→winit の CursorIcon 対応どおり)。
pub fn resize_interaction(direction: Vec2) -> mouse::Interaction {
    if direction.length_squared() < SOLVE_EPS * SOLVE_EPS {
        return mouse::Interaction::default();
    }
    let mut angle = direction.y.atan2(direction.x).to_degrees();
    // 軸としては 180° 対称なので [0, 180) へ畳む。
    if angle < 0.0 {
        angle += 180.0;
    }
    if !(22.5..157.5).contains(&angle) {
        mouse::Interaction::ResizingHorizontally
    } else if angle < 67.5 {
        mouse::Interaction::ResizingDiagonallyDown
    } else if angle < 112.5 {
        mouse::Interaction::ResizingVertically
    } else {
        mouse::Interaction::ResizingDiagonallyUp
    }
}

