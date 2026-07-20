//! U1a-1の公開境界とevent-loop source境界。

use std::path::Path;

use motolii_ui::{run_shell, ShellError};
use syn::visit::Visit;

#[test]
fn public_shell_api_is_toolkit_free() {
    let entry: fn() -> Result<(), ShellError> = run_shell;
    let _ = entry;
    let name = std::any::type_name::<ShellError>();
    for forbidden in [
        "egui::",
        "eframe::",
        "egui_wgpu::",
        "egui_winit::",
        "winit::",
        "wgpu::",
    ] {
        assert!(
            !name.contains(forbidden),
            "public ShellError leaks toolkit type: {name}"
        );
    }
}

#[test]
fn event_loop_modules_cannot_render_join_or_read_back() {
    for (path, source) in [
        ("src/app.rs", include_str!("../src/app.rs")),
        ("src/shell.rs", include_str!("../src/shell.rs")),
    ] {
        let file = syn::parse_file(source).unwrap_or_else(|error| panic!("{path}: {error}"));
        let mut visitor = ForbiddenCallVisitor {
            allow_shell_join: path == "src/shell.rs",
            ..Default::default()
        };
        visitor.visit_file(&file);
        assert!(
            visitor.findings.is_empty(),
            "{path} contains event-loop forbidden calls: {:?}",
            visitor.findings
        );
        if path == "src/app.rs" {
            assert_eq!(
                visitor.register_once_in_constructor, 1,
                "app must call the register-once seam exactly once in its constructor"
            );
            assert_eq!(visitor.shell_join_after_event_loop, 0);
        } else {
            assert_eq!(
                visitor.shell_join_after_event_loop, 1,
                "shell must own exactly one post-event-loop join"
            );
            let run_native = source.find("let run_result = eframe::run_native").unwrap();
            let join = source.find("render_worker.join()").unwrap();
            assert!(
                run_native < join,
                "render worker join must follow run_native return"
            );
        }
    }
}

#[derive(Default)]
struct ForbiddenCallVisitor {
    findings: Vec<String>,
    current_function: Option<String>,
    register_once_in_constructor: u32,
    allow_shell_join: bool,
    shell_join_after_event_loop: u32,
}

impl<'ast> Visit<'ast> for ForbiddenCallVisitor {
    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let previous = self.current_function.replace(node.sig.ident.to_string());
        syn::visit::visit_block(self, &node.block);
        self.current_function = previous;
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let previous = self.current_function.replace(node.sig.ident.to_string());
        syn::visit::visit_block(self, &node.block);
        self.current_function = previous;
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = &*node.func {
            let last = path
                .path
                .segments
                .last()
                .map(|segment| segment.ident.to_string());
            if matches!(
                last.as_deref(),
                Some(
                    "render_frame"
                        | "render_graph"
                        | "render_graph_cached"
                        | "download_rgba"
                        | "poll_wait"
                        | "recv"
                        | "join"
                )
            ) {
                self.findings.push(last.unwrap_or_default());
            }
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let method = node.method.to_string();
        if method == "join"
            && self.allow_shell_join
            && self.current_function.as_deref() == Some("run_shell")
        {
            self.shell_join_after_event_loop += 1;
        } else if matches!(method.as_str(), "recv" | "join" | "poll" | "poll_wait") {
            self.findings.push(method.clone());
        }
        if matches!(method.as_str(), "register_once" | "register_native_texture") {
            if method == "register_once" && self.current_function.as_deref() == Some("new") {
                self.register_once_in_constructor += 1;
            } else {
                self.findings.push(format!(
                    "{method} reachable from {:?}",
                    self.current_function
                ));
            }
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

#[test]
fn no_toolkit_dependency_escaped_the_ui_crate() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(manifest.ends_with("crates/motolii-ui"));
}
