//! 窓を開く薄殻。
//!
//! ```text
//! cargo run -p motolii-shell-iced
//! cargo run -p motolii-shell-iced -- --intent-log /tmp/intents.jsonl
//! ```
//!
//! egui shell(`cargo run -p motolii-ui --bin motolii-blitz-shell`)は**そのまま在る**。
//! 既定 bin の切り替えは M-5 で、UX 台本が iced 側で通ってからである。
//!
//! 引数は M-0 の範囲だけ。`--project` / `--status-log` / `--screenshot` は M-1。

use std::path::PathBuf;

use motolii_shell_iced::{view, IntentLog, Message, NativePrompts, Shell};

fn main() -> iced::Result {
    let intent_log = match parse_intent_log(std::env::args().skip(1)) {
        Ok(path) => path,
        Err(reason) => {
            eprintln!("motolii-shell-iced: {reason}");
            std::process::exit(2);
        }
    };

    // 記録先が開けないなら**起動前に**言って落ちる。窓を開いてから
    // 「実は記録していなかった」を起こさない。
    let log = match intent_log.as_deref().map(IntentLog::create).transpose() {
        Ok(log) => log,
        Err(reason) => {
            eprintln!("motolii-shell-iced: {reason}");
            std::process::exit(2);
        }
    };

    // `BootFn` は `Fn() -> _`(**`FnOnce` ではない**)なので、move で握った
    // `IntentLog` をそのまま渡せない。1回しか呼ばれない物を1回だけ渡す口として
    // `RefCell::take` を使う。2度目が来たら記録先は `None` になる — 起きないはずの
    // ことを黙って握り潰すのではなく、記録しないことがそのまま見えるようにしてある。
    let log = std::cell::RefCell::new(log);
    iced::application(
        move || Host::new(log.borrow_mut().take()),
        Host::update,
        Host::view,
    )
    .title("Motolii")
    .window_size((980.0, 650.0))
    .run()
}

/// 殻 + 記録先。**殻の側には記録の都合を1つも足さない**(egui 版 `Harness` と同じ分担)。
struct Host {
    shell: Shell,
    intent_log: Option<IntentLog>,
}

impl Host {
    fn new(intent_log: Option<IntentLog>) -> Self {
        Self {
            shell: Shell::new(NativePrompts),
            intent_log,
        }
    }

    fn update(&mut self, message: Message) {
        self.shell.update(message);
        if let Some(log) = self.intent_log.as_mut() {
            log.flush(&self.shell);
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

/// `--intent-log <path>`。付けなければ `None`。
fn parse_intent_log(args: impl Iterator<Item = String>) -> Result<Option<PathBuf>, String> {
    let args: Vec<String> = args.collect();
    let mut path = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--intent-log" => {
                let next = args
                    .get(i + 1)
                    .ok_or_else(|| "--intent-log には path が要る".to_owned())?;
                path = Some(PathBuf::from(next));
                i += 2;
            }
            other => {
                if let Some(rest) = other.strip_prefix("--intent-log=") {
                    path = Some(PathBuf::from(rest));
                    i += 1;
                } else {
                    return Err(format!("知らない引数: {other}"));
                }
            }
        }
    }
    Ok(path)
}
