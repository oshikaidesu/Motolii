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
//!
//! 全範囲書き出し(`export`/`export_with_cancel`/`export_with_progress`)は
//! 「範囲(半開 `start..end`)書き出し」(`export_range`/`export_range_with_cancel`/
//! `export_range_with_progress`)の特殊形として実装する(`0..duration_frames`
//! を渡すだけ) — 実ループは [`export_range_with_progress`] の1本だけで、
//! 進捗コールバックなしの口(`export_range_with_cancel` 含む)はそこへ
//! 無視コールバック(`|_| {}`)で委譲する。2本目のループを持たない。単一フレーム
//! → PNG の静止画書き出しは `export_still`(alpha の既知限界は同関数の doc 参照)。
//!
//! ## 進捗コールバック([`ExportProgress`])
//! `export_range_with_progress` はフレームを1本書くたびに `on_progress` を
//! **同期呼び出し**する(`FnMut` — 呼ぶたびに UI 側の状態を書き換えられる形)。
//! 中断チェック(`Cancel`)と進捗報告は**同じループの同じ点**(1フレーム処理の
//! 境界)で見ている — 中断がフレーム境界で効くなら、進捗もフレーム境界で刻む、
//! という揃え。
//!
//! ## shell(UI)側の非ブロッキング化への示唆(この crate が決められる範囲)
//! この crate は**スレッドも async ランタイムも持ち込まない**(依存を増やさない
//! ことが背骨2 に沿う設計)。`export_range_with_progress` 自体は最後まで
//! 同期・ブロッキングのまま呼び出し元のスレッドを占有する。UI を固まらせない
//! 形にするのは呼び手(shell)の責務で、この crate が用意したのは「フレーム毎に
//! 制御が一度 `on_progress` へ返る点」だけである。妥当な選択肢:
//! - **別スレッドで回す**: `std::thread::spawn` で本関数を呼び、`on_progress`
//!   は `std::sync::mpsc::Sender` 等で結果を UI スレッドへ送るだけにする
//!   (`Engine`/`StoreView` を跨がせる Send 境界の確認は呼び手側の仕事)。
//!   `iced::Task::perform`/`Task::run` から spawn_blocking 相当を使うのが
//!   最短経路。
//!   キャンセルは `Cancel` を `Clone` して両スレッドに持たせ、UI 側の
//!   `Message::CancelExport` から `.cancel()` を呼べば次のフレーム境界で
//!   止まる(ループ側の実装は変えなくてよい)。
//! - フレーム毎に yield する非同期版(`async fn`化)は要求していない —
//!   1フレームの描画+encode 自体は短く、スレッド1本に丸ごと逃がす方が
//!   この crate の「フレームは `Engine::render_frame` からしか来ない」という
//!   単純さを壊さない。

use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use motolii_core::{FrameDesc, PixelFormat, RationalTime};
use motolii_engine::{Engine, EngineError};
use motolii_media::{Encoder, MediaError};
use motolii_store::StoreView;

mod lottie;
pub use lottie::{export_lottie, LottieExport, LottieExportError, UnsupportedForLottie};

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
    #[error("静止画の書き出しに失敗: {0}")]
    Still(String),
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

/// フレーム毎の進捗報告。`export_range_with_progress` が1フレーム書き終える
/// たびに渡す — UI 側は `frames_done as f64 / frames_total as f64` で割合を
/// 出せる(`frames_total <= 0` の防波堤は呼び手側、`motolii-export-pane` の
/// `progress_fraction` と同じ規約)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExportProgress {
    /// encoder へ渡し終えたフレーム数(`ExportReport::frames_written` と同じ数え方)。
    pub frames_done: i64,
    /// この呼び出しが書く予定の総フレーム数(半開区間の長さ。`start >= end` なら 0)。
    pub frames_total: i64,
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

/// **署名不変**(既存呼び手互換)。中身は「comp の全区間」を
/// [`export_range_with_cancel`] へ委譲するだけ — 全範囲書き出しは範囲書き出しの
/// 特殊形であって、別のループを持たない。
pub fn export_with_cancel(
    engine: &mut Engine,
    view: &StoreView<'_>,
    job: &ExportJob,
    cancel: &Cancel,
) -> Result<ExportReport, ExportError> {
    let duration_frames = composition_duration_frames(view)?;
    export_range_with_cancel(engine, view, job, 0..duration_frames, cancel)
}

/// [`export_range_with_cancel`] の中断口なし版(comp の全範囲書き出しに対する
/// [`export`] と同じ関係)。
pub fn export_range(
    engine: &mut Engine,
    view: &StoreView<'_>,
    job: &ExportJob,
    range: Range<i64>,
) -> Result<ExportReport, ExportError> {
    export_range_with_cancel(engine, view, job, range, &Cancel::new())
}

/// 範囲(半開 `start..end`)を指定した書き出し。**フレーム範囲の型は export 側の
/// 自前**(`std::ops::Range<i64>`) — comp 側へ依存を逆向きに張らない。
///
/// `range` が空(`start >= end`)なら0フレーム書いた成功として報告する
/// (`frames_written == 0`)。「報告 = 現物」を守るのはループの中身であって、
/// 範囲そのものの妥当性検査ではない。
///
/// **署名不変**(既存呼び手互換)。中身は [`export_range_with_progress`] へ
/// 無視コールバック(`|_| {}`)で委譲するだけ — 実ループは1本しか持たない。
pub fn export_range_with_cancel(
    engine: &mut Engine,
    view: &StoreView<'_>,
    job: &ExportJob,
    range: Range<i64>,
    cancel: &Cancel,
) -> Result<ExportReport, ExportError> {
    export_range_with_progress(engine, view, job, range, cancel, |_| {})
}

/// [`export_with_cancel`] の進捗コールバック版。**署名不変**の呼び手互換を
/// 保つため既存口は変えず、こちらは新設の口(comp の全区間を
/// [`export_range_with_progress`] へ委譲する — 全範囲書き出しは範囲書き出しの
/// 特殊形であって、別のループを持たない)。
pub fn export_with_progress(
    engine: &mut Engine,
    view: &StoreView<'_>,
    job: &ExportJob,
    cancel: &Cancel,
    on_progress: impl FnMut(ExportProgress),
) -> Result<ExportReport, ExportError> {
    let duration_frames = composition_duration_frames(view)?;
    export_range_with_progress(engine, view, job, 0..duration_frames, cancel, on_progress)
}

/// [`export_range_with_cancel`] の進捗コールバック版 — **実ループはここだけ**。
/// 他の全 export 系口(`export`/`export_with_cancel`/`export_with_progress`/
/// `export_range`/`export_range_with_cancel`)はこの関数へ委譲する。
///
/// `on_progress` はフレームを1本 encoder へ渡し終えるたびに1回、同期呼び出しで
/// 呼ぶ(`FnMut` — 呼ぶたびに値を返さない、UI 側の状態を書き換えるための口)。
/// **中断チェックと同じループの同じ点**(1フレーム処理の境界)で見ているので、
/// 「どこまで進んだところで止まったか」は進捗の最後の値と一致する。
///
/// `range` が空(`start >= end`)なら `frames_total == 0` で `on_progress` は
/// 一度も呼ばれず、0フレーム書いた成功として報告する — 既存の空範囲契約と同じ。
pub fn export_range_with_progress(
    engine: &mut Engine,
    view: &StoreView<'_>,
    job: &ExportJob,
    range: Range<i64>,
    cancel: &Cancel,
    mut on_progress: impl FnMut(ExportProgress),
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

    let frames_total = (range.end - range.start).max(0);
    let mut encoder = Encoder::open(&job.out_path, &desc, fps, job.qp0)?;
    let mut written = 0i64;

    for frame in range {
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
        on_progress(ExportProgress {
            frames_done: written,
            frames_total,
        });
    }

    encoder.finish()?;

    Ok(ExportReport {
        out_path: job.out_path.clone(),
        frames_written: written,
    })
}

/// 単一フレーム → PNG の薄い口。**動画書き出しと同じ評価経路
/// (`Engine::render_frame`)を再利用する** — 静止画専用の速い道は作らない
/// (背骨2 と同じ精神)。
///
/// **alpha の既知限界(裁定16)**: `render_frame` が返すのは premultiplied
/// RGBA8。PNG は straight(non-premultiplied)alpha を前提にする形式なので、
/// この口はその値を**そのまま**書く — premultiplied のバイト列を straight
/// alpha PNG として書くのは、comp の背景が不透明(alpha=1 が敷き詰められる
/// 通常の書き出し)なら実害が無いが、comp 背景が透明で覆われていない画素が
/// 残る場合、その画素の RGB は「本来より暗い値」のまま alpha 付きで出る
/// (un-premultiply していないため)。動画書き出しがそもそも alpha を運べない
/// のと同根の限界であり、この口を直す(un-premultiply する)のは範囲外 —
/// 直す口は裁定16 が特定した fork 2箇所と同じ並びで別途扱う。
pub fn export_still(
    engine: &mut Engine,
    view: &StoreView<'_>,
    frame: i64,
    out_path: &Path,
) -> Result<ExportReport, ExportError> {
    let composition = view
        .composition()
        .map_err(|e| ExportError::Desc(e.to_string()))?
        .ok_or(ExportError::NoComposition)?;
    let comp = composition.spec();
    let fps = composition.fps;

    let t = RationalTime::try_from_frame(frame, fps)
        .map_err(|e| ExportError::Desc(e.to_string()))?;
    // **preview/動画書き出しと同じ関数**(上の doc 参照)。
    let rgba = engine.render_frame(view, t)?;

    image::save_buffer(out_path, &rgba, comp.width, comp.height, image::ColorType::Rgba8)
        .map_err(|e| ExportError::Still(e.to_string()))?;

    Ok(ExportReport {
        out_path: out_path.to_path_buf(),
        frames_written: 1,
    })
}

/// comp の尺(フレーム数)だけを読む。[`export_with_cancel`] が「全範囲」を
/// [`export_range_with_cancel`] へ言い換えるための下ごしらえ。
fn composition_duration_frames(view: &StoreView<'_>) -> Result<i64, ExportError> {
    let composition = view
        .composition()
        .map_err(|e| ExportError::Desc(e.to_string()))?
        .ok_or(ExportError::NoComposition)?;
    Ok(composition.duration_frames)
}

/// 中断時に「壊れた成果物」を残さない。消せなかったことは呼び手に伝えない —
/// **消えていることの方が重要**で、消せない事情は次の書き出しで上書きされる。
fn remove_partial(path: &Path) {
    let _ = std::fs::remove_file(path);
}
