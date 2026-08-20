//! wraps: iced — 窓を開けるだけ。判断は `motolii_shell` にあり、ここには置かない。
//!
//! ここが薄いのは運転席のため — 窓を開けずに `Shell` を直接動かせる形にしてある。
//!
//! `--fixture`(トンマナ検分器具)は「MV制作の途中に見える」Document で起動する
//! (既定起動は従来どおり空)。`--fixture --screenshot <path>` は窓を開かず
//! 1フレームを PNG へ書いて終了する(`motolii_shell::screenshot`)。

fn main() -> iced::Result {
    let args: Vec<String> = std::env::args().collect();
    let fixture = args.iter().any(|a| a == "--fixture");
    let screenshot_path = args
        .iter()
        .position(|a| a == "--screenshot")
        .and_then(|i| args.get(i + 1))
        .cloned();

    // `--screenshot` は窓を一切開かない一発ツール(検分器具の口)。fixture でしか
    // 意味を持たないので、`--fixture` の有無に関わらずここでは常に fixture を組む。
    if let Some(path) = screenshot_path {
        let (shell, _task) = motolii_shell::Shell::new_fixture();
        motolii_shell::screenshot::write_png(&shell, std::path::Path::new(&path))
            .unwrap_or_else(|error| panic!("screenshot を書き出せない: {error}"));
        return Ok(());
    }

    let boot: fn() -> (motolii_shell::Shell, iced::Task<motolii_shell::Message>) = if fixture {
        motolii_shell::Shell::new_fixture
    } else {
        motolii_shell::Shell::new
    };

    iced::application(
        boot,
        motolii_shell::Shell::update,
        motolii_shell::Shell::view,
    )
    .title(motolii_shell::Shell::title)
    .subscription(motolii_shell::Shell::subscription)
    .run()
}
