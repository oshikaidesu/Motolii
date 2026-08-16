//! P14: モックのHTML/CSSから、色と寸法のトークンをRustへ吐き出す。
//!
//! **これは変換機構の第一段**。狙いは1つで、
//! `timeline_blitz/html.rs` の冒頭にあるあの表 —
//!
//! ```text
//! | .rulerlabel color | #c0c0c0 | timeline_egui/ruler.rs:75 |
//! | .tick background  | #6a6a6a | timeline_egui/ruler.rs:84 |
//! ```
//!
//! — を人間が手で写すのをやめること。値の系譜は既に
//! `timeline_egui` → mock(HTML) → `timeline_blitz/html.rs` と一周していて、
//! eguiへ戻すならもう一周する。**そこを機械にやらせる。**
//!
//! やり方は「CSSを自前で解釈する」ではない。**Blitzをビルド時のコンパイラとして使う**:
//! Styloにカスケードを解かせ、Taffyに寸法を確定させ、その**計算済みの値**を読み出す。
//! だから `!important`、後勝ちの上書き、メディアクエリ、継承が全部効いた後の値が出る。
//!
//! 実行時にはBlitzは要らない。出力はツールキット非依存(`[u8;3]` と `f32`)。
//!
//! 使い方:
//!   mock_tokens <input.html> [W] [H] > tokens.rs
//!   mock_tokens <input.html> --prefix timeline   … class名で絞る

use std::collections::{BTreeMap, BTreeSet};

use blitz_dom::util::ToColorColor as _;
use blitz_dom::{DocumentConfig, StyleThreading};
use blitz_html::HtmlDocument;
use blitz_traits::shell::{ColorScheme, Viewport};

/// 1つのclassについて集めた値。同じclassの要素が複数あれば全部入る。
#[derive(Default)]
struct Collected {
    background: BTreeMap<String, usize>,
    color: BTreeMap<String, usize>,
    border_color: BTreeMap<String, usize>,
    border_width: BTreeMap<String, usize>,
    font_size: BTreeMap<String, usize>,
    height: BTreeMap<String, usize>,
    width: BTreeMap<String, usize>,
    count: usize,
}

/// 面の大きさ。既定の3つは意図的に選んである:
///   1280x500  … 設計の基準
///   1600x900  … 広げたとき
///   1000x600  … モックの `@media(max-width:1050px)` の下側
const DEFAULT_SIZES: [(u32, u32); 3] = [(1280, 500), (1600, 900), (1000, 600)];

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: mock_tokens <input.html> [--prefix <s>] [--sizes 1280x500,1600x900]");
        std::process::exit(2);
    }
    let path = args[1].clone();
    let prefix = args
        .iter()
        .position(|a| a == "--prefix")
        .and_then(|i| args.get(i + 1).cloned());
    let sizes: Vec<(u32, u32)> = args
        .iter()
        .position(|a| a == "--sizes")
        .and_then(|i| args.get(i + 1))
        .map(|s| {
            s.split(',')
                .filter_map(|p| {
                    let (w, h) = p.split_once('x')?;
                    Some((w.trim().parse().ok()?, h.trim().parse().ok()?))
                })
                .collect()
        })
        .unwrap_or_else(|| DEFAULT_SIZES.to_vec());
    assert!(!sizes.is_empty(), "面の大きさが1つも無い");

    let raw = std::fs::read_to_string(&path).expect("input html");
    let html = inline_stylesheets(&path, &raw);

    // **同じ文書を複数の面の大きさで解く。**
    // 大きさを変えても動かない値だけが「作者が決めた固定値」。
    // 動く値は `1fr` / `flex` / `%` / メディアクエリの結果であって、定数にしてはいけない。
    let runs: Vec<(String, BTreeMap<String, Collected>)> = sizes
        .iter()
        .map(|&(w, h)| (format!("{w}x{h}"), extract(&html, w, h)))
        .collect();

    let mut classes: BTreeSet<String> = BTreeSet::new();
    for (_, r) in &runs {
        classes.extend(r.keys().cloned());
    }
    if let Some(p) = &prefix {
        classes.retain(|k| k.starts_with(p.as_str()));
    }

    println!("// 自動生成: mock_tokens。手で編集しない。直すなら元のHTML/CSSを直す。");
    println!("// 出所: {path}");
    println!("// 生成の仕方: Styloにカスケードを解かせ、Taffyに寸法を確定させた");
    println!("// **計算済みの値**を読み出したもの。CSSのテキストではない。");
    println!("//");
    println!(
        "// 面の大きさ {} で解いて、**全部で同じだった値だけ**を定数にしている。",
        sizes.iter().map(|(w, h)| format!("{w}x{h}")).collect::<Vec<_>>().join(" / ")
    );
    println!("// 大きさで動く値は定数にせず、下の「面の大きさで動くもの」へ回す。");
    println!("//");
    println!("// 色は sRGB の [u8; 3]。ツールキット型は使わない(egui/Skia/CSSのどれからも読める)。");
    println!();

    let mut conflicts: Vec<String> = Vec::new();
    let mut responsive: Vec<String> = Vec::new();

    for class in &classes {
        let ident = to_ident(class);
        let mut lines = Vec::new();
        for (suffix, prop, pick) in PROPS {
            resolve_across_runs(
                &mut lines, &mut conflicts, &mut responsive, &runs, class, &ident, suffix, prop,
                *pick,
            );
        }
        if lines.is_empty() {
            continue;
        }
        let n = runs[0].1.get(class).map(|c| c.count).unwrap_or(0);
        println!("// .{class}  ({n} 要素)");
        for l in lines {
            println!("{l}");
        }
        println!();
    }

    responsive.sort();
    responsive.dedup();
    conflicts.sort();
    conflicts.dedup();

    if !responsive.is_empty() {
        println!("// ---- 面の大きさで動くもの(定数にしない) ----");
        println!("// `1fr` / `flex` / `%` / メディアクエリの結果。**実行時にレイアウトへ解かせる。**");
        println!("// 焼き込むとパネルを広げた瞬間に嘘になる。");
        for r in &responsive {
            println!("// {r}");
        }
        println!();
    }

    if !conflicts.is_empty() {
        println!("// ---- 一意に決まらなかったもの ----");
        println!("// 同じclassの要素が別々の値を持っている。**機械が決めてはいけない場所**なので");
        println!("// 出力しない。class名を分けるか、モック側で揃えるかは人が決める。");
        for c in &conflicts {
            println!("// {c}");
        }
    }
}

type Pick = fn(&Collected) -> &BTreeMap<String, usize>;

/// (定数の接尾辞, CSSでの名前, どのフィールドを見るか)
const PROPS: &[(&str, &str, Pick)] = &[
    ("BG", "background", |c| &c.background),
    ("FG", "color", |c| &c.color),
    ("BORDER", "border-color", |c| &c.border_color),
    ("BORDER_W", "border-width", |c| &c.border_width),
    ("FONT_SIZE", "font-size", |c| &c.font_size),
    ("HEIGHT", "height", |c| &c.height),
    ("WIDTH", "width", |c| &c.width),
];

/// 全ての面の大きさで同じ値になったときだけ定数にする。
fn resolve_across_runs(
    lines: &mut Vec<String>,
    conflicts: &mut Vec<String>,
    responsive: &mut Vec<String>,
    runs: &[(String, BTreeMap<String, Collected>)],
    class: &str,
    ident: &str,
    suffix: &str,
    prop: &str,
    pick: Pick,
) {
    let mut per_run: Vec<(String, Option<(String, usize, usize)>)> = Vec::new();
    for (label, map) in runs {
        let v = map.get(class).and_then(|c| {
            let values = pick(c);
            match values.len() {
                0 => None,
                1 => {
                    let (v, n) = values.iter().next().unwrap();
                    Some((v.clone(), *n, c.count))
                }
                // その面の中で既に割れている
                _ => Some((format!("\u{2}{}", describe(values)), 0, c.count)),
            }
        });
        per_run.push((label.clone(), v));
    }

    let present: Vec<&(String, usize, usize)> =
        per_run.iter().filter_map(|(_, v)| v.as_ref()).collect();
    if present.is_empty() {
        return;
    }

    if let Some(split) = present.iter().find(|(v, ..)| v.starts_with('\u{2}')) {
        conflicts.push(format!(".{class} {prop}: {}", &split.0[1..]));
        return;
    }

    let first = &present[0].0;
    if present.iter().all(|(v, ..)| v == first) {
        let (v, n, total) = present[0];
        let (expr, note) = split_value(v);
        let src = if *n < *total {
            format!("  ({n}/{total} 要素だけが持つ)")
        } else {
            String::new()
        };
        match note {
            Some(hex) => lines.push(format!("pub const {ident}_{suffix}: {expr}  // {hex}{src}")),
            None => lines.push(format!(
                "pub const {ident}_{suffix}: {expr}{}",
                if src.is_empty() {
                    String::new()
                } else {
                    format!("  //{src}")
                }
            )),
        }
    } else {
        let detail: Vec<String> = per_run
            .iter()
            .map(|(label, v)| match v {
                Some((v, ..)) => format!("{label}={}", short(v)),
                None => format!("{label}=なし"),
            })
            .collect();
        responsive.push(format!(".{class} {prop}: {}", detail.join("  ")));
    }
}

/// 表示用に短くする。色なら16進、寸法なら数字だけ。
fn short(v: &str) -> String {
    match split_value(v) {
        (_, Some(hex)) => hex,
        (expr, None) => expr
            .trim_end_matches(';')
            .trim_start_matches("f32 = ")
            .to_string(),
    }
}

fn describe(values: &BTreeMap<String, usize>) -> String {
    values
        .iter()
        .map(|(v, n)| format!("{} ×{n}", short(v)))
        .collect::<Vec<_>>()
        .join(" / ")
}

/// 1つの面の大きさで文書を解いて、classごとに値を集める
fn extract(html: &str, w: u32, h: u32) -> BTreeMap<String, Collected> {
    let mut doc = HtmlDocument::from_html(
        html,
        DocumentConfig {
            style_threading: StyleThreading::Sequential,
            ..Default::default()
        },
    );
    doc.set_viewport(Viewport {
        window_size: (w, h),
        hidpi_scale: 1.0,
        zoom: 1.0,
        color_scheme: ColorScheme::Dark,
    });
    doc.resolve(0.0);
    doc.resolve(0.0);

    let mut by_class = BTreeMap::new();
    walk(&doc, doc.root_element().id, &mut by_class);
    by_class
}

fn bump(m: &mut BTreeMap<String, usize>, key: String) {
    *m.entry(key).or_insert(0) += 1;
}

/// 値は "型と式" + 任意の注記 を `\u{1}` で区切って持っている
fn split_value(v: &str) -> (String, Option<String>) {
    match v.split_once('\u{1}') {
        Some((expr, note)) => (expr.to_string(), Some(note.to_string())),
        None => (v.to_string(), None),
    }
}

fn walk(doc: &HtmlDocument, id: usize, out: &mut BTreeMap<String, Collected>) {
    if let Some(node) = doc.get_node(id) {
        if let Some(el) = node.element_data() {
            if let Some(class_attr) = el.attr(blitz_dom::local_name!("class")) {
                // 先頭のclassをそのノードの素性とみなす(モックの書き方に合わせる)
                if let Some(class) = class_attr.split_whitespace().next() {
                    if let Some(style) = node.primary_styles() {
                        let e = out.entry(class.to_string()).or_default();
                        e.count += 1;

                        let current = style.clone_color();

                        let bg = style.clone_background_color().resolve_to_absolute(&current);
                        if bg.alpha > 0.0 {
                            bump(&mut e.background, color_token(bg.as_color_color().components));
                        }
                        bump(&mut e.color, color_token(current.as_color_color().components));

                        // 枠は**使用値**(Taffyが確定させた border box)で見る。
                        // 計算値の `border_top_width` は border-style:none でも
                        // `medium`(=3px)を返すので、そのまま読むと枠の無い要素にも
                        // 3px の枠があることになる。実際1度そうなった。
                        let used = node.final_layout.border;
                        let bw = used.top.max(used.left);
                        if bw > 0.0 {
                            bump(&mut e.border_width, format!("f32 = {bw:.1};"));
                            let b = style.get_border();
                            let side = if used.top > 0.0 {
                                b.border_top_color.clone()
                            } else {
                                b.border_left_color.clone()
                            };
                            let bc = side.resolve_to_absolute(&current);
                            if bc.alpha > 0.0 {
                                bump(&mut e.border_color, color_token(bc.as_color_color().components));
                            }
                        }

                        let fs = style.get_font().font_size.used_size.0.px();
                        bump(&mut e.font_size, format!("f32 = {fs:.1};"));

                        // 寸法は使用値(確定した矩形)から取る。CSSの指定値ではない。
                        let size = node.final_layout.size;
                        if size.height > 0.0 {
                            bump(&mut e.height, format!("f32 = {:.1};", size.height));
                        }
                        if size.width > 0.0 {
                            bump(&mut e.width, format!("f32 = {:.1};", size.width));
                        }
                    }
                }
            }
        }
        for c in node.children.iter() {
            walk(doc, *c, out);
        }
    }
}

/// stylo の型名を書かずに済むよう、変換済みの成分だけ受ける
fn color_token(components: [f32; 4]) -> String {
    let [r, g, b, a] = components;
    let to8 = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    let (r, g, b) = (to8(r), to8(g), to8(b));
    let hex = format!("#{r:02x}{g:02x}{b:02x}");
    if a >= 0.999 {
        format!("[u8; 3] = [0x{r:02x}, 0x{g:02x}, 0x{b:02x}];\u{1}{hex}")
    } else {
        format!(
            "[u8; 4] = [0x{r:02x}, 0x{g:02x}, 0x{b:02x}, 0x{:02x}];\u{1}{hex} alpha {a:.2}",
            to8(a)
        )
    }
}

/// `timelineRow` → `TIMELINE_ROW`
fn to_ident(class: &str) -> String {
    let mut out = String::new();
    for (i, ch) in class.chars().enumerate() {
        if ch.is_ascii_uppercase() && i > 0 {
            out.push('_');
            out.push(ch);
        } else if ch == '-' || ch == '.' {
            out.push('_');
        } else {
            out.push(ch.to_ascii_uppercase());
        }
    }
    out
}

/// `<link rel=stylesheet>` を実体に置き換える。Blitzに取得経路を持たせないため。
fn inline_stylesheets(path: &str, html: &str) -> String {
    let base = std::path::Path::new(path)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_default();
    let mut out = String::with_capacity(html.len() + 4096);
    let mut rest = html;
    while let Some(start) = rest.find("<link") {
        let Some(end) = rest[start..].find('>') else {
            break;
        };
        let tag = &rest[start..start + end + 1];
        out.push_str(&rest[..start]);
        if tag.contains("stylesheet") {
            if let Some(href) = attr_value(tag, "href") {
                let p = base.join(&href);
                match std::fs::read_to_string(&p) {
                    Ok(css) => {
                        out.push_str("<style>");
                        out.push_str(&css);
                        out.push_str("</style>");
                    }
                    Err(_) => eprintln!("mock_tokens: 読めない stylesheet: {href}"),
                }
            }
        } else {
            out.push_str(tag);
        }
        rest = &rest[start + end + 1..];
    }
    out.push_str(rest);
    out
}

fn attr_value(tag: &str, name: &str) -> Option<String> {
    let key = format!("{name}=\"");
    let i = tag.find(&key)? + key.len();
    let j = tag[i..].find('"')? + i;
    Some(tag[i..j].to_string())
}
