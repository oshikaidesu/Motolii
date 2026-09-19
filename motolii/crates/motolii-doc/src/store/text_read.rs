use super::{TextDocument, TextStyleId};
use unicode_segmentation::UnicodeSegmentation;

pub fn graphemes(text: &str) -> Vec<&str> { text.graphemes(true).collect() }

pub fn style_ids(document:&TextDocument, text:&str) -> Vec<TextStyleId> {
    let count=text.graphemes(true).count();
    let default=document.styles.first().map(|s|s.id).unwrap_or(TextStyleId(0));
    let mut ids=Vec::with_capacity(count);
    for run in &document.runs { ids.extend(std::iter::repeat_n(run.style,(run.len as usize).min(count-ids.len()))); if ids.len()==count { break; } }
    ids.resize(count,default);ids
}
