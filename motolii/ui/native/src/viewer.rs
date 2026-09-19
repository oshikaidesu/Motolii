use crate::doc::core::ResolvedCamera;
use crate::doc::store::{Animate, Revision, StoreView};
use crate::doc::store::{LayerId, PropertyId};
use crate::render::{engine::Window, playback::Clock};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum View {
    User,
    Camera,
}
impl View {
    pub(crate) fn parse(name: &str) -> Result<Self, String> {
        match name {
            "User" | "Stage" => Ok(Self::User),
            "Camera" => Ok(Self::Camera),
            _ => Err(format!("Unknown view {name}")),
        }
    }
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::User => "User",
            Self::Camera => "Camera",
        }
    }
}

/// Ephemeral viewing state. Dropping it does not own, save or edit the work.
pub(crate) struct ViewerState {
    pub selected_ids: Vec<LayerId>,
    pub selected_keys: Vec<KeySel>,
    pub selection_bounds: HashMap<View, HashMap<LayerId, [f32; 4]>>,
    pub color_target: Option<ColorSlot>,
    pub clock: Clock,
    pub clock_revision: Revision,
    pub frame: i64,
    pub picked_color: Option<[f64; 4]>,
    pub pick_serial: u64,
    pub stage_snap: [Option<f64>; 2],
    pub stage_pointer: Option<[f64; 2]>,
    pub stage_view_scale: f64,
    pub stage_held: Option<String>,
    pub stage_window: Option<Window>,
    pub stage_view: View,
    pub user_camera: ResolvedCamera,
    pub animate: Animate,
}

impl ViewerState {
    pub(crate) fn new(view: &StoreView<'_>, revision: Revision) -> Self {
        let clock = Clock::from_view(view, 60.0);
        clock.sync_view(view);
        Self {
            selected_ids: view.layers().first().copied().into_iter().collect(),
            selected_keys: Vec::new(),
            selection_bounds: HashMap::new(),
            color_target: None,
            clock,
            clock_revision: revision,
            frame: 0,
            picked_color: None,
            pick_serial: 0,
            stage_snap: [None, None],
            stage_pointer: None,
            stage_view_scale: 1.0,
            stage_held: None,
            stage_window: None,
            stage_view: View::User,
            user_camera: Default::default(),
            animate: Animate::Off,
        }
    }

    pub(crate) fn selected(&self) -> Option<LayerId> {
        self.selected_ids.last().copied()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct KeySel {
    pub layer: LayerId,
    pub property: Option<PropertyId>,
    pub at_sec: f64,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) enum ColorSlot {
    TextFill {
        layer: LayerId,
        style: crate::doc::store::TextStyleId,
    },
    /// ShapeNode の木の中の葉。index の列で指す。
    ShapeFill { layer: LayerId, path: Vec<usize> },
    /// 同じ葉の線の色。線が無ければ色を付けた時に生える。
    ShapeStroke { layer: LayerId, path: Vec<usize> },
    /// 2色gradientの端。`end=false` が最小offset、`end=true` が最大offset。
    /// VecのindexをUIへ漏らさないので、stopの並び順が違う文書でも同じ端を指せる。
    /// index is the persisted stop identity; legacy gradients use their ordinal.
    ShapeGradientPoint {
        layer: LayerId,
        path: Vec<usize>,
        index: usize,
    },
    /// 色の型の property なら何でも(効果の param の色など)。名前がそのまま宛先。
    Property { layer: LayerId, property: String },
    /// Composition の背景。層ではないが、同じ見本と輪で触る。
    Background,
    ShapeGradientStop {
        layer: LayerId,
        path: Vec<usize>,
        end: bool,
    },
}

impl ColorSlot {
    pub(crate) fn layer(&self) -> Option<LayerId> {
        match self {
            Self::TextFill { layer, .. }
            | Self::ShapeFill { layer, .. }
            | Self::ShapeStroke { layer, .. }
            | Self::ShapeGradientPoint { layer, .. }
            | Self::Property { layer, .. }
            | Self::ShapeGradientStop { layer, .. } => Some(*layer),
            Self::Background => None,
        }
    }

    pub(crate) fn is_shape_fill(&self) -> bool {
        matches!(
            self,
            Self::ShapeFill { .. }
                | Self::ShapeGradientStop { .. }
                | Self::ShapeGradientPoint { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::{blank_project, Intent};

    #[test]
    fn independent_viewers_do_not_duplicate_or_edit_the_document() {
        let mut doc = blank_project();
        doc.apply_all([Intent::AddLayer(LayerId(1)), Intent::AddLayer(LayerId(2))])
            .unwrap();
        let revision = doc.revision();
        let history = doc.history_depth();
        let mut first = ViewerState::new(&doc.view(), revision.clone());
        let second = ViewerState::new(&doc.view(), revision.clone());
        first.selected_ids = vec![LayerId(2)];
        first.clock.seek_frame(30);
        first.frame = 30;
        first.stage_view_scale = 2.0;
        first.stage_pointer = Some([120.0, 240.0]);
        assert_eq!(first.selected(), Some(LayerId(2)));
        assert_eq!(second.selected(), Some(LayerId(1)));
        assert_eq!(second.clock.current_frame(), 0);
        assert_eq!(second.frame, 0);
        assert_eq!(second.stage_view_scale, 1.0);
        assert_eq!(second.stage_pointer, None);
        assert_eq!(doc.revision(), revision);
        assert_eq!(doc.history_depth(), history);
        drop(first);
        drop(second);
        assert_eq!(doc.view().layers(), vec![LayerId(1), LayerId(2)]);
        assert_eq!(doc.revision(), revision);
        assert_eq!(doc.history_depth(), history);
    }

    #[test]
    fn primary_selection_is_derived_not_a_second_copy() {
        let doc = blank_project();
        let mut viewer = ViewerState::new(&doc.view(), doc.revision());
        viewer.selected_ids = vec![LayerId(1), LayerId(2)];
        assert_eq!(viewer.selected(), Some(LayerId(2)));
        viewer.selected_ids.retain(|id| *id != LayerId(2));
        assert_eq!(viewer.selected(), Some(LayerId(1)));
        viewer.selected_ids.clear();
        assert_eq!(viewer.selected(), None);
    }
}
