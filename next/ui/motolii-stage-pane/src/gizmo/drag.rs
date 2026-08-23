use glam::{Affine2, Vec2};
use iced::Point;

use motolii_store::LayerId;

use super::*;

/// 進行中の drag。**Document でも Session でもない widget 内だけの一時状態**
/// ([`crate::Interaction`] と同格)。開始時点の対象・座標系を凍結して持つ —
/// drag 中に shell が transient で対象を動かしても解が発振しない
/// (Inspector drag の `start_value` 凍結と同じ判断)。
#[derive(Debug, Clone, Copy)]
pub struct GizmoDragState {
    handle: GizmoHandle,
    start: GizmoTarget,
    parent_from_screen: Affine2,
    start_cursor_parent: Vec2,
    last_cursor: Point,
    moved: bool,
    last_value: Option<GizmoValue>,
}

impl GizmoDragState {
    /// press で drag を開始する。screen→親の逆行列が立たない(親鎖の scale 0 等)
    /// なら `None` — 解けない drag を始めない(掴めない物は掴めないまま)。
    pub fn begin(
        target: GizmoTarget,
        layout: &GizmoLayout,
        handle: GizmoHandle,
        cursor: Point,
    ) -> Option<Self> {
        let parent_from_screen = checked_inverse(layout.screen_from_parent)?;
        let start_cursor_parent = parent_from_screen.transform_point2(Vec2::new(cursor.x, cursor.y));
        Some(Self {
            handle,
            start: target,
            parent_from_screen,
            start_cursor_parent,
            last_cursor: cursor,
            moved: false,
            last_value: None,
        })
    }

    pub fn layer(&self) -> LayerId {
        self.start.layer
    }

    pub fn handle(&self) -> GizmoHandle {
        self.handle
    }

    pub fn property(&self) -> GizmoProperty {
        self.handle.property()
    }

    /// 1回でも `update` が走ったか。release 時の Commit/Cancel(空クリック)判定。
    pub fn moved(&self) -> bool {
        self.moved
    }

    /// 直近の計算値。release は cursor 位置を運ばない(`ButtonReleased` に位置が
    /// 無い)ので、Commit はこれをそのまま使う(Inspector drag の `last_value` と
    /// 同じ持ち回し)。
    pub fn last_value(&self) -> Option<GizmoValue> {
        self.last_value
    }

    /// cursor 移動で新しい値を解く。
    pub fn update(&mut self, cursor: Point, shift: bool) -> GizmoValue {
        self.last_cursor = cursor;
        self.moved = true;
        let cursor_parent = self
            .parent_from_screen
            .transform_point2(Vec2::new(cursor.x, cursor.y));
        let value = match self.handle {
            GizmoHandle::Body => GizmoValue::Position(move_value(
                &self.start,
                self.start_cursor_parent,
                cursor_parent,
            )),
            GizmoHandle::Scale(handle) => {
                GizmoValue::Scale(scale_value(&self.start, handle, cursor_parent, shift))
            }
            GizmoHandle::Rotate => GizmoValue::Rotation(rotation_value(
                &self.start,
                self.start_cursor_parent,
                cursor_parent,
                shift,
            )),
            GizmoHandle::Anchor => {
                let (anchor, position) = anchor_value(&self.start, cursor_parent);
                GizmoValue::Anchor { anchor, position }
            }
        };
        self.last_value = Some(value);
        value
    }

    /// 修飾キーの途中変更(Shift の押し離し)を、cursor が動かなくても即時に
    /// 反映する(AE/Figma と同じ生の手触り)。まだ1回も動いていなければ何もしない
    /// (空クリックを Move に化けさせない)。
    pub fn refresh(&mut self, shift: bool) -> Option<GizmoValue> {
        self.moved.then(|| self.update(self.last_cursor, shift))
    }
}

// ---------------------------------------------------------------------------
// canvas::Program(翻訳だけ — StageOverlay と同じ規律)
// ---------------------------------------------------------------------------

