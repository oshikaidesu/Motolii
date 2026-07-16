//! U0a: motolii-ui の公開 API が Slint 型を漏らさないことを走査する。

use std::fs;
use std::path::{Path, PathBuf};

fn crate_src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn collect_rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|err| panic!("read_dir {}: {err}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

fn slint_leaks_in_public_items(source: &str) -> Vec<String> {
    let mut violations = Vec::new();
    let mut in_test_module = false;

    for (line_no, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("#[cfg(test)]") {
            in_test_module = true;
            continue;
        }
        if trimmed == "#[cfg(test)]" || trimmed.starts_with("mod tests") {
            in_test_module = true;
            continue;
        }
        if in_test_module {
            continue;
        }
        if !trimmed.starts_with("pub ") && !trimmed.starts_with("pub(") {
            continue;
        }
        if trimmed.starts_with("pub(crate)") || trimmed.starts_with("pub(super)") {
            continue;
        }
        if trimmed.contains("slint::") {
            violations.push(format!("line {}: {trimmed}", line_no + 1));
        }
    }

    violations
}

#[test]
fn public_items_do_not_reference_slint_types() {
    let root = crate_src_root();
    let mut files = Vec::new();
    collect_rust_files(&root, &mut files);
    assert!(
        !files.is_empty(),
        "motolii-ui/src must contain Rust sources"
    );

    let mut all = Vec::new();
    for file in files {
        let text = fs::read_to_string(&file).unwrap();
        for violation in slint_leaks_in_public_items(&text) {
            all.push(format!("{}: {violation}", file.display()));
        }
    }

    assert!(
        all.is_empty(),
        "public API must not expose slint types: {all:#?}"
    );
}

#[test]
fn exported_types_are_slint_free() {
    fn assert_no_slint_in_type_name<T>() {
        let name = std::any::type_name::<T>();
        assert!(
            !name.contains("slint::"),
            "exported type leaks slint in type_name: {name}"
        );
    }

    assert_no_slint_in_type_name::<motolii_ui::UiCrateInfo>();
    assert_no_slint_in_type_name::<motolii_ui::UiError>();
    assert_no_slint_in_type_name::<Result<motolii_ui::UiCrateInfo, motolii_ui::UiError>>();
}

#[test]
fn crate_info_reports_linked_slint() {
    let info = motolii_ui::crate_info().expect("slint should be linked in motolii-ui");
    assert_eq!(info.crate_id, motolii_ui::CRATE_ID);
    assert!(info.slint_linked);
}
