//! owns: 有理数フレーム時刻(`RationalTime`)と frame 記述。rerun の `TimeInt` は
//!       i64 + TimeType(sequence/duration/timestamp)であって、30000/1001 のような
//!       有理 fps を正確に持てない。映像編集はここを落とせないので Motolii が持つ。
//!
//! 旧 workspace `crates/motolii-core` からの移植(2026-08-20 リセット)。再実装ではない。
//!
//! **持ってきたが落とした物**(同日、「スクラッチは最低限・保守はしたくない」の裁定):
//! `camera.rs` 561行 / `canonical.rs` 169行 / `time_map.rs` 375行 / `quality.rs` 151行
//! = 1,256行。`next/` から参照が0で、使わないコードを抱えると (a) 保守対象になり
//! (b) `check.sh` が `owns:` の重さとして数えてしまう(自前実装の量を偽る)。
//! **要る日に旧 workspace から持ってくればよい** — `crates/motolii-core/src/` に在る。
//! `camera.rs` は裁定14 で上流の `Projection::Orthographic` へ寄せた関心事なので、
//! そもそも要らない公算が高い。
//!
//! motolii-core: 全クレート共通の語彙(時間型・フレーム記述子・正準座標)。
//!
//! 仕様: docs/specs/M1-vertical-slice.md「インターフェース契約」

mod camera;
mod frame;
mod time;
mod wide_div;

pub use camera::{
    camera_projection, camera_screen_from_world_at_z, camera_screen_from_world_z0,
    distance_from_camera, CameraProjection, ResolvedCamera, CAMERA_BASE_VERTICAL_FOV_DEGREES,
};
pub use frame::{CompSpec, LayerPlacement,
    premultiply_rgba_f32, premultiply_rgba_u8, ColorSpace, CpuFrame, FrameDesc, FrameDescError,
    PixelFormat,
};
pub use time::{format_ffmpeg_seek_before_frame, Fps, FpsError, RationalTime, RationalTimeError};
