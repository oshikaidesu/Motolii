//! U0d-3: product sourceにtoolkit raw input分岐を置かないAST主審判。

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use cargo_metadata::MetadataCommand;
use proc_macro2::{Delimiter, TokenStream, TokenTree};
use syn::visit::Visit;
use syn::{Attribute, ExprMethodCall, ItemExternCrate, ItemUse, Macro, Meta, UseTree};

const FORBIDDEN_BARE_IDENTIFIERS: &[&str] = &[
    "KeyCode",
    "PhysicalKey",
    "NamedKey",
    "ModifiersState",
    "MouseButton",
    "KeyEvent",
    "ElementState",
    "RawKeyEvent",
];

const FORBIDDEN_KEY_METHODS: &[&str] = &["key_pressed", "key_released", "key_down"];
const FORBIDDEN_UI_INPUT_METHODS: &[&str] = &["input", "input_mut"];

const FORBIDDEN_PATHS: &[&[&str]] = &[
    &["egui", "Key"],
    &["egui", "Modifiers"],
    &["egui", "PointerButton"],
    &["egui", "Event"],
    &["egui", "InputState"],
    &["egui", "RawInput"],
    &["winit", "keyboard"],
    &["winit", "event"],
    &["winit", "event", "KeyEvent"],
    &["winit", "event", "ElementState"],
    &["winit", "event", "RawKeyEvent"],
    &["winit", "event", "WindowEvent"],
    &["winit", "event", "DeviceEvent"],
    &["WindowEvent", "KeyboardInput"],
    &["WindowEvent", "ModifiersChanged"],
    &["DeviceEvent", "Key"],
];

#[derive(Debug)]
struct UsePath {
    segments: Vec<String>,
    alias: Option<String>,
    glob: bool,
}

fn collect_use_paths(tree: &UseTree, prefix: &mut Vec<String>, out: &mut Vec<UsePath>) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_use_paths(&path.tree, prefix, out);
            prefix.pop();
        }
        UseTree::Name(name) => {
            let mut segments = prefix.clone();
            if name.ident != "self" {
                segments.push(name.ident.to_string());
            }
            out.push(UsePath {
                alias: None,
                segments,
                glob: false,
            });
        }
        UseTree::Rename(rename) => {
            let mut segments = prefix.clone();
            if rename.ident != "self" {
                segments.push(rename.ident.to_string());
            }
            out.push(UsePath {
                segments,
                alias: Some(rename.rename.to_string()),
                glob: false,
            });
        }
        UseTree::Glob(_) => out.push(UsePath {
            segments: prefix.clone(),
            alias: None,
            glob: true,
        }),
        UseTree::Group(group) => {
            for item in &group.items {
                collect_use_paths(item, prefix, out);
            }
        }
    }
}

fn path_is_forbidden(segments: &[String]) -> bool {
    segments
        .iter()
        .any(|segment| FORBIDDEN_BARE_IDENTIFIERS.contains(&segment.as_str()))
        || FORBIDDEN_PATHS.iter().any(|pattern| {
            segments.windows(pattern.len()).any(|window| {
                window
                    .iter()
                    .map(String::as_str)
                    .eq(pattern.iter().copied())
            })
        })
}

fn toolkit_glob_is_forbidden(segments: &[String]) -> bool {
    matches!(segments.first().map(String::as_str), Some("egui" | "winit"))
}

#[derive(Debug)]
struct TokenPath {
    segments: Vec<String>,
    alias: bool,
    glob: bool,
}

fn paths_in_tokens(tokens: TokenStream) -> Vec<TokenPath> {
    fn collect(tokens: TokenStream, prefix: &[String], paths: &mut Vec<TokenPath>) {
        let trees: Vec<_> = tokens.into_iter().collect();
        let mut index = 0;
        while index < trees.len() {
            match &trees[index] {
                TokenTree::Group(group) => {
                    collect(group.stream(), prefix, paths);
                    index += 1;
                }
                TokenTree::Ident(ident) => {
                    let mut segments = prefix.to_vec();
                    segments.push(ident.to_string());
                    let mut cursor = index + 1;
                    let mut expanded_group = false;
                    let mut glob = false;
                    while cursor + 2 < trees.len()
                        && matches!(&trees[cursor], TokenTree::Punct(punct) if punct.as_char() == ':')
                        && matches!(&trees[cursor + 1], TokenTree::Punct(punct) if punct.as_char() == ':')
                    {
                        match &trees[cursor + 2] {
                            TokenTree::Ident(next) => {
                                segments.push(next.to_string());
                                cursor += 3;
                            }
                            TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => {
                                collect(group.stream(), &segments, paths);
                                cursor += 3;
                                expanded_group = true;
                                break;
                            }
                            TokenTree::Punct(punct) if punct.as_char() == '*' => {
                                cursor += 3;
                                glob = true;
                                break;
                            }
                            _ => break,
                        }
                    }
                    if !expanded_group {
                        let alias = matches!(
                            trees.get(cursor),
                            Some(TokenTree::Ident(next)) if next == "as"
                        );
                        paths.push(TokenPath {
                            segments,
                            alias,
                            glob,
                        });
                    }
                    index = cursor.max(index + 1);
                }
                TokenTree::Punct(punct) if punct.as_char() == '*' && !prefix.is_empty() => {
                    paths.push(TokenPath {
                        segments: prefix.to_vec(),
                        alias: false,
                        glob: true,
                    });
                    index += 1;
                }
                TokenTree::Punct(_) | TokenTree::Literal(_) => index += 1,
            }
        }
    }

    let mut paths = Vec::new();
    collect(tokens, &[], &mut paths);
    paths
}

struct RawInputVisitor {
    forbid_ui_input_methods: bool,
    violations: Vec<String>,
}

impl RawInputVisitor {
    fn method_is_forbidden(&self, method: &str) -> bool {
        FORBIDDEN_KEY_METHODS.contains(&method)
            || (self.forbid_ui_input_methods && FORBIDDEN_UI_INPUT_METHODS.contains(&method))
    }

    fn inspect_segments(&mut self, segments: Vec<String>, origin: &str) {
        if path_is_forbidden(&segments) {
            self.violations
                .push(format!("{origin}: {}", segments.join("::")));
        }
    }

    fn inspect_tokens(&mut self, tokens: TokenStream, origin: &str) {
        for path in paths_in_tokens(tokens) {
            if (path.alias || path.glob) && toolkit_glob_is_forbidden(&path.segments) {
                self.violations
                    .push(format!("{origin}: toolkit alias or glob"));
            }
            if path
                .segments
                .iter()
                .any(|item| self.method_is_forbidden(item))
            {
                self.violations
                    .push(format!("{origin}: {}", path.segments.join("::")));
            }
            self.inspect_segments(path.segments, origin);
        }
    }
}

impl<'ast> Visit<'ast> for RawInputVisitor {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        self.inspect_segments(
            path.segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect(),
            "path",
        );
        syn::visit::visit_path(self, path);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        if self.method_is_forbidden(&call.method.to_string()) {
            self.violations.push(format!("method: {}", call.method));
        }
        syn::visit::visit_expr_method_call(self, call);
    }

    fn visit_item_use(&mut self, item_use: &'ast ItemUse) {
        let mut paths = Vec::new();
        collect_use_paths(&item_use.tree, &mut Vec::new(), &mut paths);
        for path in paths {
            if ((path.glob || path.alias.is_some()) && toolkit_glob_is_forbidden(&path.segments))
                || path_is_forbidden(&path.segments)
            {
                self.violations
                    .push(format!("use: {}", path.segments.join("::")));
            }
        }
        syn::visit::visit_item_use(self, item_use);
    }

    fn visit_item_extern_crate(&mut self, item: &'ast ItemExternCrate) {
        if item.rename.is_some() && matches!(item.ident.to_string().as_str(), "egui" | "winit") {
            self.violations
                .push(format!("extern crate alias: {}", item.ident));
        }
        syn::visit::visit_item_extern_crate(self, item);
    }

    fn visit_attribute(&mut self, attribute: &'ast Attribute) {
        if let Meta::List(list) = &attribute.meta {
            self.inspect_tokens(list.tokens.clone(), "attribute");
        }
        syn::visit::visit_attribute(self, attribute);
    }

    fn visit_macro(&mut self, item: &'ast Macro) {
        self.inspect_tokens(item.tokens.clone(), "macro");
        syn::visit::visit_macro(self, item);
    }
}

fn audit_source(source: &str, forbid_ui_input_methods: bool) -> Result<Vec<String>, syn::Error> {
    let file = syn::parse_file(source)?;
    let mut visitor = RawInputVisitor {
        forbid_ui_input_methods,
        violations: Vec::new(),
    };
    visitor.visit_file(&file);
    Ok(visitor.violations)
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("motolii-ui must live under workspace/crates")
        .to_path_buf()
}

fn collect_rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in
        fs::read_dir(dir).unwrap_or_else(|error| panic!("read_dir {}: {error}", dir.display()))
    {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_rust_files(&path, out);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            out.push(path);
        }
    }
}

fn product_sources() -> Vec<PathBuf> {
    let root = workspace_root();
    let metadata = MetadataCommand::new()
        .manifest_path(root.join("Cargo.toml"))
        .no_deps()
        .exec()
        .expect("workspace cargo metadata must resolve");
    let workspace_members: BTreeSet<_> = metadata.workspace_members.iter().collect();
    let mut files = Vec::new();
    for package in metadata.packages {
        if !workspace_members.contains(&package.id) {
            continue;
        }
        let member = package
            .manifest_path
            .parent()
            .expect("workspace member manifest must have a parent")
            .as_std_path();
        let relative = member
            .strip_prefix(&root)
            .expect("workspace member must live under workspace root");
        if !matches!(
            relative
                .components()
                .next()
                .and_then(|item| item.as_os_str().to_str()),
            Some("crates" | "plugins")
        ) {
            continue;
        }
        let src = member.join("src");
        if src.is_dir() {
            collect_rust_files(&src, &mut files);
        }
    }
    files.sort();
    files
}

#[test]
fn workspace_product_sources_have_no_raw_toolkit_input() {
    let files = product_sources();
    assert!(!files.is_empty(), "workspace product source set is empty");

    let mut violations = Vec::new();
    let ui_source = workspace_root().join("crates/motolii-ui/src");
    for path in files {
        let source = fs::read_to_string(&path).unwrap();
        match audit_source(&source, path.starts_with(&ui_source)) {
            Ok(found) => {
                violations.extend(
                    found
                        .into_iter()
                        .map(|violation| format!("{}: {violation}", path.display())),
                );
            }
            Err(error) => violations.push(format!("{}: parse failed: {error}", path.display())),
        }
    }

    assert!(
        violations.is_empty(),
        "raw toolkit input is forbidden outside a specification-approved adapter: {violations:#?}"
    );
}

#[test]
fn audit_rejects_paths_aliases_methods_and_macro_tokens() {
    let rejected = [
        "use egui::{Key, Modifiers as EguiModifiers};",
        "use winit::event::KeyEvent as WinitKey;",
        "use egui as e; fn raw(_: e::Key) {}",
        "mod nested { use egui as e; fn raw(_: e::Key) {} }",
        "extern crate winit as windowing; fn raw(_: windowing::event::KeyEvent) {}",
        "fn raw(_: egui::PointerButton) {}",
        "fn raw(ctx: &egui::Context) { ctx.input(|i| i.key_pressed (egui::Key::A)); }",
        "fn raw(ctx: &egui::Context) { ctx.input(|i| i.modifiers.ctrl || !i.keys_down.is_empty()); }",
        "fn raw(ui: &egui::Ui) { ui.input(|i| !i.events.is_empty()); }",
        "use egui::{InputState, RawInput}; fn raw(_: &InputState, _: &RawInput) {}",
        "use winit::event::{DeviceEvent, RawKeyEvent}; fn raw(_: DeviceEvent, _: RawKeyEvent) {}",
        "use winit::event; fn raw(_: event::DeviceEvent, _: event::WindowEvent) {}",
        "macro_rules! raw { () => { fn f(_: egui::Key) {} } }",
        "macro_rules! raw { () => { use egui::{Key, InputState, RawInput}; } }",
        "macro_rules! raw { () => { use winit::event::{DeviceEvent}; } }",
        "macro_rules! raw { () => { use winit::event; fn f(_: event::DeviceEvent) {} } }",
        "raw! { use egui::{Event, PointerButton}; }",
        "macro_rules! raw { () => { use egui::{{Key}}; } }",
        "macro_rules! raw { () => { use egui::{Key, {InputState, RawInput}}; } }",
        "raw! { use winit::event::{{DeviceEvent}}; }",
        "macro_rules! raw { () => { use egui::{*}; } }",
        "macro_rules! raw { () => { use winit::{*}; } }",
        "raw!(winit::event::KeyEvent);",
        "use winit::*;",
        "use egui as e;",
        "use winit::event as event;",
        "macro_rules! raw { () => { use egui as e; } }",
    ];

    for source in rejected {
        let violations = audit_source(source, true).unwrap();
        assert!(
            !violations.is_empty(),
            "raw input fixture unexpectedly passed: {source}"
        );
    }
}

#[test]
fn audit_ignores_literals_comments_and_domain_modifiers() {
    let accepted = r###"
        use motolii_ui::Modifiers;
        const NORMAL: &str = "egui::Key";
        const RAW: &str = r#"winit::event::KeyEvent"#;
        const BYTES: &[u8] = b"KeyCode";
        const RAW_BYTES: &[u8] = br#"ModifiersState"#;
        const CHARACTER: char = 'K';
        // egui::PointerButton
        /* winit::keyboard /* WindowEvent::KeyboardInput */ */
        fn normalized(_: Modifiers) {}
    "###;

    assert!(audit_source(accepted, true).unwrap().is_empty());
    assert!(
        audit_source("fn domain<T>(value: &T) { value.input(|_| {}); }", false)
            .unwrap()
            .is_empty()
    );
    assert!(audit_source(
        "use first as duplicate; mod nested { use second as duplicate; }",
        false,
    )
    .unwrap()
    .is_empty());
    assert!(
        audit_source("use second as first; use first as second;", false)
            .unwrap()
            .is_empty()
    );
}
