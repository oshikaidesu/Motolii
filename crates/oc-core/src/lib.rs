//! oc-core: 全クレート共通の語彙(時間型・フレーム記述子)。
//!
//! 仕様: docs/specs/M1-vertical-slice.md「インターフェース契約」

mod frame;
mod time;
mod time_map;

pub use frame::{
    premultiply_rgba_f32, premultiply_rgba_u8, ColorSpace, CpuFrame, FrameDesc, PixelFormat,
};
pub use time::{Fps, RationalTime};
pub use time_map::TimeMap;
