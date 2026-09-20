//! 絵 — 作品の値から、画面に出る形を解く側。
//! 箱・並べ・文字の置き場・切り・写し・道・線、そして層の一覧を組み立てる所。
//! コアは値と時刻と評価と契約だけを持ち、ここは `&StoreView` を受け取って読むだけ。

pub mod boxes;
pub mod connect;
pub mod flow;
pub mod frame;
pub mod motion_time;
pub mod path;
pub mod resolve;
pub mod shape_props;
pub mod shapes;
pub mod text;

pub(crate) use crate::doc::core::RationalTime;
pub(crate) use crate::doc::eval::Value;
pub(crate) use crate::doc::store::layout::*;
pub(crate) use crate::doc::store::scratch::{Frame, LayoutCache, Memo, Scratch, Slot};
pub(crate) use crate::doc::store::{
    property, EffectScope, LayerId, LayerProjection, LayerSource, PropertyId, ResolvedEffect,
    ResolvedLayer, ResolvedMask, StoreError, StoreView,
};
pub(crate) use std::collections::{HashMap, HashSet};
