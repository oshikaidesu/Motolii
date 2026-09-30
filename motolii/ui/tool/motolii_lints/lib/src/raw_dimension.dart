import 'package:analyzer/analysis_rule/analysis_rule.dart';
import 'package:analyzer/analysis_rule/rule_context.dart';
import 'package:analyzer/analysis_rule/rule_visitor_registry.dart';
import 'package:analyzer/dart/ast/ast.dart';
import 'package:analyzer/dart/ast/token.dart';
import 'package:analyzer/dart/ast/visitor.dart';
import 'package:analyzer/error/error.dart';

import 'dimension_use.dart';

/// Files that define the scale and may hold raw numbers.
const scaleFiles = {
  'lib/theme/editor_theme.dart',
  'lib/theme/metrics.dart', // Surface, Dn and UiScale: the visual canon
  'lib/theme/neutral.dart',
};

/// The one file that holds the UI Scale mechanism (its setting, the root that rebuilds the tree, the readout): it may say `scale`.
const scaleMechanismFile = 'lib/app/ui_scale.dart';

/// The scale a file's raw numbers should be taken from: the canon is [Surface] (and [Dn] for text), one class for the whole window.
({String uri, String cls}) scaleFor(String path) => (uri: 'package:motolii_ui/theme/metrics.dart', cls: 'Surface');

/// The only raw numbers a measurement may carry: nothing, or a hairline.
bool structural(num value) => value == 0 || value == .5 || value == 1;

class RawDimension extends AnalysisRule {
  static const LintCode code = LintCode(
    'raw_dimension',
    'Raw number in a measurement; take it from Surface / Dn (lib/theme/metrics.dart), or Surface.px(n) with a reason',
    correctionMessage: 'Use a canonical Surface / Dn token (quick fix when one matches).',
    severity: DiagnosticSeverity.WARNING,
  );

  RawDimension()
    : super(
        name: 'raw_dimension',
        description:
            'Sizes, paddings, radii and font sizes come from the metric scale, '
            'not from literals, so the whole UI can be retuned in one place.',
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
    if (excusedFile(context.definingUnit.content)) return;
    final visitor = _Visitor(this, context.definingUnit.content);
    registry.addIntegerLiteral(this, visitor);
    registry.addDoubleLiteral(this, visitor);
  }
}

/// The literal itself, or the `-literal` around it.
AstNode measured(Literal node) {
  final parent = node.parent;
  if (parent is PrefixExpression && parent.operator.type == TokenType.MINUS) {
    return parent;
  }
  return node;
}

/// Whether [node] sits where a measurement is expected (see [dimensionUse]).
bool isMeasurement(AstNode node) => dimensionUse(node).measured;

/// The node to report for a numeric [node] of [value], or null when it is
/// not a raw measurement. Shared by the plugin and `bin/check.dart`.
AstNode? rawMeasurement(Literal node, num? value) {
  if (value == null || structural(value)) return null;
  final target = measured(node);
  return isMeasurement(target) ? target : null;
}

class _Visitor extends SimpleAstVisitor<void> {
  _Visitor(this.rule, this.content);
  final AnalysisRule rule;
  final String content;

  void _check(Literal node, num? value) {
    final target = rawMeasurement(node, value);
    if (target != null && !excusedNode(content, target)) rule.reportAtNode(target);
  }

  @override
  void visitIntegerLiteral(IntegerLiteral node) => _check(node, node.value);

  @override
  void visitDoubleLiteral(DoubleLiteral node) => _check(node, node.value);
}

/// An exception is a comment that says why the Surface tokens cannot express this: `// surface: <reason>` on the line or the line
/// above (a reason of at least eight characters). A bare `// surface:` excuses nothing.
final _excuse = RegExp(r'//\s*surface:\s*(\S.{7,})');

bool excused(String content, int offset) {
  final lineStart = content.lastIndexOf('\n', offset == 0 ? 0 : offset - 1) + 1;
  final lineEnd = content.indexOf('\n', offset);
  final line = content.substring(lineStart, lineEnd < 0 ? content.length : lineEnd);
  if (_excuse.hasMatch(line)) return true;
  if (lineStart == 0) return false;
  final prevStart = content.lastIndexOf('\n', lineStart - 2) + 1;
  return _excuse.hasMatch(content.substring(prevStart, lineStart - 1));
}

/// [excused] for a node: its line, or a declaration around it that says `// surface-block: <reason>` above itself (a painter, a
/// canvas function: geometry that is drawn in its own pixels).
bool excusedNode(String content, AstNode node) {
  if (excused(content, node.offset)) return true;
  for (AstNode? n = node.parent; n != null; n = n.parent) {
    if (n is! Declaration) continue;
    // the comments before a declaration hang on its first real token (after its documentation and annotations)
    final first = n.metadata.isEmpty ? n.firstTokenAfterCommentAndMetadata : n.metadata.first.beginToken;
    Token? c = first.precedingComments;
    while (c != null) {
      if (_excuseBlock.hasMatch(c.lexeme)) return true;
      c = c.next;
    }
  }
  return false;
}

final _excuseBlock = RegExp(r'//\s*surface-block:\s*\S.{7,}');

/// A file that says, in its first lines, that it is a fixture of its own and not product UI: `// surface-file: <reason>`.
final _excuseFile = RegExp(r'^//\s*surface-file:\s*\S.{7,}', multiLine: true);
bool excusedFile(String content) {
  final head = content.length > 400 ? content.substring(0, 400) : content;
  return _excuseFile.hasMatch(head);
}
