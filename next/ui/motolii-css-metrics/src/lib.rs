//! wraps: blitz-dom+stylo(CSS 計算値抽出)。見た目を決めない・書き換えない器具
//!
//! 移植元 = 旧 workspace `crates/motolii-ui/src/css_metrics/`(歴史証拠として
//! 不変)。HTML モックを blitz-dom+stylo で解いて計算済み CSS 値(矩形・余白・
//! 文字寸)を抽出する — [`extract`] が入口。bin(`src/main.rs`)は CLI の皮を
//! 被せて呼ぶだけで、道具本体はここに置く。**呼び手をこの crate の外(pane の
//! oracle テスト)からも取れるようにする**のが理由 — 抽出ロジックを bin の中に
//! 閉じ込めると、テストは JSON ファイルを経由するか bin を subprocess で叩く
//! しかなくなる。関数として公開すればテストは直接呼べる。
//!
//! next/ での用途(裁定183 検証層): モック(`next/reference/mocks/*.html`、
//! 正本 — ここでは書き換えない)を機械で測り、実装(taffy container / 既存
//! pane)の実寸と照合する ±1px oracle の分母側。
//!
//! 素の html は `<link rel="stylesheet">` で css を引くが、blitz-html は
//! `<link>` の href を解決しない(移植元の実測)ので、Blitz へ渡す前に
//! この器具側で `<style>` へ inline する([`inline_stylesheets`])。next/ の
//! mock は panel CSS を `<style>` に内蔵し、`<link>` は
//! `/src/tokens/mock-candidates.css`(色トークン候補)だけを指す — これは
//! `next/reference/` 下に実在しないので解決失敗の警告を出して素通しし、
//! `var(--mock-role-*, fallback)` の fallback 側が効く(寸法はすべて fallback
//! に書かれているので抽出値は変わらない)。
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

/// 既知の1枚の名乗り・mock ファイル名・既定 viewport。
pub struct Panel {
    pub name: &'static str,
    /// `next/reference/mocks/` 内のファイル名。
    pub file: &'static str,
    pub default_viewport: (u32, u32),
}

impl Panel {
    /// mock への絶対パス。`CARGO_MANIFEST_DIR`(この crate)起点なので
    /// cwd に依存しない — bin もテストも同じ論理で通る。
    pub fn html_path(&self) -> PathBuf {
        mock_path(self.file)
    }
}

/// `next/reference/mocks/<file>` への絶対パス(cwd 非依存)。
pub fn mock_path(file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../reference/mocks")
        .join(file)
}

/// 既知の2枚(next/ の library mock)。旧 timeline-library.html は next/ に
/// 移植されていない(timeline-semantics.html は意味論 mock で寸法正本ではない)
/// ため登録しない — bin は既知名以外を html への直接パスとして扱うので、
/// 増えたらここへ足すだけで良い。
pub const PANELS: &[Panel] = &[
    Panel {
        name: "inspector",
        file: "inspector-library.html",
        // inspector-library.html `.inspectorShell { width: min(100%, 496px) }`
        // — 496 を上回る幅で min を 496 側へ倒す(移植元と同じ値)。
        default_viewport: (520, 900),
    },
    Panel {
        name: "browser",
        file: "browser-library.html",
        default_viewport: (900, 600),
    },
];

/// 文書1つを layout まで解いて、`<body>` 配下の全 Element(+生成 pseudo box)
/// を JSON の配列で返す。呼び出し元(bin / テスト)が使う唯一の入口。
///
/// `html_path` は相対でも絶対でも良い — `<link>` の解決([`resolve_href`])は
/// `html_path` 自身からの `.parent()` 連鎖だけで行うので、cwd には依存しない。
pub fn extract(html_path: &Path, viewport: (u32, u32)) -> Result<Vec<Value>, String> {
    let raw = std::fs::read_to_string(html_path)
        .map_err(|error| format!("{}: {error}", html_path.display()))?;
    let html = inline_stylesheets(&raw, html_path);

    // blitz-net の Provider は使わない — link は inline 済みで、mock は <img> を
    // 持たない(移植元の実測)。net_provider を省いても layout の値は変わらない。
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
    // 移植元(motolii-blitz-dump)と同じ理由(hoisted paint child の座標が
    // 1回遅れる)で2回。
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

// ---------------------------------------------------------------------------
// selector 照会 — next/ で足した薄い読み出し口(抽出結果は JSON 行の配列の
// ままだと呼び手が path 文字列を手で漁ることになるため)。対応するのは
// 「空白区切りの子孫結合子 + 単純 selector(tag / #id / .class / ::before /
// ::after の複合)」だけ — mock の既知値照合に要るのはこれで全部で、
// ここに CSS engine をもう1個作らない。
// ---------------------------------------------------------------------------

/// `selector` に合う行を文書順で全部返す。
pub fn select<'a>(rows: &'a [Value], selector: &str) -> Vec<&'a Value> {
    let parts: Vec<Simple> = selector.split_whitespace().map(Simple::parse).collect();
    if parts.is_empty() {
        return Vec::new();
    }
    rows.iter().filter(|row| matches(row, &parts)).collect()
}

/// `selector` に合う最初の行。**存在しなければ Err**(発注の受入条件)。
pub fn select_one<'a>(rows: &'a [Value], selector: &str) -> Result<&'a Value, String> {
    select(rows, selector)
        .into_iter()
        .next()
        .ok_or_else(|| format!("selector {selector:?} に合う要素が無い"))
}

/// 単純 selector 1個(複合可): `tag#id.class1.class2::before` の形。
struct Simple {
    tag: Option<String>,
    id: Option<String>,
    classes: Vec<String>,
    pseudo: Option<String>,
}

impl Simple {
    fn parse(part: &str) -> Simple {
        let (rest, pseudo) = match part.find("::") {
            Some(i) => (&part[..i], Some(part[i + 2..].to_string())),
            None => (part, None),
        };
        let mut tag = None;
        let mut id = None;
        let mut classes = Vec::new();
        // `.` / `#` の位置で区切る(selector も path セグメントも ASCII 前提
        // — mock の class/id 命名の実態)。
        let boundary: Vec<usize> = rest
            .char_indices()
            .filter(|&(_, c)| c == '.' || c == '#')
            .map(|(i, _)| i)
            .collect();
        let mut segments: Vec<&str> = Vec::new();
        let mut prev = 0usize;
        for &b in &boundary {
            segments.push(&rest[prev..b]);
            prev = b;
        }
        segments.push(&rest[prev..]);
        for segment in segments {
            if segment.is_empty() {
                continue;
            }
            if let Some(class) = segment.strip_prefix('.') {
                classes.push(class.to_string());
            } else if let Some(ident) = segment.strip_prefix('#') {
                id = Some(ident.to_string());
            } else {
                tag = Some(segment.to_string());
            }
        }
        Simple { tag, id, classes, pseudo }
    }

    /// path の1セグメント(`tag#id.cls1.cls2` または末尾 `::before` 付き)に
    /// 合うか。
    fn matches_segment(&self, segment: &str) -> bool {
        let other = Simple::parse(segment);
        if self.pseudo != other.pseudo {
            return false;
        }
        if let Some(tag) = &self.tag {
            if other.tag.as_deref() != Some(tag.as_str()) {
                return false;
            }
        }
        if let Some(id) = &self.id {
            if other.id.as_deref() != Some(id.as_str()) {
                return false;
            }
        }
        self.classes
            .iter()
            .all(|c| other.classes.iter().any(|oc| oc == c))
    }
}

/// 行の `path`(`body > div.a > button.b` / pseudo は `…::before`)へ子孫結合子
/// selector を照合する。末尾 part は行自身のセグメントに一致し、前段の part は
/// 祖先セグメント列の順序保存部分列に一致すること。
fn matches(row: &Value, parts: &[Simple]) -> bool {
    let Some(path) = row["path"].as_str() else {
        return false;
    };
    let segments: Vec<&str> = path.split(" > ").collect();
    let (last_part, ancestor_parts) = parts.split_last().expect("parts は非空");
    let Some((&last_segment, ancestor_segments)) = segments.split_last() else {
        return false;
    };
    if !last_part.matches_segment(last_segment) {
        return false;
    }
    // 祖先 part を順序を保って貪欲に消化する。
    let mut it = ancestor_segments.iter();
    'parts: for part in ancestor_parts {
        for segment in it.by_ref() {
            if part.matches_segment(segment) {
                continue 'parts;
            }
        }
        return false;
    }
    true
}

// ---------------------------------------------------------------------------
// 以下は移植元そのまま(パス正本の差し替え以外ロジック不変)。
// ---------------------------------------------------------------------------

/// `<link rel="stylesheet" href="...">` を対応する css の中身で `<style>` に
/// 置き換える。href が `/` 始まりなら html 自身のディレクトリから、それ以外も
/// html 自身のディレクトリからの相対で解決する。`html_path` の相対/絶対は
/// 問わない — cwd に依存させないため、すべて `html_path` 自身からの
/// `.parent()` 連鎖だけで解く([`resolve_href`])。解決に失敗した `<link>` は
/// 警告して素通しする(next/ の mock の `/src/tokens/mock-candidates.css` が
/// これに当たる — 寸法は `var(--…, fallback)` の fallback 側に書かれている
/// ので抽出値には効かない)。
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

/// `name="value"` を雑に拾う(mock は属性値を全部二重引用符で書いている —
/// 移植元の実測、next/ の2枚も同じ。単引用符 fixture が増えたらここを広げる)。
fn extract_attr(tag: &str, name: &str) -> Option<String> {
    let needle = format!("{name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let end = tag[start..].find('"')? + start;
    Some(tag[start..end].to_string())
}

/// `href`(`<link>` の)を実ファイルへ解く。`html_path` の cwd 非依存を保つため、
/// 根はすべて `html_path` 自身から `.parent()` で辿る — ハードコードした
/// 相対文字列は使わない。
fn resolve_href(href: &str, html_path: &Path) -> PathBuf {
    let html_dir = html_path.parent().unwrap_or_else(|| Path::new("."));
    match href.strip_prefix('/') {
        Some(stripped) => {
            // 第一候補: html 自身の隣。
            let first = html_dir.join(stripped);
            if first.exists() {
                first
            } else {
                // 第二候補: 一段上(移植元の実測: Vite dev server が `public/`
                // と project root を両方 `/` へ重ねて出す構成の写し)。
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
/// で、path は `parent_path` に suffix をそのまま付けるだけにする(mock は
/// 行の帯を `::before` で作る — 例: `.propertyRow::before{width:3px}`。
/// ここを歩かないとその実体を取り逃がす)。
///
/// 絶対座標は DOM の親子関係をそのまま辿って `final_layout.location` を足し
/// 上げる。inline formatting context が anonymous wrapper を挟む場合はズレ得る
/// が、対象は行・見出し・セルなど block 単位の箱なので実害は無い(移植元から
/// の既知の単純化)。
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
    // (blitz-dom `layout/construct.rs` の `flush_pseudo_elements` — 移植元の
    // 実測)。ここへ辿り着くのは `node.children`(実 DOM の子)経由か、
    // 呼び出し元が `node.before`/`node.after` を明示して来た時だけなので、
    // 後者を弾かず両方受ける。前者経由で無名 box に当たることはない —
    // inline formatting context の匿名 wrapper は `layout_children` 側にしか
    // 入らず、`children`(DOM の子)には出てこない。
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
/// `border_radius` / `border_color` は四辺・四隅を丸ごと出す(移植元の実測:
/// top 代表値だと `border-bottom` だけの行区切り線を取り逃がす)。
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
