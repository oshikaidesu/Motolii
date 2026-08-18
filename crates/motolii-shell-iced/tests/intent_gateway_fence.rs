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

/// この crate の製品ソース全部。**足したらここへ足す**(走査漏れは静かな穴になる)。
const SCANNED: &[(&str, &str)] = &[
    ("lib.rs", include_str!("../src/lib.rs")),
    ("main.rs", include_str!("../src/main.rs")),
    ("intent_log.rs", include_str!("../src/intent_log.rs")),
    ("jsonl.rs", include_str!("../src/jsonl.rs")),
    ("launch.rs", include_str!("../src/launch.rs")),
    ("message.rs", include_str!("../src/message.rs")),
    ("prompts.rs", include_str!("../src/prompts.rs")),
    ("shell.rs", include_str!("../src/shell.rs")),
    ("status_log.rs", include_str!("../src/status_log.rs")),
    ("view.rs", include_str!("../src/view.rs")),
    ("window_input.rs", include_str!("../src/window_input.rs")),
];

/// 禁止する呼び出しと、代わりに通すべき intent。
/// `motolii_ui::blitz_shell` が `pub` で出している「製品状態を進める関数」である。
const FORBIDDEN: &[(&str, &str)] = &[
    ("create_project_file(", "UiIntent::NewProject"),
    ("reseat_project(", "UiIntent::NewProject / OpenProject"),
    ("admit_dropped_paths(", "UiIntent::AdmitPaths"),
    ("ProjectSeat::open(", "UiIntent::OpenProject"),
];

/// `ShellGateway::dispatch` を通らずに済む唯一の道は `Shell::update` である、
/// という主張を機械で保つ。ここが増えたら殻の設計が変わったということ。
const GATEWAY_CALL: &str = "self.gateway.dispatch(";

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
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut on_disk: Vec<String> = std::fs::read_dir(&src)
        .expect("src/ を読める")
        .filter_map(|entry| {
            let name = entry.expect("dir entry").file_name();
            let name = name.to_string_lossy().into_owned();
            name.ends_with(".rs").then_some(name)
        })
        .collect();
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
