//! 再生の音声座席: soundtrack を実 device で鳴らし、playhead を audio clock に貼り付ける。
//!
//! クロックの正本は `motolii-transport` の `Transport`(供給済みサンプル数)であり、
//! **ここで時刻の補間を発明しない** — `next_frame_plan()` が返す聴感時刻を
//! そのまま playhead(秒)へ写すだけである。
//!
//! soundtrack が無い project は従来どおり壁時計(`advance_playhead`)で進む。
//! soundtrack があるのに鳴らせない(program が組めない / device が開けない)ときは
//! **黙って無音で成功した顔をしない** — `WallClock` へ明示的に落ち、理由を
//! status に出す(`WallClockReason::AudioUnavailable`)。
//!
//! 座席の寿命 = 再生の寿命。`playing` が立っているあいだだけ `AudioSeat` が
//! `PlaybackSession`(cpal stream + mix thread)を抱え、pause / スクラブ / 終端で
//! drop して device を手放す。シーク(playhead の移動・ループの折り返し)は
//! `reseek` で **session を開き直す**(`MixProducer` は `start_frame` から前へ
//! 流すだけなので、途中参加は開き直しが正規の形である)。

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use motolii_audio::{AudioProgram, PcmCache, CANONICAL_SAMPLE_RATE};
use motolii_core::{Fps, Quality};
use motolii_doc::Document;
use motolii_transport::{PlaybackSession, PlaybackSessionError, Transport, TransportError};

/// 壁時計再生になっている理由。status の文言と「黙らない」判定の分岐がここで割れる。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WallClockReason {
    /// soundtrack が無い。**従来の無音再生のままが正しい**ので status は変えない。
    NoSoundtrack,
    /// soundtrack はあるのに鳴らせない。理由を status に出す。
    AudioUnavailable(String),
}

/// 再生1回ぶんの clock の形。`playing` のあいだだけ editor が `Some` で持つ。
pub(crate) enum AudioPlayback {
    /// 壁時計(従来の `advance_playhead`)。
    WallClock(WallClockReason),
    /// 実 device 再生。playhead は `Transport` の聴感時刻に同期する。
    Synced(AudioSeat),
}

/// 実 device の座席。drop で cpal stream / mix thread ごと手放す
/// (`PlaybackSession` の寿命設計に従い、停止中に device を握り続けない)。
pub(crate) struct AudioSeat {
    session: PlaybackSession,
    /// reseek(開き直し)のために同じ program を持ち続ける。decode し直さない。
    program: Arc<AudioProgram>,
    /// audio が最後に書いた playhead。UI がこれと違う値にしていたら
    /// 「手で動かした」印であり、reseek の合図になる。
    last_synced: f32,
}

impl AudioSeat {
    /// `playhead`(秒)から default device で鳴らし始める。
    pub(crate) fn open(
        program: Arc<AudioProgram>,
        playhead: f32,
        fps: Fps,
    ) -> Result<Self, PlaybackSessionError> {
        let session = PlaybackSession::open_default(
            Arc::clone(&program),
            playhead_to_start_frame(playhead),
            fps,
            // Preview の既定品質。DRS は映像レンダ計測が来てから効くもので、
            // ここ(音+playhead だけ)では実質関与しない。
            Quality::DRAFT,
            None,
        )?;
        Ok(Self {
            session,
            program,
            last_synced: playhead,
        })
    }

    /// audio clock で1フレームぶん進んだ playhead と、まだ再生中か。
    pub(crate) fn follow(&mut self, comp: f32) -> Result<(f32, bool), TransportError> {
        let step = follow_audio_clock(self.session.transport_mut(), comp)?;
        self.last_synced = step.0;
        Ok(step)
    }

    /// 別の時刻から鳴らし直す。旧 session を先に手放してから開く
    /// (同じ device を2本同時に握らない)。
    pub(crate) fn reseek(self, playhead: f32, fps: Fps) -> Result<Self, PlaybackSessionError> {
        let Self {
            session, program, ..
        } = self;
        drop(session);
        Self::open(program, playhead, fps)
    }

    pub(crate) fn last_synced(&self) -> f32 {
        self.last_synced
    }
}

/// 再生開始の分岐を1か所に畳む: soundtrack 無し → 壁時計、
/// program / device が開けない → 理由付き壁時計、開けた → 同期再生。
pub(crate) fn open_playback(
    document: &Document,
    project_root: Option<&Path>,
    caches: &mut HashMap<(String, u32), Arc<PcmCache>>,
    playhead: f32,
    fps: Fps,
) -> AudioPlayback {
    if document.soundtrack.is_none() {
        // 従来の無音再生。device に触りにも行かない。
        return AudioPlayback::WallClock(WallClockReason::NoSoundtrack);
    }
    let program = match AudioProgram::from_document(document, project_root, caches) {
        Ok(program) => Arc::new(program),
        Err(error) => {
            return AudioPlayback::WallClock(WallClockReason::AudioUnavailable(format!(
                "soundtrack: {error}"
            )))
        }
    };
    match AudioSeat::open(program, playhead, fps) {
        Ok(seat) => AudioPlayback::Synced(seat),
        Err(error) => {
            AudioPlayback::WallClock(WallClockReason::AudioUnavailable(format!("device: {error}")))
        }
    }
}

/// `Transport` の聴感時刻を playhead(秒)へ写す。終端で止まる(巻き戻さない)
/// — 壁時計側の `advance_playhead` と同じ出口の形。
pub(crate) fn follow_audio_clock(
    transport: &mut Transport,
    comp: f32,
) -> Result<(f32, bool), TransportError> {
    let plan = transport.next_frame_plan()?;
    let at = plan.timeline_time.as_seconds_f64() as f32;
    if at >= comp {
        Ok((comp, false))
    } else {
        Ok((at, true))
    }
}

/// UI が playhead を動かしたか。audio が書いた値は bit そのままで戻ってくるので、
/// 厳密比較でよい(閾値を発明しない)。
pub(crate) fn playhead_moved(current: f32, last_synced: f32) -> bool {
    current != last_synced
}

/// playhead(秒)→ 正準サンプルフレーム(48kHz)。`PlaybackSession` の `start_frame`。
pub(crate) fn playhead_to_start_frame(playhead: f32) -> u64 {
    (playhead.max(0.0) as f64 * CANONICAL_SAMPLE_RATE as f64).round() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_audio::{program_from_sources, MixSource};
    use motolii_core::RationalTime;
    use motolii_core::TimeMap;
    use motolii_doc::{AudioOutOfRange, DocParam};
    use motolii_transport::test_transport_headless;

    /// **playhead は供給済みサンプル数から来る。壁時計 dt は式に入らない。**
    /// 供給が止まっていれば、何度呼んでも同じ時刻に居る。
    #[test]
    fn the_playhead_follows_supplied_frames_not_wall_time() {
        let mut transport = test_transport_headless(48_000, false);
        let counters = Arc::clone(transport.counters());

        counters.advance_supplied_for_simulation(24_000); // 0.5s
        let (at, keep) = follow_audio_clock(&mut transport, 16.0).expect("clock");
        assert!((at - 0.5).abs() < 1e-6, "0.5s供給 → 0.5s: {at}");
        assert!(keep);

        // 供給が進まない = 時間も進まない(壁時計なら進んでしまうところ)
        let (again, keep) = follow_audio_clock(&mut transport, 16.0).expect("clock");
        assert_eq!(again, at, "供給0で進んではいけない");
        assert!(keep);
    }

    /// **聴感時刻 = 供給済み − デバイス待ち。** リングの充填は引かない
    /// (その規約は transport 側の正本テストが持つ。ここは写像の確認だけ)。
    #[test]
    fn the_playhead_subtracts_the_device_wait() {
        let mut transport = test_transport_headless(48_000, false);
        let counters = Arc::clone(transport.counters());

        counters.advance_supplied_for_simulation(48_000); // 1.0s供給
        transport.device_wait().set_wait_frames(4_800); // 0.1s はまだ device の中
        let (at, keep) = follow_audio_clock(&mut transport, 16.0).expect("clock");
        assert!((at - 0.9).abs() < 1e-6, "聴こえているのは0.9sまで: {at}");
        assert!(keep);
    }

    /// **composition の終端で止まる。** 巻き戻さない(壁時計側と同じ出口)。
    #[test]
    fn the_audio_clock_stops_at_the_composition_end() {
        let mut transport = test_transport_headless(48_000, false);
        let counters = Arc::clone(transport.counters());

        counters.advance_supplied_for_simulation(96_000); // 2.0s供給
        let (at, keep) = follow_audio_clock(&mut transport, 1.0).expect("clock");
        assert_eq!(at, 1.0, "終端を越えない");
        assert!(!keep, "終端で止まる");
    }

    /// **UI が動かした playhead は厳密比較で見つかる。** audio が書いた値は
    /// bit そのまま戻るので、同じ値なら動かしていない。
    #[test]
    fn a_moved_playhead_asks_for_a_reseek() {
        assert!(!playhead_moved(0.5, 0.5));
        assert!(playhead_moved(0.0, 0.5), "to_start で 0 へ跳んだ");
        assert!(playhead_moved(0.5 + f32::EPSILON, 0.5));
    }

    /// 秒 → 正準48kHzサンプルフレーム。session の `start_frame` の単位。
    #[test]
    fn seconds_map_onto_canonical_sample_frames() {
        assert_eq!(playhead_to_start_frame(0.0), 0);
        assert_eq!(playhead_to_start_frame(0.5), 24_000);
        assert_eq!(playhead_to_start_frame(-1.0), 0, "負の時刻は頭へ");
    }

    /// **soundtrack が無い project は壁時計のまま。** device に触りにも行かない
    /// (このテストは device の無い CI でも決定的に通る)。
    #[test]
    fn a_project_without_a_soundtrack_stays_on_the_wall_clock() {
        let (document, _names) = crate::timeline_editor::lab_fixture();
        assert!(document.soundtrack.is_none(), "fixture は soundtrack を持たない前提");
        let mut caches = HashMap::new();
        let fps = document.composition.fps;
        match open_playback(&document, None, &mut caches, 0.0, fps) {
            AudioPlayback::WallClock(WallClockReason::NoSoundtrack) => {}
            AudioPlayback::WallClock(WallClockReason::AudioUnavailable(reason)) => {
                panic!("soundtrack 無しは『鳴らせない』ではない: {reason}")
            }
            AudioPlayback::Synced(_) => panic!("鳴らすものが無いのに device を開いた"),
        }
    }

    /// 実 device 再生: session の origin が playhead に一致し、reseek で移る。
    /// device が無い環境では skip(`playback_shared.rs` と同じ soft gate)。
    #[test]
    fn a_real_device_session_starts_and_reseeks_at_the_playhead() {
        let silence = Arc::new(
            PcmCache::from_interleaved(
                vec![0.0f32; (CANONICAL_SAMPLE_RATE as usize) * 2 * 2], // 2s stereo
                motolii_audio::canonical_format(),
            )
            .expect("cache"),
        );
        let program = Arc::new(program_from_sources(
            vec![MixSource {
                pcm: silence,
                timeline_start: RationalTime::ZERO,
                timeline_duration: RationalTime::from_seconds(2),
                time_map: TimeMap::constant_speed(RationalTime::ZERO, 1, 1).expect("time map"),
                gain: DocParam::const_f64(1.0),
                out_of_range: AudioOutOfRange::Silence,
                enabled: true,
            }],
            1.0,
            RationalTime::from_seconds(2),
        ));
        let fps = Fps::try_new(30, 1).expect("fps");

        // device が開けない環境(ヘッドレスCI)は判定対象外
        let Ok(mut seat) = AudioSeat::open(Arc::clone(&program), 1.0, fps) else {
            return;
        };
        let (at, keep) = seat.follow(2.0).expect("clock");
        assert!(at >= 1.0 - 1e-4, "origin は playhead: {at}");
        assert!(keep);

        std::thread::sleep(std::time::Duration::from_millis(150));
        let (later, _) = seat.follow(2.0).expect("clock");
        assert!(later > at, "実 device の callback で clock が進む: {at} → {later}");

        let mut seat = seat.reseek(0.25, fps).expect("reseek");
        let (at, keep) = seat.follow(2.0).expect("clock");
        assert!(at >= 0.25 - 1e-4, "reseek 後の origin: {at}");
        assert!(at < 1.0, "reseek で古い位置を引きずらない: {at}");
        assert!(keep);
    }
}
