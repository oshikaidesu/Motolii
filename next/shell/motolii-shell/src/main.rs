//! wraps: iced — 凍結ホストの窓。product front は Makepad(裁定251/252)。
//! 判断は `motolii_shell` にあり、ここには置かない。
//!
//! ここが薄いのは運転席のため — 窓を開けずに `Shell` を直接動かせる形にしてある。
//!
//! `--fixture`(トンマナ検分器具)は「MV制作の途中に見える」Document で起動する
//! (既定起動は従来どおり空)。`--fixture --screenshot <path>` は窓を開かず
//! 1フレームを PNG へ書いて終了する(`motolii_shell::screenshot`)。
//!
//! `--screenshot` と併用できる状態フラグ4本(実機報告の検分用): `--checkerboard`
//! は Stage 下縁状態帯(裁定163)の市松トグルを ON にした状態を、`--transparent-bg` は背景
//! プリセット「Transparent」(alpha=0)を適用した状態を、`--observe` は観測カメラ
//! (裁定157)を有効にした状態を、**`--browser-open`(B3)は header の "Browser"
//! ボタンを押した状態**を、それぞれ実際の `Message` 経由(`Shell::update`)で再現する —
//! (旧 `--settings-open` は S2(裁定182/188)で撤去: Settings は OS 窓へ移住し、
//! 単窓オフスクリーン合成の screenshot 器具では写せない —
//! `motolii_shell::screenshot` 冒頭 doc「Settings は器具対象外」参照)—
//! ボタン/ホイール/中ボタンドラッグを実際に操作した時と同じ経路
//! (`--observe` は `stage::StageOverlay` が計算するのと同じ形の
//! `Message::Stage(stage::Message::Observe(_))` を1回出すだけ — Stage の
//! ホイールを実際に回した状態を再現する)。実窓起動でも同じ4本を
//! `Shell::boot` 後に適用するので、実窓観測の argv が「指定しただけで
//! 無視される」ことはない。

fn apply_visual_actions(shell: &mut motolii_shell::Shell, args: &[String]) {
    let checkerboard = args.iter().any(|a| a == "--checkerboard");
    let transparent_bg = args.iter().any(|a| a == "--transparent-bg");
    let observe = args.iter().any(|a| a == "--observe");
    let browser_open = args.iter().any(|a| a == "--browser-open");

    if transparent_bg {
        // SET+ 結線(第5波): 旧 `settings_pane::Message` は
        // `sections::Message::Legacy` が包む(`Shell::update_settings` doc)。
        let _ = shell.update(motolii_shell::Message::Settings(
            motolii_shell::settings_pane::sections::Message::Legacy(
                motolii_shell::settings_pane::Message::BackgroundPreset(
                    motolii_shell::settings_pane::BackgroundPreset::Transparent,
                ),
            ),
        ));
    }
    if checkerboard {
        // 裁定163: 市松トグルは Stage 下縁状態帯へ引っ越した。
        let _ = shell.update(motolii_shell::Message::Stage(
            motolii_shell::stage::Message::ToggleCheckerboard,
        ));
    }
    if browser_open {
        let _ = shell.update(motolii_shell::Message::Browser(
            motolii_shell::browser_pane::Message::ToggleBrowserPanel,
        ));
    }
    if observe {
        // Stage のホイール/中ボタンドラッグが実際に計算する物と同じ形の
        // 観測カメラ値を1回出す。枠が画面内に収まるようズームアウトする。
        let _ = shell.update(motolii_shell::Message::Stage(
            motolii_shell::stage::Message::Observe(motolii_engine::ObservationCamera {
                pan: [100.0, 60.0],
                zoom: 0.7,
            }),
        ));
    }
}

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
        let (mut shell, _task) = motolii_shell::Shell::new_fixture();
        apply_visual_actions(&mut shell, &args);
        motolii_shell::screenshot::write_png(&mut shell, std::path::Path::new(&path))
            .unwrap_or_else(|error| panic!("screenshot を書き出せない: {error}"));
        return Ok(());
    }

    // S1(裁定182/188): `iced::application` → `iced::daemon` 形(窓の浮かしの
    // 骨格 — `docs/reviews/2026-08-22-multiwindow-probe.md` §Q3)。daemon は
    // 自分では窓を開かないので、boot(`Shell::boot`/`boot_fixture`)が main 窓を
    // 1枚開く。main 窓1枚のときの挙動・見た目は従前どおり(窓の性質は
    // `window::Settings::default()` のまま)。main 窓を閉じたら exit
    // (`Message::WindowClosed` — probe 注意点1「窓ゼロ状態を作らない」)。
    let launch_args = args.clone();
    let boot = move || {
        let (mut shell, task) = if fixture {
            motolii_shell::Shell::boot_fixture()
        } else {
            motolii_shell::Shell::boot()
        };
        apply_visual_actions(&mut shell, &launch_args);
        (shell, task)
    };

    iced::daemon(
        boot,
        motolii_shell::Shell::update,
        motolii_shell::Shell::view_window,
    )
    .title(motolii_shell::Shell::window_title)
    .subscription(motolii_shell::Shell::subscription)
    // `.theme(...)` が無いと `Program::theme` の既定実装が常に `None` を返し、
    // 実窓は tokens と無関係な iced 組み込み Light/Dark(OS 設定依存)へ落ちる
    // (`tokens::theme_from_colors` の doc comment に該当行の引用あり)。
    // daemon 形では窓別引数が増えるだけ(probe §Q3)— 全窓同一 theme を
    // そのまま表現する。
    .theme(|shell: &motolii_shell::Shell, _window: iced::window::Id| {
        motolii_shell::tokens::theme_from_colors(&shell.colors())
    })
    .run()
}
