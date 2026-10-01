import 'package:analyzer/analysis_rule/analysis_rule.dart';
import 'package:analyzer/analysis_rule/rule_context.dart';
import 'package:analyzer/analysis_rule/rule_visitor_registry.dart';
import 'package:analyzer/dart/ast/ast.dart';
import 'package:analyzer/dart/ast/visitor.dart';
import 'package:analyzer/error/error.dart';

import 'raw_dimension.dart' show scaleFiles;

class RawColor extends AnalysisRule {
  static const LintCode code = LintCode(
    'raw_color',
    'Raw colour outside the theme; take it from EditorTheme or EditorInk',
    correctionMessage:
        'Name the colour as a token in foundation/theme.dart and read that.',
    severity: DiagnosticSeverity.WARNING,
  );

  RawColor()
    : super(
        name: 'raw_color',
        description:
            'Colours come from the theme (EditorTheme, EditorInk), not from '
            'Color(0x…) literals, so the whole look can be retuned in one place.',
      );

  @override
  LintCode get diagnosticCode => code;

  @override
  void registerNodeProcessors(
    RuleVisitorRegistry registry,
    RuleContext context,
  ) {
    final path = context.definingUnit.file.path.replaceAll('\\', '/');
    if (context.isInTestDirectory || scaleFiles.any(path.endsWith)) return;
    registry.addInstanceCreationExpression(this, _Visitor(this));
  }
}

/// `Color(<integer literal>)` — a hex colour written where a token belongs.
/// `Color(0xff000000 | n)` and `Color.fromARGB(...)` are computed, not named,
/// and pass.
AstNode? rawColor(InstanceCreationExpression node) {
  final name = node.constructorName;
  if (name.type.name.lexeme != 'Color' || name.name != null) return null;
  final args = node.argumentList.arguments;
  if (args.length != 1 || args.first is! IntegerLiteral) return null;
  return node;
}

class _Visitor extends SimpleAstVisitor<void> {
  _Visitor(this.rule);
  final AnalysisRule rule;

  @override
  void visitInstanceCreationExpression(InstanceCreationExpression node) {
    final target = rawColor(node);
    if (target != null) rule.reportAtNode(target);
  }
}
