//! 画の骨格の表(`assets/strokes/strokes.bin`、出所と licence は隣の LICENSES.md)。
//! Text Morph の Strokes 方式だけが読む。書式は `scripts/strokes/build_strokes.py` の頭に。

use std::collections::HashMap;
use std::sync::OnceLock;

static TABLE: &[u8] = include_bytes!("../../assets/strokes/strokes.bin");

/// 1 字の画: 筆順に並んだ折れ線。座標は em の箱を 0..255 に写した物(y は下向き)。
pub type Template = Vec<Vec<[f32; 2]>>;

fn parse(bytes: &[u8]) -> HashMap<char, Template> {
    let mut out = HashMap::new();
    if bytes.len() < 9 || &bytes[..4] != b"MSTK" || bytes[4] != 1 { return out; }
    let count = u32::from_le_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]) as usize;
    let mut i = 9;
    for _ in 0..count {
        if i + 5 > bytes.len() { break; }
        let cp = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
        let strokes = bytes[i + 4] as usize;
        i += 5;
        let mut template = Vec::with_capacity(strokes);
        for _ in 0..strokes {
            let points = bytes[i] as usize;
            i += 1;
            let stroke: Vec<[f32; 2]> = (0..points).map(|k| [bytes[i + 2 * k] as f32, bytes[i + 2 * k + 1] as f32]).collect();
            i += 2 * points;
            template.push(stroke);
        }
        if let Some(ch) = char::from_u32(cp) { out.insert(ch, template); }
    }
    out
}

fn table() -> &'static HashMap<char, Template> {
    static PARSED: OnceLock<HashMap<char, Template>> = OnceLock::new();
    PARSED.get_or_init(|| parse(TABLE))
}

/// この字の画(無ければ None: Melt へ落ちる)。
pub fn template(ch: char) -> Option<&'static Template> {
    table().get(&ch)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 表が読め、かな・漢字は KanjiVG の筆順(永は 5 画)、欧文は Relief の線(a は 2 本)。
    #[test]
    fn the_table_has_kana_kanji_and_latin() {
        assert!(table().len() > 6000);
        assert_eq!(template('永').map(Vec::len), Some(5));
        assert_eq!(template('あ').map(Vec::len), Some(3));
        assert_eq!(template('a').map(Vec::len), Some(2));
        assert!(template('永').unwrap().iter().all(|s| s.len() >= 2 && s.iter().all(|p| (0.0..=255.0).contains(&p[0]) && (0.0..=255.0).contains(&p[1]))));
        assert_eq!(template('\u{E000}'), None);
    }
}
