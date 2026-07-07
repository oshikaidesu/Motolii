//! oc-core: 全クレート共通の語彙(時間型・フレーム記述子)。
//!
//! 仕様: docs/specs/M1-vertical-slice.md「インターフェース契約」

mod frame;
mod time;

pub use frame::{ColorSpace, CpuFrame, FrameDesc, PixelFormat};
pub use time::{Fps, RationalTime};
