//! Subprocess helper for D1m lock integration tests (not a product entry).

use std::io::Write;
use std::path::Path;
use std::thread;
use std::time::Duration;

use motolii_doc::{ProjectSession, ResourceLimits};

/// lock 獲得の合図。親テスト(`tests/d1m_session_lock.rs`)が同じ文字列を待つので、
/// 変えるときは必ず両方を揃える。
const READY_LINE: &str = "d1m-lock-holder: ready";

fn main() {
    let path = std::env::args().nth(1).expect("project path");
    let hold_ms: u64 = std::env::args()
        .nth(2)
        .expect("hold ms")
        .parse()
        .expect("hold ms parse");
    let _session =
        ProjectSession::acquire(Path::new(&path), &ResourceLimits::production()).expect("acquire");

    // 合図は acquire が成功した「後」でしか出さない。ここより前に出すと、
    // 親が「まだ握っていない子」を握ったものと誤認する。
    let mut stdout = std::io::stdout();
    writeln!(stdout, "{READY_LINE}").expect("write ready line");
    stdout.flush().expect("flush ready line");

    // 以降 stdout へは何も書かない。親は一行読んだ時点で読み口を閉じるため、
    // 追記すると SIGPIPE で落ちて hold が途中で切れる。
    thread::sleep(Duration::from_millis(hold_ms));
}
