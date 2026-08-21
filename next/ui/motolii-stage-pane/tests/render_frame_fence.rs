//! **構造的隔離の柵**(縫い目調査 `docs/reviews/2026-08-21-camera-seam-survey.md`
//! §3「担保案」、裁定160 切片10で `motolii-shell/src/stage.rs` からこの crate へ
//! 移設): `Engine::render_frame_with_view_camera` の呼び手は
//! [`motolii_stage_pane::observation_preview_source`] の1箇所だけ — export
//! (`screenshot.rs`・`main.rs`・`motolii-export`、いずれも `motolii-shell` root
//! crate 側)は観測カメラの存在自体を知らない。ソーステキストを grep するだけの
//! 軽い柵(`tonmana_token_fence.rs`(`motolii-shell`)と同型の「ソースを読んで
//! 具体的な行で落とす」手段)。呼び手が増えたら意図的な変更かどうかをこの試験が
//! 問い直す。
//!
//! **`src/` 直下の `#[cfg(test)]` ではなく独立した integration test にしてある
//! 理由**: この柵のソースコード自身が `.render_frame_with_view_camera(` という
//! 文字列を含む(grep パターンの文字列リテラル・この doc comment)。`src/` 内に
//! 置くと自分自身の柵コードを1件として数えてしまい、`call_sites.len() == 1` が
//! 常に2件以上になって落ちる(実際に発生した自己参照バグ、この分割で回避)。
//!
//! 対になる「`motolii-shell` 側は直接呼ばない」柵は
//! `motolii-shell/tests/suite/observation_camera_drive.rs::
//! motolii_shell_never_calls_render_frame_with_view_camera_directly` にある —
//! 呼び手が2箇所に増えることも、assembler が pane を経由せず直接呼ぶことも
//! 両方この2本の柵で拾う。

#[test]
fn render_frame_with_view_camera_is_only_called_from_this_crate() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut call_sites: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(&src_dir).expect("src/ を読める") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("read src file");
        for (line_no, line) in text.lines().enumerate() {
            // `.render_frame_with_view_camera(` = 実際の呼び出し(メソッド呼び出し
            // 構文)。コメント内の言及(バッククォート付きの名指し)は
            // `render_frame_with_view_camera(` の直前が `.` ではないので拾わない。
            if line.contains(".render_frame_with_view_camera(") {
                call_sites.push(format!("{}:{}: {}", path.display(), line_no + 1, line.trim()));
            }
        }
    }
    assert_eq!(
        call_sites.len(),
        1,
        "render_frame_with_view_camera の呼び手が1箇所ではない(export 経路が\
         観測カメラを知ってしまった可能性 — 裁定157の最重要不変):\n{}",
        call_sites.join("\n")
    );
    assert!(
        call_sites[0].contains("lib.rs"),
        "唯一の呼び手が lib.rs 以外にある: {}",
        call_sites[0]
    );
}
