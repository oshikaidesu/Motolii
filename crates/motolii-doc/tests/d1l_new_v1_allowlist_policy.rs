//! D1l: `Document::new_v1()` の非製品 allowlist 機械 gate (2026-07-16 §2.2/§5.3)。

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    Attribute, Expr, ExprCall, ExprPath, File, ForeignItem, ImplItem, Item, Meta, MetaList,
    TraitItem,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TriValue {
    True,
    False,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NewV1Violation {
    path: PathBuf,
    line: usize,
    text: String,
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/motolii-doc -> workspace root")
        .to_path_buf()
}

fn is_migrate_rs(path: &Path) -> bool {
    path.file_name() == Some(OsStr::new("migrate.rs"))
        && path
            .parent()
            .is_some_and(|p| p.file_name() == Some(OsStr::new("src")))
        && path
            .parent()
            .and_then(|p| p.parent())
            .is_some_and(|p| p.file_name() == Some(OsStr::new("motolii-doc")))
}

fn parse_nested_metas(tokens: &proc_macro2::TokenStream) -> Vec<Meta> {
    syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated
        .parse2(tokens.clone())
        .map(|punctuated| punctuated.into_iter().collect())
        .unwrap_or_default()
}

fn eval_cfg_meta_when_test_false(meta: &Meta) -> TriValue {
    match meta {
        Meta::Path(path) => {
            if path.is_ident("test") {
                TriValue::False
            } else {
                TriValue::Unknown
            }
        }
        Meta::NameValue(_) => TriValue::Unknown,
        Meta::List(list) => eval_cfg_list_when_test_false(list),
    }
}

fn eval_cfg_list_when_test_false(list: &MetaList) -> TriValue {
    if list.path.is_ident("all") {
        let nested = parse_nested_metas(&list.tokens);
        if nested.is_empty() {
            return TriValue::Unknown;
        }
        let mut all_true = true;
        for meta in &nested {
            match eval_cfg_meta_when_test_false(meta) {
                TriValue::False => return TriValue::False,
                TriValue::Unknown => all_true = false,
                TriValue::True => {}
            }
        }
        if all_true {
            TriValue::True
        } else {
            TriValue::Unknown
        }
    } else if list.path.is_ident("any") {
        let nested = parse_nested_metas(&list.tokens);
        if nested.is_empty() {
            return TriValue::Unknown;
        }
        let mut all_false = true;
        for meta in &nested {
            match eval_cfg_meta_when_test_false(meta) {
                TriValue::True => return TriValue::True,
                TriValue::Unknown => all_false = false,
                TriValue::False => {}
            }
        }
        if all_false {
            TriValue::False
        } else {
            TriValue::Unknown
        }
    } else if list.path.is_ident("not") {
        let nested = parse_nested_metas(&list.tokens);
        if nested.len() != 1 {
            return TriValue::Unknown;
        }
        match eval_cfg_meta_when_test_false(&nested[0]) {
            TriValue::True => TriValue::False,
            TriValue::False => TriValue::True,
            TriValue::Unknown => TriValue::Unknown,
        }
    } else {
        TriValue::Unknown
    }
}

fn has_cfg_test(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("cfg") {
            return false;
        }
        let list = match attr.meta.require_list() {
            Ok(list) => list,
            Err(_) => return false,
        };
        let meta = match syn::parse2::<Meta>(list.tokens.clone()) {
            Ok(meta) => meta,
            Err(_) => return false,
        };
        eval_cfg_meta_when_test_false(&meta) == TriValue::False
    })
}

fn is_new_v1_call(func: &Expr) -> bool {
    let Expr::Path(ExprPath { path, .. }) = func else {
        return false;
    };
    let segments = &path.segments;
    segments.len() >= 2
        && segments[segments.len() - 2].ident == "Document"
        && segments[segments.len() - 1].ident == "new_v1"
}

fn item_attrs(item: &Item) -> &[Attribute] {
    match item {
        Item::Const(item) => &item.attrs,
        Item::Enum(item) => &item.attrs,
        Item::ExternCrate(item) => &item.attrs,
        Item::Fn(item) => &item.attrs,
        Item::ForeignMod(item) => &item.attrs,
        Item::Impl(item) => &item.attrs,
        Item::Macro(item) => &item.attrs,
        Item::Mod(item) => &item.attrs,
        Item::Static(item) => &item.attrs,
        Item::Struct(item) => &item.attrs,
        Item::Trait(item) => &item.attrs,
        Item::TraitAlias(item) => &item.attrs,
        Item::Type(item) => &item.attrs,
        Item::Union(item) => &item.attrs,
        Item::Use(item) => &item.attrs,
        _ => &[],
    }
}

fn impl_item_attrs(item: &ImplItem) -> &[Attribute] {
    match item {
        ImplItem::Const(item) => &item.attrs,
        ImplItem::Fn(item) => &item.attrs,
        ImplItem::Type(item) => &item.attrs,
        ImplItem::Macro(item) => &item.attrs,
        ImplItem::Verbatim(_) | _ => &[],
    }
}

fn trait_item_attrs(item: &TraitItem) -> &[Attribute] {
    match item {
        TraitItem::Const(item) => &item.attrs,
        TraitItem::Fn(item) => &item.attrs,
        TraitItem::Type(item) => &item.attrs,
        TraitItem::Macro(item) => &item.attrs,
        TraitItem::Verbatim(_) | _ => &[],
    }
}

fn foreign_item_attrs(item: &ForeignItem) -> &[Attribute] {
    match item {
        ForeignItem::Fn(item) => &item.attrs,
        ForeignItem::Static(item) => &item.attrs,
        ForeignItem::Type(item) => &item.attrs,
        ForeignItem::Macro(item) => &item.attrs,
        ForeignItem::Verbatim(_) | _ => &[],
    }
}

struct NewV1Visitor<'a> {
    path: &'a Path,
    source: &'a str,
    cfg_test_depth: usize,
    in_migrate: bool,
    violations: Vec<NewV1Violation>,
}

impl<'a> NewV1Visitor<'a> {
    fn allowed(&self) -> bool {
        self.in_migrate || self.cfg_test_depth > 0
    }

    fn record_violation(&mut self, line: usize) {
        let text = self
            .source
            .lines()
            .nth(line.saturating_sub(1))
            .unwrap_or("")
            .trim()
            .to_string();
        self.violations.push(NewV1Violation {
            path: self.path.to_path_buf(),
            line,
            text,
        });
    }

    fn with_cfg_test_depth<R>(&mut self, attrs: &[Attribute], f: impl FnOnce(&mut Self) -> R) -> R {
        let enters_cfg_test = has_cfg_test(attrs);
        if enters_cfg_test {
            self.cfg_test_depth += 1;
        }
        let result = f(self);
        if enters_cfg_test {
            self.cfg_test_depth -= 1;
        }
        result
    }
}

impl<'a> Visit<'_> for NewV1Visitor<'a> {
    fn visit_item(&mut self, item: &Item) {
        self.with_cfg_test_depth(item_attrs(item), |visitor| visit::visit_item(visitor, item));
    }

    fn visit_impl_item(&mut self, item: &ImplItem) {
        self.with_cfg_test_depth(impl_item_attrs(item), |visitor| {
            visit::visit_impl_item(visitor, item)
        });
    }

    fn visit_trait_item(&mut self, item: &TraitItem) {
        self.with_cfg_test_depth(trait_item_attrs(item), |visitor| {
            visit::visit_trait_item(visitor, item)
        });
    }

    fn visit_foreign_item(&mut self, item: &ForeignItem) {
        self.with_cfg_test_depth(foreign_item_attrs(item), |visitor| {
            visit::visit_foreign_item(visitor, item)
        });
    }

    fn visit_expr_call(&mut self, call: &ExprCall) {
        if is_new_v1_call(&call.func) && !self.allowed() {
            self.record_violation(call.func.span().start().line);
        }
        visit::visit_expr_call(self, call);
    }
}

fn find_disallowed_new_v1_calls(path: &Path, source: &str) -> Vec<NewV1Violation> {
    let file: File = match syn::parse_file(source) {
        Ok(file) => file,
        Err(err) => panic!("failed to parse {}: {err}", path.display()),
    };

    let mut visitor = NewV1Visitor {
        path,
        source,
        cfg_test_depth: 0,
        in_migrate: is_migrate_rs(path),
        violations: Vec::new(),
    };
    visitor.visit_file(&file);
    visitor.violations
}

fn scan_workspace_src() -> Vec<NewV1Violation> {
    let root = workspace_root().join("crates");
    let mut violations = Vec::new();
    scan_rust_sources(&root, &mut violations);
    violations
}

fn scan_rust_sources(dir: &Path, violations: &mut Vec<NewV1Violation>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|err| {
        panic!("read {}: {err}", dir.display());
    });
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_rust_sources(&path, violations);
            continue;
        }
        if path.extension() != Some(OsStr::new("rs")) {
            continue;
        }
        if !path
            .components()
            .any(|c| c.as_os_str() == OsStr::new("src"))
        {
            continue;
        }
        let source = fs::read_to_string(&path).unwrap_or_else(|err| {
            panic!("read {}: {err}", path.display());
        });
        violations.extend(find_disallowed_new_v1_calls(&path, &source));
    }
}

#[test]
fn workspace_src_has_no_disallowed_new_v1_calls() {
    let violations = scan_workspace_src();
    assert!(
        violations.is_empty(),
        "Document::new_v1() is allowlisted only in migrate.rs legacy generation and #[cfg(test)] items; violations: {violations:#?}"
    );
}

#[test]
fn detector_flags_non_allowlisted_src_call() {
    let sample = r#"
        pub fn bad() -> Document {
            Document::new_v1()
        }
    "#;
    let hits = find_disallowed_new_v1_calls(Path::new("lib.rs"), sample);
    assert_eq!(hits.len(), 1);
    assert!(hits[0].text.contains("Document::new_v1()"));
}

#[test]
fn detector_allows_cfg_test_module() {
    let sample = r#"
        #[cfg(test)]
        mod tests {
            fn ok() {
                let _ = Document::new_v1();
            }
        }
    "#;
    assert!(find_disallowed_new_v1_calls(Path::new("lib.rs"), sample).is_empty());
}

#[test]
fn detector_allows_nested_cfg_test_module() {
    let sample = r#"
        #[cfg(test)]
        mod tests {
            mod nested {
                fn ok() {
                    let _ = Document::new_v1();
                }
            }
        }
    "#;
    assert!(find_disallowed_new_v1_calls(Path::new("lib.rs"), sample).is_empty());
}

#[test]
fn detector_allows_migrate_rs_legacy_generation() {
    let sample = r#"
        pub fn legacy_seed() -> Document {
            Document::new_v1()
        }
    "#;
    let path = Path::new("crates/motolii-doc/src/migrate.rs");
    assert!(find_disallowed_new_v1_calls(path, sample).is_empty());
}

#[test]
fn detector_ignores_comment_and_string_literals() {
    let sample = r##"
        pub fn new_v1() {
            // Document::new_v1()
            let _ = "Document::new_v1()";
            let _ = r#"Document::new_v1()"#;
        }
    "##;
    assert!(find_disallowed_new_v1_calls(Path::new("lib.rs"), sample).is_empty());
}

#[test]
fn detector_skips_compile_fail_doc_example() {
    let sample = r#"
        /// ```compile_fail
        /// let _ = motolii_doc::Document::new_v1();
        /// ```
        pub fn new_v1() {}
    "#;
    assert!(find_disallowed_new_v1_calls(Path::new("lib.rs"), sample).is_empty());
}

#[test]
fn detector_allows_cfg_all_test_unix() {
    let sample = r#"
        #[cfg(all(test, unix))]
        mod tests {
            fn ok() {
                let _ = Document::new_v1();
            }
        }
    "#;
    assert!(find_disallowed_new_v1_calls(Path::new("lib.rs"), sample).is_empty());
}

#[test]
fn detector_allows_cfg_not_not_test() {
    let sample = r#"
        #[cfg(not(not(test)))]
        fn ok() {
            let _ = Document::new_v1();
        }
    "#;
    assert!(find_disallowed_new_v1_calls(Path::new("lib.rs"), sample).is_empty());
}

#[test]
fn detector_rejects_cfg_any_feature_or_test() {
    let sample = r#"
        #[cfg(any(feature = "x", test))]
        fn bad() {
            let _ = Document::new_v1();
        }
    "#;
    assert!(
        !find_disallowed_new_v1_calls(Path::new("lib.rs"), sample).is_empty(),
        "cfg(any(feature, test)) must not be treated as test-only"
    );
}

#[test]
fn detector_rejects_cfg_not_test() {
    let sample = r#"
        #[cfg(not(test))]
        fn bad() {
            let _ = Document::new_v1();
        }
    "#;
    assert!(
        !find_disallowed_new_v1_calls(Path::new("lib.rs"), sample).is_empty(),
        "cfg(not(test)) must not be treated as test-only"
    );
}

#[test]
fn detector_rejects_cfg_unix() {
    let sample = r#"
        #[cfg(unix)]
        fn bad() {
            let _ = Document::new_v1();
        }
    "#;
    assert!(
        !find_disallowed_new_v1_calls(Path::new("lib.rs"), sample).is_empty(),
        "cfg(unix) must not be treated as test-only"
    );
}

#[test]
fn detector_rejects_cfg_all_unix_any_test_feature() {
    let sample = r#"
        #[cfg(all(unix, any(test, feature = "x")))]
        fn bad() {
            let _ = Document::new_v1();
        }
    "#;
    assert!(
        !find_disallowed_new_v1_calls(Path::new("lib.rs"), sample).is_empty(),
        "cfg(all(unix, any(test, feature))) must not be treated as test-only"
    );
}

#[test]
fn detector_allows_cfg_test_method_in_non_test_impl() {
    let sample = r#"
        impl Document {
            #[cfg(test)]
            fn test_only() {
                let _ = Document::new_v1();
            }
        }
    "#;
    assert!(find_disallowed_new_v1_calls(Path::new("lib.rs"), sample).is_empty());
}

#[test]
fn detector_rejects_non_test_sibling_method_in_impl() {
    let sample = r#"
        impl Document {
            #[cfg(test)]
            fn test_only() {
                let _ = Document::new_v1();
            }

            fn production() {
                let _ = Document::new_v1();
            }
        }
    "#;
    assert!(
        !find_disallowed_new_v1_calls(Path::new("lib.rs"), sample).is_empty(),
        "non-test sibling impl method must be flagged"
    );
}

#[test]
fn detector_allows_nested_test_only_context() {
    let sample = r#"
        #[cfg(test)]
        mod outer {
            #[cfg(all(test, unix))]
            fn ok() {
                let _ = Document::new_v1();
            }
        }
    "#;
    assert!(find_disallowed_new_v1_calls(Path::new("lib.rs"), sample).is_empty());
}
