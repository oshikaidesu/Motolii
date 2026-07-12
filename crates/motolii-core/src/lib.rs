//! motolii-core: 全クレート共通の語彙(時間型・フレーム記述子・正準座標)。
//!
//! 仕様: docs/specs/M1-vertical-slice.md「インターフェース契約」

mod camera;
mod canonical;
mod frame;
mod quality;
mod time;
mod time_map;

pub use camera::CompCamera;
pub use canonical::{
    CanonicalPoint, CanonicalSize, PixelPoint, PixelSize, ViewportTransform, ViewportTransformError,
};
pub use frame::{
    premultiply_rgba_f32, premultiply_rgba_u8, ColorSpace, CpuFrame, FrameDesc, FrameDescError,
    PixelFormat,
};
pub use quality::{Quality, SampleTier};
pub use time::{Fps, FpsError, RationalTime, RationalTimeError};
pub use time_map::{OverrunMode, TimeMap, TimeMapError};
