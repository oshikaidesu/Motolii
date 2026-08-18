//! CSS 計算値の抽出器具の中身 — [`extract`] が入口。bin([`main.rs`](../css_metrics/main.rs)
//! ではなく `src/css_metrics/main.rs`)は CLI の皮を被せて呼ぶだけで、道具本体は
//! ここに置く。**呼び手を `motolii-ui` の外(`motolii-shell-iced` の
//! oracle テスト)からも取れるようにする**のが唯一の理由 — 抽出ロジックを
//! bin の中に閉じ込めると、テストは JSON ファイルを経由するか(実行のたび
//! 生成物が要る)、bin を subprocess で叩くしかなくなる。関数として公開すれば
//! テストは直接呼べる。
//!
//! **見た目を決めない・パネルを書き換えない。** 入力は
//! `docs/mocks-ui/public/{inspector,browser,timeline}-library.html`(正本、
//! ここでは書き換えない)そのもの。素の html は `<link rel="stylesheet">` で
//! css を引くが、blitz-html は `<link>` の href を解決しない(実測) ので、
//! Blitz へ渡す前にこの器具側で `<style>` へ inline する([`inline_stylesheets`])。
//!
//! GPU 不要。`document.resolve()` で layout だけ解き、`Node::final_layout`
//! (taffy `Layout` — border box の場所・大きさ・padding・border・margin)と
//! `Node::primary_styles()`(stylo `ComputedValues`)を読み戻すだけで、
//! 描画(`blitz-paint` / wgpu)は一切呼ばない。

use std::path::{Path, PathBuf};

use blitz_dom::{local_name, BaseDocument, DocumentConfig, NodeData, StyleThreading};
use blitz_html::HtmlDocument;
use blitz_traits::shell::{ColorScheme, Viewport};
use serde_json::{json, Value};
use style::properties::ComputedValues;
use style_traits::ToCss;

/// 既知の1枚の名乗り・html への相対パス・既定 viewport。
pub struct Panel {
    pub name: &'static str,
    pub html: &'static str,
    pub default_viewport: (u32, u32),
}

/// 既知の3枚。`docs/mocks-ui/public/` からの相対パスと既定 viewport。
pub const PANELS: &[Panel] = &[
    Panel {
        name: "inspector",
        html: "docs/mocks-ui/public/inspector-library.html",
        // inspector-library.css:18 `.inspectorShell { width: min(100%, 496px) }`
        default_viewport: (520, 900),
    },
    Panel {
        name: "browser",
        html: "docs/mocks-ui/public/browser-library.html",
        default_viewport: (900, 600),
    },
    Panel {
        name: "timeline",
        html: "docs/mocks-ui/public/timeline-library.html",
        // timeline-library.css:1 `body{min-width:960px}` — footer / keyTools まで収まる高さ。
        default_viewport: (1200, 760),
    },
];

/// 文書1つを layout まで解いて、`<body>` 配下の全 Element(+生成 pseudo box)
/// を JSON の配列で返す。呼び出し元(bin / テスト)が使う唯一の入口。
///
/// `html_path` は相対でも絶対でも良い — `<link>` の解決([`resolve_href`])は
/// `html_path` 自身からの `.parent()` 連鎖だけで行うので、cwd には依存しない
/// (bin は repo root からの相対、oracle テストは `CARGO_MANIFEST_DIR` からの
/// 絶対、のどちらで呼んでも同じ論理で通る)。
pub fn extract(html_path: &Path, viewport: (u32, u32)) -> Result<Vec<Value>, String> {
    let raw = std::fs::read_to_string(html_path)
        .map_err(|error| format!("{}: {error}", html_path.display()))?;
    let html = inline_stylesheets(&raw, html_path);

    // blitz-net の Provider は使わない — link は inline 済みで、3枚とも <img> を
    // 持たない(実測)。net_provider を省いても layout の値は変わらない。
    let mut document = HtmlDocument::from_html(
        &html,
        DocumentConfig {
            style_threading: StyleThreading::Sequential,
            ..Default::default()
        },
    );
    document.set_viewport(Viewport {
        window_size: viewport,
        hidpi_scale: 1.0,
        zoom: 1.0,
        color_scheme: ColorScheme::Dark,
    });
    // motolii-blitz-dump と同じ理由(hoisted paint child の座標が1回遅れる)で2回。
    document.resolve(0.0);
    document.resolve(0.0);

    let root = document.root_element();
    let body_id = root
        .children
        .iter()
        .copied()
        .find(|&id| {
            document
                .get_node(id)
                .is_some_and(|node| node.data.is_element_with_tag_name(&local_name!("body")))
        })
        .ok_or_else(|| "body 要素が見つからない".to_string())?;

    let mut rows = Vec::new();
    walk(&document, body_id, "", "", (0.0, 0.0), &mut rows);
    Ok(rows)
}

/// `<link rel="stylesheet" href="...">` を対応する css の中身で `<style>` に
/// 置き換える。href が `/` 始まりなら html 自身のディレクトリ(= `public/`)
/// から、それ以外はやはり html 自身のディレクトリからの相対で解決する(3枚の
/// html が両方の書き方を使う — 実測: inspector/browser は絶対、timeline は
/// 相対)。`html_path` の相対/絶対は問わない — cwd に依存させないため、
/// すべて `html_path` 自身からの `.parent()` 連鎖だけで解く
/// ([`resolve_href`])。
fn inline_stylesheets(html: &str, html_path: &Path) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(start) = rest.find("<link") {
        out.push_str(&rest[..start]);
        let Some(end_rel) = rest[start..].find('>') else {
            out.push_str(&rest[start..]);
            rest = "";
            break;
        };
        let end = start + end_rel + 1;
        let tag = &rest[start..end];
        if tag.contains("stylesheet") {
            match extract_attr(tag, "href") {
                Some(href) => {
                    let css_path = resolve_href(&href, html_path);
                    match std::fs::read_to_string(&css_path) {
                        Ok(css) => {
                            out.push_str("<style>\n");
                            out.push_str(&css);
                            out.push_str("\n</style>");
                        }
                        Err(error) => {
                            eprintln!(
                                "css-metrics: {} ({href}) を読めない: {error}",
                                css_path.display()
                            );
                        }
                    }
                }
                None => out.push_str(tag),
            }
        } else {
            out.push_str(tag);
        }
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

/// `name="value"` を雑に拾う(3枚の html は属性値を全部二重引用符で書いている
/// — 実測。単引用符 fixture が増えたらここを広げる)。
fn extract_attr(tag: &str, name: &str) -> Option<String> {
    let needle = format!("{name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let end = tag[start..].find('"')? + start;
    Some(tag[start..end].to_string())
}

/// `href`(`<link>` の)を実ファイルへ解く。`html_path` の cwd 非依存を保つため、
/// 根はすべて `html_path` 自身から `.parent()` で辿る — ハードコードした
/// 相対文字列は使わない(テストが `CARGO_MANIFEST_DIR` から絶対パスで
/// `html_path` を渡す時にも、bin が repo root から相対で渡す時にも同じ論理で通る)。
fn resolve_href(href: &str, html_path: &Path) -> PathBuf {
    let html_dir = html_path.parent().unwrap_or_else(|| Path::new("."));
    match href.strip_prefix('/') {
        Some(stripped) => {
            // 第一候補: html 自身の隣(`public/` — inspector/browser の
            // `/xxx-library.css` はここに実在する)。
            let first = html_dir.join(stripped);
            if first.exists() {
                first
            } else {
                // 第二候補: `public/` の一段上。実測:
                // `inspector-library.html` の `/src/tokens/mock-candidates.css`
                // は `public/` の外に居る(Vite dev server が `public/` と
                // project root を両方 `/` へ重ねて出す構成の写し)。
                html_dir.parent().unwrap_or(html_dir).join(stripped)
            }
        }
        None => html_dir.join(href),
    }
}

/// 1要素ぶんの行を作って `out` に積み、子(実 DOM の子 +
/// `::before`/`::after` の生成 box)へ再帰する。`AnonymousBlock` / text /
/// comment は飛ばす(実 DOM に無い箱・生成物と、行を持たないノード)。
///
/// `pseudo_suffix` が空でなければこの呼び出しは `::before`/`::after` の box
/// で、path は `parent_path` に suffix をそのまま付けるだけにする(この mock
/// は行の帯・見出しの accent bar を軒並み `::before` で作っている —
/// `.propertyRow::before{width:3px}` / `.panelHeader::before{width:3px;
/// height:13px}` — 実測。ここを歩かないと `dims::ROW_BAND_W` /
/// `HEADER_ACCENT_W/H` が指す実体を1個も取り逃がす)。
///
/// 絶対座標は DOM の親子関係をそのまま辿って `final_layout.location` を足し
/// 上げる。inline formatting context が anonymous wrapper を挟む場合はズレ得る
/// が、対象は行・見出し・セルなど block 単位の箱なので実害は無い(既知の
/// 単純化 — RETURN 側に明記)。
fn walk(
    document: &BaseDocument,
    node_id: usize,
    parent_path: &str,
    pseudo_suffix: &str,
    parent_abs: (f32, f32),
    out: &mut Vec<Value>,
) {
    let Some(node) = document.get_node(node_id) else {
        return;
    };
    // `::before`/`::after` の生成 box は `NodeData::Element` ではなく
    // `NodeData::AnonymousBlock`(タグは placeholder の "div")で作られる
    // (blitz-dom `layout/construct.rs` の `flush_pseudo_elements` — 実測)。
    // ここへ辿り着くのは `node.children`(実 DOM の子)経由か、呼び出し元が
    // `node.before`/`node.after` を明示して来た時だけなので、後者を弾かず
    // 両方受ける。前者経由で無名 box に当たることはない — inline formatting
    // context の匿名 wrapper は `layout_children` 側にしか入らず、
    // `children`(DOM の子)には出てこない。
    let element = match &node.data {
        NodeData::Element(e) | NodeData::AnonymousBlock(e) => e,
        _ => return,
    };

    let layout = node.final_layout;
    let abs_x = parent_abs.0 + layout.location.x;
    let abs_y = parent_abs.1 + layout.location.y;

    let tag = element.name.local.to_string();
    let id = node.attr(local_name!("id")).map(|s| s.to_string());
    let classes: Vec<String> = node
        .attr(local_name!("class"))
        .map(|s| s.split_whitespace().map(str::to_string).collect())
        .unwrap_or_default();

    let own_path = if !pseudo_suffix.is_empty() {
        format!("{parent_path}{pseudo_suffix}")
    } else {
        let mut segment = tag.clone();
        if let Some(i) = &id {
            segment.push('#');
            segment.push_str(i);
        }
        for c in &classes {
            segment.push('.');
            segment.push_str(c);
        }
        if parent_path.is_empty() {
            segment
        } else {
            format!("{parent_path} > {segment}")
        }
    };

    let computed = node
        .primary_styles()
        .map(|style| describe_style(&style))
        .unwrap_or(Value::Null);

    out.push(json!({
        "path": own_path,
        "id": id,
        "classes": classes,
        "tag": tag,
        "box": { "x": abs_x, "y": abs_y, "w": layout.size.width, "h": layout.size.height },
        "padding": edges(layout.padding),
        "border": edges(layout.border),
        "margin": edges(layout.margin),
        "computed": computed,
    }));

    for &child_id in &node.children {
        walk(document, child_id, &own_path, "", (abs_x, abs_y), out);
    }
    if let Some(before_id) = node.before {
        walk(
            document,
            before_id,
            &own_path,
            "::before",
            (abs_x, abs_y),
            out,
        );
    }
    if let Some(after_id) = node.after {
        walk(
            document,
            after_id,
            &own_path,
            "::after",
            (abs_x, abs_y),
            out,
        );
    }
}

fn edges(r: taffy::Rect<f32>) -> Value {
    json!({ "top": r.top, "right": r.right, "bottom": r.bottom, "left": r.left })
}

/// stylo の computed value を CSS 文字列へ落とす。
///
/// `border_radius` / `border_color` は最初 top 側だけの代表値で試したが、この
/// mock は角丸をほぼ使わず、線は `border-bottom` だけの行区切りが大半
/// (`.panelHeader` など)なので top 代表だと実際に見える線を取り逃がす
/// (実測 — `.panelHeader` は `border-top-color` が `currentcolor` を返す一方
/// `border-bottom-color` が本物の `#3b3b3b`)。四辺・四隅を丸ごと出す。
fn describe_style(style: &ComputedValues) -> Value {
    let font_size = style.clone_font_size().computed_size().to_css_string();
    let font_family = style.clone_font_family().to_css_string();
    let background = style.clone_background_color().to_css_string();
    let color = style.clone_color().to_css_string();
    let row_gap = style.clone_row_gap().to_css_string();
    let column_gap = style.clone_column_gap().to_css_string();
    json!({
        "background": background,
        "color": color,
        "font_size": font_size,
        "font_family": font_family,
        "border_radius": {
            "top_left": style.clone_border_top_left_radius().to_css_string(),
            "top_right": style.clone_border_top_right_radius().to_css_string(),
            "bottom_right": style.clone_border_bottom_right_radius().to_css_string(),
            "bottom_left": style.clone_border_bottom_left_radius().to_css_string(),
        },
        "border_color": {
            "top": style.clone_border_top_color().to_css_string(),
            "right": style.clone_border_right_color().to_css_string(),
            "bottom": style.clone_border_bottom_color().to_css_string(),
            "left": style.clone_border_left_color().to_css_string(),
        },
        "gap": { "row": row_gap, "column": column_gap },
    })
}
