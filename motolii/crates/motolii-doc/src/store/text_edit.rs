use super::{TextDocument, TextRun, TextStyleId};
use unicode_segmentation::UnicodeSegmentation;
use unicode_script::{UnicodeScript, Script};

pub fn graphemes(text: &str) -> Vec<&str> { text.graphemes(true).collect() }

pub fn style_ids(document:&TextDocument, text:&str) -> Vec<TextStyleId> {
    let count=text.graphemes(true).count();
    let default=document.styles.first().map(|s|s.id).unwrap_or(TextStyleId(0));
    let mut ids=Vec::with_capacity(count);
    for run in &document.runs { ids.extend(std::iter::repeat_n(run.style,(run.len as usize).min(count-ids.len()))); if ids.len()==count { break; } }
    ids.resize(count,default);ids
}

pub fn set_runs(document:&mut TextDocument, ids:&[TextStyleId]) {
    let mut runs:Vec<TextRun>=Vec::new();
    for &style in ids {
        if let Some(last)=runs.last_mut().filter(|r|r.style==style) { last.len+=1; }
        else { runs.push(TextRun{len:1,style}); }
    }
    document.runs=if runs.len()==1 && document.styles.first().is_some_and(|s|s.id==runs[0].style) { Vec::new() } else { runs };
}

pub fn preserve_replacement(document:&mut TextDocument, old:&str, new:&str) {
    let a=graphemes(old);let b=graphemes(new);let ids=style_ids(document,old);
    let prefix=a.iter().zip(&b).take_while(|(a,b)|a==b).count();
    let suffix=a[prefix..].iter().rev().zip(b[prefix..].iter().rev()).take_while(|(a,b)|a==b).count();
    let inherit_at=if a.len()==prefix+suffix {prefix.saturating_sub(1)} else {prefix};
    let inherited=ids.get(inherit_at).or_else(||ids.last()).copied().unwrap_or_else(||document.styles.first().map(|s|s.id).unwrap_or(TextStyleId(0)));
    let mut next=ids[..prefix].to_vec();
    next.extend(std::iter::repeat_n(inherited,b.len()-prefix-suffix));
    next.extend_from_slice(&ids[ids.len()-suffix..]);
    set_runs(document,&next);
}

fn scripts(text:&str) -> Vec<Script> {
    let mut previous=Script::Common;
    text.graphemes(true).map(|g| {
        let mut script=g.chars().map(|c|c.script()).find(|s|!matches!(s,Script::Common|Script::Inherited)).unwrap_or(Script::Common);
        if script==Script::Common && matches!(previous,Script::Hiragana|Script::Katakana) &&
            g.chars().all(|c| c.script_extension().iter().all(|s|matches!(s,Script::Hiragana|Script::Katakana)) && c.script_extension().contains_script(previous)) { script=previous; }
        previous=script;script
    }).collect()
}

/// One label per grapheme: the script, and for a cased letter its case after
/// a dash (`latin-upper`). A scope names either half; [`in_scope`] reads it.
pub fn classifications(text:&str) -> Vec<&'static str> {
    text.graphemes(true).zip(scripts(text)).map(|(g,s)| {
        let case=if g.chars().any(char::is_uppercase) {"-upper"} else if g.chars().any(char::is_lowercase) {"-lower"} else {""};
        match (s,case) {
            (Script::Hiragana,_)=>"hiragana",(Script::Katakana,_)=>"katakana",(Script::Han,_)=>"han",
            (Script::Latin,"-upper")=>"latin-upper",(Script::Latin,"-lower")=>"latin-lower",(Script::Latin,_)=>"latin",
            (_,"-upper")=>"other-upper",(_,"-lower")=>"other-lower",_=>"other",
        }
    }).collect()
}

pub const SCOPES:&[&str]=&["all","selection","hiragana","katakana","han","latin","upper","lower"];

pub fn in_scope(kind:&str, scope:&str) -> bool {
    let (script,case)=kind.split_once('-').unwrap_or((kind,""));
    script==scope || case==scope
}

pub fn selected(text:&str, start:usize, end:usize, scope:&str) -> Vec<bool> {
    let mut offset=0;
    text.graphemes(true).zip(classifications(text)).map(|(g,kind)| {
        let from=offset;offset+=g.encode_utf16().count();
        match scope { "all"=>true,"selection"=>from<end && offset>start,_=>in_scope(kind,scope) }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scripts_and_utf16_ranges_never_split_a_grapheme() {
        let text="か\u{3099}カ漢😀a";
        assert_eq!(classifications(text),["hiragana","katakana","han","other","latin-lower"]);
        assert_eq!(classifications("Ab1Ж"),["latin-upper","latin-lower","other","other-upper"]);
        assert_eq!(selected("Ab1Ж",0,0,"upper"),[true,false,false,true]);
        assert_eq!(selected("Ab1Ж",0,0,"latin"),[true,true,false,false]);
        assert_eq!(selected(text,1,2,"selection"),[true,false,false,false,false]);
        assert_eq!(selected(text,5,6,"selection"),[false,false,false,true,false]);
        assert_eq!(classifications("スーパー。"),["katakana","katakana","katakana","katakana","other"]);
    }
}
