//! wraps: fontdb — システムフォント一覧(family →実在 path)。
//!
//! # 出自(2026-08-22 追い発注「フォントが選べる・選ばなくても落ちない」)
//!
//! `docs/reviews/2026-08-22-persona-lyric-mv.md` を塞いだレーンの自己申告
//! FINDING(`next/shell/motolii-shell/tests/suite/lyric_text_layer_drive.rs`
//! モジュール doc): `TextField::FontFamily` は `FontRef::family` しか編集せず、
//! `FontRef::path` を編集する入口がリポ全体のどこにも無かった。既定値
//! (`FontRef::default()`)も空 path のままなので、テキストレイヤーに文字を
//! 打った瞬間 `motolii-vector::text::shape_text` が
//! `db.load_font_file(&font.path)` で空文字列を開こうとして
//! `TextShapeError::FontFile` → `render_frame` 全体が `Err` になっていた
//! (1レイヤーのフォント欠落が合成全体を消す)。
//!
//! この crate は**列挙するだけ**——「実在する家族名 → 実在するファイル path」
//! の表を返す。書き込み(`FontRef` を組み立てて `Intent::SetTextDocument` を
//! 出す)は呼び手(`motolii-inspector-pane::text`)の仕事。
//!
//! # やらないこと
//!
//! シェイピング・描画はしない(`motolii-vector::text::shape_text` の仕事の
//! まま)。`motolii-vector` の module doc が明記するとおり「システムフォント
//! 走査はしない(試験が決定的で速い)」という判断は**そこでは**維持する ——
//! 走査は UI(pick_list/既定値解決)側の責務としてこの新しい crate へ切り出し、
//! engine 側の決定的な path 解決とは責務を分けたまま。
//!
//! # 依存増分ゼロ
//!
//! `Cargo.toml` のコメント参照 — fontdb 0.23.0 は既に `next/Cargo.lock` に
//! 居る(`motolii-vector` の cosmic-text 0.19 が引いている)。

use std::sync::OnceLock;

/// 列挙された1フォント。**実在する path を持つ face だけ**がここに現れる
/// ([`system_fonts`] のフィルタ参照) — `motolii_store::FontRef` へ写す時に
/// 「解決できない path」を混入させない、という本発注の頑健化方針をこの型の
/// 契約自体で表現する。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontEntry {
    /// family 名(`fontdb::FaceInfo::families` の先頭 — 英語USが最初に来る
    /// 慣習、無ければそのフォントの1本目の名前、fontdb doc 参照)。
    pub family: String,
    /// フォント実体の在処。**必ずローカルファイルとして開ける**
    /// (`fontdb::Source::File` 由来のみ通す、`Binary`/`SharedFile` は除外 —
    /// `FontRef::path` は「開けるファイルパス」という契約なので、メモリ上の
    /// バイト列しか持たない face をここに含めても書き先が無い)。
    pub path: String,
    /// face の様子(`Style` の `Debug`、`text.rs::justify_row` と同じ
    /// 「型の Debug をそのまま表示語彙にする」流儀)。同じ family でも
    /// 複数 face(Regular/Bold/Italic 等)がある時、どれを代表に選んだかを
    /// 呼び手が説明できるようにするための情報(v1 は代表1本しか返さないが、
    /// 「なぜこの1本か」を捨てない)。
    pub style: String,
}

/// family 名だけでは Regular/Bold/Italic のどれを指すか決まらないので、
/// 「一般的な既定として一番使いやすい face」を選ぶための点数(**低いほど良い**)。
/// Normal style・標準 weight(400)・標準 stretch に近いほど 0 に近づく。
fn face_score(face: &fontdb::FaceInfo) -> i32 {
    let style_penalty = match face.style {
        fontdb::Style::Normal => 0,
        fontdb::Style::Oblique => 5,
        fontdb::Style::Italic => 10,
    };
    let weight_penalty = (i32::from(face.weight.0) - i32::from(fontdb::Weight::NORMAL.0)).abs();
    let stretch_penalty = if face.stretch == fontdb::Stretch::Normal {
        0
    } else {
        5
    };
    style_penalty + weight_penalty + stretch_penalty
}

/// `fontdb::Database::load_system_fonts` を1回だけ実行し、family ごとに
/// 最も「素直な」1 face を選んで返す。**非公開/内部フォントは除く**
/// (macOS の `.AppleSystemUIFont` 等 `.` 始まりの family — fontdb がそのまま
/// 返してくることを実機で確認済み。利用者が選ぶ対象ではない)。
fn scan_system_fonts() -> Vec<FontEntry> {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();

    let mut best: std::collections::HashMap<String, (FontEntry, i32)> =
        std::collections::HashMap::new();
    for face in db.faces() {
        let fontdb::Source::File(path) = &face.source else {
            // Binary/SharedFile は実在 path を持たない — この crate の唯一の
            // 仕事(`FontRef::path` を埋める)には使えない候補なので飛ばす。
            continue;
        };
        let Some(path_str) = path.to_str() else {
            continue;
        };
        let Some((family, _lang)) = face.families.first() else {
            continue;
        };
        if family.is_empty() || family.starts_with('.') {
            continue;
        }

        let score = face_score(face);
        let candidate = FontEntry {
            family: family.clone(),
            path: path_str.to_owned(),
            style: format!("{:?}", face.style),
        };
        match best.get(family.as_str()) {
            Some((_, existing_score)) if *existing_score <= score => {}
            _ => {
                best.insert(family.clone(), (candidate, score));
            }
        }
    }

    let mut entries: Vec<FontEntry> = best.into_values().map(|(entry, _)| entry).collect();
    entries.sort_by(|a, b| a.family.to_lowercase().cmp(&b.family.to_lowercase()));
    entries
}

/// システムフォント一覧。**プロセス内で1回だけ走査する**(`OnceLock` —
/// pick_list の `view()` が呼ばれるたびにファイルシステムを再走査しない)。
/// 実行中にフォントが増減しても反映されない(v1 の割り切り — 走査コストの方が
/// 「起動中にフォントが増えた」への追従より優先度が高い、`ui-quality-bar` に
/// この種の即応性は要求が無い)。
pub fn system_fonts() -> &'static [FontEntry] {
    static CACHE: OnceLock<Vec<FontEntry>> = OnceLock::new();
    CACHE.get_or_init(scan_system_fonts).as_slice()
}

/// family 名から1件引く(pick_list の選択・`TextField::FontFamily` 手打ちの
/// 追従の両方が使う)。[`system_fonts`] は family ごとに一意なので `.find` で
/// 十分(重複 family を複数返さない、[`scan_system_fonts`] の集約参照)。
pub fn find_family(family: &str) -> Option<&'static FontEntry> {
    system_fonts().iter().find(|entry| entry.family == family)
}

/// 既定として選ぶ1本。**見つかれば**よく知られた汎用ファミリを優先する ——
/// UI が最初に見せる文字が奇抜な装飾/記号フォントにならないための見た目上の
/// 配慮に過ぎず、無ければアルファベット順の先頭で妥協する。どちらの道でも
/// [`system_fonts`] が既に「実在 path を持つ face だけ」に絞ってあるので、
/// この関数が `Some` を返す限り結果は常に解決可能。**システムフォントが
/// 1つも見つからない環境でだけ `None`**(この crate では直せない — RETURN の
/// engine 側 finding 参照)。
pub fn default_font() -> Option<&'static FontEntry> {
    const PREFERRED: [&str; 7] = [
        "Helvetica",
        "Arial",
        "Helvetica Neue",
        "Liberation Sans",
        "DejaVu Sans",
        "Noto Sans",
        "Verdana",
    ];
    let fonts = system_fonts();
    for name in PREFERRED {
        if let Some(entry) = fonts.iter().find(|entry| entry.family == name) {
            return Some(entry);
        }
    }
    fonts.first()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// このマシン(darwin、実機)にシステムフォントが1つも無いことは無い —
    /// 0件ならこの crate の存在意義そのものが崩れるので、まずここで固定する。
    #[test]
    fn system_fonts_returns_at_least_one_entry_on_this_machine() {
        assert!(
            !system_fonts().is_empty(),
            "このマシンにシステムフォントが1つも見つからない"
        );
    }

    /// [`FontEntry::path`] の契約(「開けるファイル」)を実際に確かめる —
    /// `motolii-vector::text::shape_text` が `load_font_file` で開けない path を
    /// この crate が返さない、という頑健化の要石。
    #[test]
    fn every_entry_points_at_a_file_that_actually_exists() {
        for entry in system_fonts() {
            assert!(
                std::path::Path::new(&entry.path).is_file(),
                "{} の path が実在しない: {}",
                entry.family,
                entry.path
            );
        }
    }

    #[test]
    fn default_font_resolves_to_an_existing_path() {
        let entry = default_font().expect("このマシンではシステムフォントが有るはず");
        assert!(!entry.path.is_empty(), "既定フォントの path が空");
        assert!(
            std::path::Path::new(&entry.path).is_file(),
            "既定フォントの path が実在しない: {}",
            entry.path
        );
    }

    #[test]
    fn find_family_agrees_with_system_fonts() {
        let first = system_fonts().first().expect("catalog が空");
        let found = find_family(&first.family).expect("同じ family 文字列で引けるはず");
        assert_eq!(found.path, first.path, "find_family が別の path を返した");
    }

    #[test]
    fn find_family_returns_none_for_a_family_that_does_not_exist() {
        assert!(find_family("Definitely Not A Real Font Family 12345").is_none());
    }
}
