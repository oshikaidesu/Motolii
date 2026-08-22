//! CSS 宣言リスト(`"display:flex; gap:4px; …"`)→ `taffy::Style` の変換口。
//!
//! モック転写レーンが mock の CSS 宣言をほぼコピペで写すための最小 parser。
//! **値の解釈は taffy の `parse` feature(upstream の cssparser 実装)へ委譲**し、
//! こちら側は宣言の分解・property 束縛・shorthand(padding/gap/flex)の展開だけを持つ。
//!
//! fail-closed: 対応外の property・解釈できない値・素の CSS として不正な形は、
//! 黙って無視せず必ず [`CssDeclError`] で落とす。転写レーンが対応外の宣言へ
//! 出会ったことが、握り潰されずに検収まで届くようにするための柵。
//!
//! 対応サブセット(property 単位):
//! - `display`(`flex` / `grid` / `block` / `none` — taffy の語彙)
//! - `flex-direction` / `flex-wrap` / `flex-grow` / `flex-shrink` / `flex-basis`
//! - `flex`(shorthand: `none` / `auto` / `<grow>` / `<grow> <shrink>` /
//!   `<grow> <basis>` / `<grow> <shrink> <basis>`)
//! - `justify-content` / `align-items`
//! - `gap`(1〜2値: `<row-gap> <column-gap>`)
//! - `padding`(1〜4値の CSS shorthand)
//! - `width` / `height` / `min-width` / `min-height` / `max-width` / `max-height`
//! - `grid-template-columns` / `grid-template-rows`(`minmax()` / `repeat()` /
//!   `fr` / `px` / `%` / `auto` — taffy の track parser の語彙)
//!
//! 値の単位は taffy parser の対応どおり `px` / `%` / `auto`(+素の `0`)のみ。
//! `em`・`rem`・`calc()` 等は Err — mock 側が px 正本である前提(裁定117 系の
//! tokens 運用)に合わせて狭く保つ。

use std::fmt;

/// CSS 宣言の変換に失敗した箇所の申告。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CssDeclError {
    /// `prop: value` の形をしていない断片。
    MalformedDeclaration {
        /// 元の断片。
        declaration: String,
    },
    /// このサブセットが対応しない property(fail-closed の本体)。
    UnsupportedProperty {
        /// property 名(小文字化済み)。
        property: String,
    },
    /// property は対応内だが値が解釈できない。
    InvalidValue {
        /// property 名(小文字化済み)。
        property: String,
        /// 元の値の字面。
        value: String,
        /// 解釈できなかった理由(taffy parser のメッセージ等)。
        reason: String,
    },
}

impl fmt::Display for CssDeclError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedDeclaration { declaration } => {
                write!(f, "`prop: value` の形をしていない宣言: {declaration:?}")
            }
            Self::UnsupportedProperty { property } => {
                write!(f, "対応外の property(fail-closed): {property:?}")
            }
            Self::InvalidValue {
                property,
                value,
                reason,
            } => {
                write!(f, "{property}: {value:?} を解釈できない — {reason}")
            }
        }
    }
}

impl std::error::Error for CssDeclError {}

/// CSS 宣言リストから `taffy::Style` を作る(`Style::default()` 起点)。
///
/// ```
/// use motolii_taffy::style_from_css_decl;
///
/// let style = style_from_css_decl(
///     "display:flex; gap:4px; justify-content:space-between",
/// )
/// .unwrap();
/// assert_eq!(style.justify_content, Some(taffy::JustifyContent::SPACE_BETWEEN));
/// ```
pub fn style_from_css_decl(decl: &str) -> Result<taffy::Style, CssDeclError> {
    let mut style = taffy::Style::default();
    apply_css_decl(&mut style, decl)?;
    Ok(style)
}

/// CSS 宣言リストを既存の `taffy::Style` へ上書き適用する
/// (base style + 差分、の合成用。[`style_from_css_decl`] はこれの糖衣)。
pub fn apply_css_decl(style: &mut taffy::Style, decl: &str) -> Result<(), CssDeclError> {
    for fragment in decl.split(';') {
        let fragment = fragment.trim();
        if fragment.is_empty() {
            continue;
        }
        let Some((property, value)) = fragment.split_once(':') else {
            return Err(CssDeclError::MalformedDeclaration {
                declaration: fragment.to_owned(),
            });
        };
        let property = property.trim().to_ascii_lowercase();
        let value = value.trim();
        if property.is_empty() || value.is_empty() {
            return Err(CssDeclError::MalformedDeclaration {
                declaration: fragment.to_owned(),
            });
        }

        match property.as_str() {
            "display" => style.display = parse_taffy(&property, value)?,
            "flex-direction" => style.flex_direction = parse_taffy(&property, value)?,
            "flex-wrap" => style.flex_wrap = parse_taffy(&property, value)?,
            "flex-grow" => style.flex_grow = parse_flex_number(&property, value)?,
            "flex-shrink" => style.flex_shrink = parse_flex_number(&property, value)?,
            "flex-basis" => style.flex_basis = parse_dimension(&property, value)?,
            "flex" => apply_flex_shorthand(style, value)?,
            "justify-content" => style.justify_content = Some(parse_taffy(&property, value)?),
            "align-items" => style.align_items = Some(parse_taffy(&property, value)?),
            "gap" => style.gap = parse_gap(&property, value)?,
            "padding" => style.padding = parse_padding(&property, value)?,
            "width" => style.size.width = parse_dimension(&property, value)?,
            "height" => style.size.height = parse_dimension(&property, value)?,
            "min-width" => style.min_size.width = parse_dimension(&property, value)?,
            "min-height" => style.min_size.height = parse_dimension(&property, value)?,
            "max-width" => style.max_size.width = parse_dimension(&property, value)?,
            "max-height" => style.max_size.height = parse_dimension(&property, value)?,
            "grid-template-columns" => {
                style.grid_template_columns = parse_track_list(&property, value)?;
            }
            "grid-template-rows" => {
                style.grid_template_rows = parse_track_list(&property, value)?;
            }
            _ => {
                return Err(CssDeclError::UnsupportedProperty { property });
            }
        }
    }
    Ok(())
}

/// taffy の `parse` feature が生やす `FromStr`(cssparser 実装)への橋。
fn parse_taffy<T>(property: &str, value: &str) -> Result<T, CssDeclError>
where
    T: std::str::FromStr,
    T::Err: fmt::Display,
{
    value.parse().map_err(|e: T::Err| CssDeclError::InvalidValue {
        property: property.to_owned(),
        value: value.to_owned(),
        reason: e.to_string(),
    })
}

/// `width`/`height`/`flex-basis` 系。taffy の `Dimension` parser は素の `0` を
/// 受けない(`px`/`%`/`auto` のみ)ので、CSS で合法な `0` だけこちらで足す。
fn parse_dimension(property: &str, value: &str) -> Result<taffy::Dimension, CssDeclError> {
    if value == "0" {
        return Ok(taffy::Dimension::length(0.0));
    }
    parse_taffy(property, value)
}

/// `gap`/`padding` の成分。`Dimension` 同様、素の `0` だけこちらで足す。
fn parse_length_percentage(
    property: &str,
    value: &str,
) -> Result<taffy::LengthPercentage, CssDeclError> {
    if value == "0" {
        return Ok(taffy::LengthPercentage::length(0.0));
    }
    parse_taffy(property, value)
}

/// `flex-grow`/`flex-shrink`: 有限・非負の素の数。
fn parse_flex_number(property: &str, value: &str) -> Result<f32, CssDeclError> {
    let invalid = |reason: &str| CssDeclError::InvalidValue {
        property: property.to_owned(),
        value: value.to_owned(),
        reason: reason.to_owned(),
    };
    let n: f32 = value
        .parse()
        .map_err(|_| invalid("素の数(例: 0, 1, 2.5)ではない"))?;
    if !n.is_finite() || n < 0.0 {
        return Err(invalid("flex の係数は有限・非負のみ"));
    }
    Ok(n)
}

/// `flex` shorthand(CSS 仕様の既定に合わせる: 数1つなら shrink=1・basis=0%)。
fn apply_flex_shorthand(style: &mut taffy::Style, value: &str) -> Result<(), CssDeclError> {
    const PROPERTY: &str = "flex";
    let invalid = |reason: &str| CssDeclError::InvalidValue {
        property: PROPERTY.to_owned(),
        value: value.to_owned(),
        reason: reason.to_owned(),
    };

    match value {
        "none" => {
            style.flex_grow = 0.0;
            style.flex_shrink = 0.0;
            style.flex_basis = taffy::Dimension::auto();
            return Ok(());
        }
        "auto" => {
            style.flex_grow = 1.0;
            style.flex_shrink = 1.0;
            style.flex_basis = taffy::Dimension::auto();
            return Ok(());
        }
        _ => {}
    }

    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.as_slice() {
        [grow] => {
            style.flex_grow = parse_flex_number(PROPERTY, grow)?;
            style.flex_shrink = 1.0;
            style.flex_basis = taffy::Dimension::percent(0.0);
        }
        [grow, second] => {
            style.flex_grow = parse_flex_number(PROPERTY, grow)?;
            // 2値形は `<grow> <shrink>` か `<grow> <basis>`(CSS 仕様どおり両対応)。
            if let Ok(shrink) = parse_flex_number(PROPERTY, second) {
                style.flex_shrink = shrink;
                style.flex_basis = taffy::Dimension::percent(0.0);
            } else {
                style.flex_shrink = 1.0;
                style.flex_basis = parse_dimension(PROPERTY, second)?;
            }
        }
        [grow, shrink, basis] => {
            style.flex_grow = parse_flex_number(PROPERTY, grow)?;
            style.flex_shrink = parse_flex_number(PROPERTY, shrink)?;
            style.flex_basis = parse_dimension(PROPERTY, basis)?;
        }
        _ => return Err(invalid("値は 1〜3 個(または none / auto)")),
    }
    Ok(())
}

/// `gap`: 1値(両方向)か 2値(`<row-gap> <column-gap>`)。
/// taffy の `Style::gap` は `width` = column-gap・`height` = row-gap。
fn parse_gap(
    property: &str,
    value: &str,
) -> Result<taffy::Size<taffy::LengthPercentage>, CssDeclError> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.as_slice() {
        [both] => {
            let g = parse_length_percentage(property, both)?;
            Ok(taffy::Size { width: g, height: g })
        }
        [row, column] => Ok(taffy::Size {
            width: parse_length_percentage(property, column)?,
            height: parse_length_percentage(property, row)?,
        }),
        _ => Err(CssDeclError::InvalidValue {
            property: property.to_owned(),
            value: value.to_owned(),
            reason: "gap の値は 1〜2 個".to_owned(),
        }),
    }
}

/// `padding`: CSS shorthand(1値=全辺 / 2値=縦・横 / 3値=上・横・下 / 4値=上右下左)。
fn parse_padding(
    property: &str,
    value: &str,
) -> Result<taffy::Rect<taffy::LengthPercentage>, CssDeclError> {
    let mut sides = Vec::with_capacity(4);
    for part in value.split_whitespace() {
        sides.push(parse_length_percentage(property, part)?);
    }
    let (top, right, bottom, left) = match sides.as_slice() {
        [all] => (*all, *all, *all, *all),
        [vertical, horizontal] => (*vertical, *horizontal, *vertical, *horizontal),
        [top, horizontal, bottom] => (*top, *horizontal, *bottom, *horizontal),
        [top, right, bottom, left] => (*top, *right, *bottom, *left),
        _ => {
            return Err(CssDeclError::InvalidValue {
                property: property.to_owned(),
                value: value.to_owned(),
                reason: "padding の値は 1〜4 個".to_owned(),
            });
        }
    };
    Ok(taffy::Rect {
        left,
        right,
        top,
        bottom,
    })
}

/// `grid-template-columns/rows`: 括弧の外の空白で track を区切り、各 track の解釈は
/// taffy の `GridTemplateComponent` parser(`minmax()`/`repeat()`/`fr`/`px`/`%`/`auto`)
/// へ委譲する。line 名(`[name]`)等の対応外は taffy 側の Err がそのまま届く。
fn parse_track_list<S: taffy::CheapCloneStr>(
    property: &str,
    value: &str,
) -> Result<Vec<taffy::GridTemplateComponent<S>>, CssDeclError> {
    let mut tracks = Vec::new();
    for part in split_outside_parens(value) {
        tracks.push(parse_taffy(property, part)?);
    }
    if tracks.is_empty() {
        return Err(CssDeclError::InvalidValue {
            property: property.to_owned(),
            value: value.to_owned(),
            reason: "track が 1 つも無い".to_owned(),
        });
    }
    Ok(tracks)
}

/// 括弧深度 0 の空白でだけ区切る(`repeat(3, 64px)` の中の空白は区切らない)。
fn split_outside_parens(value: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = None::<usize>;
    for (i, c) in value.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            c if c.is_whitespace() && depth == 0 => {
                if let Some(s) = start.take() {
                    parts.push(&value[s..i]);
                }
                continue;
            }
            _ => {}
        }
        if start.is_none() {
            start = Some(i);
        }
    }
    if let Some(s) = start {
        parts.push(&value[s..]);
    }
    parts
}
