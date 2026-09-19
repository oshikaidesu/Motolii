import 'package:analyzer/analysis_rule/analysis_rule.dart';
import 'package:analyzer/analysis_rule/rule_context.dart';
import 'package:analyzer/analysis_rule/rule_visitor_registry.dart';
import 'package:analyzer/dart/ast/ast.dart';
import 'package:analyzer/dart/ast/visitor.dart';
import 'package:analyzer/error/error.dart';

/// The libraries the window does not build on: Material and Cupertino bring
/// their own metrics, ripples and motion, which the leaves in
/// foundation/leaves.dart replaced on `flutter/widgets` alone.
const foreignImports = {
  'package:flutter/material.dart',
  'package:flutter/cupertino.dart',
};

class MaterialImport extends AnalysisRule {
  static const LintCode code = LintCode(
    'material_import',
    'Material import in the window; build on flutter/widgets and the leaves',
    correctionMessage:
        'Import package:flutter/widgets.dart and take the control from '
        'foundation/leaves.dart (EditorPress, EditorIconButton, '
        'EditorTextField, …) or the glyph from foundation/glyphs.dart.',
    severity: DiagnosticSeverity.WARNING,
  );

  MaterialImport()
    : super(
        name: 'material_import',
        description:
            'The window is built on flutter/widgets and its own leaves; a '
            'Material or Cupertino import brings a second set of metrics, '
            'ripples and motion the theme would have to silence again.',
      );

  @override
  LintCode get diagnosticCode => code;

  @override
  void registerNodeProcessors(
    RuleVisitorRegistry registry,
    RuleContext context,
  ) {
    if (context.isInTestDirectory) return;
    registry.addImportDirective(this, _Visitor(this));
  }
}

/// The directive when it imports one of [foreignImports].
AstNode? materialImport(ImportDirective node) =>
    foreignImports.contains(node.uri.stringValue) ? node : null;

class _Visitor extends SimpleAstVisitor<void> {
  _Visitor(this.rule);
  final AnalysisRule rule;

  @override
  void visitImportDirective(ImportDirective node) {
    final target = materialImport(node);
    if (target != null) rule.reportAtNode(target);
  }
}
