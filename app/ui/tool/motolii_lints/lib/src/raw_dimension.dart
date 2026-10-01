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

/// Constructors whose positional numbers are measurements.
const measuredTypes = {
  'EdgeInsets', 'EdgeInsetsDirectional', 'BorderRadius', 'Radius', 'Size',
};

/// Files that define the scale and may hold raw numbers.
const scaleFiles = {
  'lib/foundation/metrics.dart',
  'lib/foundation/theme.dart',
  'lib/foundation/panel_catalog.dart',
};

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
    final visitor = _Visitor(this);
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
  _Visitor(this.rule);
  final AnalysisRule rule;

  void _check(Literal node, num? value) {
    final target = rawMeasurement(node, value);
    if (target != null) rule.reportAtNode(target);
  }

  @override
  void visitIntegerLiteral(IntegerLiteral node) => _check(node, node.value);

  @override
  void visitDoubleLiteral(DoubleLiteral node) => _check(node, node.value);
}
