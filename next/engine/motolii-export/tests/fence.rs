//! 背骨2 の柵 — **export は第二の合成器を建てられない**。
//!
//! 文言や慣習ではなく依存グラフで守る。`motolii-export` の依存に
//! `motolii-compositor` が入った瞬間、export は `Compositor::headless()` を
//! 呼べるようになり「書き出し専用の速い道」が作れてしまう。
//!
//! 2026-08-20 の敵対的レビューで、まさにその依存が(`CompSpec` を取るためだけに)
//! 入っていた。型を `motolii-core` へ出して切った。この試験はそれが戻らないことを見る。

#[test]
fn export_cannot_build_a_second_compositor() {
    let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .expect("Cargo.toml");

    // dev-dependencies 側は試験が絵を確かめるために使うので対象外。
    let deps = manifest
        .split("[dev-dependencies]")
        .next()
        .expect("dependencies 節");

    assert!(
        !deps.contains("motolii-compositor"),
        "motolii-export が motolii-compositor を依存に持っている。\n\
         これが入ると export は第二の Compositor を建てられる(背骨2)。\n\
         型が要るだけなら motolii-core へ出すこと。"
    );
}
