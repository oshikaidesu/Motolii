//! **非有限角度からの retrogade quat 構造柵**(`motolii-stage-pane` の
//! `tests/inverse_fence.rs`/`src/gizmo.rs::checked_inverse` と同型の「ソースを
//! 読んで具体的な行で落とす」軽い柵。src/ 内に置くと柵コード自身の
//! `.from_axis_angle(` という文字列リテラルを1件として数える自己参照になるため
//! integration test に分離)。
//!
//! ## 縛る不変
//!
//! `glam::Quat::from_axis_angle(axis, angle)`(glam-0.30.10)自体は `axis` が
//! 正規化済み単位ベクトルである限り `angle` がどんな値(NaN/±inf 含む)でも
//! その場では panic しない(`glam_assert!(axis.is_normalized())` は axis しか
//! 見ない)。しかし `angle` が非有限だと生成される quat 自体が非正規化(NaN)に
//! なり、その quat がそのまま(または他の quat と掛け合わされて)
//! `glam::Mat4::from_quat` へ渡ると、そちらの `glam_assert!
//! (rotation.is_normalized())` が**結果を返す前に**panic する
//! (`debug-glam-assert`/`glam-assert` feature が rerun フォーク経由で
//! ワークスペース全体へ unify されているため — AGENTS.md 「glam の `inverse()`
//! は自己アサートする」と同型。2026-08-22 実測: `camera::ResolvedCamera{
//! roll_degrees: NaN, .. }` で `camera_projection(..).view_matrix()` が
//! `assertion failed: rotation.is_normalized()` で panic)。
//!
//! これは「`Quat::from_axis_angle` を呼んでから結果を `is_finite()` で
//! 後始末する」形では防げない — 非正規化 quat は `from_axis_angle` の**戻り値の
//! 時点**では panic を起こさず、離れた場所(`Mat4::from_quat` 側)の assert まで
//! 静かに運ばれてしまうため。
//!
//! この製品での唯一の正解の形は `camera::safe_axis_angle`(angle を先に
//! `is_finite()` で検査し、非有限なら 0.0 へ丸めてから `from_axis_angle` を呼ぶ)。
//! 新しく実行時値(store から解決された property など、コンパイル時定数でない
//! 角度)を quat 化する開発者は:
//!
//! 1. `safe_axis_angle` を経由する(`camera::camera_projection` の `roll` と
//!    同じ形)、または
//! 2. angle が構造的に常に有限であることの根拠(例: コンパイル時定数)を
//!    `SAFE-GLAM-ASSERT:` から始まるコメントで直前に添える(`camera_projection`
//!    の `base`(定数 `PI`)と `safe_axis_angle` 自身の内部呼び出しが現在の例)。
//!
//! どちらでもない生 `.from_axis_angle(` 呼び出しが増えたらここで落ちる。
//!
//! **既知の穴**: この柵はテキストパターンマッチであって型システムではない —
//! `SAFE-GLAM-ASSERT:` 注記さえ書けば実際には非有限になりうる呼び出しも通って
//! しまう(注記の正しさまでは検証しない)。それでも「サイレントに素通りする生
//! `from_axis_angle`」をゼロにする効果はある。
//!
//! **意図的にスコープ外**: `Mat4::from_quat`(`view_matrix()`)・
//! `Mat4::perspective_infinite_reverse_rh`(`projection_matrix()`)・
//! `Quat` の `Mul`(`roll * base`)はいずれも glam_assert を持つか、それに
//! 準ずる不変を要求するが、この柵では追跡しない — この crate では
//! いずれも「唯一の入口(`safe_axis_angle`/定数)を通過した後の quat/f32」しか
//! 受け取らないことを `camera.rs` の doc コメントで示しており、`.inverse()` の
//! ように**呼び出し側が任意の値を渡せる汎用ユーティリティ**ではないため
//! (`camera_projection`/`view_matrix`/`projection_matrix` は1箇所のみに実装が
//! 存在し、そこで既に不変が成立している)。将来これらへの新しい呼び出し口が
//! 増えたら、その時点でこの柵を拡張すること。

use std::path::Path;

/// 対象ファイル(`src/` 直下の全 *.rs)。新規ファイルを足したらここへ追加する
/// (`inverse_fence.rs` と同じ理由でディレクトリ自動列挙より明示リストを採る)。
const SOURCE_FILES: &[&str] = &["camera.rs", "frame.rs", "lib.rs", "time.rs", "wide_div.rs"];

/// 注記を探す窓(行数)。`safe_axis_angle` 内の `SAFE-GLAM-ASSERT` は呼び出し行の
/// 2行上、`base` のものは3行上 — 余裕を見て6行まで遡る(`inverse_fence.rs` と同じ)。
const ANNOTATION_WINDOW: usize = 6;

fn read_src(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}

#[test]
fn every_raw_from_axis_angle_call_is_wrapped_or_annotated_safe() {
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
            if !line.contains("from_axis_angle(") {
                continue;
            }
            let window_start = idx.saturating_sub(ANNOTATION_WINDOW);
            let annotated = lines[window_start..=idx]
                .iter()
                .any(|context_line| context_line.contains("SAFE-GLAM-ASSERT"));
            if !annotated {
                violations.push(format!("{file}:{}: {}", idx + 1, line.trim()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "生の `.from_axis_angle(` 呼び出しが `SAFE-GLAM-ASSERT:` 注記なしで見つかった。\
         実行時角度(NaN/±inf になりうる値、例: store から解決された property)を \
         そのまま渡すと、その場では panic しないが生成される quat が非正規化(NaN)に \
         なり、離れた場所の `Mat4::from_quat` の `glam_assert!(rotation.is_normalized())` \
         まで運ばれて結果を返す前に panic する(camera::ResolvedCamera.roll_degrees の \
         実測済み欠陥と同じ経路)。`camera::safe_axis_angle` を経由するか、angle が \
         構造的に常に有限である根拠を直前に `SAFE-GLAM-ASSERT:` コメントで添えること:\n{}",
        violations.join("\n")
    );
}
