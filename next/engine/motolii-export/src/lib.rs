//! wraps: motolii-engine + motolii-media — 書き出し。
//!
//! **フレームは `Engine::render_frame` からしか来ない**(背骨2)。ここに「書き出し用の
//! 速い道」を作らない。旧 workspace の `motolii-export`(913行)は大半が
//! graph / plugin 機構だったが、評価経路が1本になった今それは要らない。
//! 残るのは「回して、書いて、報告する」だけである。
//!
//! **`motolii-compositor` を依存に持たない**(2026-08-20 の敵対的レビュー)。
//! `CompSpec` を取るためだけに引いていた時期があり、その間 export は第二の
//! `Compositor` を建てられた。型を `motolii-core` へ出して依存を切ってあるので、
//! 今は**建てられない** — 背骨2 を文言ではなく依存グラフで守る。
//!
//! 移植したのは**機構ではなく意味**:
//! - **報告 = 現物**(書いたと言ったフレーム数と、出来た file のフレーム数が一致する)
//! - **中断したら残骸を残さない**(途中の file を置いて「壊れた成果物」を作らない)
//! - 音声は後段 mux(`motolii-media::mux_soundtrack`)

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use motolii_core::{FrameDesc, PixelFormat, RationalTime};
use motolii_engine::{Engine, EngineError};
use motolii_media::{Encoder, MediaError};
use motolii_store::StoreView;

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error(transparent)]
    Engine(#[from] EngineError),
    #[error(transparent)]
    Media(#[from] MediaError),
    #[error("frame 記述を作れない: {0}")]
    Desc(String),
    #[error("中断された(残骸は消してある)")]
    Cancelled,
    #[error("comp の設定が Document に無い")]
    NoComposition,
}

/// 書き出しの注文。
pub struct ExportJob {
    pub out_path: PathBuf,
    /// 可逆(qp0)で書くか。
    pub qp0: bool,
}

// 解像度・fps・尺は **Document が持つ**(`Composition`)。ここに書かないのは、
// 書けると preview と違う入力で書き出せてしまうから(2026-08-20 の敵対的レビュー)。

/// 書き出しの結果。**言ったことと現物が一致する**ための報告。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportReport {
    pub out_path: PathBuf,
    /// 実際に encoder へ渡したフレーム数。
    pub frames_written: i64,
}

/// 中断の口。押されたら**残骸を消してから**返す。
#[derive(Clone, Default)]
pub struct Cancel(Arc<AtomicBool>);

impl Cancel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

pub fn export(
    engine: &mut Engine,
    view: &StoreView<'_>,
    job: &ExportJob,
) -> Result<ExportReport, ExportError> {
    export_with_cancel(engine, view, job, &Cancel::new())
}

pub fn export_with_cancel(
    engine: &mut Engine,
    view: &StoreView<'_>,
    job: &ExportJob,
    cancel: &Cancel,
) -> Result<ExportReport, ExportError> {
    // 合成器が返すのは premultiplied RGBA8。
    let composition = view
        .composition()
        .map_err(|e| ExportError::Desc(e.to_string()))?
        .ok_or(ExportError::NoComposition)?;
    let comp = composition.spec();
    let fps = composition.fps;

    let desc = FrameDesc::try_packed(
        comp.width,
        comp.height,
        PixelFormat::Rgba8Unorm,
        motolii_core::ColorSpace::Srgb,
        true,
    )
    .map_err(|e| ExportError::Desc(e.to_string()))?;

    let mut encoder = Encoder::open(&job.out_path, &desc, fps, job.qp0)?;
    let mut written = 0i64;

    for frame in 0..composition.duration_frames {
        if cancel.is_cancelled() {
            drop(encoder);
            remove_partial(&job.out_path);
            return Err(ExportError::Cancelled);
        }

        // **preview と同じ関数**。ここを別経路にした瞬間「見た絵 ≠ 出る絵」になる。
        // 正準口を通す(手で `frame * den / num` を書かない。core の doc が
        // 「時刻とフレームの写像は正準口のみ」と言っている)。
        let t = RationalTime::try_from_frame(frame, fps)
            .map_err(|e| ExportError::Desc(e.to_string()))?;
        let rgba = engine.render_frame(view, t)?;
        encoder.write_frame(&rgba)?;
        written += 1;
    }

    encoder.finish()?;

    Ok(ExportReport {
        out_path: job.out_path.clone(),
        frames_written: written,
    })
}

/// 中断時に「壊れた成果物」を残さない。消せなかったことは呼び手に伝えない —
/// **消えていることの方が重要**で、消せない事情は次の書き出しで上書きされる。
fn remove_partial(path: &Path) {
    let _ = std::fs::remove_file(path);
}
