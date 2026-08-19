//! owns: 有理数フレーム時刻(`RationalTime`)と frame 記述。rerun の `TimeInt` は
//!       i64 + TimeType(sequence/duration/timestamp)であって、30000/1001 のような
//!       有理 fps を正確に持てない。映像編集はここを落とせないので Motolii が持つ。
//!
//! 旧 workspace `crates/motolii-core` からの移植(2026-08-20 リセット)。再実装ではない。
//!
//! motolii-core: 全クレート共通の語彙(時間型・フレーム記述子・正準座標)。
//!
//! 仕様: docs/specs/M1-vertical-slice.md「インターフェース契約」

mod camera;
mod canonical;
mod frame;
mod quality;
mod time;
mod time_map;
mod wide_div;

pub use camera::{CompCamera, CompCameraError};
pub use canonical::{
    CanonicalPoint, CanonicalSize, PixelPoint, PixelSize, ViewportTransform, ViewportTransformError,
};
pub use frame::{
    premultiply_rgba_f32, premultiply_rgba_u8, ColorSpace, CpuFrame, FrameDesc, FrameDescError,
    PixelFormat,
};
pub use quality::{Quality, SampleTier};
pub use time::{format_ffmpeg_seek_before_frame, Fps, FpsError, RationalTime, RationalTimeError};
pub use time_map::{OverrunMode, TimeMap, TimeMapError};
