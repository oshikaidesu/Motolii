//! 同梱の効果の登録表。契約は doc の `store::kind`、実装はこの隣の file たち。
//! コアはこの表を知らない — 作品を開く側が `with_programs` で渡す。

pub fn placement_program(plugin_id: &str) -> Option<crate::doc::store::kind::PlacementProgram> {
    super::placement::program(plugin_id).or_else(|| super::blob::program(plugin_id))
}

pub fn sampling_program(plugin_id: &str) -> Option<crate::doc::store::kind::SamplingProgram> {
    super::motion::program(plugin_id)
}

pub fn snap_program(plugin_id: &str) -> Option<crate::doc::store::kind::SnapProgram> {
    super::overlay::program(plugin_id)
}

/// 同梱の効果一式。コアの外から書類へ渡す(コアはこの関数を知らない)。
pub fn bundled() -> crate::doc::store::kind::Programs {
    crate::doc::store::kind::Programs { placement: placement_program, sampling: sampling_program, snap: snap_program }
}

/// 「どこに見えているか」を答える口。束ねを解く・親を移す・札を変える時、
/// コアはこの答えを使って書いた値を補正する(コアは解き方を知らない)。
pub fn geometry() -> crate::doc::store::kind::Geometry {
    use crate::picture::resolve::{camera, transform};
    crate::doc::store::kind::Geometry {
        local: transform::local_transform,
        local3d: transform::local_transform3d,
        world: transform::world_transform3d,
        worlds: transform::world_transforms3d,
        camera: camera::resolve_camera,
    }
}
