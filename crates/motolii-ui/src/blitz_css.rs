//! パネルのCSSを**Rustの外**に置くための共有の口。
//!
//! CSSがRustの文字列リテラルに埋まっていると、色を1つ変えるのに `motolii-ui` の
//! 再ビルドが要る(2026-08-16 実測: 3.1〜23秒)。外に出すと、その往復から
//! **ビルドが消える** — ファイルを保存して dump を実行し直すだけになる(実測 0.69秒)。
//!
//! 既定では各パネルの `.css` はバイナリへ埋め込まれる(`include_str!`)。
//! `MOTOLII_BLITZ_CSS_DIR` を設定すると、そのディレクトリから実行時に読み直す:
//!
//! ```text
//! export MOTOLII_BLITZ_CSS_DIR=crates/motolii-ui/src
//! ./target/fast/motolii-blitz-dump timeline /tmp/t.png
//! ```
//!
//! 製品のバイナリは埋め込みを使うので、実行時にファイルを要求しない。

use std::borrow::Cow;

/// `relative` は `MOTOLII_BLITZ_CSS_DIR` からの相対path
/// (例 `timeline_blitz/timeline.css`)。読めなければ埋め込みへ落ちる。
pub(crate) fn template(relative: &str, embedded: &'static str) -> Cow<'static, str> {
    let Some(dir) = std::env::var_os("MOTOLII_BLITZ_CSS_DIR") else {
        return Cow::Borrowed(embedded);
    };
    let path = std::path::Path::new(&dir).join(relative);
    match std::fs::read_to_string(&path) {
        Ok(text) => Cow::Owned(text),
        Err(error) => {
            eprintln!(
                "blitz-css: {} を読めないので埋め込みで描く: {error}",
                path.display()
            );
            Cow::Borrowed(embedded)
        }
    }
}

/// `{name}` を差し替える。**長い名前から先に**渡すこと
/// (`surface_hi` を `surface` より先に置換しないと `{surface_hi}` が壊れる)。
pub(crate) fn fill(template: &str, values: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (name, value) in values {
        out = out.replace(&format!("{{{name}}}"), value);
    }
    out
}
