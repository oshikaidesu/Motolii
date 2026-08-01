//! U0a: motolii-uiの公開APIがUI toolkit型を漏らさないことを走査する。

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use syn::{visit::Visit, UseTree};

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

const TOOLKIT_PATHS: &[&str] = &[
    "egui::",
    "eframe::",
    "egui_wgpu::",
    "egui_winit::",
    "egui_tiles::",
    "winit::",
    "slint::",
];

fn toolkit_leaks_in_public_items(source: &str) -> Vec<String> {
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
        if TOOLKIT_PATHS.iter().any(|path| trimmed.contains(path)) {
            violations.push(format!("line {}: {trimmed}", line_no + 1));
        }
    }

    violations
}

#[test]
fn public_items_do_not_reference_toolkit_types() {
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
        for violation in toolkit_leaks_in_public_items(&text) {
            all.push(format!("{}: {violation}", file.display()));
        }
    }

    assert!(
        all.is_empty(),
        "public API must not expose UI toolkit types: {all:#?}"
    );
}

#[test]
fn exported_types_are_toolkit_free() {
    fn assert_no_toolkit_in_type_name<T>() {
        let name = std::any::type_name::<T>();
        assert!(
            !TOOLKIT_PATHS.iter().any(|path| name.contains(path)),
            "exported type leaks UI toolkit in type_name: {name}"
        );
    }

    assert_no_toolkit_in_type_name::<motolii_ui::UiCrateInfo>();
    assert_no_toolkit_in_type_name::<motolii_ui::UiError>();
    assert_no_toolkit_in_type_name::<motolii_ui::UiStateOwner>();
    assert_no_toolkit_in_type_name::<motolii_ui::UiStateLifetime>();
    assert_no_toolkit_in_type_name::<motolii_ui::DomainIntent>();
    assert_no_toolkit_in_type_name::<motolii_ui::DomainIntentError>();
    assert_no_toolkit_in_type_name::<motolii_ui::DocumentCommandRequest>();
    assert_no_toolkit_in_type_name::<motolii_ui::DocumentCommandRequestError>();
    assert_no_toolkit_in_type_name::<motolii_ui::CommandId>();
    assert_no_toolkit_in_type_name::<motolii_ui::CommandIdError>();
    assert_no_toolkit_in_type_name::<motolii_ui::CommandMetadata>();
    assert_no_toolkit_in_type_name::<motolii_ui::CommandRegistry>();
    assert_no_toolkit_in_type_name::<motolii_ui::CommandRegistryError>();
    assert_no_toolkit_in_type_name::<motolii_ui::InputPhase>();
    assert_no_toolkit_in_type_name::<motolii_ui::ImeGateState>();
    assert_no_toolkit_in_type_name::<motolii_ui::SafetyInterrupt>();
    assert_no_toolkit_in_type_name::<motolii_ui::NormalizedInput>();
    assert_no_toolkit_in_type_name::<motolii_ui::RouterOutput>();
    assert_no_toolkit_in_type_name::<motolii_ui::InputRouter>();
    assert_no_toolkit_in_type_name::<motolii_ui::InputRouterError>();
    assert_no_toolkit_in_type_name::<motolii_ui::InteractionState>();
    assert_no_toolkit_in_type_name::<motolii_ui::InteractionStateMachine>();
    assert_no_toolkit_in_type_name::<motolii_ui::InteractionTransitionError>();
    assert_no_toolkit_in_type_name::<motolii_ui::DiagnosticEnvelope>();
    assert_no_toolkit_in_type_name::<motolii_ui::DiagnosticReasonCode>();
    assert_no_toolkit_in_type_name::<motolii_ui::DiagnosticActionKind>();
    assert_no_toolkit_in_type_name::<motolii_ui::DiagnosticSubject>();
    assert_no_toolkit_in_type_name::<motolii_ui::DiagnosticFact>();
    assert_no_toolkit_in_type_name::<motolii_ui::DiagnosticRecoverability>();
    assert_no_toolkit_in_type_name::<motolii_ui::UnsupportedDiagnosticSource>();
    assert_no_toolkit_in_type_name::<motolii_ui::Gesture>();
    assert_no_toolkit_in_type_name::<motolii_ui::EffectiveTrigger>();
    assert_no_toolkit_in_type_name::<motolii_ui::KeymapResolution>();
    assert_no_toolkit_in_type_name::<motolii_ui::KeymapDiagnostic>();
    assert_no_toolkit_in_type_name::<motolii_ui::KeymapCodecLimits>();
    assert_no_toolkit_in_type_name::<motolii_ui::LimitKind>();
    assert_no_toolkit_in_type_name::<motolii_ui::KeymapCodecError>();
    assert_no_toolkit_in_type_name::<motolii_ui::OpaqueOperationReason>();
    assert_no_toolkit_in_type_name::<motolii_ui::KeymapCodecDiagnostic>();
    assert_no_toolkit_in_type_name::<motolii_ui::KeymapApplyError>();
    assert_no_toolkit_in_type_name::<motolii_ui::LoadedKeymap>();
    assert_no_toolkit_in_type_name::<motolii_ui::TimelineBar>();
    assert_no_toolkit_in_type_name::<motolii_ui::TimelineHit>();
    assert_no_toolkit_in_type_name::<motolii_ui::TimelineKey>();
    assert_no_toolkit_in_type_name::<motolii_ui::TimelineMetrics>();
    assert_no_toolkit_in_type_name::<motolii_ui::TimelineProjection>();
    assert_no_toolkit_in_type_name::<motolii_ui::TimelineProjectionError>();
    assert_no_toolkit_in_type_name::<motolii_ui::TimelineUnsupported>();
    assert_no_toolkit_in_type_name::<motolii_ui::TimelineViewport>();
    assert_no_toolkit_in_type_name::<Result<motolii_ui::UiCrateInfo, motolii_ui::UiError>>();
}

#[test]
fn crate_info_reports_linked_toolkit() {
    let info = motolii_ui::crate_info().expect("egui should be linked in motolii-ui");
    assert_eq!(info.crate_id, motolii_ui::CRATE_ID);
    assert!(info.toolkit_linked);
}

#[test]
fn parameter_control_is_private_module_with_exact_reexports() {
    let lib = {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("lib.rs");
        fs::read_to_string(path).unwrap()
    };
    let ast = syn::parse_file(&lib).unwrap();

    #[derive(Default)]
    struct ParameterControlExtractor {
        declared_private: bool,
        exported: BTreeSet<String>,
        invalid_reexport: bool,
    }

    impl<'ast> Visit<'ast> for ParameterControlExtractor {
        fn visit_item_mod(&mut self, i: &'ast syn::ItemMod) {
            if i.ident == "parameter_control" && matches!(i.vis, syn::Visibility::Inherited) {
                self.declared_private = true;
            }
            syn::visit::visit_item_mod(self, i);
        }

        fn visit_item_use(&mut self, i: &'ast syn::ItemUse) {
            if !matches!(&i.vis, syn::Visibility::Public(_)) {
                return;
            }

            fn collect(
                pc_depth: Option<usize>,
                tree: &UseTree,
                exported: &mut BTreeSet<String>,
                invalid: &mut bool,
            ) {
                match tree {
                    UseTree::Path(node) => {
                        let next_depth = match pc_depth {
                            Some(depth) => Some(depth + 1),
                            None if node.ident == "parameter_control" => Some(0),
                            None => None,
                        };
                        collect(next_depth, &node.tree, exported, invalid);
                    }
                    UseTree::Name(node) => match pc_depth {
                        Some(0) => {
                            exported.insert(node.ident.to_string());
                        }
                        Some(_) => {
                            *invalid = true;
                        }
                        None => {}
                    },
                    UseTree::Rename(_) => {
                        if pc_depth.is_some() {
                            *invalid = true;
                        }
                    }
                    UseTree::Glob(_) => {
                        if pc_depth.is_some() {
                            *invalid = true;
                        }
                    }
                    UseTree::Group(node) => {
                        for item in &node.items {
                            collect(pc_depth, item, exported, invalid);
                        }
                    }
                }
            }

            collect(
                None,
                &i.tree,
                &mut self.exported,
                &mut self.invalid_reexport,
            );
            syn::visit::visit_item_use(self, i);
        }
    }

    let mut extractor = ParameterControlExtractor::default();
    extractor.visit_file(&ast);

    assert!(
        extractor.declared_private,
        "parameter_control must be declared as a private module in lib.rs"
    );
    assert!(
        !extractor.invalid_reexport,
        "parameter_control reexports must be direct identifiers: pub use parameter_control::{{...}}"
    );

    let expected = BTreeSet::from_iter([
        "HostParameterControl".to_string(),
        "map_parameter_control".to_string(),
        "ParameterControlError".to_string(),
        "ParameterControlSpec".to_string(),
    ]);
    assert_eq!(extractor.exported, expected);
}
