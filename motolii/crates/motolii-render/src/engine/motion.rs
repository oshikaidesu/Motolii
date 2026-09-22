//! 時間で重ねる描き方: Motion Blur の写しを 1 枚の板から作る、隣のコマの時刻、粒子の層の点をこのコマで解く。

#[allow(unused_imports)]
use crate::picture::resolved::{ResolvedEffect, ResolvedLayer, ResolvedMask};
use std::collections::HashMap;

use crate::doc::core::{CompSpec, ResolvedCamera};
use crate::doc::store::{LayerId, LayerSource, RationalTime, ShapeNode, StoreView, TextDocument};
use crate::render::compositor::{BlendMode as CompositeBlendMode, Layer, LayerWithPasses};
use crate::render::engine::translate::translate_effect_passes;
use crate::render::engine::{Engine, EngineError};

impl Engine {

}

