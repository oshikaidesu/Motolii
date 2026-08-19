//! 構造フェンス — 新しい殻でも、副作用は**単一ゲートウェイ**の外から起こせない。
//!
//! egui shell 側の同じ柵(`crates/motolii-ui/tests/shell_intent_gateway_fence.rs`)を
//! この crate へ持ち越したものである。持ち越しが要るのは、M-0 で
//! `ShellGateway` を公開した際、その隣に居た `create_project_file` /
//! `reseat_project` / `admit_dropped_paths` も**既に `pub` だった**からで、
//! 新しい殻はそれらを直接呼んで journal を迂回できてしまう。
//!
//! ここで塞いでおかないと、iced 側だけ「`--intent-log` に載らない操作」が
//! 生えて、replay oracle が静かに嘘になる。
//!
//! 走査の規約は egui 版と同じ: `#[cfg(test)]` から先と `//` で始まる行
//! (この禁止リストを説明する doc コメント自身)は落とす。
//!
//! M-3 から Timeline の編集がある。編集の唯一の書き口
//! (`ProjectSeat::editor_mut`)を禁止リストへ足したので、canvas / pane が
//! intent を通らずにエディタを動かす道は名前ごと塞がっている。
//!
//! M-2(2026-08-19)で `resume.rs` が加わった。`--project` の解決は**窓より前**、
//! journal がまだ無い時点の話で、egui 版 `blitz_shell/runner.rs` が同じ理由で
//! `ProjectSeat::open` を直接呼んでいるのと同じ扱いである(あちらの fence は
//! この禁止をそもそも持たない)。開いた座席は `ShellGateway::seated` /
//! `resumed` に渡され、journal の第1行に `OpenProject` を合成で持つので、
//! journal を迂回してはいない。[`BOOT_TIME_PROJECT_OPEN`] がこの1箇所だけを
//! 名指しで免除する。

/// この crate の製品ソース全部。**足したらここへ足す**(走査漏れは静かな穴になる)。
const SCANNED: &[(&str, &str)] = &[
    ("lib.rs", include_str!("../src/lib.rs")),
    ("main.rs", include_str!("../src/main.rs")),
    ("browser.rs", include_str!("../src/browser.rs")),
    ("browser_pane.rs", include_str!("../src/browser_pane.rs")),
    ("inspector_model.rs", include_str!("../src/inspector_model.rs")),
    ("inspector_pane.rs", include_str!("../src/inspector_pane.rs")),
    ("intent_log.rs", include_str!("../src/intent_log.rs")),
    ("jsonl.rs", include_str!("../src/jsonl.rs")),
    ("launch.rs", include_str!("../src/launch.rs")),
    ("message.rs", include_str!("../src/message.rs")),
    ("prompts.rs", include_str!("../src/prompts.rs")),
    ("resume.rs", include_str!("../src/resume.rs")),
    ("shell.rs", include_str!("../src/shell.rs")),
    ("shortcuts.rs", include_str!("../src/shortcuts.rs")),
    ("stage_arbiter.rs", include_str!("../src/stage_arbiter.rs")),
    ("stage_bridge.rs", include_str!("../src/stage_bridge.rs")),
    ("stage_island.rs", include_str!("../src/stage_island.rs")),
    ("status_log.rs", include_str!("../src/status_log.rs")),
    ("theme/mod.rs", include_str!("../src/theme/mod.rs")),
    ("theme/style.rs", include_str!("../src/theme/style.rs")),
    ("view.rs", include_str!("../src/view.rs")),
    (
        "widgets/context_menu.rs",
        include_str!("../src/widgets/context_menu.rs"),
    ),
    (
        "widgets/drop_zone.rs",
        include_str!("../src/widgets/drop_zone.rs"),
    ),
    (
        "widgets/key_button.rs",
        include_str!("../src/widgets/key_button.rs"),
    ),
    ("widgets/mod.rs", include_str!("../src/widgets/mod.rs")),
    (
        "widgets/scrub_value.rs",
        include_str!("../src/widgets/scrub_value.rs"),
    ),
    ("window_input.rs", include_str!("../src/window_input.rs")),
    ("timeline/mod.rs", include_str!("../src/timeline/mod.rs")),
    ("timeline/canvas.rs", include_str!("../src/timeline/canvas.rs")),
    ("timeline/keys.rs", include_str!("../src/timeline/keys.rs")),
    ("timeline/pane.rs", include_str!("../src/timeline/pane.rs")),
    (
        "timeline/semantics.rs",
        include_str!("../src/timeline/semantics.rs"),
    ),
    (
        "timeline/waveform.rs",
        include_str!("../src/timeline/waveform.rs"),
    ),
];

/// 禁止する呼び出しと、代わりに通すべき intent。
/// `motolii_ui::blitz_shell` が `pub` で出している「製品状態を進める関数」である。
///
/// M-3 で `editor_mut(` を足した: `ProjectSeat::editor_mut` は Timeline エディタの
/// **唯一の書き口**で、これさえ出て来なければ移動・トリム・削除・Undo など
/// エディタの可変 API は1つも呼べない(egui shell が持つ `project_mut()` の穴を
/// この crate に作らない、の機械化)。
const FORBIDDEN: &[(&str, &str)] = &[
    ("create_project_file(", "UiIntent::NewProject"),
    ("reseat_project(", "UiIntent::NewProject / OpenProject"),
    ("admit_dropped_paths(", "UiIntent::AdmitPaths"),
    ("ProjectSeat::open(", "UiIntent::OpenProject"),
    (
        "editor_mut(",
        "UiIntent::SelectLayer / MoveClips / TrimClip / DeleteSelection / Undo / Redo / SetPlayhead / StepPlayhead",
    ),
];

/// `ShellGateway::dispatch` を通らずに済む唯一の道は `Shell::update` である、
/// という主張を機械で保つ。ここが増えたら殻の設計が変わったということ。
const GATEWAY_CALL: &str = "self.gateway.dispatch(";

/// `ProjectSeat::open(` 1つだけを免除する file。**窓より前**(journal が無い時点)
/// の `--project` 解決がここに在り、egui 版 `runner.rs` と同じ理由で正当
/// (module doc 参照)。他の禁止パターンはここにも普通に効く。
const BOOT_TIME_PROJECT_OPEN: &[&str] = &["resume.rs"];

fn product_source(source: &str) -> String {
    let body = match source.find("#[cfg(test)]") {
        Some(at) => &source[..at],
        None => source,
    };
    body.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn no_shell_code_touches_product_state_outside_the_gateway() {
    let mut breaches: Vec<String> = Vec::new();
    for (name, source) in SCANNED {
        let product = product_source(source);
        for (pattern, replacement) in FORBIDDEN {
            if *pattern == "ProjectSeat::open(" && BOOT_TIME_PROJECT_OPEN.contains(name) {
                continue;
            }
            let hits = product.matches(pattern).count();
            if hits > 0 {
                breaches.push(format!(
                    "  src/{name}: `{pattern}` が {hits} 箇所 — {replacement} を dispatch する"
                ));
            }
        }
    }
    assert!(
        breaches.is_empty(),
        "ゲートウェイの外から製品状態を直接動かしている:\n{}\n\
         journal を通らない副作用は --intent-log に載らず replay で再現できない。\
         `ShellGateway::dispatch(UiIntent::…)` へ寄せること",
        breaches.join("\n")
    );
}

/// **走査漏れが無い**ことの担保。[`SCANNED`] は手で並べた表なので、`src/` に
/// file を足して表に足し忘れると、この柵は静かに何も守らなくなる。
#[test]
fn every_product_source_is_scanned() {
    fn walk(dir: &std::path::Path, prefix: &str, out: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir).expect("src/ を読める") {
            let entry = entry.expect("dir entry");
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = entry.path();
            if path.is_dir() {
                // 子ディレクトリも掘る(M-3 で `timeline/` が増えた —
                // ここが平らな read_dir のままだと、下の file は柵を素通りする)。
                walk(&path, &format!("{prefix}{name}/"), out);
            } else if name.ends_with(".rs") {
                out.push(format!("{prefix}{name}"));
            }
        }
    }
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut on_disk: Vec<String> = Vec::new();
    walk(&src, "", &mut on_disk);
    on_disk.sort();
    let mut listed: Vec<String> = SCANNED.iter().map(|(name, _)| (*name).to_owned()).collect();
    listed.sort();
    assert_eq!(
        on_disk, listed,
        "SCANNED が src/ の実体と噛み合っていない — 足した file は表にも足すこと\
         (表に無い file はフェンスを素通りする)"
    );
}

/// 禁止リストが**空振りしていない**ことの担保。名前が実在の API と噛み合って
/// いなければ、この柵は何も守らないのに緑になる。
#[test]
fn the_forbidden_names_are_real_public_api_of_motolii_ui() {
    // 参照するだけ。呼びはしない(呼んだらこのファイル自身が上のテストに落ちる)。
    let _: fn(&std::path::Path) -> Result<(), String> =
        motolii_ui::blitz_shell::create_project_file;
    let _: fn(&std::path::Path) -> Result<motolii_ui::blitz_shell::ProjectSeat, String> =
        motolii_ui::blitz_shell::ProjectSeat::open;
    // `editor_mut` — Timeline エディタの唯一の書き口が実在することの担保。
    let _ = motolii_ui::blitz_shell::ProjectSeat::editor_mut;
}

/// 副作用の唯一の入口が `Shell::update` の中に**実在する**こと。
/// (禁止リストだけでは「何も呼んでいない殻」でも緑になる)
#[test]
fn the_only_door_is_shell_update() {
    let shell = product_source(include_str!("../src/shell.rs"));
    let doors = shell.matches(GATEWAY_CALL).count();
    assert!(
        doors > 0,
        "`{GATEWAY_CALL}` が shell.rs に1つも無い — 殻が製品状態へ着く道を失ったか、\
         綴りが変わった(綴り違いのフェンスは何も守らない)"
    );

    for (name, source) in SCANNED {
        if *name == "shell.rs" {
            continue;
        }
        assert_eq!(
            product_source(source).matches(".dispatch(").count(),
            0,
            "src/{name} が dispatch を呼んでいる — 入口は Shell::update だけである"
        );
    }
}
