//! Timeline egui Lab の薄い起動殻。
//!
//! エディタ本体は `crates/motolii-ui/src/timeline_editor/` にあり、shell の
//! Timeline pane と同じ実装である。この example は fixture を座らせて窓を開くだけ。
//! 引数を1つ渡すと従来どおり、そのパスへ screenshot(BMP)を撮って終了する。
//!
//! 実行: `cargo run --profile fast -p motolii-ui --example timeline_egui_lab`

fn main() {
    if let Err(error) = motolii_ui::timeline_editor::run_lab(std::env::args().nth(1)) {
        eprintln!("timeline_egui_lab: {error}");
        std::process::exit(1);
    }
}
