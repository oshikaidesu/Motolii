//! Subprocess helper for D1m lock integration tests (not a product entry).

use std::path::Path;
use std::thread;
use std::time::Duration;

use motolii_doc::{ProjectSession, ResourceLimits};

fn main() {
    let path = std::env::args().nth(1).expect("project path");
    let hold_ms: u64 = std::env::args()
        .nth(2)
        .expect("hold ms")
        .parse()
        .expect("hold ms parse");
    let _session =
        ProjectSession::acquire(Path::new(&path), &ResourceLimits::production()).expect("acquire");
    thread::sleep(Duration::from_millis(hold_ms));
}
