//! トークンファイルの変更を見張る `Subscription`(debug ビルドのみ)と、
//! notify の raw event を束ねるデバウンス。`lib.rs` から分割
//! (SP-8、中身は移送のみ)。

use crate::{Colors, Dimensions};

/// トークンファイルの変更を見張る `Subscription`。**debug ビルドのみ実際に見張る**
/// — release はホットリロードを前提にしない(裁定117)ので何も発行しない。
///
/// 発行するのは `()` だけ(このモジュールは `Message` 型を知らない)。呼び出し側
/// (`Shell::subscription`)が `.map(|_| Message::TokensFileChanged)` で繋ぐ。
#[cfg(debug_assertions)]
pub fn watch_subscription() -> iced::Subscription<()> {
    iced::Subscription::run(watch_stream)
}

#[cfg(not(debug_assertions))]
pub fn watch_subscription() -> iced::Subscription<()> {
    iced::Subscription::none()
}

/// 1回の保存につき notify が出す raw event を1回の通知へ束ねる窓。
///
/// **実測(2026-08-20)**: 多くのエディタ/OS はファイル1本の保存で write+rename
/// 等、**複数の raw event を連続して出す**(notify 自体のドキュメントにも
/// 明記されている一般的挙動)。束ねずに全部 `Message::TokensFileChanged` へ流すと
/// 1回の保存で `Tokens::load()`(file I/O + JSON parse ×2)が複数回走り、
/// そのたび `view()` が再構築される — Stage の Handle 自体には触れない
/// (`refresh_frame` は revision/playhead が同じなら早期 return する)ので
/// チラつきの直接原因ではないが、無駄な再描画の連打であることに変わりはない
/// (発注書の容疑者2)。
const TOKENS_WATCH_DEBOUNCE: std::time::Duration = std::time::Duration::from_millis(80);

/// 受信を1回に束ねる**純粋なロジック**(テスト可能)。`is_significant` を
/// 満たす最初の1件をブロッキングで待ち(満たさない件は無視して待ち続ける —
/// notify のエラー event を通知扱いしない、という元の挙動を保つ)、その後
/// `window` 以内に来た追加分は種類を問わず全部飲み込んで捨てる。戻り値は
/// 「1回分の通知が来た」を表すだけで、束ねた個数は数えない(呼び出し側は
/// 「変わった」以上の情報を必要としない)。
///
/// 送信側が消えた(`Err(RecvError)`)場合は `None` — 監視終了のサイン。
fn debounce_recv<T>(
    rx: &std::sync::mpsc::Receiver<T>,
    window: std::time::Duration,
    is_significant: impl Fn(&T) -> bool,
) -> Option<()> {
    loop {
        let item = rx.recv().ok()?;
        if is_significant(&item) {
            break;
        }
    }
    while rx.recv_timeout(window).is_ok() {}
    Some(())
}

#[cfg(debug_assertions)]
fn watch_stream() -> impl iced::futures::Stream<Item = ()> {
    iced::stream::channel(
        8,
        |mut output: iced::futures::channel::mpsc::Sender<()>| async move {
            let dims_path = Dimensions::debug_source_path();
            let colors_path = Colors::debug_source_path();

            // notify の watcher は監視対象スレッドでコールバックを呼ぶ実装のため、
            // 受信は専用の OS スレッドへ逃がす(async executor を止めない)。
            // `try_send` は poll を要らないので、executor を挟まず同期コールバックから
            // 直接呼べる — 詰まっていたら単に取りこぼす(M16: 見張りが完璧でなくても
            // shell 自体は止めない)。
            std::thread::spawn(move || {
                use notify::Watcher;

                let (tx, rx) = std::sync::mpsc::channel();
                let mut watcher = match notify::recommended_watcher(tx) {
                    Ok(watcher) => watcher,
                    // 見張れなくても shell 自体は動く(M16)。token は起動時の値のまま。
                    Err(_) => return,
                };
                if watcher
                    .watch(&dims_path, notify::RecursiveMode::NonRecursive)
                    .is_err()
                {
                    return;
                }
                if watcher
                    .watch(&colors_path, notify::RecursiveMode::NonRecursive)
                    .is_err()
                {
                    return;
                }

                // **デバウンス**: 1回の保存が出す連続 raw event を1回の通知へ束ねる。
                // エラー event(`Result::Err`)は「変わった」の合図として扱わない
                // (元の実装の `if event.is_err() { continue; }` と同じ意味)。
                loop {
                    if debounce_recv(&rx, TOKENS_WATCH_DEBOUNCE, |event| event.is_ok()).is_none() {
                        // 送信側(watcher)が消えた = 監視を続けられない。
                        return;
                    }
                    if let Err(error) = output.try_send(()) {
                        // 詰まっているだけ(容量超過)なら次の束ねへ進めばよい。
                        // 受け手(Shell)がもう無い(disconnected)なら見張りを終える。
                        if error.is_disconnected() {
                            return;
                        }
                    }
                }
            });

            // 実際の送信は上の OS スレッドが行う。この Future 自体は消費されないまま
            // stream を生かしておくためだけに待ち続ける。
            std::future::pending::<()>().await;
        },
    )
}

#[cfg(test)]
mod debounce_tests {
    use super::debounce_recv;
    use std::time::Duration;

    /// **容疑者2の柵**: 1回の保存で notify が出す連続バーストを、1回の
    /// `debounce_recv` 呼び出しへ束ねる。束ねた後は channel が空になっている
    /// こと(=呼び出し側が2回目を呼んでも新しい通知が無い)まで確かめる。
    #[test]
    fn a_burst_of_events_collapses_into_one_notification() {
        let (tx, rx) = std::sync::mpsc::channel::<()>();
        // 保存直後の連続 raw event を模す(sleep なし — 実際のバーストと同じく
        // ほぼ同時に届く)。
        for _ in 0..5 {
            tx.send(()).unwrap();
        }

        let window = Duration::from_millis(50);
        let result = debounce_recv(&rx, window, |_| true);
        assert!(result.is_some(), "束ねた1回の通知が出ない");

        // 束ねた後は空 — 5件が5回の通知に化けていないことの直接証拠。
        assert!(
            rx.try_recv().is_err(),
            "バーストの一部が束ねられずに残っている(デバウンス欠如)"
        );
    }

    /// エラー扱いの event は「変わった」の合図にしない(notify のエラー通知を
    /// トークン再読込のトリガにしない、元の実装の意味を保つ)。
    #[test]
    fn insignificant_events_do_not_trigger_a_notification_on_their_own() {
        let (tx, rx) = std::sync::mpsc::channel::<Result<(), ()>>();
        tx.send(Err(())).unwrap();
        tx.send(Err(())).unwrap();
        tx.send(Ok(())).unwrap();

        let window = Duration::from_millis(50);
        let result = debounce_recv(&rx, window, |event| event.is_ok());
        assert!(result.is_some(), "有効な event が来ているのに通知が出ない");
        assert!(
            rx.try_recv().is_err(),
            "Ok の後に残りが無いはず(全部1回へ束ねられているべき)"
        );
    }

    /// 送信側が消えたら `None` — 監視を終える合図として使える。
    #[test]
    fn a_closed_channel_yields_none() {
        let (tx, rx) = std::sync::mpsc::channel::<()>();
        drop(tx);
        assert!(debounce_recv(&rx, Duration::from_millis(10), |_| true).is_none());
    }
}

