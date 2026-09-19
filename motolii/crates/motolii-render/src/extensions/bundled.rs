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
