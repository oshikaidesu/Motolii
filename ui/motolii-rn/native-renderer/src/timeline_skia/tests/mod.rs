pub use super::draw::{first_absolute_tick, format_ruler_time, ruler_label_step_secs};
pub use super::geometry::{
    body_top, bx, empty_real_guide_rows, logical_height, snap_to_frame, surface_width,
};
pub use super::hit::trim_edge_width;
pub use super::layout::{scale_for, KEY_HIT_PX, SURF_X};
pub use super::*;

pub fn session() -> (TimelineSession, i32, f64) {
    (TimelineSession::default(), 1, 0.27)
}

pub fn lx_for_bar(bar: f64) -> f64 {
    lx_for_bar_in(&TimelineScene::default(), bar)
}

pub fn lx_for_bar_in(scene: &TimelineScene, bar: f64) -> f64 {
    f64::from(SURF_X)
        + (bar - f64::from(scene.view_a)) / f64::from(scene.view_b - scene.view_a) * surface_width()
}

pub fn phys(lx: f64, ly: f64) -> (f64, f64) {
    (lx, ly)
}

pub fn bx_default(b: f32) -> f32 {
    bx(&TimelineScene::default(), b)
}

mod hit_draw;
mod identity;
mod key_real;
mod product;
mod select_move_trim;
mod snap;
mod view_select;
