import 'package:analyzer/analysis_rule/analysis_rule.dart';
import 'package:analyzer/analysis_rule/rule_context.dart';
import 'package:analyzer/analysis_rule/rule_visitor_registry.dart';
import 'package:analyzer/dart/ast/ast.dart';
import 'package:analyzer/dart/ast/visitor.dart';
import 'package:analyzer/error/error.dart';

import 'raw_dimension.dart' show excused, excusedFile, scaleFiles;

/// Housing the Surface Grammar does not allow as grouping: a `Card`, and a rounded, outlined box (a `BoxDecoration` that has both a
/// `borderRadius` and a `border`) used to gather a section. A section is told by a hairline and a change of level
/// (`Surface.divider`, `Surface.raised`), not by a box. A control's or badge's own outline says so in a `// surface:` comment.
class CardHousing extends AnalysisRule {
  static const LintCode code = LintCode(
    'card_housing',
    'A card is housing; separate a section with a hairline and a level (Surface.divider, Surface.raised)',
    correctionMessage: 'Drop the box. If it is a control or a badge, say so: // surface: <why a token cannot express it>.',
    severity: DiagnosticSeverity.WARNING,
  );

  CardHousing()
    : super(
        name: 'card_housing',
        description: 'Cards and rounded, outlined boxes gather sections with housing; the Surface Grammar uses a hairline and a level.',
      );

  @override
  LintCode get diagnosticCode => code;

  @override
  void registerNodeProcessors(RuleVisitorRegistry registry, RuleContext context) {
    final path = context.definingUnit.file.path.replaceAll('\\', '/');
    if (context.isInTestDirectory || scaleFiles.any(path.endsWith) || excusedFile(context.definingUnit.content)) return;
    registry.addInstanceCreationExpression(this, _Visitor(this, context.definingUnit.content));
  }
}

/// The node to report for [node], or null: `Card(...)`, or `BoxDecoration(border: …, borderRadius: …)`.
AstNode? cardHousing(InstanceCreationExpression node) {
  final type = node.constructorName.type.name.lexeme;
  if (type == 'Card') return node;
  if (type != 'BoxDecoration') return null;
  final names = {for (final a in node.argumentList.arguments) if (a is NamedArgument) a.name.lexeme};
  return names.contains('border') && names.contains('borderRadius') ? node : null;
}

class _Visitor extends SimpleAstVisitor<void> {
  _Visitor(this.rule, this.content);
  final AnalysisRule rule;
  final String content;

  @override
  void visitInstanceCreationExpression(InstanceCreationExpression node) {
    final target = cardHousing(node);
    if (target != null && !excused(content, target.offset)) rule.reportAtNode(target);
  }
}
