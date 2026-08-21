//! wraps: iced — 窓を開けるだけ。判断は `motolii_shell` にあり、ここには置かない。
//!
//! ここが薄いのは運転席のため — 窓を開けずに `Shell` を直接動かせる形にしてある。
//!
//! `--fixture`(トンマナ検分器具)は「MV制作の途中に見える」Document で起動する
//! (既定起動は従来どおり空)。`--fixture --screenshot <path>` は窓を開かず
//! 1フレームを PNG へ書いて終了する(`motolii_shell::screenshot`)。
//!
//! `--screenshot` と併用できる状態フラグ4本(実機報告の検分用): `--checkerboard`
//! は Settings の市松トグルを ON にした状態を、`--transparent-bg` は背景
//! プリセット「Transparent」(alpha=0)を適用した状態を、`--settings-open` は
//! 歯車ボタンを押した状態を、`--observe` は観測カメラ(裁定157)を有効にした
//! 状態を、それぞれ実際の `Message` 経由(`Shell::update`)で再現する —
//! ボタン/ホイール/中ボタンドラッグを実際に操作した時と同じ経路
//! (`--observe` は `stage::StageOverlay` が計算するのと同じ形の
//! `Message::Stage(stage::Message::Observe(_))` を1回出すだけ — Stage の
//! ホイールを実際に回した状態を再現する)。

fn main() -> iced::Result {
    let args: Vec<String> = std::env::args().collect();
    let fixture = args.iter().any(|a| a == "--fixture");
    let screenshot_path = args
        .iter()
        .position(|a| a == "--screenshot")
        .and_then(|i| args.get(i + 1))
        .cloned();
    let checkerboard = args.iter().any(|a| a == "--checkerboard");
    let transparent_bg = args.iter().any(|a| a == "--transparent-bg");
    let settings_open = args.iter().any(|a| a == "--settings-open");
    let observe = args.iter().any(|a| a == "--observe");

    // `--screenshot` は窓を一切開かない一発ツール(検分器具の口)。fixture でしか
    // 意味を持たないので、`--fixture` の有無に関わらずここでは常に fixture を組む。
    if let Some(path) = screenshot_path {
        let (mut shell, _task) = motolii_shell::Shell::new_fixture();
        if transparent_bg {
            let _ = shell.update(motolii_shell::Message::Settings(
                motolii_shell::settings_pane::Message::BackgroundPreset(
                    motolii_shell::settings_pane::BackgroundPreset::Transparent,
                ),
            ));
        }
        if checkerboard {
            let _ = shell.update(motolii_shell::Message::Settings(
                motolii_shell::settings_pane::Message::ToggleCheckerboard,
            ));
        }
        if settings_open {
            let _ = shell.update(motolii_shell::Message::Settings(
                motolii_shell::settings_pane::Message::ToggleSettingsPanel,
            ));
        }
        if observe {
            // Stage のホイール/中ボタンドラッグが実際に計算する物と同じ形の
            // 観測カメラ値を1回出す(`stage::StageOverlay::update` が publish
            // する `Message::Stage(stage::Message::Observe(_))` と同じ経路)。
            // **ズームアウト側**(zoom<1)を選ぶ — ズームインすると comp の
            // フレーム枠が画面の外側へ広がって screenshot 内に写らなくなる
            // (数学的には正しい挙動 — 観測がフレームの内側へ寄るほど枠は
            // 視界の外へ出る)ので、枠が実際に写ることを目視で確かめる
            // instrument としては枠が画面内に収まるズームアウト+パンの
            // 組み合わせを選ぶ。
            let _ = shell.update(motolii_shell::Message::Stage(
                motolii_shell::stage::Message::Observe(motolii_engine::ObservationCamera {
                    pan: [100.0, 60.0],
                    zoom: 0.7,
                }),
            ));
        }
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
    // `.theme(...)` が無いと `Program::theme` の既定実装が常に `None` を返し、
    // 実窓は tokens と無関係な iced 組み込み Light/Dark(OS 設定依存)へ落ちる
    // (`tokens::theme_from_colors` の doc comment に該当行の引用あり)。
    .theme(|shell: &motolii_shell::Shell| motolii_shell::tokens::theme_from_colors(&shell.colors()))
    .run()
}
