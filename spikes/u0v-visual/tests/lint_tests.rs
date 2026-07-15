use regex::Regex;
use std::fs;
use walkdir::WalkDir;

const SKIP: &[&str] = &["generated/", "tokens/", "target/"];

#[test]
fn no_raw_color_literals_outside_tokens() {
    let manifest = u0v_visual::manifest_dir();
    let hex = Regex::new(r"#([0-9a-fA-F]{3}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})\b").unwrap();
    let mut violations = Vec::new();
    for entry in WalkDir::new(&manifest) {
        let entry = entry.unwrap();
        let path = entry.path();
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = path.strip_prefix(&manifest).unwrap().to_string_lossy();
        if SKIP.iter().any(|s| rel.contains(s)) {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !matches!(ext, "slint" | "rs") {
            continue;
        }
        let content = fs::read_to_string(path).unwrap();
        for (i, line) in content.lines().enumerate() {
            if line.contains("AUTO-GENERATED") || line.trim_start().starts_with("//") {
                continue;
            }
            if hex.is_match(line) {
                violations.push(format!("{}:{}: {}", rel, i + 1, line.trim()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "raw color literals found:\n{}",
        violations.join("\n")
    );
}

#[test]
fn ui_declares_required_regions() {
    let app = fs::read_to_string(u0v_visual::manifest_dir().join("ui/app.slint")).unwrap();
    for region in u0v_visual::REQUIRED_REGIONS {
        assert!(
            app.contains(&format!("region-id: \"{region}\"")),
            "missing region marker: {region}"
        );
    }
}

#[test]
fn pseudo_locale_strings_are_longer_than_english() {
    let i18n = u0v_visual::manifest_dir().join("i18n");
    let en = fs::read_to_string(i18n.join("en/LC_MESSAGES/u0v-visual.po")).unwrap();
    let pseudo = fs::read_to_string(i18n.join("pseudo/LC_MESSAGES/u0v-visual.po")).unwrap();
    let parse = |s: &str| -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();
        let mut id = String::new();
        let mut msg = String::new();
        let mut in_msg = false;
        for line in s.lines() {
            if line.starts_with("msgid ") {
                id = line.trim_start_matches("msgid ").trim_matches('"').to_string();
                in_msg = false;
            } else if line.starts_with("msgstr ") {
                msg = line.trim_start_matches("msgstr ").trim_matches('"').to_string();
                in_msg = true;
            } else if in_msg && line.starts_with('"') {
                msg.push_str(line.trim_matches('"'));
            } else if line.is_empty() && !id.is_empty() && id != "" {
                if !id.is_empty() {
                    map.insert(id.clone(), msg.clone());
                }
                id.clear();
                msg.clear();
                in_msg = false;
            }
        }
        map
    };
    let en_map = parse(&en);
    let pseudo_map = parse(&pseudo);
    for (key, en_val) in &en_map {
        if key.is_empty() {
            continue;
        }
        let Some(pseudo_val) = pseudo_map.get(key) else { continue };
        assert!(
            pseudo_val.len() > en_val.len(),
            "{key}: pseudo must be longer than en"
        );
    }
}
