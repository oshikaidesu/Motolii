use crate::doc::store::{LayerId,PropertyId};
#[derive(Clone,Debug)]
pub(crate) struct KeySel {pub layer:LayerId,pub property:Option<PropertyId>,pub at_sec:f64}
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize)]
pub(crate) enum ColorSlot {
    TextFill {
        layer: LayerId,
        style: crate::doc::store::TextStyleId,
    },
    TextStroke {
        layer: LayerId,
        style: crate::doc::store::TextStyleId,
    },
    /// ShapeNode の木の中の葉。index の列で指す。
    ShapeFill { layer: LayerId, path: Vec<usize> },
    /// 2色gradientの端。`end=false` が最小offset、`end=true` が最大offset。
    /// VecのindexをUIへ漏らさないので、stopの並び順が違う文書でも同じ端を指せる。
    ShapeGradientStop {
        layer: LayerId,
        path: Vec<usize>,
        end: bool,
    },
}

impl ColorSlot {
    pub(crate) fn layer(&self) -> LayerId {
        match self {
            Self::TextFill { layer, .. }
            | Self::TextStroke { layer, .. }
            | Self::ShapeFill { layer, .. }
            | Self::ShapeGradientStop { layer, .. } => *layer,
        }
    }

    pub(crate) fn is_shape_fill(&self) -> bool {
        matches!(
            self,
            Self::ShapeFill { .. } | Self::ShapeGradientStop { .. }
        )
    }
}

