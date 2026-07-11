//! 受け入れテスト用のCPU参照実装・期待値生成(M2E-2保護領域)。
//!
//! 被試験クレートの`src/**`に同居させない(監査E-1)。
//! 製品の`ViewportTransform`等に依存しない(監査E-6の循環参照回避)。

mod composite;
mod graph;
mod luma;
mod overlay;
mod yuv;

pub use composite::{premul_add_u8, premul_multiply_u8, premul_over_u8, to_u8};
pub use graph::expected_fixed_graph;
pub use luma::expected_luma;
pub use overlay::{
    expected_circle_over_pattern, expected_line_over_pattern, expected_rect_frame,
    expected_rect_over_pattern,
};
pub use yuv::yuv_to_rgba_reference;
