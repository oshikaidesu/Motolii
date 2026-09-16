
use cosmic_text::fontdb;
use cosmic_text::{
    Align, Attrs, Buffer, Command, Family, FeatureTag, FontFeatures, FontSystem, Metrics, Shaping,
    SwashCache,
};

use crate::doc::vector::{Contour, Point, Vertex};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphFont {
    pub path: String,
    pub family: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextJustify {
    #[default]
    Left,
    Right,
    Center,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextFeature {
    pub tag: String,
    pub value: u32,
}

/// CSS `text-autospace`(CSS Text 4 §8.4): 漢字と欧文の字・数字の境に、字送りの 1/8(0.125ic)を足す。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum TextAutospace {
    #[default]
    Normal,
    NoAutospace,
}

/// CSS `text-spacing-trim`(CSS Text 4 §8.5): 全角の約物を、行の端と隣どうしで半角に詰める。
/// 隣どうしは font の `chws`、行頭は行分割の後で左半分を削る(halt と同じ量)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum TextSpacingTrim {
    #[default]
    Normal,
    SpaceAll,
    SpaceFirst,
    TrimStart,
}

/// CSS `hanging-punctuation`(CSS Text 4 §9.2.1): `none | [ first || [ force-end | allow-end ] || last ]`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct HangingPunctuation {
    pub first: bool,
    pub end: HangEnd,
    pub last: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum HangEnd {
    #[default]
    None,
    AllowEnd,
    ForceEnd,
}

impl TextAutospace {
    pub const CHOICES: &'static [&'static str] = &["Normal", "No Autospace"];
    pub fn to_enum_value(self) -> i64 { self as i64 }
    pub fn from_enum_value(v: i64) -> Option<Self> { [Self::Normal, Self::NoAutospace].get(usize::try_from(v).ok()?).copied() }
}

impl TextSpacingTrim {
    pub const CHOICES: &'static [&'static str] = &["Normal", "Space All", "Space First", "Trim Start"];
    pub fn to_enum_value(self) -> i64 { self as i64 }
    pub fn from_enum_value(v: i64) -> Option<Self> { [Self::Normal, Self::SpaceAll, Self::SpaceFirst, Self::TrimStart].get(usize::try_from(v).ok()?).copied() }
}

impl HangingPunctuation {
    /// 文法の順(first, end, last)で全部の組合せ。
    pub const CHOICES: &'static [&'static str] = &[
        "None", "First", "Allow End", "Force End", "Last", "First Allow End", "First Force End", "First Last",
        "Allow End Last", "Force End Last", "First Allow End Last", "First Force End Last",
    ];
    const TABLE: [(bool, HangEnd, bool); 12] = [
        (false, HangEnd::None, false), (true, HangEnd::None, false), (false, HangEnd::AllowEnd, false), (false, HangEnd::ForceEnd, false),
        (false, HangEnd::None, true), (true, HangEnd::AllowEnd, false), (true, HangEnd::ForceEnd, false), (true, HangEnd::None, true),
        (false, HangEnd::AllowEnd, true), (false, HangEnd::ForceEnd, true), (true, HangEnd::AllowEnd, true), (true, HangEnd::ForceEnd, true),
    ];
    pub fn to_enum_value(self) -> i64 {
        Self::TABLE.iter().position(|&(first, end, last)| first == self.first && end == self.end && last == self.last).unwrap_or(0) as i64
    }
    pub fn from_enum_value(v: i64) -> Option<Self> {
        Self::TABLE.get(usize::try_from(v).ok()?).map(|&(first, end, last)| Self { first, end, last })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextLayout {
    /// 折返しと揃えの幅。None なら point text(揃えは効かない — cosmic-text は幅が無いと補正 0)。
    pub wrap_width: Option<f32>,
    /// 折り返すか。幅は揃え(Center/Right)にも要るので、point text は幅を持って折り返さない。
    pub wrap: bool,
    pub size: f32,
    pub line_height: Option<f32>,
    pub tracking: f32,
    pub justify: TextJustify,
    pub features: Vec<TextFeature>,
    pub autospace: TextAutospace,
    pub spacing_trim: TextSpacingTrim,
    pub hanging: HangingPunctuation,
}

impl TextLayout {
    pub fn new(size: f32) -> Self {
        Self {
            wrap_width: None,
            wrap: false,
            size,
            line_height: None,
            tracking: 0.0,
            justify: TextJustify::default(),
            features: Vec::new(),
            autospace: TextAutospace::default(),
            spacing_trim: TextSpacingTrim::default(),
            hanging: HangingPunctuation::default(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TextShapeError {
    #[error("font file {path} could not be read: {source}")]
    FontFile {
        path: String,
        source: std::io::Error,
    },
    #[error("OpenType feature tag {0:?} is not 4 bytes")]
    FeatureTag(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineMeasure {
    pub baseline_y: f32,
    pub width: f32,
    pub glyph_xs: Vec<f32>,
    /// 字ごとの元の文字の byte の位置(組み直しても同じ字を対にする。移り方と物差し)。
    pub glyph_bytes: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ShapedText {
    pub contours: Vec<Contour>,
    pub contour_styles: Vec<usize>,
    /// 輪郭ごとの文字の番(組んだ順)。morph が同じ番同士を対にする。
    pub contour_glyphs: Vec<usize>,
    pub lines: Vec<LineMeasure>,
}

/// 書体の台帳と shaper は process に 1 つ(cosmic-text の指示: 作るのは 1 秒級、一度だけ共有せよ)。
/// OS の書体を全部積み、locale は ja-JP — 漢字の後詰めがヒラギノになり、絵文字・欧文は OS の
/// fallback(Apple Color Emoji・.SF NS)へ落ちる。`path` は台帳に無い書体を足す口として残す。
fn font_system() -> std::sync::MutexGuard<'static, FontSystem> {
    static SYSTEM: std::sync::OnceLock<std::sync::Mutex<FontSystem>> = std::sync::OnceLock::new();
    SYSTEM
        .get_or_init(|| {
            let mut db = fontdb::Database::new();
            db.load_system_fonts();
            std::sync::Mutex::new(FontSystem::new_with_locale_and_db("ja-JP".to_owned(), db))
        })
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

/// Families available to the same database used to shape the document.
pub fn font_families() -> &'static [String] {
    static FAMILIES: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    FAMILIES.get_or_init(|| {
        let system = font_system();
        let names: std::collections::BTreeSet<_> = system.db().faces()
            .flat_map(|face| face.families.iter().map(|(name, _)| name.clone()))
            .collect();
        names.into_iter().collect()
    })
}

/// 台帳に在る family の一覧。face を 1 枚ずつ名前で照合すると、組む度に台帳を全走査する
/// —— 書体見本のように 1 行ごとに組み直す所では、それが組む時間の大半になる。
/// 台帳の枚数が変わった時だけ作り直す。
fn family_is_known(db: &fontdb::Database, family: &str) -> bool {
    static KNOWN: std::sync::OnceLock<std::sync::Mutex<(usize, std::collections::HashSet<String>)>> = std::sync::OnceLock::new();
    let mut cache = KNOWN
        .get_or_init(|| std::sync::Mutex::new((usize::MAX, std::collections::HashSet::new())))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if cache.0 != db.len() {
        cache.1 = db.faces().flat_map(|f| f.families.iter().map(|(name, _)| name.clone())).collect();
        cache.0 = db.len();
    }
    cache.1.contains(family)
}

/// family が台帳に無ければ path の file を足す。path も読めなければ、その時だけ誤り。
fn ensure_font(system: &mut FontSystem, font: &GlyphFont) -> Result<(), TextShapeError> {
    let known = |db: &fontdb::Database| family_is_known(db, &font.family);
    if known(system.db()) || font.path.is_empty() {
        return Ok(());
    }
    system
        .db_mut()
        .load_font_file(&font.path)
        .map_err(|source| TextShapeError::FontFile { path: font.path.clone(), source })
}

pub struct StyledText<'a> {
    pub text: &'a str,
    pub font: &'a GlyphFont,
    pub layout: &'a TextLayout,
    pub style: usize,
}

pub fn shape_text(content: &str, font: &GlyphFont, layout: &TextLayout) -> Result<ShapedText, TextShapeError> {
    shape_rich_text(&[StyledText { text: content, font, layout, style: 0 }], layout)
}

pub fn shape_rich_text(spans: &[StyledText<'_>], layout: &TextLayout) -> Result<ShapedText, TextShapeError> {
    let mut guard = font_system();
    let font_system = &mut *guard;
    let attributes = span_attributes(font_system, spans)?;
    let metrics = Metrics::new(layout.size, layout.line_height.unwrap_or(layout.size * 1.2));
    let align = to_align(layout.justify);

    let mut buffer = Buffer::new(font_system, metrics);
    // 幅が無いと Align::Center / Right が常に 0 補正になる。幅は wrap_size か枠の幅。
    buffer.set_size(layout.wrap_width, None);
    buffer.set_wrap(if layout.wrap { cosmic_text::Wrap::WordOrGlyph } else { cosmic_text::Wrap::None });
    let default_attrs = attributes.first().cloned().unwrap_or_else(Attrs::new);
    buffer.set_rich_text(spans.iter().zip(attributes.iter()).map(|(span, attrs)| (span.text, attrs.clone())), &default_attrs, Shaping::Advanced, Some(align));
    buffer.shape_until_scroll(font_system, false);

    let mut swash_cache = SwashCache::new();
    let mut out = ShapedText::default();
    let mut glyph_ordinal = 0usize;
    let content: String = spans.iter().map(|s| s.text).collect();
    let line_starts: Vec<usize> = std::iter::once(0).chain(content.match_indices('\n').map(|(i, _)| i + 1)).collect();
    let runs: Vec<_> = buffer.layout_runs().collect();
    for (i, run) in runs.iter().enumerate() {
        let base = line_starts.get(run.line_i).copied().unwrap_or(0);
        let edge = (i == 0 || runs[i - 1].line_i != run.line_i, i + 1 == runs.len() || runs[i + 1].line_i != run.line_i);
        emit_run(font_system, &mut swash_cache, run, 0.0, 0.0, base, layout, edge, &mut glyph_ordinal, &mut out);
    }
    Ok(out)
}

/// 障害物を避けて流す組み方(CSS `shape-outside`、CSS Exclusions の `wrap-flow: both`)。
/// 行ごとに `segments(行の上, 行の下)` が空いている横の区間を返し、行はその区間を左から順に埋める。
/// 区間ごとに残りの文字をその幅で組み、1 行目だけを取る(折り返しの規則は cosmic-text のまま)。
/// 語が入らない区間は飛ばし、どの区間にも入らない行は 1 行下へ(CSS の float の横に入らない語は下へ)。
pub fn shape_rich_text_around(
    spans: &[StyledText<'_>],
    layout: &TextLayout,
    segments: &dyn Fn(f32, f32) -> Vec<(f32, f32)>,
) -> Result<ShapedText, TextShapeError> {
    let mut guard = font_system();
    let font_system = &mut *guard;
    let attributes = span_attributes(font_system, spans)?;
    let align = to_align(layout.justify);
    let full = layout.wrap_width.unwrap_or(f32::MAX);
    let min_size = spans.iter().map(|s| s.layout.size).fold(layout.size, f32::min).max(1.0);
    // 文字全体を 1 本に、スタイルの切れ目は byte で持つ。
    let text: String = spans.iter().map(|s| s.text).collect();
    let mut bounds = Vec::with_capacity(spans.len());
    let mut at = 0usize;
    for span in spans {
        bounds.push((at, at + span.text.len()));
        at += span.text.len();
    }
    let pieces = |from: usize, to: usize| -> Vec<(&str, Attrs)> {
        spans.iter().zip(&bounds).zip(&attributes).filter_map(|((_, &(a, b)), attrs)| {
            let (a, b) = (a.max(from), b.min(to));
            (a < b).then(|| (&text[a..b], attrs.clone()))
        }).collect()
    };
    let default_attrs = attributes.first().cloned().unwrap_or_else(Attrs::new);
    let line_height = layout.line_height.unwrap_or(layout.size * 1.2);

    let mut swash_cache = SwashCache::new();
    let mut out = ShapedText::default();
    let mut glyph_ordinal = 0usize;
    let mut start = 0usize;
    let mut top = 0.0f32;
    let mut rows = 0;
    while start < text.len() && rows < 4096 {
        rows += 1;
        // 段落の終わり(改行)まで。
        let paragraph_end = text[start..].find('\n').map_or(text.len(), |i| start + i);
        let paragraph_start = text[..start].rfind('\n').map_or(0, |i| i + 1);
        if start == paragraph_end {
            start += 1;
            top += line_height;
            continue;
        }
        let spaces = |mut i: usize| { while i < paragraph_end && text[i..].starts_with(' ') { i += 1; } i };
        let mut row_height = line_height;
        let open = segments(top, top + line_height);
        let only = open.len() == 1 && (open[0].1 - open[0].0) >= full - 0.5;
        for (x0, x1) in open {
            let width = x1 - x0;
            if start >= paragraph_end || width < min_size {
                continue;
            }
            // 組む窓は区間に入る字数より十分長く(残り全部を毎回組まない)。
            let window = ((width / (min_size * 0.25)) as usize + 32).min(paragraph_end - start);
            let mut end = start + window;
            while !text.is_char_boundary(end) { end += 1; }
            let mut buffer = Buffer::new(font_system, Metrics::new(layout.size, line_height));
            buffer.set_size(Some(width), None);
            buffer.set_wrap(if only { cosmic_text::Wrap::WordOrGlyph } else { cosmic_text::Wrap::Word });
            buffer.set_rich_text(pieces(start, end), &default_attrs, Shaping::Advanced, Some(align));
            buffer.shape_until_scroll(font_system, false);
            let Some(run) = buffer.layout_runs().next() else { continue };
            let Some(consumed) = run.glyphs.iter().map(|g| g.end).max() else { continue };
            if run.line_w > width + 0.5 && !only {
                continue;
            }
            let edge = (paragraph_start == start, spaces(start + consumed) >= paragraph_end);
            emit_run(font_system, &mut swash_cache, &run, x0, top, start, layout, edge, &mut glyph_ordinal, &mut out);
            row_height = row_height.max(run.line_height);
            start = spaces(start + consumed);
        }
        if start >= paragraph_end && paragraph_end < text.len() {
            start = paragraph_end + 1;
        }
        top += row_height;
    }
    Ok(out)
}

fn to_align(justify: TextJustify) -> Align {
    match justify {
        TextJustify::Left => Align::Left,
        TextJustify::Right => Align::Right,
        TextJustify::Center => Align::Center,
    }
}

/// 組んだ 1 行を輪郭と行の寸法へ(`dx`・`dy` だけずらして)。`edge` = (段落の最初の行か, 最後の行か)。
/// 文字組みの 3 法(autospace・spacing-trim の行頭・hanging)は行分割の後にここで 1 度、字の x をずらすだけ。
#[allow(clippy::too_many_arguments)]
fn emit_run(font_system: &mut FontSystem, swash_cache: &mut SwashCache, run: &cosmic_text::LayoutRun<'_>, dx: f32, dy: f32, byte_base: usize, layout: &TextLayout, edge: (bool, bool), glyph_ordinal: &mut usize, out: &mut ShapedText) {
    let (shift, extra, width) = line_laws(run, layout, edge);
    let mut glyph_xs = Vec::with_capacity(run.glyphs.len());
    let mut glyph_bytes = Vec::with_capacity(run.glyphs.len());
    for (i, glyph) in run.glyphs.iter().enumerate() {
        glyph_bytes.push(byte_base + glyph.start);
        let x = glyph.x + shift + extra[i];
        let pen_x = dx + x + glyph.font_size * glyph.x_offset;
        let pen_y = dy + run.line_y + glyph.y - glyph.font_size * glyph.y_offset;
        glyph_xs.push(dx + x);
        let cache_key = glyph.physical((0.0, 0.0), 1.0).cache_key;
        let ordinal = *glyph_ordinal;
        *glyph_ordinal += 1;
        let Some(commands) = swash_cache.get_outline_commands(font_system, cache_key) else {
            continue; // 空白など、輪郭を持たない glyph。
        };
        let before = out.contours.len();
        commands_to_contours(commands, pen_x, pen_y, &mut out.contours);
        out.contour_styles.extend(std::iter::repeat_n(glyph.metadata, out.contours.len() - before));
        out.contour_glyphs.extend(std::iter::repeat_n(ordinal, out.contours.len() - before));
    }
    out.lines.push(LineMeasure { baseline_y: dy + run.line_y, width, glyph_xs, glyph_bytes });
}

/// 字の種(CSS Text 4 §8.4.1・§8.5.2・§9.2.1 の定義を Unicode の性質で写す)。
mod class {
    use unicode_general_category::{get_general_category as category, GeneralCategory as G};
    use unicode_script::{Script, UnicodeScript};
    use unicode_width::UnicodeWidthChar;

    /// East Asian Width = F(Fullwidth)。
    fn fullwidth(c: char) -> bool { matches!(c, '\u{3000}' | '\u{FF01}'..='\u{FF60}' | '\u{FFE0}'..='\u{FFE6}') }
    /// East Asian Width = W か F。
    fn wide(c: char) -> bool { c.width().unwrap_or(0) == 2 }
    fn cjk_block(c: char) -> bool { ('\u{3000}'..='\u{303F}').contains(&c) }
    fn punctuation(c: char) -> bool { matches!(category(c), G::ConnectorPunctuation | G::DashPunctuation | G::OpenPunctuation | G::ClosePunctuation | G::InitialPunctuation | G::FinalPunctuation | G::OtherPunctuation) }

    /// ideographs: U+3041–U+30FF(約物を除く)、CJK Strokes、Katakana Phonetic Extensions、Han(scx が名指しで Han を含む。
    /// Common / Inherited の字は scx が全部の script を含むので、名指しの列で見る)。
    pub(super) fn ideograph(c: char) -> bool {
        let han = c.script() == Script::Han || (!matches!(c.script(), Script::Common | Script::Inherited) && c.script_extension().iter().any(|s| s == Script::Han));
        (('\u{3041}'..='\u{30FF}').contains(&c) && !punctuation(c)) || ('\u{31C0}'..='\u{31FF}').contains(&c) || han
    }
    /// non-ideographic letters: L* と M*、ただし ideograph と W/F を除く。
    pub(super) fn letter(c: char) -> bool {
        matches!(category(c), G::UppercaseLetter | G::LowercaseLetter | G::TitlecaseLetter | G::ModifierLetter | G::OtherLetter | G::NonspacingMark | G::SpacingMark | G::EnclosingMark) && !ideograph(c) && !wide(c)
    }
    /// non-ideographic numerals: Nd、ただし F を除く。
    pub(super) fn numeral(c: char) -> bool { category(c) == G::DecimalNumber && !fullwidth(c) }

    /// fullwidth opening punctuation: CJK 記号ブロックか F の Ps、と ‘ “。
    pub(super) fn fullwidth_opening(c: char) -> bool { (category(c) == G::OpenPunctuation && (cjk_block(c) || fullwidth(c))) || matches!(c, '\u{2018}' | '\u{201C}') }

    /// hanging-punctuation first: Ps・Pf・Pi、' "、U+3000。
    pub(super) fn hangs_first(c: char) -> bool { matches!(category(c), G::OpenPunctuation | G::FinalPunctuation | G::InitialPunctuation) || matches!(c, '\'' | '"' | '\u{3000}') }
    /// hanging-punctuation last: Pe・Pf・Pi、' "。
    pub(super) fn hangs_last(c: char) -> bool { matches!(category(c), G::ClosePunctuation | G::FinalPunctuation | G::InitialPunctuation) || matches!(c, '\'' | '"') }
    /// 行末にぶら下がる句読点(仕様の表そのまま)。
    pub(super) fn stop(c: char) -> bool { matches!(c, '\u{002C}' | '\u{002E}' | '\u{060C}' | '\u{06D4}' | '\u{3001}' | '\u{3002}' | '\u{FF0C}' | '\u{FF0E}' | '\u{FE50}' | '\u{FE51}' | '\u{FE52}' | '\u{FF61}' | '\u{FF64}') }
}

/// 1 行に 3 法を掛けた結果: (行全体のずれ, 字ごとの足し, 測った行の幅)。
/// 行頭の詰めと、ぶら下げは行の測りから外れる(CSS「it is not considered when measuring the line's contents for alignment」)ので、
/// Center / Right は短くなった分だけ寄せ直す。autospace は測りに入る(CSS「additive with letter-spacing」)。
fn line_laws(run: &cosmic_text::LayoutRun<'_>, layout: &TextLayout, (first_line, last_line): (bool, bool)) -> (f32, Vec<f32>, f32) {
    let glyphs = run.glyphs;
    let n = glyphs.len();
    let ch = |i: usize| run.text.get(glyphs[i].start..glyphs[i].end).and_then(|s| s.chars().next()).unwrap_or('\0');
    let mut extra = vec![0.0f32; n];
    if n == 0 {
        return (0.0, extra, run.line_w);
    }
    // text-spacing-trim: 行頭の全角の開き約物を半角に(左半分を削る)。normal は行頭を詰めない。
    let trim_here = match layout.spacing_trim {
        TextSpacingTrim::TrimStart => true,
        TextSpacingTrim::SpaceFirst => !first_line,
        TextSpacingTrim::Normal | TextSpacingTrim::SpaceAll => false,
    };
    // 削るのは全角の字送りの空いた左半分。書体が既に詰めている(palt などでプロポーショナル)字は「足しも引きもしない」(§8.5.1)。
    let start_cut = if trim_here && class::fullwidth_opening(ch(0)) { (glyphs[0].w - glyphs[0].font_size * 0.5).max(0.0) } else { 0.0 };
    // hanging-punctuation: 端の 1 字を行の測りから外す。詰めた字は残りの半分がぶら下がる(§8.5.3 の第 3 式)。
    let hang_start = if layout.hanging.first && first_line && class::hangs_first(ch(0)) { glyphs[0].w - start_cut } else { 0.0 };
    let last = n - 1;
    let fits = layout.wrap_width.is_none_or(|w| run.line_w <= w + 0.5);
    let hang_end = if layout.hanging.last && last_line && class::hangs_last(ch(last)) {
        glyphs[last].w
    } else if class::stop(ch(last)) && (layout.hanging.end == HangEnd::ForceEnd || (layout.hanging.end == HangEnd::AllowEnd && !fits)) {
        glyphs[last].w
    } else {
        0.0
    };
    // text-autospace: 漢字と欧文の字・数字が直に隣る境に 1/8 の字送り(隣る字の間に空白や約物があれば境ではない)。
    let mut total = 0.0f32;
    if layout.autospace == TextAutospace::Normal {
        for i in 1..n {
            let (a, b) = (ch(i - 1), ch(i));
            let alpha = |c: char| class::letter(c) || class::numeral(c);
            let ideograph_size = if class::ideograph(a) && alpha(b) { Some(glyphs[i - 1].font_size) } else if alpha(a) && class::ideograph(b) { Some(glyphs[i].font_size) } else { None };
            if let Some(size) = ideograph_size { total += size * 0.125; }
            extra[i] = total;
        }
    }
    let width = run.line_w - start_cut - hang_start - hang_end + total;
    let align = match (layout.wrap_width, layout.justify) {
        (None, _) | (_, TextJustify::Left) => 0.0,
        (_, TextJustify::Center) => 0.5,
        (_, TextJustify::Right) => 1.0,
    };
    let shift = align * (run.line_w - width) - start_cut - hang_start;
    (shift, extra, width)
}

fn span_attributes<'a>(font_system: &mut FontSystem, spans: &[StyledText<'a>]) -> Result<Vec<Attrs<'a>>, TextShapeError> {
    for span in spans { ensure_font(font_system, span.font)?; }
    let mut attributes = Vec::new();
    for span in spans {
        let mut features = FontFeatures::new();
        // text-spacing-trim の隣どうしの詰めは font の `chws`(仕様: halt/chws を使ってよい、hwid は使ってはならない)。
        if span.layout.spacing_trim != TextSpacingTrim::SpaceAll {
            features.set(FeatureTag::new(b"chws"), 1);
        }
        for feature in &span.layout.features {
            let tag: &[u8;4] = feature.tag.as_bytes().try_into().map_err(|_| TextShapeError::FeatureTag(feature.tag.clone()))?;
            features.set(FeatureTag::new(tag), feature.value);
        }
        attributes.push(Attrs::new().family(Family::Name(&span.font.family))
            .metrics(Metrics::new(span.layout.size, span.layout.line_height.unwrap_or(span.layout.size * 1.2)))
            .letter_spacing(span.layout.tracking / 1000.0).font_features(features).metadata(span.style));
    }
    Ok(attributes)
}

fn commands_to_contours(commands: &[Command], pen_x: f32, pen_y: f32, out: &mut Vec<Contour>) {
    let at = |x: f32, y: f32| Point {
        x: f64::from(pen_x + x),
        y: f64::from(pen_y - y), // zeno は y-up、正準空間は y-down。
    };
    let mut vertices: Vec<Vertex> = Vec::new();
    for command in commands {
        match *command {
            Command::MoveTo(p) => {
                if !vertices.is_empty() {
                    out.push(Contour {
                        vertices: std::mem::take(&mut vertices),
                        closed: false,
                    });
                }
                vertices.push(Vertex::corner(at(p.x, p.y)));
            }
            Command::LineTo(p) => vertices.push(Vertex::corner(at(p.x, p.y))),
            Command::CurveTo(c1, c2, p) => {
                let to = at(p.x, p.y);
                if let Some(last) = vertices.last_mut() {
                    last.out_tangent = at(c1.x, c1.y).sub(last.point);
                }
                vertices.push(Vertex {
                    point: to,
                    in_tangent: at(c2.x, c2.y).sub(to),
                    out_tangent: Point::ZERO,
                });
            }
            Command::QuadTo(q, p) => {
                let to = at(p.x, p.y);
                let control = at(q.x, q.y);
                if let Some(last) = vertices.last_mut() {
                    last.out_tangent = control.sub(last.point).scale(2.0 / 3.0);
                }
                vertices.push(Vertex {
                    point: to,
                    in_tangent: control.sub(to).scale(2.0 / 3.0),
                    out_tangent: Point::ZERO,
                });
            }
            Command::Close => {
                if vertices.len() > 1 {
                    let first = vertices[0];
                    let last = vertices[vertices.len() - 1];
                    let dx = last.point.x - first.point.x;
                    let dy = last.point.y - first.point.y;
                    if (dx * dx + dy * dy) < 1e-9 {
                        vertices[0].in_tangent = last.in_tangent;
                        vertices.pop();
                    }
                }
                if !vertices.is_empty() {
                    out.push(Contour {
                        vertices: std::mem::take(&mut vertices),
                        closed: true,
                    });
                }
            }
        }
    }
    if !vertices.is_empty() {
        out.push(Contour {
            vertices,
            closed: false,
        });
    }
}

/// A specimen must not silently display a different fallback family.
pub fn font_supports_sample(family: &str, text: &str) -> bool {
    let mut system = font_system();
    let Some(id) = system.db().query(&fontdb::Query {
        families: &[fontdb::Family::Name(family)], ..Default::default()
    }) else { return false };
    let Some(font) = system.get_font(id, fontdb::Weight::NORMAL) else { return false };
    text.chars().filter(|c| !c.is_whitespace()).all(|c| font.as_swash().charmap().map(c) != 0)
}

/// What a family is, read from the font itself once: how many faces and
/// weights, whether it is monospaced, its variable axes, the scripts its
/// charmap covers, and whether it carries colour glyphs. Labels for a shelf,
/// not shaping input.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct FontFacts {
    pub family: String,
    pub styles: usize,
    pub weights: usize,
    pub monospaced: bool,
    /// fvar の軸の tag(`wght` など)。無ければ静的。
    pub axes: Vec<String>,
    /// 覆う文字種の名前(Latin / Kana / Kanji / Hangul / Cyrillic / Arabic)。
    pub scripts: Vec<&'static str>,
    pub color: bool,
}

const SCRIPT_PROBES: &[(&str, char)] = &[("Latin", 'A'), ("Kana", 'あ'), ("Kanji", '永'), ("Hangul", '한'), ("Cyrillic", 'Я'), ("Arabic", 'ع')];

pub fn font_facts() -> &'static [FontFacts] {
    static FACTS: std::sync::OnceLock<Vec<FontFacts>> = std::sync::OnceLock::new();
    FACTS.get_or_init(|| {
        let mut system = font_system();
        let mut by_family: std::collections::BTreeMap<String, Vec<(fontdb::ID, fontdb::Weight, bool)>> = std::collections::BTreeMap::new();
        for face in system.db().faces() {
            if let Some((name, _)) = face.families.first() {
                by_family.entry(name.clone()).or_default().push((face.id, face.weight, face.monospaced));
            }
        }
        by_family.into_iter().map(|(family, faces)| {
            let weights = faces.iter().map(|f| f.1).collect::<std::collections::BTreeSet<_>>().len();
            let monospaced = faces.iter().any(|f| f.2);
            let (axes, scripts, color) = match system.get_font(faces[0].0, faces[0].1) {
                Some(font) => {
                    let swash = font.as_swash();
                    let axes = swash.variations().map(|v| String::from_utf8_lossy(&v.tag().to_be_bytes()).into_owned()).collect();
                    let charmap = swash.charmap();
                    let scripts = SCRIPT_PROBES.iter().filter(|(_, c)| charmap.map(*c) != 0).map(|(name, _)| *name).collect();
                    (axes, scripts, swash.color_palettes().len() > 0)
                }
                None => (Vec::new(), Vec::new(), false),
            };
            FontFacts { family, styles: faces.len(), weights, monospaced, axes, scripts, color }
        }).collect()
    })
}

#[cfg(test)]
mod law_tests {
    use super::*;

    /// ヒラギノ角ゴで 100 級。書体が無ければ飛ばす(定規は CSS Text 4、字送りは書体)。
    fn shaped(text: &str, edit: impl Fn(&mut TextLayout)) -> Option<ShapedText> {
        let font = GlyphFont { path: String::new(), family: "Hiragino Sans".to_owned() };
        if !font_supports_sample(&font.family, "漢A1「」。") { eprintln!("skipped: Hiragino Sans missing"); return None; }
        let mut layout = TextLayout::new(100.0);
        edit(&mut layout);
        Some(shape_text(text, &font, &layout).unwrap())
    }
    fn xs(shaped: &ShapedText) -> Vec<f32> { shaped.lines[0].glyph_xs.clone() }

    /// CSS Text 4 §8.4.1: 漢字と欧文の字・数字の境に 1/8 の字送り。漢字どうし・no-autospace は 0。
    #[test]
    fn autospace_is_an_eighth_of_the_ideograph_advance_at_script_boundaries() {
        let Some(plain) = shaped("漢A1漢字 B", |l| l.autospace = TextAutospace::NoAutospace) else { return };
        let spaced = shaped("漢A1漢字 B", |_| {}).unwrap();
        let (a, b) = (xs(&plain), xs(&spaced));
        let gaps: Vec<f32> = (1..a.len()).map(|i| (b[i] - b[i - 1]) - (a[i] - a[i - 1])).collect();
        // 境: 漢|A(+12.5)、A|1(0)、1|漢(+12.5)、漢|字(0)、字| (0)、 |B(0 — 空白が間にある)。
        assert_eq!(gaps, vec![12.5, 0.0, 12.5, 0.0, 0.0, 0.0], "{gaps:?}");
        assert_eq!(spaced.lines[0].width - plain.lines[0].width, 25.0);
    }

    /// CSS Text 4 §8.5: trim-start は行頭の全角の開き括弧の左半分を削る。normal は行頭を詰めない。
    #[test]
    fn trim_start_kerns_half_of_the_opening_bracket_at_line_start() {
        let Some(normal) = shaped("「漢」", |_| {}) else { return };
        let trimmed = shaped("「漢」", |l| l.spacing_trim = TextSpacingTrim::TrimStart).unwrap();
        assert_eq!(xs(&normal)[0], 0.0);
        assert_eq!(xs(&trimmed)[0], -50.0);
        assert_eq!(xs(&trimmed)[1], xs(&normal)[1] - 50.0);
        assert_eq!(normal.lines[0].width - trimmed.lines[0].width, 50.0);
        // 書体が既に詰めた字(palt でプロポーショナル)は足しも引きもしない(§8.5.1)。
        let palt = |l: &mut TextLayout| l.features = vec![TextFeature { tag: "palt".into(), value: 1 }];
        let proportional = shaped("「漢」", |l| palt(l)).unwrap();
        let bracket = xs(&proportional)[1] - xs(&proportional)[0];
        let trimmed = shaped("「漢」", |l| { palt(l); l.spacing_trim = TextSpacingTrim::TrimStart }).unwrap();
        assert_eq!(xs(&trimmed)[0], -(bracket - 50.0).max(0.0), "palt 「 advance {bracket}");
    }

    /// CSS Text 4 §9.2.1: first は最初の行の行頭の開き括弧を箱の外へ、last は最後の行の行末の閉じ括弧を外へ。
    /// ぶら下がった字は行の測りに入らないので、Center は残りの字で寄せ直す。
    #[test]
    fn hanging_first_and_last_move_the_brackets_outside_the_line() {
        let Some(plain) = shaped("「漢」", |_| {}) else { return };
        let hung = shaped("「漢」", |l| l.hanging = HangingPunctuation { first: true, end: HangEnd::None, last: true }).unwrap();
        let bracket = xs(&plain)[1] - xs(&plain)[0];
        assert_eq!(xs(&hung)[0], -bracket, "「 hangs by its own advance");
        assert_eq!(xs(&hung)[1], 0.0, "漢 sits on the edge");
        assert_eq!(hung.lines[0].width, plain.lines[0].width - 2.0 * bracket);
        let centred = shaped("「漢」", |l| { l.wrap_width = Some(1000.0); l.justify = TextJustify::Center; l.hanging = HangingPunctuation { first: true, end: HangEnd::None, last: true } }).unwrap();
        assert_eq!(xs(&centred)[1], (1000.0 - hung.lines[0].width) * 0.5);
    }

    /// force-end は行末の句点をぶら下げる。allow-end は入り切る行では何もしない。
    #[test]
    fn end_stops_hang_only_when_forced_or_not_fitting() {
        let Some(plain) = shaped("漢字。", |_| {}) else { return };
        let allowed = shaped("漢字。", |l| l.hanging.end = HangEnd::AllowEnd).unwrap();
        let forced = shaped("漢字。", |l| l.hanging.end = HangEnd::ForceEnd).unwrap();
        assert_eq!(xs(&allowed), xs(&plain));
        assert_eq!(xs(&forced), xs(&plain));
        assert_eq!(forced.lines[0].width, plain.lines[0].width - 100.0);
    }

    /// 貼れるか: 欧文だけの行に autospace と trim-start は掛からない(境も全角の約物も無い)。
    /// hanging は仕様の通り欧文の約物にも掛かる(§9.2.1 の表に U+002E、first/last は Ps/Pe)。
    #[test]
    fn latin_only_lines_are_untouched_except_by_hanging() {
        let Some(plain) = shaped("Hello, world (again).", |_| {}) else { return };
        let pressed = shaped("Hello, world (again).", |l| l.spacing_trim = TextSpacingTrim::TrimStart).unwrap();
        assert_eq!(xs(&plain), xs(&pressed));
        assert_eq!(plain.lines[0].width, pressed.lines[0].width);
        let hung = shaped("Hello, world (again).", |l| l.hanging.end = HangEnd::ForceEnd).unwrap();
        let stop = plain.lines[0].width - xs(&plain).last().unwrap();
        assert_eq!(hung.lines[0].width, plain.lines[0].width - stop, "the full stop hangs, as CSS says it does in Latin too");
    }

    /// 選択肢の番と値は往復する(窓・台本が番で書く)。
    #[test]
    fn choices_round_trip() {
        for i in 0..HangingPunctuation::CHOICES.len() as i64 { assert_eq!(HangingPunctuation::from_enum_value(i).unwrap().to_enum_value(), i); }
        assert_eq!(HangingPunctuation::from_enum_value(HangingPunctuation::CHOICES.len() as i64), None);
        for i in 0..4 { assert_eq!(TextSpacingTrim::from_enum_value(i).unwrap().to_enum_value(), i); }
        assert_eq!(TextAutospace::from_enum_value(1), Some(TextAutospace::NoAutospace));
    }
}

#[cfg(test)]
mod facts_tests {
    use super::*;

    /// 台帳の書体が事実を持つ: ヒラギノ角ゴは仮名と漢字を覆い、10 の太さの静的書体。
    #[test]
    fn facts_come_from_the_font_itself() {
        let facts = font_facts();
        let Some(hiragino) = facts.iter().find(|f| f.family == "Hiragino Sans") else {
            eprintln!("skipped: Hiragino Sans missing");
            return;
        };
        assert!(hiragino.scripts.contains(&"Kana") && hiragino.scripts.contains(&"Kanji") && hiragino.scripts.contains(&"Latin"));
        assert!(hiragino.weights >= 5, "{hiragino:?}");
        assert!(hiragino.axes.is_empty());
        assert!(!hiragino.monospaced);
        assert!(facts.iter().all(|f| f.styles >= 1));
    }
}


