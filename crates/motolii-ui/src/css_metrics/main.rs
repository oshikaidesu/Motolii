//! CSS 計算値の抽出器具の CLI の皮。中身([`extract`](motolii_ui::css_metrics::extract))は
//! `motolii-ui` の lib 側(`src/css_metrics/mod.rs`)にある — `motolii-shell-iced`
//! 側の oracle テストが同じ関数を直接呼べるようにするため、ロジックを bin に
//! 閉じ込めていない。この main.rs は引数を読んで呼び、JSON をファイルへ書くだけ。
//!
//! ```text
//! cargo run -p motolii-ui --bin motolii-css-metrics -- inspector out/inspector.json
//! cargo run -p motolii-ui --bin motolii-css-metrics -- browser   out/browser.json
//! cargo run -p motolii-ui --bin motolii-css-metrics -- timeline  out/timeline.json
//! cargo run -p motolii-ui --bin motolii-css-metrics -- all       out/
//! cargo run -p motolii-ui --bin motolii-css-metrics -- inspector out/i.json --viewport 520x900
//! ```
//!
//! `all` は3枚を既定 viewport で `<out>/<panel>.json` へ書く。`--viewport WxH`
//! は1枚指定のときだけ既定を上書きする(3枚は幅の要求が違うので `all` では
//! 個別の既定値をそのまま使う)。既知の3名以外を渡すと html への直接パスとして
//! 扱う(`docs/mocks-ui/` 以外の fixture を後で足すときのため)。

use std::path::{Path, PathBuf};

use motolii_ui::css_metrics::{extract, PANELS};
use serde_json::Value;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut positional: Vec<String> = Vec::new();
    let mut viewport_override: Option<(u32, u32)> = None;
    let mut it = args.into_iter();
    while let Some(arg) = it.next() {
        if arg == "--viewport" {
            let value = it.next().unwrap_or_else(|| {
                eprintln!("--viewport には WxH が要る");
                std::process::exit(2);
            });
            viewport_override = Some(parse_viewport(&value));
        } else {
            positional.push(arg);
        }
    }

    let (panel, out) = match positional.as_slice() {
        [panel, out] => (panel.clone(), PathBuf::from(out)),
        _ => {
            eprintln!(
                "usage: motolii-css-metrics <inspector|browser|timeline|all|<path.html>> <out.json|out/> [--viewport WxH]"
            );
            std::process::exit(2);
        }
    };

    let mut failures: Vec<String> = Vec::new();
    if panel == "all" {
        std::fs::create_dir_all(&out).expect("out dir");
        for p in PANELS {
            let out_path = out.join(format!("{}.json", p.name));
            let viewport = viewport_override.unwrap_or(p.default_viewport);
            report(
                p.name,
                run(Path::new(p.html), viewport, &out_path),
                &mut failures,
            );
        }
    } else {
        ensure_parent_dir(&out);
        match PANELS.iter().find(|p| p.name == panel) {
            Some(p) => {
                let viewport = viewport_override.unwrap_or(p.default_viewport);
                report(
                    p.name,
                    run(Path::new(p.html), viewport, &out),
                    &mut failures,
                );
            }
            // 既知名でなければ html への直接パスとして扱う。
            None => {
                let viewport = viewport_override.unwrap_or((1200, 800));
                report(
                    &panel,
                    run(Path::new(&panel), viewport, &out),
                    &mut failures,
                );
            }
        }
    }

    if !failures.is_empty() {
        for failure in &failures {
            eprintln!("FAILED {failure}");
        }
        std::process::exit(1);
    }
}

fn ensure_parent_dir(out: &Path) {
    if let Some(parent) = out.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).expect("out dir");
        }
    }
}

fn parse_viewport(value: &str) -> (u32, u32) {
    let (w, h) = value.split_once('x').unwrap_or_else(|| {
        eprintln!("--viewport は WxH の形(例: 900x600) — 貰ったのは {value:?}");
        std::process::exit(2);
    });
    let parse = |s: &str| {
        s.parse::<u32>().unwrap_or_else(|_| {
            eprintln!("--viewport の数が読めない: {s:?}");
            std::process::exit(2);
        })
    };
    (parse(w), parse(h))
}

fn report(name: &str, result: Result<(), String>, failures: &mut Vec<String>) {
    match result {
        Ok(()) => println!("ok {name}"),
        Err(message) => failures.push(format!("{name}: {message}")),
    }
}

/// [`extract`] を呼んで結果を JSON ファイルへ書くだけ。
fn run(html_path: &Path, viewport: (u32, u32), out: &Path) -> Result<(), String> {
    let rows = extract(html_path, viewport)?;
    let count = rows.len();
    let payload = serde_json::to_string_pretty(&Value::Array(rows)).map_err(|e| e.to_string())?;
    std::fs::write(out, payload).map_err(|error| error.to_string())?;
    println!("  {} ({count} 要素)", out.display());
    Ok(())
}
