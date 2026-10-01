//! 「どこに見えているか」を答える口 — 見た目を保つ編集(束ねを解く・親を移す・札を変える)が使う。
//! 解くのは絵の側。読み口を名指しするので、契約の表(`kind`)ではなく読み口の隣に置く
//! (`kind` が `StoreView` を知ると、契約と読み口が互いを指して輪になる)。

use std::collections::HashMap;

use crate::doc::core::{RationalTime, ResolvedCamera};
use crate::doc::store::{LayerId, StoreError, StoreView};

/// 「見えている所」を答える口。作品を編集する時、見た目を保つために必要になる
/// (束ねを解く・親を移す・札を変える)。解くのは絵の側で、コアは答えだけ受け取る。
#[derive(Clone, Copy)]
pub struct Geometry {
    /// 層の、親の空間での変換。
    pub local: fn(&StoreView<'_>, LayerId, RationalTime) -> Result<glam::Affine2, StoreError>,
    /// 層の、親の空間での変換(奥行き込み)。
    pub local3d: fn(&StoreView<'_>, LayerId, RationalTime) -> Result<glam::Affine3A, StoreError>,
    /// 層の、世界の変換(奥行き込み)。
    pub world: fn(&StoreView<'_>, LayerId, RationalTime) -> Result<glam::Affine3A, StoreError>,
    /// その時刻の全層の世界の変換。
    pub worlds: fn(&StoreView<'_>, RationalTime) -> Result<HashMap<LayerId, glam::Affine3A>, StoreError>,
    /// その時刻に効いている観測の姿勢。
    pub camera: fn(&StoreView<'_>, RationalTime) -> Result<ResolvedCamera, StoreError>,
}

impl Geometry {
    /// 誰も答えない書類。見た目の補正は起きず、書いた値がそのまま残る。
    pub const NONE: Self = Self {
        local: |_, _, _| Ok(glam::Affine2::IDENTITY),
        local3d: |_, _, _| Ok(glam::Affine3A::IDENTITY),
        world: |_, _, _| Ok(glam::Affine3A::IDENTITY),
        worlds: |_, _| Ok(HashMap::new()),
        camera: |_, _| Ok(ResolvedCamera::default()),
    };
}
