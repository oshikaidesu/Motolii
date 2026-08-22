//! owns: 自動保存(AUTOSAVE、SET+ B12 第2切片の shell 結線、2026-08-22)。
//!
//! この module は「一定間隔ごとに `()` を1つ発行する」購読口だけを持つ
//! (`transport::tick_subscription` と同じ「翻訳だけ」の役割分担)。実際に
//! 保存してよいかどうかの判断(dirty 判定・再生中/ドラッグ中のスキップ・
//! `auto_save_enabled` の有無)は一切ここに置かない — `Shell::subscription`
//! が `auto_save_enabled` を見てこの購読自体を差し込むかどうかを決め、
//! `Shell::run_auto_save`(`Message::AutoSaveTick` の受け口)が
//! `motolii_store::Document::auto_save` を呼ぶ(dirty 判定はその内部の
//! revision 比較に委ねる)。
//!
//! **`iced::time::every` はこの workspace では使えない**
//! (`next/reference/KNOWN.md`「iced::time::everyはこのworkspaceでは使えない」
//! — fork の `iced_futures::backend::default::time` は `tokio`/`smol` feature
//! が無いと空モジュールに落ちる。`next/Cargo.toml` の `iced` 依存はどちらも
//! 有効化していない)。代わりに `motolii_tokens_rs::watch_subscription` と
//! 同じ手口(`iced::stream::channel` + 専用 OS スレッド)で自前のタイマーを
//! 組む — `std::thread::sleep` は専用スレッド側で行い、async 側は
//! `pending().await` で stream を生かしておくだけ(executor を止めない、
//! `watch_stream` doc と同じ理由)。
//!
//! OWNS-JUSTIFICATION(B): 探索対象=`iced::time::every` — `next/reference/KNOWN.md`
//! に「このworkspaceでは使えない」と具体的に明記(forkの`iced_futures`が
//! tokio/smol featureを有効化していないため空モジュールに落ちる)。上流APIの
//! 欠落を検証した上で自前タイマー購読を組んだ(裁定215 棚卸し 2026-08-23 #25)。

use std::time::Duration;

/// AUTOSAVE 間隔(秒)ごとに `()` を発行する購読。**`interval_secs` が識別子の
/// 一部**(`iced::Subscription::run_with` — 値が変われば古い購読は消え、新しい
/// 周期のタイマースレッドへ差し替わる)なので、Settings で間隔を変えれば次の
/// `Shell::subscription()` 呼び出しで新しい周期が反映される。呼び出し側
/// (`Shell::subscription`)が `.map(|()| Message::AutoSaveTick)` で繋ぐ
/// (`transport::tick_subscription`/`motolii_tokens_rs::watch_subscription` と
/// 同じ「`()` を発行するだけ、`Message` 型を知らない」形)。
pub fn tick_subscription(interval_secs: u64) -> iced::Subscription<()> {
    iced::Subscription::run_with(interval_secs.max(1), tick_stream)
}

fn tick_stream(interval_secs: &u64) -> impl iced::futures::Stream<Item = ()> {
    let interval_secs = *interval_secs;
    iced::stream::channel(
        1,
        move |mut output: iced::futures::channel::mpsc::Sender<()>| async move {
            // notify watcher と同じ理由(`watch_stream` doc 参照): sleep は
            // ブロッキングなので async executor 上ではなく専用 OS スレッドで行う。
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_secs(interval_secs));
                if let Err(error) = output.try_send(()) {
                    // 受け手(Shell)が無くなった(subscription が差し替わった)
                    // なら、このスレッドはもう役目を終えている。
                    if error.is_disconnected() {
                        return;
                    }
                }
            });

            std::future::pending::<()>().await;
        },
    )
}
