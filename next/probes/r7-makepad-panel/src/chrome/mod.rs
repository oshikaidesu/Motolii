//! 共通 chrome 部品。他パネルから `ChromeFace` 等を import する。
//! Splash eval: `ScrollYView` 禁止（白紙）。未登録 `Chrome*` を `panel.splash` に直書きするな。
//! 実験は `--hot`。登録してから葉へ `ChromeGallery{}` だけ載せる。
//! Dock 葉は `View` / `SolidView` / `Label` / `ButtonFlat` の直載せだけ。
//! 閉集合: 面 / 文字 / 線 / ボタン / 行 / 数値 / 検索。Document を持たない。
//! 色・寸法: `ui-scale-and-z.html` 候補B と r7 `panel.splash`。新色は置かない。
//!
//! 別名は代入だけ。`set_type_default()` を SolidView / View / Label / ButtonFlat 等の
//! Makepad 基底に使うな（型既定を書き換え、Fit の中の Fill が 0px、全面灰色・エラー無し）。
use makepad_widgets::*;

pub mod gallery;
pub mod parts;

mod button {
    use makepad_widgets::*;
    script_mod! {
        use mod.prelude.widgets.*
        use mod.widgets.*
        mod.widgets.ChromeButton = ButtonFlat{
            width: Fit
            height: 24
            padding: Inset{left: 8 right: 8}
            draw_bg.color: #x3e3e3e
            draw_bg.color_hover: #x464646
            draw_bg.color_down: #x242424
            draw_bg.border_size: 0.0
            draw_bg.border_radius: 0.0
            draw_text.color: #xb8b8b8
            draw_text.text_style: theme.font_regular{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
        }
    }
}

pub fn script_mod(vm: &mut ScriptVm) {
    button::script_mod(vm);
    parts::script_mod(vm);
    gallery::script_mod(vm);
}

#[cfg(test)]
mod type_default_fence {
    use std::fs;
    use std::path::Path;

    fn overwrites_makepad_base(line: &str) -> bool {
        let Some(idx) = line.find("set_type_default()") else {
            return false;
        };
        let rest = line[idx + "set_type_default()".len()..].trim_start();
        let Some(rest) = rest.strip_prefix("do") else {
            return false;
        };
        let rest = rest.trim_start();
        if rest.starts_with("mod.widgets.") || rest.starts_with("#(") {
            return false;
        }
        rest.starts_with(|c: char| c.is_ascii_alphabetic())
    }

    fn walk(dir: &Path, hits: &mut Vec<String>) {
        for entry in fs::read_dir(dir).expect("r7 src") {
            let entry = entry.expect("entry");
            let path = entry.path();
            if path.is_dir() {
                walk(&path, hits);
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }
            let text = fs::read_to_string(&path).expect("read");
            for (number, line) in text.lines().enumerate() {
                if overwrites_makepad_base(line) {
                    hits.push(format!("{}:{}: {line}", path.display(), number + 1));
                }
            }
        }
    }

    #[test]
    fn chrome_does_not_set_type_default_on_makepad_bases() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut hits = Vec::new();
        walk(&root, &mut hits);
        assert!(
            hits.is_empty(),
            "set_type_default on Makepad bases (gray, no error):\n{}",
            hits.join("\n")
        );
        let face = format!(
            "mod.widgets.ChromeFace = set_type_default() do {}",
            "SolidView{"
        );
        let ink = format!("mod.widgets.ChromeInk = set_type_default() do {}", "Label{");
        assert!(overwrites_makepad_base(&face));
        assert!(overwrites_makepad_base(&ink));
        assert!(!overwrites_makepad_base(
            "mod.widgets.ChromeGallery = set_type_default() do mod.widgets.ChromeGalleryBase{"
        ));
    }
}
