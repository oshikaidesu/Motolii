//! owns: 実時間再生(A2、2026-08-21)— Space で Play/Pause、tick で
//! `Session::playhead` を `PlaybackClock::position()` へ追随させる。
//!
//! **設計**(発注書「旧 PlaybackSession の形を移植」+「デバイス抽象はフェイクで」
//! の両方を1つの型で満たす): [`Transport`] は `Option<motolii_audio::
//! PlaybackSession>` を持つだけの薄い口。`PlaybackSession` 自体が実
//! cpal/producer を束ねる型なので、`Transport` はハードウェアを一切知らない
//! (`open_real_playback` — 本番の唯一の実デバイス起動経路 — だけがそれを知る)。
//!
//! - **Play**(停止中→再生): `Transport::start` が `open_real_playback`(本番)
//!   または `PlaybackSession::for_simulation`(試験)で組んだセッションを採用する。
//! - **Pause**: `Shell::freeze_playhead_from_transport`(呼び出し側)が
//!   `position_frame` で今の位置を `Session::playhead` へ確定させてから
//!   `Transport::stop` — セッションを丸ごと Drop する(stream 停止・producer
//!   スレッド join)。
//! - **再生中の scrub(seek)**: `Transport::seek` が `PlaybackSession::seek`
//!   (`motolii-audio`側)へ委譲する。**論理位置は純粋 counters 非依存の
//!   `PlaybackClock::seek`**(A1)なので実デバイス無しで即時反映・試験可能
//!   (ORACLE (c))。audio内容の追従(`MixProducer::seek`)は producer が
//!   生きている本番経路だけの話 — リングに既に積んだ分はそのまま流れきる
//!   既知の制約(`producer.rs` doc参照、実機確認課題)。

use std::sync::Arc;

use motolii_audio::PlaybackSession;
use motolii_core::Fps;
use motolii_store::Document;

use crate::state::Session;

/// 本番の実デバイス起動経路の型。試験は [`PlaybackSession::for_simulation`] で
/// 直接 `PlaybackSession` を組んで `Transport::start_with_session` へ渡すので、
/// この関数自体は呼ばない(「実デバイス経路はテスト不能でよい」の対象)。
pub type OpenPlayback = fn(&Document, &Session) -> Result<PlaybackSession, String>;

/// 再生状態そのもの。**Document ではない**(`Shell` の他の transient 状態と
/// 同じ「front だけが持つ」身分 — undo に一切乗らない)。
#[derive(Default)]
pub struct Transport {
    session: Option<PlaybackSession>,
}

impl Transport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_running(&self) -> bool {
        self.session
            .as_ref()
            .is_some_and(|session| session.clock().is_running())
    }

    /// 実デバイス(または試験の場合はフェイク)を開いて再生を開始する。
    pub fn start(&mut self, open: OpenPlayback, doc: &Document, session: &Session) -> Result<(), String> {
        let opened = open(doc, session)?;
        self.start_with_session(opened);
        Ok(())
    }

    /// 既に組み立てた [`PlaybackSession`] を採用する。**ORACLE の試験専用の縫い目**
    /// (`PlaybackSession::for_simulation` で作ったフェイクを注入する)— 本番は
    /// [`Self::start`] がこれを内部で呼ぶだけで、直接は呼ばない。
    pub fn start_with_session(&mut self, session: PlaybackSession) {
        self.session = Some(session);
    }

    /// 現在位置を凍結して停止する。**呼ぶ前に位置を読んでおくこと**
    /// (`position_frame` → `Session::playhead` へ書く → この関数、の順。
    /// `Shell::toggle_playback` 参照)— この関数自体はセッションを Drop する
    /// だけで位置を保存しない。
    pub fn stop(&mut self) {
        self.session = None;
    }

    /// 今の再生位置を comp フレームへ(再生していなければ `None`)。
    pub fn position_frame(&self, fps: Fps) -> Option<i64> {
        let clock = self.session.as_ref()?.clock();
        clock.position().ok()?.try_to_frame_floor(fps).ok()
    }

    /// 再生中の scrub = seek(発注書 ORACLE (c))。再生していなければ no-op
    /// (呼び出し側 `Shell::scrub_to` が非再生中は素の `Session::playhead`
    /// 書き換えだけで済ませる — ここまで来る時点で既に再生中の場合だけ)。
    pub fn seek(&mut self, at: motolii_core::RationalTime) {
        if let Some(session) = self.session.as_mut() {
            session.seek(at);
        }
    }
}

/// 本番の唯一の実デバイス起動経路(A2)。`Document` の soundtrack から
/// `AudioProgram` を組み、デフォルト出力デバイスへ `session.playhead` から
/// 再生を開始する。**この関数はテストしない**(ORACLE「実デバイス経路は
/// テスト不能でよい」)— 純粋なロジックは全部 `motolii_audio` 側(`PlaybackClock`
/// 自体・`select_device_sample_rate` 等)で個別に試験済み。
pub fn open_real_playback(doc: &Document, session: &Session) -> Result<PlaybackSession, String> {
    let mut caches = std::collections::HashMap::new();
    let program = motolii_audio::AudioProgram::from_view(&doc.view(), &mut caches)
        .map_err(|error| format!("音声プログラムを組めない: {error}"))?;

    let composition = doc
        .view()
        .composition()
        .ok()
        .flatten()
        .ok_or_else(|| "comp が無いので再生できない".to_owned())?;
    let at = motolii_core::RationalTime::try_from_frame(session.playhead, composition.fps)
        .map_err(|error| format!("playhead を時刻へ写せない: {error}"))?;

    PlaybackSession::open_default(Arc::new(program), at)
        .map_err(|error| format!("再生を開始できない: {error}"))
}

/// 再生中のtick購読(`Shell::subscription`が`transport.is_running()`の間だけ
/// batchへ含める)。**`iced::time::every`は使わない** — この workspace の
/// `iced`依存は`tokio`/`smol` featureを有効化していない(`thread-pool`
/// executorのみ)ため`iced_futures::backend::native::thread_pool::time`は
/// 空(`every`が無い、実測)。`tokens::watch_subscription`(notify監視)と同じ
/// `iced::stream::channel` + 専用OSスレッドの形をtick用に踏襲する。
pub fn tick_subscription() -> iced::Subscription<()> {
    iced::Subscription::run(tick_stream)
}

/// 目標fps(~60)。playhead追随の粗さはこの間隔が上限になる — comp fpsより
/// 十分細かければ「1フレームも飛ばさず見える」水準は満たす(正確な
/// フレーム同期はStage描画側の責務で、tickは「動いていることを知らせる」
/// だけ)。
const TICK_INTERVAL: std::time::Duration = std::time::Duration::from_millis(16);

fn tick_stream() -> impl iced::futures::Stream<Item = ()> {
    iced::stream::channel(
        8,
        |mut output: iced::futures::channel::mpsc::Sender<()>| async move {
            std::thread::spawn(move || loop {
                std::thread::sleep(TICK_INTERVAL);
                if let Err(error) = output.try_send(()) {
                    // 受け手(Shell)がもう無い(disconnected)なら見張りを終える
                    // (`tokens::watch_stream`と同じ後始末)。詰まっているだけ
                    // (容量超過)なら次のtickへ進めばよい。
                    if error.is_disconnected() {
                        return;
                    }
                }
            });

            // 実際の送信は上のOSスレッドが行う。この Future 自体は消費されない
            // まま stream を生かしておくためだけに待ち続ける
            // (`tokens::watch_stream`と同じ形)。
            std::future::pending::<()>().await;
        },
    )
}
