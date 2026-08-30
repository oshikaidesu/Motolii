
//! パネルの関数は「出ている物」だけが呼ばれる。そこで hook を作ると、出ている
//! パネルが変わった瞬間に hook の枠が入れ替わって落ちる(dioxus の rules of hooks)。
//! hook を作ってよいのは `app.rs` だけ。

use std::fs;

const PANELS: &[&str] = &[
    "src/ui/browser.rs",
    "src/ui/inspector.rs",
    "src/ui/utility.rs",
    "src/ui/timeline_shell.rs",
    "src/ui/dock.rs",
];

#[test]
fn only_the_app_root_creates_hooks() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for path in PANELS {
        let source = fs::read_to_string(root.join(path)).expect(path);
        for (n, line) in source.lines().enumerate() {
            let line = line.trim();
            if line.starts_with("//") {
                continue;
            }
            assert!(
                !(line.contains("use_signal") || line.contains("use_hook")),
                "{path}:{} が hook を作っている。app.rs で作って渡すこと\n  {line}",
                n + 1
            );
        }
    }
}
