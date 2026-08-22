//! owns: 実デバイス再生セッション(A2)— [`device`](実cpal結線) + [`producer`]
//! (mixプロデューサスレッド) + [`clock::PlaybackClock`] を束ねる。
//!
//! OWNS-JUSTIFICATION(A): 発注書「旧PlaybackSessionの形を移植 — スクラッチ禁止」
//! を直接引用した明示指示(裁定215 棚卸し 2026-08-23 #10)。
//!
//! 旧 `crates/motolii-transport/src/playback.rs::PlaybackSession` の形を移植
//! (発注書「旧PlaybackSessionの形を移植 — スクラッチ禁止」)。**旧との違い**:
//! 旧は`Transport`(DRS・映像フレームドロップ込み)と束ねていたが、この crate の
//! 対象は音声クロックだけなので([`clock`]モジュールdoc参照)、束ねる相手は
//! [`crate::PlaybackClock`]。`negotiate_output`/`open_negotiated_shared`/
//! `MixProducer::spawn`の3手順を1回の`open_default`/`open_on_device`へ畳んだ点は
//! 旧`PlaybackSession::open_on_device`と同じ形。

use std::sync::Arc;

use cpal::traits::HostTrait;

use motolii_core::RationalTime;

use crate::clock::{DeviceWaitLatency, PlaybackClock, PlaybackCounters};
use crate::convert::{canonical_format, time_to_canonical_frames};
use crate::device::{negotiate_output, NegotiatedOutput, OutputStream};
use crate::error::{AudioError, Result};
use crate::producer::MixProducer;
use crate::program::AudioProgram;

/// リング容量(フレーム数)。旧`crates/motolii-transport/src/playback.rs`の
/// `channel(format.channels, 4_096)`と同じ値 — 実測済みの余裕(約85ms @48k)。
const RING_CAPACITY_FRAMES: usize = 4_096;

/// device出力 + mix producer + [`PlaybackClock`] を束ねた再生セッション。Drop で
/// stream/producerスレッドを止める。
///
/// **soundtrackが無いDocumentでも開ける**: `AudioProgram::sources()`が空でも
/// `composition_duration()`さえ非ゼロなら`mix_audio`は正規の無音PCMを返す
/// (`mix.rs::mix_audio`— sourceが無ければ`any=false`のまま0.0を書く)ので、
/// このセッションは実デバイスへ無音を流しながら`frames_supplied`を進める —
/// 「絵だけの comp でも Play が成立する」という発注書の判断はここで満たす。
pub struct PlaybackSession {
    clock: PlaybackClock,
    negotiated: Option<NegotiatedOutput>,
    _stream: Option<OutputStream>,
    _producer: Option<MixProducer>,
}

impl PlaybackSession {
    /// デフォルト出力デバイスで、`at`(タイムライン時刻)から再生開始する。
    pub fn open_default(program: Arc<AudioProgram>, at: RationalTime) -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or(AudioError::NoOutputDevice)?;
        Self::open_on_device(program, at, &device)
    }

    /// 指定デバイスで再生を開始する(テスト/明示選択用)。
    pub fn open_on_device(
        program: Arc<AudioProgram>,
        at: RationalTime,
        device: &cpal::Device,
    ) -> Result<Self> {
        let format = canonical_format();
        let negotiated = negotiate_output(device, format)?;

        let counters = Arc::new(PlaybackCounters::default());
        let device_wait = Arc::new(DeviceWaitLatency::default());
        let (ring_prod, ring_cons) =
            rtrb::RingBuffer::<f32>::new(RING_CAPACITY_FRAMES * format.channels as usize);

        let stream = OutputStream::open_negotiated_shared(
            device,
            &negotiated,
            ring_cons,
            Arc::clone(&counters),
            Arc::clone(&device_wait),
        )?;

        let start_frame = time_to_canonical_frames(at);
        let producer = MixProducer::spawn(
            program,
            ring_prod,
            start_frame,
            negotiated.device_sample_rate,
        )?;

        let mut clock =
            PlaybackClock::new(Arc::clone(&counters), Arc::clone(&device_wait), negotiated.device_sample_rate)?;
        clock.start(at);

        Ok(Self {
            clock,
            negotiated: Some(negotiated),
            _stream: Some(stream),
            _producer: Some(producer),
        })
    }

    pub fn clock(&self) -> &PlaybackClock {
        &self.clock
    }

    pub fn clock_mut(&mut self) -> &mut PlaybackClock {
        &mut self.clock
    }

    pub fn negotiated(&self) -> Option<&NegotiatedOutput> {
        self.negotiated.as_ref()
    }

    /// 走行状態を変えずに`at`へ跳ぶ(発注書「再生中の scrub は seek」)。**論理
    /// 位置は常に純粋に即時反映**(`PlaybackClock::seek`— counters を触らない
    /// 原点の付け替えだけ、A1参照)。実producerが生きていれば([`MixProducer::
    /// seek`])audio内容も追いつかせる — こちらは次ループ反映・リングに
    /// 既に積んだ分はそのまま流れきる既知の制約(`producer.rs` doc参照、実機
    /// 確認課題)。試験(`for_simulation`)にはproducerが無いのでここは
    /// no-op — 論理位置のseekだけがORACLEの対象で、それは常に成立する。
    pub fn seek(&mut self, at: RationalTime) {
        self.clock.seek(at);
        if let Some(producer) = self._producer.as_ref() {
            producer.seek(time_to_canonical_frames(at));
        }
    }

    /// **試験専用の縫い目**(ORACLE「デバイス抽象はフェイクで — A1と同じ手」)。
    /// 実cpal/producerを一切開かず、呼び出し側が組んだ`PlaybackClock`(A1の
    /// `fake_clock`ヘルパと同じ形 — `Arc<PlaybackCounters>`を直接進める)だけを
    /// 持つセッションを作る。`motolii-shell`側の`Transport`はこれと本番の
    /// `open_default`の戻り値を区別しない(同じ`PlaybackSession`型)ので、
    /// tick/pause/seekのロジックは実デバイス無しで検証できる。
    #[doc(hidden)]
    pub fn for_simulation(clock: PlaybackClock) -> Self {
        Self {
            clock,
            negotiated: None,
            _stream: None,
            _producer: None,
        }
    }
}
