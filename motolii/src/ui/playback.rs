use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::doc::store::{Document, Fps, RationalTime};
use crate::render::audio::{AudioProgram, AudioProgramCache, PlaybackSession, WaveformTrack};

/// Compositionが持つ時間の物差し。UIは秒とframeを自分で換算しない。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct CompositionTimebase {
    fps: Fps,
    duration_frames: i64,
}

impl CompositionTimebase {
    pub(super) fn from_document(doc: &Document) -> Option<Self> {
        let composition = doc.view().composition().ok().flatten()?;
        Some(Self {
            fps: composition.fps,
            duration_frames: composition.duration_frames.max(0),
        })
    }

    fn ordinary_default(duration_sec: f64) -> Self {
        let fps = Fps::try_new(30, 1).expect("30fps is valid");
        Self {
            fps,
            duration_frames: (duration_sec.max(0.0) * 30.0).round() as i64,
        }
    }

    pub(super) fn fps(&self) -> Fps {
        self.fps
    }

    #[cfg(test)]
    pub(super) fn duration_frames(&self) -> i64 {
        self.duration_frames
    }

    pub(super) fn duration_sec(&self) -> f64 {
        RationalTime::try_from_frame(self.duration_frames, self.fps)
            .map(|t| t.as_seconds_f64())
            .unwrap_or(0.0)
    }

    pub(super) fn frame_at(&self, sec: f64) -> i64 {
        RationalTime::try_new((sec.max(0.0) * 1_000_000.0).round() as i64, 1_000_000)
            .and_then(|t| t.try_to_frame_round(self.fps))
            .unwrap_or(0)
            .clamp(0, self.duration_frames)
    }

    fn seconds_at_frame(&self, frame: i64) -> f64 {
        RationalTime::try_from_frame(frame.clamp(0, self.duration_frames), self.fps)
            .map(|time| time.as_seconds_f64())
            .unwrap_or(0.0)
    }

    fn frame_duration_sec(&self) -> f64 {
        1.0 / self.fps.as_f64().max(f64::EPSILON)
    }

    pub(super) fn time_at(&self, sec: f64) -> RationalTime {
        RationalTime::try_from_frame(self.frame_at(sec), self.fps).unwrap_or(RationalTime::ZERO)
    }

    fn fps_label(&self) -> String {
        if self.fps.den() == 1 {
            format!("{}fps", self.fps.num())
        } else {
            format!("{:.3}fps", self.fps.as_f64())
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum VisualFallback {
    NoAudio,
    Program(String),
    Device(String),
}

/// Transportが観測できる状態。audioが無くても同じ操作でvisual playbackは続く。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum PlaybackHealth {
    AudioReady,
    VisualOnly(VisualFallback),
}

impl PlaybackHealth {
    pub(super) fn visible_label(&self) -> Option<&'static str> {
        match self {
            Self::AudioReady | Self::VisualOnly(VisualFallback::NoAudio) => None,
            Self::VisualOnly(VisualFallback::Program(_)) => Some("Audio unavailable"),
            Self::VisualOnly(VisualFallback::Device(_)) => Some("Visual playback"),
        }
    }
}

struct PlaybackState {
    playing: bool,
    anchor: (Instant, f64),
    timebase: CompositionTimebase,
    duration: f64,
    program: Option<Arc<AudioProgram>>,
    cache: AudioProgramCache,
    session: Option<PlaybackSession>,
    health: PlaybackHealth,
}

/// Space、seek、現在位置の唯一のowner。
///
/// `Clock` aliasを残すことで、Stage/Timeline/Inspectorはこの部品を読むだけで
/// audio device由来の同じ時刻へ移る。
pub(super) struct PlaybackController {
    state: Mutex<PlaybackState>,
}

/// 既存surfaceの小さな互換口。実体と責任は`PlaybackController`一つ。
pub(super) type Clock = PlaybackController;

impl PlaybackController {
    pub(super) fn from_document(doc: &Document, fallback_duration_sec: f64) -> Self {
        let timebase = CompositionTimebase::from_document(doc)
            .unwrap_or_else(|| CompositionTimebase::ordinary_default(fallback_duration_sec));
        let mut cache = AudioProgramCache::default();
        let (program, health) = match AudioProgram::from_view(&doc.view(), &mut cache) {
            Ok(program) if program.sources().is_empty() => {
                (None, PlaybackHealth::VisualOnly(VisualFallback::NoAudio))
            }
            Ok(program) => (Some(Arc::new(program)), PlaybackHealth::AudioReady),
            Err(err) => (
                None,
                PlaybackHealth::VisualOnly(VisualFallback::Program(err.to_string())),
            ),
        };
        Self::from_parts(timebase, program, health, cache)
    }

    fn from_parts(
        timebase: CompositionTimebase,
        program: Option<Arc<AudioProgram>>,
        health: PlaybackHealth,
        cache: AudioProgramCache,
    ) -> Self {
        let duration = timebase.duration_sec();
        Self {
            state: Mutex::new(PlaybackState {
                playing: false,
                anchor: (Instant::now(), 0.0),
                timebase,
                duration,
                program,
                cache,
                session: None,
                health,
            }),
        }
    }

    #[cfg(test)]
    fn visual_only(timebase: CompositionTimebase) -> Self {
        Self::from_parts(
            timebase,
            None,
            PlaybackHealth::VisualOnly(VisualFallback::NoAudio),
            AudioProgramCache::default(),
        )
    }

    pub(super) fn health(&self) -> PlaybackHealth {
        self.state.lock().unwrap().health.clone()
    }

    pub(super) fn duration(&self) -> f64 {
        self.state.lock().unwrap().duration
    }

    /// Document revision後の唯一の再投影口。decode済みPCMとpeak pyramidはcacheで再利用する。
    pub(super) fn sync_document(&self, doc: &Document) {
        let mut state = self.state.lock().unwrap();
        let now = Instant::now();
        let old_duration = state.duration;
        let position = position_of(&state, now, old_duration);
        let was_playing = state.playing;
        state.session = None;

        let timebase = CompositionTimebase::from_document(doc)
            .unwrap_or_else(|| CompositionTimebase::ordinary_default(old_duration));
        let built = AudioProgram::from_view(&doc.view(), &mut state.cache);
        let (program, health) = match built {
            Ok(program) if program.sources().is_empty() => {
                (None, PlaybackHealth::VisualOnly(VisualFallback::NoAudio))
            }
            Ok(program) => (Some(Arc::new(program)), PlaybackHealth::AudioReady),
            Err(err) => (
                None,
                PlaybackHealth::VisualOnly(VisualFallback::Program(err.to_string())),
            ),
        };
        state.timebase = timebase;
        state.duration = timebase.duration_sec();
        let waveform_tracks = program
            .as_ref()
            .map_or(0, |program| program.waveform_tracks().len());
        state.program = program;
        state.health = health;
        state.anchor = (now, position.min(state.duration));
        println!("PROBE room=playback verdict=document-sync waveform-tracks={waveform_tracks}");
        if was_playing {
            let resume_at = position.min(state.duration);
            Self::open_audio_or_keep_visual(&mut state, resume_at);
        }
    }

    /// Timelineへ渡すimmutableなdata port。生成やcache規則はaudio側から出さない。
    pub(super) fn waveform_tracks(&self) -> Vec<WaveformTrack> {
        self.state
            .lock()
            .unwrap()
            .program
            .as_ref()
            .map(|program| program.waveform_tracks().to_vec())
            .unwrap_or_default()
    }

    pub(super) fn playing(&self) -> bool {
        self.state.lock().unwrap().playing
    }

    pub(super) fn toggle(&self) {
        let mut state = self.state.lock().unwrap();
        let now = Instant::now();
        let duration = state.duration;
        if state.playing {
            let position = position_of(&state, now, duration);
            // Drop stream and ring together. Resume/reseek cannot emit old queued samples.
            state.session = None;
            state.anchor = (now, position);
            state.playing = false;
            return;
        }

        let start = if state.anchor.1 >= duration {
            0.0
        } else {
            state.anchor.1
        };
        state.anchor = (now, start);
        state.playing = true;
        Self::open_audio_or_keep_visual(&mut state, start);
    }

    pub(super) fn seek(&self, sec: f64) {
        let mut state = self.state.lock().unwrap();
        let now = Instant::now();
        let to = sec.clamp(0.0, state.duration);
        // Reopening discards the old ring. `PlaybackSession::seek` cannot remove samples
        // already queued at the consumer side.
        state.session = None;
        state.anchor = (now, to);
        if state.playing {
            Self::open_audio_or_keep_visual(&mut state, to);
        }
    }

    pub(super) fn now_sec(&self) -> f64 {
        let mut state = self.state.lock().unwrap();
        let now = Instant::now();
        let duration = state.duration;
        let position = position_of(&state, now, duration);
        if state.playing && position >= duration {
            state.session = None;
            state.playing = false;
            state.anchor = (now, duration);
        }
        position
    }

    pub(super) fn current_frame(&self) -> i64 {
        let now = self.now_sec();
        self.state.lock().unwrap().timebase.frame_at(now)
    }

    pub(super) fn seek_frame(&self, frame: i64) {
        let seconds = self.state.lock().unwrap().timebase.seconds_at_frame(frame);
        self.seek(seconds);
    }

    pub(super) fn current_time(&self) -> RationalTime {
        let now = self.now_sec();
        self.state.lock().unwrap().timebase.time_at(now)
    }

    pub(super) fn frame_duration_sec(&self) -> f64 {
        self.state.lock().unwrap().timebase.frame_duration_sec()
    }

    pub(super) fn format_timecode(&self) -> String {
        let now = self.now_sec();
        let timebase = self.state.lock().unwrap().timebase;
        let frame = timebase.frame_at(now);
        let whole_fps = timebase.fps().as_f64().round().max(1.0) as i64;
        format!(
            "{}:{:02}:{:02}",
            frame / (whole_fps * 60),
            (frame / whole_fps) % 60,
            frame % whole_fps
        )
    }

    pub(super) fn composition_label(&self) -> String {
        let state = self.state.lock().unwrap();
        let fps = state.timebase.fps.as_f64().max(f64::EPSILON);
        let frames = (state.duration * fps).round() as i64;
        let whole = fps.round() as i64;
        let seconds = frames / whole.max(1);
        let rest = frames % whole.max(1);
        // 尺は丸めない。秒に収まらない分はコマで足す(0:20+12f)。
        let mut duration = format!("{}:{:02}", seconds / 60, seconds % 60);
        if rest != 0 {
            duration.push_str(&format!("+{rest}f"));
        }
        format!("{} · {duration}", state.timebase.fps_label())
    }

    fn open_audio_or_keep_visual(state: &mut PlaybackState, at_sec: f64) {
        let Some(program) = state.program.clone() else {
            return;
        };
        let at = state.timebase.time_at(at_sec);
        match PlaybackSession::open_default(program, at) {
            Ok(session) => {
                state.session = Some(session);
                state.health = PlaybackHealth::AudioReady;
            }
            Err(err) => {
                state.session = None;
                state.health = PlaybackHealth::VisualOnly(VisualFallback::Device(err.to_string()));
            }
        }
    }
}

fn position_of(state: &PlaybackState, now: Instant, duration: f64) -> f64 {
    if let Some(session) = &state.session {
        return session
            .clock()
            .position()
            .map(|t| t.as_seconds_f64().clamp(0.0, duration))
            .unwrap_or(state.anchor.1);
    }
    if state.playing {
        (state.anchor.1 + now.duration_since(state.anchor.0).as_secs_f64()).min(duration)
    } else {
        state.anchor.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::{Composition, Intent};

    fn document_at(fps_num: i64, fps_den: i64, duration_frames: i64) -> Document {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 1920,
            height: 1080,
            fps: Fps::try_new(fps_num, fps_den).unwrap(),
            duration_frames,
            background: Composition::default_background(),
        }))
        .unwrap();
        doc
    }

    #[test]
    fn composition_timebase_uses_document_fps_and_duration() {
        let doc = document_at(24, 1, 240);
        let timebase = CompositionTimebase::from_document(&doc).unwrap();

        assert_eq!(timebase.fps(), Fps::try_new(24, 1).unwrap());
        assert_eq!(timebase.duration_frames(), 240);
        assert_eq!(timebase.duration_sec(), 10.0);
        assert_eq!(timebase.frame_at(1.5), 36);
    }

    #[test]
    fn visual_fallback_keeps_transport_seekable_without_audio_or_device() {
        let timebase = CompositionTimebase::from_document(&document_at(30, 1, 300)).unwrap();
        let clock = PlaybackController::visual_only(timebase);

        clock.seek(3.25);
        assert_eq!(clock.now_sec(), 3.25);
        clock.toggle();
        assert!(clock.playing());
        assert!(clock.now_sec() >= 3.25);
        clock.toggle();
        assert!(!clock.playing());
        assert!(clock.now_sec() >= 3.25);
        assert_eq!(
            clock.health(),
            PlaybackHealth::VisualOnly(VisualFallback::NoAudio)
        );
    }

    #[test]
    fn seek_is_clamped_by_the_composition_timebase() {
        let timebase = CompositionTimebase::from_document(&document_at(24, 1, 240)).unwrap();
        let clock = PlaybackController::visual_only(timebase);

        clock.seek(99.0);
        assert_eq!(clock.now_sec(), 10.0);
        clock.seek(-1.0);
        assert_eq!(clock.now_sec(), 0.0);
    }

    #[test]
    fn document_sync_replaces_the_timebase_without_replacing_the_controller() {
        let clock = PlaybackController::from_document(&document_at(30, 1, 1_800), 60.0);
        clock.seek(50.0);

        clock.sync_document(&document_at(24, 1, 240));

        assert_eq!(clock.duration(), 10.0);
        assert_eq!(clock.now_sec(), 10.0);
        assert_eq!(clock.composition_label(), "24fps · 0:10");
    }

    #[test]
    fn frame_navigation_uses_the_composition_rate() {
        let clock = PlaybackController::from_document(&document_at(24, 1, 240), 10.0);
        clock.seek_frame(37);

        assert_eq!(clock.current_frame(), 37);
        assert!((clock.now_sec() - 37.0 / 24.0).abs() < 1e-9);
        assert_eq!(clock.current_time(), RationalTime::try_new(37, 24).unwrap());
        assert!((clock.frame_duration_sec() - 1.0 / 24.0).abs() < 1e-12);
    }
}
