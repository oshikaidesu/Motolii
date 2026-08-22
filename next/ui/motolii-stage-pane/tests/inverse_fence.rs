//! **退化行列 inverse の構造柵**(`marquee_fence.rs`/`viewer_bar_fence.rs` と同型の
//! 「ソースを読んで具体的な行で落とす」軽い柵。src/ 内に置くと柵コード自身の
//! `.inverse()` という文字列リテラルを1件として数える自己参照になるため
//! integration test に分離)。
//!
//! ## 縛る不変
//!
//! `glam::Affine2::inverse()`(内部で `Mat2::inverse()` を呼ぶ)は、退化行列
//! (det=0、例: レイヤーの scale を 0 にした時)へ呼ぶと `glam_assert!(det != 0.0)`
//! で**結果を返す前に自己アサートして panic** する(`debug-glam-assert`/
//! `glam-assert` feature がワークスペースのどこかで有効化され unify されているため)。
//! これは「呼んでから `is_finite()` で後始末する」形では防げない — その形が
//! `gizmo::anchor_value` の実測済み panic の根本原因だった(2026-08-22 発注 —
//! Scale X=0 で Anchor Point ハンドルを掴むと落ちる)。
//!
//! この製品での唯一の正解の形は `gizmo::checked_inverse`(det を先に見て、
//! 非有限または 0 なら inverse を呼ばずに `None` を返す)。新しい `.inverse()`
//! 呼び出しを足す開発者は:
//!
//! 1. `checked_inverse` を経由する(`anchor_value`/`gizmo_hit_test`/
//!    `GizmoDragState::begin`/`render_camera_frame_corners`/
//!    `sheet_screen_from_frame` と同じ形)、または
//! 2. det が構造的に常に非零であることの根拠を `SAFE-INVERSE:` から始まる
//!    コメントで直前に添える(`scale_value` の rot_skew — 回転+shear は
//!    det=1 で常に可逆 — が唯一の現在の例外)。
//!
//! どちらでもない生 `.inverse()` 呼び出しが増えたらここで落ちる。
//!
//! **既知の穴**: この柵はテキストパターンマッチであって型システムではない —
//! `SAFE-INVERSE:` 注記さえ書けば実際には退化しうる呼び出しも通ってしまう
//! (注記の正しさまでは検証しない)。それでも「サイレントに素通りする生
//! inverse」をゼロにする効果はある。

use std::path::Path;

/// 対象ファイル(`src/` 直下の全 *.rs)。新規ファイルを足したらここへ追加する
/// (ディレクトリを自動列挙すると `env!("CARGO_MANIFEST_DIR")` 経由の相対パス組立
/// より複雑になる割に、この crate は今のところファイル数が少ないため明示リストで足並みを揃える)。
const SOURCE_FILES: &[&str] = &[
    "gizmo.rs",
    "lib.rs",
    "marquee.rs",
    "sheets.rs",
    "viewer_bar.rs",
    "zoom.rs",
];

/// 注記を探す窓(行数)。`scale_value` の `SAFE-INVERSE:` はメソッドチェーンの
/// 都合で呼び出し行の4行上にある — 余裕を見て6行まで遡る。
const ANNOTATION_WINDOW: usize = 6;

fn read_src(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}

#[test]
fn every_raw_affine_inverse_call_is_checked_or_annotated_safe() {
    let mut violations: Vec<String> = Vec::new();

    for &file in SOURCE_FILES {
        let text = read_src(file);
        let lines: Vec<&str> = text.lines().collect();

        for (idx, line) in lines.iter().enumerate() {
            // 呼び出し構文だけ拾う(doc comment/line comment の日本語文中での
            // 言及は無視 — `///`/`//` はどちらも trim 後 `//` で始まる)。
            if line.trim_start().starts_with("//") {
                continue;
            }
            if !line.contains(".inverse()") {
                continue;
            }
            let window_start = idx.saturating_sub(ANNOTATION_WINDOW);
            let annotated = lines[window_start..=idx]
                .iter()
                .any(|context_line| context_line.contains("SAFE-INVERSE"));
            if !annotated {
                violations.push(format!("{file}:{}: {}", idx + 1, line.trim()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "生の `.inverse()` 呼び出しが `checked_inverse` 経由でも `SAFE-INVERSE:` \
         注記付きでもない状態で見つかった。退化行列(det=0、例: scale 0)へ呼ぶと \
         glam の `Mat2::inverse()` が `glam_assert!(det != 0.0)` で panic する \
         (anchor_value の実測済み欠陥と同じ経路)。`gizmo::checked_inverse` を \
         経由するか、det が構造的に常に非零である根拠を直前に `SAFE-INVERSE:` \
         コメントで添えること:\n{}",
        violations.join("\n")
    );
}
