//! U2b-1: writer所有とcallback境界をsourceで閉じる。

use syn::visit::Visit;

#[test]
fn app_callbacks_cannot_reach_document_writer_operations() {
    let source = include_str!("../src/app.rs");
    let file = syn::parse_file(source).unwrap();
    let mut visitor = WriterBoundaryVisitor::default();
    visitor.visit_file(&file);
    assert!(
        visitor.findings.is_empty(),
        "app callback reached writer boundary: {:?}",
        visitor.findings
    );
}

#[test]
fn private_runtime_is_the_only_ui_module_that_stores_the_writer() {
    let runtime = include_str!("../src/document_edit_runtime.rs");
    let app = include_str!("../src/app.rs");
    let shell = include_str!("../src/shell.rs");
    assert_eq!(
        runtime
            .lines()
            .filter(|line| line.trim() == "writer: DocumentWriter,")
            .count(),
        1,
        "private edit runtime must own exactly one writer field"
    );
    assert!(!app.contains("DocumentWriter"));
    assert!(!shell.contains("writer: DocumentWriter"));
    for forbidden in ["serde::Serialize", "serde::Deserialize", "derive(Serialize"] {
        assert!(
            !runtime.contains(forbidden),
            "runtime action/snapshot boundary became persistent: {forbidden}"
        );
    }
}

#[derive(Default)]
struct WriterBoundaryVisitor {
    findings: Vec<String>,
}

impl<'ast> Visit<'ast> for WriterBoundaryVisitor {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        if path
            .segments
            .iter()
            .any(|segment| matches!(segment.ident.to_string().as_str(), "DocumentWriter"))
        {
            self.findings.push("DocumentWriter".into());
        }
        syn::visit::visit_path(self, path);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let method = node.method.to_string();
        if matches!(method.as_str(), "apply_macro" | "undo" | "redo") {
            self.findings.push(method);
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_type_reference(&mut self, node: &'ast syn::TypeReference) {
        if node.mutability.is_some()
            && matches!(
                &*node.elem,
                syn::Type::Path(path)
                    if path.path.segments.last().is_some_and(|segment| segment.ident == "Document")
            )
        {
            self.findings.push("&mut Document".into());
        }
        syn::visit::visit_type_reference(self, node);
    }
}
