//! 窓を開く薄殻。
//!
//! ```text
//! cargo run -p motolii-shell-iced
//! cargo run -p motolii-shell-iced -- --intent-log /tmp/intents.jsonl
//! cargo run -p motolii-shell-iced -- --status-log /tmp/statuses.jsonl
//! ```
//!
//! egui shell(`cargo run -p motolii-ui --bin motolii-blitz-shell`)は**そのまま在る**。
//! 既定 bin の切り替えは M-5 で、UX 台本が iced 側で通ってからである。

use motolii_shell_iced::{view, IntentLog, Launch, Message, NativePrompts, Outcome, Shell, StatusLog};

fn main() -> iced::Result {
    let launch = match Launch::parse(std::env::args().skip(1)) {
        Ok(launch) => launch,
        Err(reason) => {
            eprintln!("motolii-shell-iced: {reason}");
            std::process::exit(2);
        }
    };

    // 記録先が開けないなら**起動前に**言って落ちる。窓を開いてから
    // 「実は記録していなかった」を起こさない。
    let logs = match open_logs(&launch) {
        Ok(logs) => logs,
        Err(reason) => {
            eprintln!("motolii-shell-iced: {reason}");
            std::process::exit(2);
        }
    };

    // `BootFn` は `Fn() -> _`(**`FnOnce` ではない**)なので、move で握った
    // 記録先をそのまま渡せない。1回しか呼ばれない物を1回だけ渡す口として
    // `RefCell::take` を使う。2度目が来たら記録先は `None` になる — 起きないはずの
    // ことを黙って握り潰すのではなく、記録しないことがそのまま見えるようにしてある。
    let logs = std::cell::RefCell::new(Some(logs));
    iced::application(
        move || Host::new(logs.borrow_mut().take().unwrap_or_default()),
        Host::update,
        Host::view,
    )
    .subscription(Host::subscription)
    // 見た目は token 正本から(`theme::product`)。iced 既定 palette の藍色を
    // 1画面も残さない — 見た目も嘘をつかない(Q0 の精神)。
    .theme(|_host: &Host| motolii_shell_iced::theme::product())
    .title("Motolii")
    .window_size((980.0, 650.0))
    // 閉じるかどうかは殻が決める(未保存なら3択)。窓に勝手に閉じさせない。
    .exit_on_close_request(false)
    .run()
}

/// 記録の口ぜんたい。**殻の側には記録の都合を1つも足さない**(egui 版 `Harness` と同じ分担)。
#[derive(Default)]
struct Logs {
    intent: Option<IntentLog>,
    status: Option<StatusLog>,
}

impl Logs {
    /// 原因を先に流す。記録の順が「intent → その結果の status」だから。
    fn flush(&mut self, shell: &Shell) {
        if let Some(log) = self.intent.as_mut() {
            log.flush(shell);
        }
        if let Some(log) = self.status.as_mut() {
            log.flush(shell);
        }
    }
}

fn open_logs(launch: &Launch) -> Result<Logs, String> {
    Ok(Logs {
        intent: launch
            .intent_log
            .as_deref()
            .map(IntentLog::create)
            .transpose()?,
        status: launch
            .status_log
            .as_deref()
            .map(StatusLog::create)
            .transpose()?,
    })
}

/// 殻 + 記録先。
struct Host {
    shell: Shell,
    logs: Logs,
}

impl Host {
    fn new(logs: Logs) -> Self {
        Self {
            shell: Shell::new(NativePrompts),
            logs,
        }
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        let outcome = self.shell.update(message);
        self.logs.flush(&self.shell);
        match outcome {
            Outcome::Stay => iced::Task::none(),
            Outcome::Close => iced::exit(),
        }
    }

    /// 走っている書き出しの返事を受けるための刻み。**書き出し中だけ**購読する
    /// (egui 版が `request_repaint_after(200ms)` で自分を起こしているのと同じ役目)。
    ///
    /// 近道キーと OS ドロップを**ここに置かない**のは意図的である。購読は
    /// `iced_test::Simulator` の外に居るので、置くと運転席から注入できなくなる —
    /// あれらは widget 木の中(`window_input`)で受ける。
    fn subscription(&self) -> iced::Subscription<Message> {
        if self.shell.export_running() {
            iced::window::frames().map(|_| Message::ExportPolled)
        } else {
            iced::Subscription::none()
        }
    }

    /// **`fn` 項目であって closure ではない。**
    ///
    /// `|host: &Host| -> Element<'_, _>` はクロージャの推論が高階の生存期間を
    /// 作れず `implementation of ViewFn is not general enough` で落ちる
    /// (`spikes/iced-rerun-embed-probe` の詰まった箇所その4)。
    fn view(&self) -> iced::Element<'_, Message> {
        view(&self.shell)
    }
}
