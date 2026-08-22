//! **構造的隔離の柵**(発注 2026-08-22「Viewer の家」、`src/viewer_bar.rs`
//! 冒頭 doc「この module の契約」)。`viewer_bar.rs` は視点タブ(B17)/画質
//! (B23)/チャンネル表示(B05 Viewer 側)の**部品+純関数だけ**を持ち、実配線
//! (`Engine`/`motolii-shell` への書き込み)は supervisor の仕事 — この不変を
//! ソーステキストの grep で縛る(`render_frame_fence.rs` と同型の「ソースを
//! 読んで具体的な行で落とす」軽い柵)。
//!
//! **`src/` 直下の `#[cfg(test)]` ではなく独立 integration test にしてある
//! 理由**は `render_frame_fence.rs` と同じ自己言及回避 — この柵は
//! `src/viewer_bar.rs` という**別ファイル**だけを読むので自己参照は起きない
//! (このファイル自身の doc comment がここに書く禁止文字列を含んでも、
//! 読む対象は `viewer_bar.rs` の方なので混入しない)が、`gizmo.rs`/
//! `render_frame_fence.rs` と同じ置き場の慣習(pane crate の構造柵は
//! `tests/` 配下)に揃えた。
//!
//! この柵は3本:
//! 1. `motolii_shell` を参照していない(実配線は supervisor 側 crate の仕事)。
//! 2. `&mut Engine` を取っていない(`crate::observation_preview_source` の
//!    ような書き口をここには作らない契約)。
//! 3. engine の render entry(`render_frame`/`render_frame_with_view_camera`/
//!    `render_frame_without_background`)を直接呼んでいない(呼ぶなら
//!    `crate::observation_preview_source`/`crate::checkerboard_preview_source`
//!    級の既存の書き口を経由するのが筋 — ここに新しい呼び手を増やさない)。

fn viewer_bar_source() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/viewer_bar.rs");
    std::fs::read_to_string(&path).expect("src/viewer_bar.rs を読める")
}

#[test]
fn viewer_bar_never_references_motolii_shell() {
    let text = viewer_bar_source();
    assert!(
        !text.contains("motolii_shell"),
        "viewer_bar.rs が motolii-shell を直接参照している(実配線は supervisor \
         側の仕事、発注書「純関数+部品として完結」の write-set 外)"
    );
}

#[test]
fn viewer_bar_never_takes_engine_by_mutable_reference() {
    let text = viewer_bar_source();
    assert!(
        !text.contains("&mut Engine"),
        "viewer_bar.rs が `&mut Engine` を取っている — この crate の他モジュール \
         (`observation_preview_source`)と違い、viewer_bar は書ける物を持たない \
         純関数+部品のみの契約(発注書「純関数+部品として完結(実配線は \
         supervisor)」)"
    );
}

#[test]
fn viewer_bar_never_calls_engine_render_entry_points_directly() {
    let text = viewer_bar_source();
    let banned_call_syntax = [
        ".render_frame(",
        ".render_frame_with_view_camera(",
        ".render_frame_without_background(",
        ".render_frame_to_texture(",
    ];
    for needle in banned_call_syntax {
        assert!(
            !text.contains(needle),
            "viewer_bar.rs が engine の render entry を直接呼んでいる: {needle} \
             (既存の書き口 `crate::observation_preview_source`/`crate:: \
             checkerboard_preview_source` を経由する筋、ここに新しい呼び手を \
             増やさない)"
        );
    }
}
