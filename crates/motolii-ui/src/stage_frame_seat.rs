//! playhead 時刻の合成フレームを Stage へ渡す席。
//!
//! 評価は **export と同じ一本**を通る(`render_worker` = `build_document_frame_graph`
//! ＋ `render_graph_cached`)。ここで第二 render 経路を作らない(絶対規律6)。
//!
//! この席が持つのは「いつ評価し直すか」と「完成したフレームをどこへ置くか」だけ:
//!
//! - **鍵が変わった時だけ submit する**。鍵は (frame grid へ落とした playhead,
//!   出力 FrameDesc, Document snapshot の同一性)。float の playhead をそのまま
//!   鍵にすると 1 フレームの中の揺れで評価が走り続ける
//! - **完成は `DisplaySlot` へ写す**。texture は VRAM に置いたまま(絶対規律1)で、
//!   Stage は毎フレームこの1枚を見る
//! - **間に合わない時は前の絵を出し続ける**。追い越された要求は捨てるだけで、
//!   slot は最後に完成したフレームを保ったままにする(絵は落とす・音は止めない)

use std::sync::Arc;

use motolii_core::{ColorSpace, Fps, FrameDesc, PixelFormat, Quality, RationalTime};
use motolii_doc::{Document, EvaluationTime};
use motolii_eval::DataTracks;
use motolii_gpu::GpuCtx;

use crate::display_slot::DisplaySlot;
use crate::render_worker::{RenderGeneration, RenderRequest, RenderWorker, RenderWorkerStartError};

/// 出力解像度を持たない Document の既定。CLI `new_document` と
/// `static_preview::bootstrap_frame_desc` と同じ値で、**ここで新しい既定を決めない**。
const FALLBACK_RESOLUTION: (u32, u32) = (1920, 1080);

/// 再評価の合図。**これが等しい間は submit しない。**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StageFrameKey {
    pub(crate) time: RationalTime,
    pub(crate) desc: FrameDesc,
}

/// 席の実測。テストと再生中の追従の観測が見る。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct StageFrameEvidence {
    /// submit した回数。
    pub(crate) submitted: u32,
    /// 完成を受けて slot へ写した回数。
    pub(crate) accepted: u32,
    /// 前の要求がまだ返っていないうちに次を出した回数。
    /// **落ちたフレーム数ではない** — 走り始めていた要求はこの後も返る。
    pub(crate) overlapped: u32,
    /// 実行中/待機中の要求があるか。
    pub(crate) outstanding: bool,
}

impl StageFrameEvidence {
    /// 一度も出ないまま捨てられたフレームの数。
    /// 追い越された「待ち」だけがここに落ちる(走っていた要求は返って `accepted` になる)。
    pub(crate) fn dropped(self) -> u32 {
        self.submitted
            .saturating_sub(self.accepted)
            .saturating_sub(u32::from(self.outstanding))
    }
}

/// Composition が所有する出力解像度から評価用の記述子を作る。
pub(crate) fn composition_frame_desc(document: &Document) -> Option<FrameDesc> {
    let (width, height) = document
        .composition
        .resolution()
        .unwrap_or(FALLBACK_RESOLUTION);
    FrameDesc::try_packed(
        width,
        height,
        PixelFormat::Rgba8Unorm,
        ColorSpace::Srgb,
        true,
    )
    .ok()
}

/// playhead(秒)を composition の frame grid へ落とす。
///
/// export はフレーム番号で回るので、preview もここで同じ格子へ載せる
/// (Preview / Export 同一評価)。float のまま評価すると、同じ1フレームを指す
/// 微差の playhead が別の要求になり、再生中に評価が積み上がる。
pub(crate) fn snap_to_frame(
    seconds: f32,
    fps: Fps,
    duration: RationalTime,
) -> Option<RationalTime> {
    let clamped = seconds.clamp(0.0, duration.as_seconds_f64() as f32);
    let raw = RationalTime::try_new((f64::from(clamped) * 1000.0).round() as i64, 1000).ok()?;
    let frame = raw.try_to_frame_round(fps).ok()?;
    RationalTime::try_from_frame(frame, fps).ok()
}

/// 今 Stage が出すべきフレームの鍵。
pub(crate) fn stage_frame_key(document: &Document, playhead_seconds: f32) -> Option<StageFrameKey> {
    let desc = composition_frame_desc(document)?;
    let time = snap_to_frame(
        playhead_seconds,
        document.composition.fps,
        document.composition.duration,
    )?;
    Some(StageFrameKey { time, desc })
}

pub(crate) struct StageFrameSeat {
    gpu: Arc<GpuCtx>,
    worker: RenderWorker,
    submitted: Option<(StageFrameKey, Arc<Document>)>,
    pending: Option<RenderGeneration>,
    slot: Option<DisplaySlot>,
    stopped: bool,
    last_error: Option<String>,
    evidence: StageFrameEvidence,
}

impl StageFrameSeat {
    /// 席を立てる。`gpu` は **Stage を塗るのと同じ device** でなければならない
    /// — 完成 texture は Rerun の texture pool へ実 handle のまま渡すので、
    /// 別 device のものは import できない。
    pub(crate) fn spawn(gpu: Arc<GpuCtx>) -> Result<Self, RenderWorkerStartError> {
        let worker = RenderWorker::spawn(Arc::clone(&gpu))?;
        Ok(Self {
            gpu,
            worker,
            submitted: None,
            pending: None,
            slot: None,
            stopped: false,
            last_error: None,
            evidence: StageFrameEvidence::default(),
        })
    }

    /// 完成が publish された時に鳴らす合図(ホストの repaint)。
    /// 評価は別 thread なので、これが無いと入力が止まった瞬間の1枚が出ない。
    pub(crate) fn on_frame_ready(&mut self, signal: impl Fn() + Send + Sync + 'static) {
        if let Err(error) = self
            .worker
            .client()
            .register_repaint_signal(Arc::new(signal))
        {
            self.last_error = Some(error.to_string());
        }
    }

    /// playhead 時刻の合成を要求する。**鍵が変わっていなければ何もしない。**
    /// 戻り値は「今回 submit したか」。
    pub(crate) fn request(&mut self, document: &Arc<Document>, playhead_seconds: f32) -> bool {
        if self.stopped {
            return false;
        }
        let Some(key) = stage_frame_key(document, playhead_seconds) else {
            return false;
        };
        let unchanged = self
            .submitted
            .as_ref()
            .is_some_and(|(seen, seen_document)| {
                *seen == key && Arc::ptr_eq(seen_document, document)
            });
        if unchanged {
            return false;
        }
        // 前の要求がまだ返っていないうちに次を出す。待ちの1件は worker 側で
        // 置き換わって消える(= 出さずに捨てるフレーム)。捨てるのはこれから作る絵
        // だけで、slot に載っている最後の完成フレームはそのまま残す。
        if self.pending.is_some() {
            self.evidence.overlapped += 1;
        }
        match self.worker.submit(RenderRequest {
            document: Arc::clone(document),
            data_tracks: Arc::new(DataTracks::new()),
            evaluation_time: EvaluationTime::new(key.time),
            desc: key.desc,
            quality: Quality::DRAFT,
        }) {
            Ok(generation) => {
                self.submitted = Some((key, Arc::clone(document)));
                self.pending = Some(generation);
                self.evidence.submitted += 1;
                self.evidence.outstanding = true;
                true
            }
            Err(error) => {
                // 閉じた/落ちた worker へ毎フレーム submit し直さない。
                self.stopped = true;
                self.last_error = Some(error.to_string());
                false
            }
        }
    }

    /// 完成していれば受けて slot へ写す。**来ていなければ何も変えない。**
    pub(crate) fn poll(&mut self) {
        let Some(result) = self.worker.try_take_latest() else {
            return;
        };
        if self.pending == Some(result.generation) {
            self.pending = None;
            self.evidence.outstanding = false;
        }
        let rendered = match result.result {
            Ok(rendered) => rendered,
            Err(error) => {
                self.last_error = Some(error.to_string());
                return;
            }
        };
        // 寸法が同じ間は同じ texture を使い回す(毎フレーム確保しない)。
        let copied = match self.slot.as_ref() {
            Some(slot) if slot.desc() == rendered.frame.desc => {
                slot.copy(&self.gpu, &rendered.frame).map(|()| None)
            }
            _ => DisplaySlot::copy_from_rendered(&self.gpu, &rendered.frame).map(Some),
        };
        match copied {
            Ok(Some(slot)) => {
                self.slot = Some(slot);
                self.evidence.accepted += 1;
            }
            Ok(None) => self.evidence.accepted += 1,
            Err(error) => self.last_error = Some(error.to_string()),
        }
    }

    /// Stage へ渡す1枚。**完成した最新のフレーム**で、次が間に合わない間もこれが残る。
    pub(crate) fn frame(&self) -> Option<&wgpu::Texture> {
        self.slot.as_ref().map(DisplaySlot::texture)
    }

    pub(crate) fn evidence(&self) -> StageFrameEvidence {
        self.evidence
    }

    /// 溜まった理由を1度だけ返す(呼び手が言う。毎フレーム同じ行を吐かない)。
    pub(crate) fn take_error(&mut self) -> Option<String> {
        self.last_error.take()
    }
}

#[cfg(test)]
mod tests {
    use motolii_gpu::download_rgba;

    use super::*;
    use crate::static_preview::bootstrap_document;

    fn live(document: Document) -> Arc<Document> {
        Arc::new(document)
    }

    /// bootstrap fixture は 1920x1080 の緑一色 clip を1本持つ。
    fn green_project() -> Arc<Document> {
        live(bootstrap_document().expect("bootstrap document"))
    }

    #[test]
    fn the_key_uses_the_composition_resolution_and_falls_back_to_full_hd() {
        let document = bootstrap_document().expect("document");
        let desc = composition_frame_desc(&document).expect("desc");
        assert_eq!((desc.width, desc.height), (1920, 1080));
        assert_eq!(desc.format, PixelFormat::Rgba8Unorm);

        let mut sized = Document::new_current();
        sized
            .composition
            .set_resolution(Some((640, 360)))
            .expect("resolution");
        let desc = composition_frame_desc(&sized).expect("desc");
        assert_eq!((desc.width, desc.height), (640, 360));
    }

    /// 鍵は frame grid へ落ちる。**同じフレームの中の揺れでは変わらない。**
    #[test]
    fn the_key_snaps_the_playhead_to_the_frame_grid() {
        let document = bootstrap_document().expect("document");
        let fps = document.composition.fps;
        let frame = 1.0 / (fps.num() as f32 / fps.den() as f32);

        let at_zero = stage_frame_key(&document, 0.0).expect("key");
        let within = stage_frame_key(&document, frame * 0.2).expect("key");
        let next = stage_frame_key(&document, frame).expect("key");

        assert_eq!(at_zero, within, "同じフレームの中の揺れで再評価しない");
        assert_ne!(at_zero, next, "隣のフレームは別の鍵");
        assert_eq!(at_zero.time, RationalTime::ZERO);
    }

    /// **playhead と Document snapshot の変化だけ**が submit を起こす。
    #[test]
    fn only_playhead_or_revision_changes_submit_a_render() {
        let Some(gpu) = motolii_testkit::gpu_or_skip() else {
            return;
        };
        let document = green_project();
        let mut seat = StageFrameSeat::spawn(Arc::new(gpu)).expect("seat");

        assert!(seat.request(&document, 0.0), "最初の要求は submit する");
        assert!(
            !seat.request(&document, 0.0),
            "playhead も snapshot も変わらなければ submit しない"
        );
        assert!(
            !seat.request(&document, 0.001),
            "同じフレームの中では submit しない"
        );
        assert!(
            seat.request(&document, 1.0),
            "playhead が動けば submit する"
        );

        // 編集後の snapshot は別の `Arc`(app の `seat_stage_documents` が配るもの)。
        let edited = live((*document).clone());
        assert!(
            seat.request(&edited, 1.0),
            "同じ時刻でも新しい snapshot なら submit する"
        );
        assert_eq!(seat.evidence().submitted, 3);
    }

    /// 完成したフレームは slot に載り、**その中身は評価結果そのもの**。
    #[test]
    fn a_completed_frame_lands_in_the_display_slot() {
        let Some(gpu) = motolii_testkit::gpu_or_skip() else {
            return;
        };
        let gpu = Arc::new(gpu);
        let document = green_project();
        let mut seat = StageFrameSeat::spawn(Arc::clone(&gpu)).expect("seat");
        assert!(seat.frame().is_none(), "評価前は出す絵を持たない");
        assert!(seat.request(&document, 0.0));

        let texture = loop {
            seat.poll();
            if seat.frame().is_some() {
                break seat.frame().expect("frame").clone();
            }
            std::thread::yield_now();
        };

        let desc = composition_frame_desc(&document).expect("desc");
        let rendered = Quality::DRAFT.render_desc(desc);
        assert_eq!(
            (texture.width(), texture.height()),
            (rendered.width, rendered.height)
        );
        assert_eq!(texture.format(), wgpu::TextureFormat::Rgba8Unorm);
        assert_eq!(seat.evidence().accepted, 1);

        let bytes = download_rgba(&gpu, &texture).expect("download");
        assert!(
            bytes.chunks_exact(4).all(|pixel| pixel == [0, 255, 0, 255]),
            "Stage へ出すのは評価済みの合成フレームそのもの"
        );
    }

    /// **間に合わない間は前の絵を出し続ける。** 新しい要求は今出ている絵を消さない。
    #[test]
    fn a_pending_request_keeps_showing_the_last_completed_frame() {
        let Some(gpu) = motolii_testkit::gpu_or_skip() else {
            return;
        };
        let document = green_project();
        let mut seat = StageFrameSeat::spawn(Arc::new(gpu)).expect("seat");
        assert!(seat.request(&document, 0.0));
        while {
            seat.poll();
            seat.frame().is_none()
        } {
            std::thread::yield_now();
        }
        let shown = seat.evidence().accepted;
        assert_eq!(shown, 1);

        assert!(seat.request(&document, 1.0), "次の時刻を要求する");
        assert!(
            seat.frame().is_some(),
            "完成するまでは最後に完成したフレームを出し続ける"
        );
        assert!(seat.request(&document, 2.0));
        assert!(
            seat.evidence().overlapped >= 1,
            "返る前に出した要求は追い越しとして数える"
        );
        assert!(
            seat.frame().is_some(),
            "落としたのは待たせた絵であって、出ている絵ではない"
        );
        assert_eq!(
            seat.evidence().accepted,
            shown,
            "受け取っていないフレームで slot を書き換えない"
        );
    }

    /// 時刻ごとに別の色になる Document。追従の判定に使う。
    fn two_shot_project() -> Arc<Document> {
        use motolii_core::TimeMap;
        use motolii_doc::{
            Clip, ClipSource, DocParam, ItemEnvelope, Track, TrackItem, RECT_LAYER_SOURCE,
        };
        use std::collections::BTreeMap;

        let mut document = Document::new_current();
        let track = document.track_ids.allocate("V1").expect("track");
        let one_second = RationalTime::from_seconds(1);
        let mut items = Vec::new();
        for (index, (start, color)) in [
            (RationalTime::ZERO, [1.0, 0.0, 0.0, 1.0]),
            (one_second, [0.0, 0.0, 1.0, 1.0]),
        ]
        .into_iter()
        .enumerate()
        {
            let layer = document
                .layers
                .allocate(&format!("shot-{index}"))
                .expect("layer");
            items.push(TrackItem::Clip(Clip {
                envelope: ItemEnvelope::new(layer),
                start,
                duration: one_second,
                time_map: TimeMap::identity(),
                source: ClipSource::Plugin {
                    plugin_id: RECT_LAYER_SOURCE.into(),
                    effect_version: 1,
                    params: BTreeMap::from([
                        ("center".into(), DocParam::const_vec2([0.0, 0.0])),
                        ("size".into(), DocParam::const_vec2([4.0, 4.0])),
                        ("color".into(), DocParam::const_color(color)),
                    ]),
                    extra: Default::default(),
                },
            }));
        }
        document.tracks.push(Track { id: track, items });
        document.composition.duration = RationalTime::from_seconds(2);
        document.validate().expect("two shot document");
        live(document)
    }

    fn wait_for_frame(seat: &mut StageFrameSeat, gpu: &GpuCtx) -> Vec<u8> {
        let accepted = seat.evidence().accepted;
        loop {
            seat.poll();
            if seat.evidence().accepted > accepted {
                break;
            }
            std::thread::yield_now();
        }
        download_rgba(gpu, seat.frame().expect("frame")).expect("download")
    }

    /// **playhead を動かすと Stage に出る絵が変わる。** これが「見たまま出る」面。
    #[test]
    fn the_frame_follows_the_playhead() {
        let Some(gpu) = motolii_testkit::gpu_or_skip() else {
            return;
        };
        let gpu = Arc::new(gpu);
        let document = two_shot_project();
        let mut seat = StageFrameSeat::spawn(Arc::clone(&gpu)).expect("seat");

        assert!(seat.request(&document, 0.0));
        let first = wait_for_frame(&mut seat, &gpu);
        assert!(seat.request(&document, 1.5));
        let second = wait_for_frame(&mut seat, &gpu);

        assert_eq!(first.chunks_exact(4).next(), Some(&[255, 0, 0, 255][..]));
        assert_eq!(second.chunks_exact(4).next(), Some(&[0, 0, 255, 255][..]));
        assert_ne!(first, second, "playhead 時刻の合成結果が出ていない");
    }

    /// 再生の速さで playhead を進めたときの追従。**間に合わなくても絵は消えない**
    /// (落ちるのは待たせた要求だけ)。実測値は `--nocapture` で見る。
    #[test]
    fn playback_never_blanks_the_stage_even_when_renders_are_dropped() {
        let Some(gpu) = motolii_testkit::gpu_or_skip() else {
            return;
        };
        let document = two_shot_project();
        let mut seat = StageFrameSeat::spawn(Arc::new(gpu)).expect("seat");

        // 60fps 相当で2秒ぶん進める。UI thread が毎フレームやることと同じ順序。
        let mut blanked_after_first = 0;
        let mut first_seen = false;
        for tick in 0..120 {
            seat.request(&document, tick as f32 / 60.0);
            seat.poll();
            match seat.frame() {
                Some(_) => first_seen = true,
                None if first_seen => blanked_after_first += 1,
                None => {}
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        let evidence = seat.evidence();
        println!(
            "playback follow: {evidence:?} dropped={} ",
            evidence.dropped()
        );
        assert!(first_seen, "再生中に1枚も出ていない");
        assert_eq!(
            blanked_after_first, 0,
            "間に合わない要求で出ている絵を消してはいけない"
        );
        assert!(
            evidence.accepted <= evidence.submitted,
            "受けた数が要求数を超えることはない"
        );
        assert!(
            evidence.dropped() <= evidence.overlapped,
            "落ちるのは追い越された待ちだけ(走り始めた要求は返る)"
        );
    }

    /// 出す絵が無い間は Stage へ何も渡さない(fixture 展示が従来のまま残る側)。
    #[test]
    fn a_seat_without_a_completed_frame_hands_nothing_to_the_stage() {
        let Some(gpu) = motolii_testkit::gpu_or_skip() else {
            return;
        };
        let seat = StageFrameSeat::spawn(Arc::new(gpu)).expect("seat");
        assert!(seat.frame().is_none());
        assert_eq!(seat.evidence(), StageFrameEvidence::default());
        assert_eq!(seat.evidence().dropped(), 0);
    }
}
