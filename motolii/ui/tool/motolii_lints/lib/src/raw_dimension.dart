import 'package:analyzer/analysis_rule/analysis_rule.dart';
import 'package:analyzer/analysis_rule/rule_context.dart';
import 'package:analyzer/analysis_rule/rule_visitor_registry.dart';
import 'package:analyzer/dart/ast/ast.dart';
import 'package:analyzer/dart/ast/token.dart';
import 'package:analyzer/dart/ast/visitor.dart';
import 'package:analyzer/error/error.dart';

/// Argument names whose numeric value is a layout measurement.
const measuredNames = {
  'width', 'height', 'minWidth', 'maxWidth', 'minHeight', 'maxHeight',
  'fontSize', 'size', 'iconSize', 'dimension', 'radius', 'thickness',
  'strokeWidth', 'indent', 'endIndent', 'spacing', 'runSpacing',
  'mainAxisSpacing', 'crossAxisSpacing', 'horizontal', 'vertical',
  'left', 'top', 'right', 'bottom', 'start', 'end',
  'toolbarHeight', 'leadingWidth', 'itemExtent',
};

/// Constructors whose positional numbers are measurements, and the text helpers whose first argument is a font size
/// (`sans(11, …)` is `fontSize: 11` by another name).
const measuredTypes = {
  'EdgeInsets', 'EdgeInsetsDirectional', 'BorderRadius', 'Radius', 'Size',
  'sans', 'mono', 'caps',
};

/// Files that define the scale and may hold raw numbers.
const scaleFiles = {
  'lib/theme/editor_metrics.dart',
  'lib/theme/editor_theme.dart',
  'lib/foundation/panel_catalog.dart',
  'lib/foundation/shell_tokens.dart',
  'lib/theme/surface.dart', // Surface and Dn: the product window's grammar
  'lib/theme/neutral.dart',
};

/// The scale a file's raw numbers should be taken from: the product window reads [Surface]; the Classic and New windows (`legacy/`),
/// the Stage chrome and the shared panel controls read [EditorMetrics] until they are retired.
({String uri, String cls}) scaleFor(String path) {
  final unix = path.replaceAll('\\', '/');
  const editorMetricsDirs = ['/lib/legacy/', '/lib/stage/', '/lib/controls/leaves', '/lib/controls/panel', '/lib/theme/editor_', '/lib/colors/color_field', '/lib/colors/hsv_triangle'];
  if (editorMetricsDirs.any(unix.contains)) {
    return (uri: 'package:motolii_stage5/theme/editor_metrics.dart', cls: 'EditorMetrics');
  }
  return (uri: 'package:motolii_stage5/theme/surface.dart', cls: 'Surface');
}

/// The only raw numbers a measurement may carry: nothing, or a hairline.
bool structural(num value) => value == 0 || value == .5 || value == 1;

class RawDimension extends AnalysisRule {
  static const LintCode code = LintCode(
    'raw_dimension',
    'Raw number in a measurement; take it from EditorMetrics',
    correctionMessage: 'Use an EditorMetrics token (quick fix when one matches).',
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

/// Whether [node] sits where a measurement is expected. Branches of a
/// conditional, parentheses and `+`/`-` operands inherit the position;
/// `*` and `/` operands are ratios, not pixels.
bool isMeasurement(AstNode node) {
  var parent = node.parent;
  while (parent is ParenthesizedExpression ||
      (parent is ConditionalExpression && parent.condition != node) ||
      (parent is BinaryExpression &&
          (parent.operator.type == TokenType.PLUS ||
              parent.operator.type == TokenType.MINUS))) {
    node = parent!;
    parent = node.parent;
  }
  if (parent is NamedArgument) {
    final name = parent.name.lexeme;
    if (!measuredNames.contains(name)) return false;
    // TextStyle(height:) is a line-height multiplier, not pixels.
    final call = parent.parent?.parent;
    if (name == 'height' && _typeName(call) == 'TextStyle') return false;
    return true;
  }
  if (parent is ArgumentList) {
    return measuredTypes.contains(_typeName(parent.parent));
  }
  return false;
}

String? _typeName(AstNode? call) {
  if (call is InstanceCreationExpression) {
    return call.constructorName.type.name.lexeme;
  }
  if (call is MethodInvocation) {
    final target = call.target;
    if (target is SimpleIdentifier) return target.name;
    if (target == null) return call.methodName.name; // a bare `sans(11, …)`
  }
  return null;
}

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
    if (target != null && !excused(content, target.offset)) rule.reportAtNode(target);
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

/// A file that says, in its first lines, that it is a fixture of its own and not product UI: `// surface-file: <reason>`.
final _excuseFile = RegExp(r'^//\s*surface-file:\s*\S.{7,}', multiLine: true);
bool excusedFile(String content) {
  final head = content.length > 400 ? content.substring(0, 400) : content;
  return _excuseFile.hasMatch(head);
}
