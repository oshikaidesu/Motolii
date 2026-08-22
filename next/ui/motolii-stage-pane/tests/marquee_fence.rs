//! **マーキーの構造柵**(`render_frame_fence.rs` と同型の「ソースを読んで
//! 具体的な行で落とす」軽い柵。src/ 内に置くと柵コード自身の文字列リテラルを
//! 1件として数える自己参照になるため integration test に分離)。
//!
//! 2本の不変を縛る:
//!
//! 1. **選択は message でしか出ない**(gizmo.rs と同じ「Document を一切書かない」
//!    規律): `src/marquee.rs` は store の書き口(`Intent`・`set_transient`・
//!    `apply_intent`)を一切名指ししない。選択の出力は `SelectLayers` message
//!    1本 — shell 側 Session への反映は結線(supervisor)の仕事で、pane が
//!    Document/Session を直接書き始めたらこの柵が問い直す。
//! 2. **hit 判定の正本は gizmo と共有**(発注 EXACT TARGET 4「優先順位を純関数で
//!    固定 — gizmo の hit 判定を再利用」): `src/marquee.rs` は
//!    `gizmo_hit_test(` を実際に呼ぶ(呼び出し構文)。マーキー側が自前の
//!    gizmo 命中判定を再発明して「見えている gizmo と触れる gizmo」の正本が
//!    割れたら(呼び出しが消えたら)ここで落ちる。

use std::path::Path;

fn marquee_source() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/marquee.rs");
    std::fs::read_to_string(&path).expect("src/marquee.rs を読める")
}

/// 柵1: marquee.rs は store の書き口を名指ししない(選択は message のみ)。
#[test]
fn marquee_never_touches_document_write_apis() {
    let text = marquee_source();
    let mut violations: Vec<String> = Vec::new();
    for (line_no, line) in text.lines().enumerate() {
        // 呼び出し/パス構文だけ拾う(doc comment の日本語文中での言及は
        // `Intent::`/`.set_transient(`/`.apply_intent(` の形を取らない)。
        for needle in ["Intent::", ".set_transient(", ".apply_intent("] {
            if line.contains(needle) {
                violations.push(format!("{}:{}: {}", needle, line_no + 1, line.trim()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "marquee.rs が Document の書き口へ触っている(選択は SelectLayers message \
         だけで運ぶ規律 — gizmo.rs と同じ):\n{}",
        violations.join("\n")
    );
}

/// 柵2: gizmo 命中判定は gizmo_hit_test の再利用(2つ目の hit 実装を作らない)。
#[test]
fn marquee_reuses_the_gizmo_hit_test_for_press_priority() {
    let text = marquee_source();
    let call_sites: Vec<&str> = text
        .lines()
        .filter(|line| {
            // 実呼び出し構文のみ(use 行や doc comment のバッククォート名指しは
            // 直後に `(layout` 等の引数が続かない)。
            line.contains("gizmo_hit_test(") && !line.trim_start().starts_with("//")
        })
        .collect();
    assert!(
        !call_sites.is_empty(),
        "marquee.rs が gizmo_hit_test を呼んでいない — press の優先順位判定が \
         gizmo の hit 正本から切り離された(2つ目の hit 実装の疑い)"
    );
}
