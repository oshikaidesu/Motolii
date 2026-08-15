//! 移植した Blitz パネル(Timeline / Browser / dock)を PNG に落とすだけの道具。
//!
//! ```text
//! cargo run -p motolii-ui --bin motolii-blitz-dump -- timeline out/timeline.png
//! cargo run -p motolii-ui --bin motolii-blitz-dump -- browser  out/browser.png
//! cargo run -p motolii-ui --bin motolii-blitz-dump -- dock     out/dock.png
//! cargo run -p motolii-ui --bin motolii-blitz-dump -- all      out/
//! ```
//!
//! **見た目をここで決めない。** HTML/CSS は3つとも既存実装がそのまま出したものを使い、
//! この bin は「文書を作る → 描く → 読み戻す → PNG にする」だけを持つ。
//! 描画経路は `browser_blitz/render.rs` の `BlitzSurface`(実証済み)を**使い回す**。
//! 新しい描画方式は作らない。
//!
//! Browser のサンプル枚数は `MOTOLII_BROWSER_MAX` / フォルダは `MOTOLII_BROWSER_DIR` で
//! 変えられる(既定は `oracle_main.rs` と同じ `docs/mocks` / `DEFAULT_MAX_ITEMS`)。

mod dock_sample;
mod gpu;

use std::path::{Path, PathBuf};

use blitz_dom::{DocumentConfig, StyleThreading};
use blitz_html::HtmlDocument;
use blitz_traits::shell::{ColorScheme, Viewport};

use motolii_ui::browser_blitz::render::BlitzSurface;
use motolii_ui::browser_blitz::{BrowserBlitzPanel, DEFAULT_MAX_ITEMS};
use motolii_ui::timeline_blitz::{project_for_blitz, timeline_html};

use gpu::Gpu;

/// timeline_blitz/mod.rs:59-66 のテストが使う面の大きさ。
const TIMELINE_W: u32 = 1000;
const TIMELINE_H: u32 = 460;
/// browser_blitz/oracle_main.rs:15-16 の写し。
const BROWSER_W: u32 = 900;
const BROWSER_H: u32 = 520;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (panel, out) = match args.as_slice() {
        [panel, out] => (panel.as_str(), PathBuf::from(out)),
        _ => {
            eprintln!("usage: motolii-blitz-dump <timeline|browser|dock|all> <out.png|out/>");
            std::process::exit(2);
        }
    };

    // blitz-net の Provider は Tokio reactor を要求する(実測の罠4)。
    // 文書構築・resolve・描画はすべてこの runtime の中で回す。
    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
    let gpu = Gpu::new(&runtime);

    let mut failures: Vec<String> = Vec::new();
    match panel {
        "all" => {
            std::fs::create_dir_all(&out).expect("out dir");
            for (name, run) in panels() {
                let path = out.join(format!("{name}.png"));
                report(name, run(&gpu, &runtime, &path), &mut failures);
            }
        }
        "timeline" | "browser" | "dock" => {
            if let Some(parent) = out.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent).expect("out dir");
                }
            }
            let run = panels()
                .into_iter()
                .find(|(name, _)| *name == panel)
                .expect("known panel")
                .1;
            report(panel, run(&gpu, &runtime, &out), &mut failures);
        }
        other => {
            eprintln!("unknown panel: {other}");
            std::process::exit(2);
        }
    }

    if !failures.is_empty() {
        for failure in &failures {
            eprintln!("FAILED {failure}");
        }
        std::process::exit(1);
    }
}

type Dump = fn(&Gpu, &tokio::runtime::Runtime, &Path) -> Result<(), String>;

fn panels() -> Vec<(&'static str, Dump)> {
    vec![
        ("timeline", dump_timeline as Dump),
        ("browser", dump_browser as Dump),
        ("dock", dump_dock as Dump),
    ]
}

fn report(name: &str, result: Result<(), String>, failures: &mut Vec<String>) {
    match result {
        Ok(()) => println!("ok {name}"),
        Err(message) => failures.push(format!("{name}: {message}")),
    }
}

/// 絵にするDocument。既定は `docs/mocks-ui/fixtures/reference-document.json`。
///
/// `Document::new_current()` は**空のDocument**で、投影が bar も key も出さない。
/// つまり Timeline は ruler と playhead しか描かれず、移植の結果が見えない。
/// 新しいfixtureはここで作らず、リポジトリに既にある参照Documentを読む。
const TIMELINE_DOC: &str = "docs/mocks-ui/fixtures/reference-document.json";

/// Timeline: `timeline_blitz::timeline_html` が出すHTMLをそのまま描く。
fn dump_timeline(gpu: &Gpu, runtime: &tokio::runtime::Runtime, out: &Path) -> Result<(), String> {
    let path = PathBuf::from(
        std::env::var("MOTOLII_TIMELINE_DOC").unwrap_or_else(|_| TIMELINE_DOC.to_string()),
    );
    let document = match motolii_doc::load_document(&path) {
        Ok(document) => document,
        Err(error) => {
            eprintln!(
                "blitz-dump: {} を読めないので空Documentで描く: {error}",
                path.display()
            );
            motolii_doc::Document::new_current()
        }
    };
    let projection = project_for_blitz(&document).map_err(|error| format!("{error:?}"))?;
    let html = timeline_html(
        &document,
        Some(&projection),
        None,
        motolii_core::RationalTime::ZERO,
        TIMELINE_W as f64,
        TIMELINE_H as f64,
    );
    dump_html(gpu, runtime, &html, TIMELINE_W, TIMELINE_H, out)
}

/// dock: `dock/css.rs` の `layout_html` が出すHTMLを、ダンプ側のスタイルシートで描く。
fn dump_dock(gpu: &Gpu, runtime: &tokio::runtime::Runtime, out: &Path) -> Result<(), String> {
    let html = dock_sample::dock_html();
    dump_html(
        gpu,
        runtime,
        &html,
        dock_sample::WIDTH,
        dock_sample::HEIGHT,
        out,
    )
}

/// Browser: `BrowserBlitzPanel` をそのまま使う。HTMLもrender経路も既存のもの。
fn dump_browser(gpu: &Gpu, _runtime: &tokio::runtime::Runtime, out: &Path) -> Result<(), String> {
    let dir = PathBuf::from(
        std::env::var("MOTOLII_BROWSER_DIR").unwrap_or_else(|_| "docs/mocks".to_string()),
    );
    let max_items = std::env::var("MOTOLII_BROWSER_MAX")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_MAX_ITEMS);

    // `BrowserBlitzPanel` は自前で reactor を張る(browser_blitz/mod.rs:85)。
    let mut panel = BrowserBlitzPanel::from_folder(
        &gpu.device,
        &gpu.adapter,
        &gpu.queue,
        gpu::FORMAT,
        BROWSER_W,
        BROWSER_H,
        &dir,
        max_items,
    )
    .map_err(|error| error.to_string())?;
    eprintln!(
        "blitz-dump: browser {} → {} 件",
        dir.display(),
        panel.items().len()
    );

    let (texture, view) = gpu.target(BROWSER_W, BROWSER_H);
    // `blitz-net` の fetch は非同期。1フレームだけ描くと画像がまだ届いておらず
    // cardが空のまま出る(`oracle_main.rs` は60秒描き続けるので気付けない)。
    // 枚数が増えなくなるまで描き直してから読み戻す。
    let mut settled = 0;
    let mut last = usize::MAX;
    for _ in 0..120 {
        panel.render(&gpu.device, &gpu.queue, &view);
        let count = panel.cached_image_count();
        settled = if count == last { settled + 1 } else { 0 };
        last = count;
        if settled >= 10 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    eprintln!(
        "blitz-dump: browser atlas images = {} / items = {}",
        panel.cached_image_count(),
        panel.items().len()
    );
    save(gpu, &texture, BROWSER_W, BROWSER_H, out)
}

/// 文書1つを描いてPNGにする。`browser_blitz/mod.rs:221-245` の `build_document` と
/// 同じ設定を使う(`StyleThreading::Sequential` / `blitz_net::Provider` / Dark)。
fn dump_html(
    gpu: &Gpu,
    runtime: &tokio::runtime::Runtime,
    html: &str,
    width: u32,
    height: u32,
    out: &Path,
) -> Result<(), String> {
    // 絵が期待どおりでない時に「HTMLが出していないのか、描けていないのか」を
    // 切り分けるための窓。既定では書かない。
    if std::env::var_os("MOTOLII_BLITZ_DUMP_HTML").is_some() {
        let path = out.with_extension("html");
        std::fs::write(&path, html).map_err(|error| error.to_string())?;
        println!("  {}", path.display());
    }

    let _guard = runtime.enter();
    let mut document = HtmlDocument::from_html(
        html,
        DocumentConfig {
            style_threading: StyleThreading::Sequential,
            net_provider: Some(blitz_net::Provider::shared(None)),
            ..Default::default()
        },
    );
    document.set_viewport(Viewport {
        window_size: (width, height),
        hidpi_scale: 1.0,
        zoom: 1.0,
        color_scheme: ColorScheme::Dark,
    });
    document.resolve(0.0);

    let mut surface = BlitzSurface::new(
        &gpu.device,
        &gpu.adapter,
        &gpu.queue,
        gpu::FORMAT,
        width,
        height,
    );
    let (texture, view) = gpu.target(width, height);
    surface.render(&mut document, &gpu.device, &gpu.queue, &view);
    save(gpu, &texture, width, height, out)
}

/// 読み戻して unpremultiply し、確認用の下地の上に重ねて書く。
/// 生のまま(透過込み)も `*-raw.png` として残す。
fn save(
    gpu: &Gpu,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
    out: &Path,
) -> Result<(), String> {
    let premultiplied = gpu::read_texture(&gpu.device, &gpu.queue, texture, width, height);
    gpu::write_png(
        out,
        &gpu::over_checkerboard(&premultiplied, width, height),
        width,
        height,
    );
    let raw = out.with_file_name(format!(
        "{}-raw.png",
        out.file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_else(|| "panel".to_string())
    ));
    gpu::write_png(&raw, &gpu::unpremultiply(&premultiplied), width, height);
    println!("  {} / {}", out.display(), raw.display());
    Ok(())
}
