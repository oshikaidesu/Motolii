//! owns: keyframe 補間(Hold / Linear / cubic-bezier)と、区間を壊さない分割。
//!       rerun の latest-at は **step 補間しか持たない**(R0-3 実測)ので、
//!       「comp 時刻 → 値」は上流に無い。AM式高度イージングの区間補間はここが正本。
//!
//! OWNS-JUSTIFICATION(B): 探した相手は **rerun の latest-at クエリ**で、**step 補間しか
//!       持たない**ことを R0-3 で**実測**した(上記)。キーフレームのイージング自体は
//!       AE/Lottie から借りた語彙であって Motolii の意見ではない — 借りたい実装が
//!       上流に無いので持っている、という (B) の形。なお区間を壊さない bezier 分割は
//!       意見4(決定性)が要求する不変量でもある(同じ `t` から必ず同じ値)。
//!
//! 旧 workspace `crates/motolii-eval` からの移植(2026-08-20 リセット)。再実装ではない。
//! bezier 分割の不変量と DataTrack 参照は移植元のまま。
//!
//! **落としたもの**(2026-08-20、軸4「保守をしたくない」):
//! `ParamSource` / `DataTracks` / `DataTrackId`。`next/` から未使用で、しかも
//! `ParamSource` は旧 workspace の `ParamDriverPlugin`(4口の1つ)の形そのものだった。
//! **口の形だけが先に入っているのは、口を決めたことにならない**(裁定13 は
//! 「2人目の利用者が現れるまで決めない」)。要る日に旧 workspace から取る。
//!
//! motolii-eval: パラメータ評価エンジン。
//!
//! 「時刻t → 値」の純関数として評価する。

mod bezier;
mod track;
mod value;

pub use bezier::cubic_bezier_ease;
pub use track::{Interp, Keyframe, KeyframeTrack, SpatialTangent, TrackError};
pub use value::{Path, PathVertex, Value};
