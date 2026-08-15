// Ported from `egui_tiles` (https://github.com/rerun-io/egui_tiles) by rerun.io / Emil Ernerfeldt.
// Licensed under MIT OR Apache-2.0. Source: upstream commit fddb9fd (0.17.0 pre-release).
//
// なぜこのファイルが新しいか: 移植元では splitter の掴み判定を `egui::Ui::interact` が、
// 掴んでいる最中かどうかを egui のメモリストアが持っていた。egui を使わないので、
// 同じ状態機械を `DockInput`(入力の写し) と `DragState`(HashMap 一つ) で持ち直す。
// 判定の順序・条件は移植元の `Response::{hovered, dragged, double_clicked}` に合わせている。

use super::drag::DragState;
use super::geom::{Pos2, Rect};
use super::{DropContext, ResizeState, TileId};

/// One frame of pointer/keyboard input, in the tree's own coordinate space.
///
/// This is the whole of what the dock needs from the windowing layer.
#[derive(Clone, Copy, Debug)]
pub struct DockInput {
    /// Where the pointer is, if it is over the window at all.
    pub pointer_pos: Option<Pos2>,

    /// Is the primary button held down right now?
    pub pointer_down: bool,

    /// Did the primary button go down this frame?
    pub pointer_pressed: bool,

    /// Did the primary button go up this frame?
    pub pointer_released: bool,

    /// Was there a double-click this frame?
    pub double_clicked: bool,

    /// Escape aborts an in-progress drag, as it does upstream.
    pub escape_pressed: bool,

    /// Seconds since the last frame, used for the drag-preview smoothing.
    pub stable_dt: f32,

    /// Physical pixels per point, used for the same rounding upstream does.
    pub pixels_per_point: f32,
}

impl Default for DockInput {
    fn default() -> Self {
        Self {
            pointer_pos: None,
            pointer_down: false,
            pointer_pressed: false,
            pointer_released: false,
            double_clicked: false,
            escape_pressed: false,
            stable_dt: 1.0 / 60.0,
            pixels_per_point: 1.0,
        }
    }
}

/// Which of a container's splitters this is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SplitterKind {
    /// Between two children of a [`super::Linear`] container.
    Linear,

    /// Between two columns of a [`super::Grid`].
    GridColumn,

    /// Between two rows of a [`super::Grid`].
    GridRow,
}

/// Identifies one splitter, in place of the upstream `ui.id().with((parent_id, "resize", i))`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SplitterId {
    pub parent_id: TileId,
    pub kind: SplitterKind,
    pub index: usize,
}

/// A splitter as it came out of one interaction pass, for drawing and for the cursor.
#[derive(Clone, Copy, Debug)]
pub struct ResizeHandle {
    pub id: SplitterId,

    /// The grab area of the splitter.
    pub rect: Rect,

    /// The line the splitter is drawn on: `x` for a vertical line, `y` for a horizontal one.
    pub line: f32,

    /// Is the drawn line vertical (i.e. does dragging it resize horizontally)?
    pub is_vertical_line: bool,

    /// The extent of the line, along the line.
    pub span: super::geom::Rangef,

    pub state: ResizeState,
}

/// Stands in for `egui::Response` in the ported resize code.
#[derive(Clone, Copy, Debug, Default)]
pub struct SplitterResponse {
    hovered: bool,
    dragged: bool,
    double_clicked: bool,
}

impl SplitterResponse {
    #[inline]
    pub fn hovered(&self) -> bool {
        self.hovered
    }

    #[inline]
    pub fn dragged(&self) -> bool {
        self.dragged
    }

    #[inline]
    pub fn double_clicked(&self) -> bool {
        self.double_clicked
    }
}

/// Resolve one splitter against this frame's input, taking over what `Ui::interact` did.
pub(crate) fn splitter_response(
    id: SplitterId,
    line_rect: Rect,
    input: &DockInput,
    drag: &mut DragState,
) -> SplitterResponse {
    let hovered = input
        .pointer_pos
        .is_some_and(|pointer| line_rect.contains(pointer));

    if input.pointer_pressed && hovered {
        drag.active_splitter = Some(id);
    }

    let dragged = input.pointer_down && drag.active_splitter == Some(id);

    SplitterResponse {
        hovered,
        dragged,
        double_clicked: hovered && input.double_clicked,
    }
}

/// Everything the interaction pass carries down the tree.
pub(crate) struct InteractContext<'a> {
    pub input: &'a DockInput,
    pub drop: &'a mut DropContext,
    pub drag: &'a mut DragState,

    /// Every splitter seen this pass, in tree order, for the host to draw and to set a cursor from.
    pub handles: &'a mut Vec<ResizeHandle>,
}
