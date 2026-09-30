import 'package:analyzer/dart/ast/ast.dart';
import 'package:analyzer/dart/ast/visitor.dart';

/// Arbitrary visual scaling is a violation: the UI Scale is a derived value the tokens carry (lib/theme/metrics.dart), applied at build
/// time, never a transform of a subtree. `Transform.scale`, `ScaleTransition`, a `textScaler` override and a scaled-viewport widget
/// are what this finds. Its excuse is `// surface: <reason>` (an animation that scales a thing is not the UI Scale).
List<AstNode> scalingViolations(CompilationUnit unit) {
  final hits = <AstNode>[];
  unit.accept(_Scaling(hits));
  return hits;
}

class _Scaling extends RecursiveAstVisitor<void> {
  _Scaling(this.hits);
  final List<AstNode> hits;

  @override
  void visitInstanceCreationExpression(InstanceCreationExpression node) {
    final type = node.constructorName.type.name.lexeme, ctor = node.constructorName.name?.name;
    if ((type == 'Transform' && ctor == 'scale') || type == 'ScaleTransition' || type.contains('ScaledViewport') || type == 'TextScaler') {
      hits.add(node);
    }
    super.visitInstanceCreationExpression(node);
  }

  @override
  void visitMethodInvocation(MethodInvocation node) {
    final target = node.target;
    final name = node.methodName.name;
    if (target is SimpleIdentifier && ((target.name == 'Transform' && name == 'scale') || (target.name == 'TextScaler' && name == 'linear'))) {
      hits.add(node);
    }
    // an unresolved constructor call is a method invocation without a target
    if (target == null && (name == 'ScaleTransition' || name.contains('ScaledViewport'))) hits.add(node);
    super.visitMethodInvocation(node);
  }

  @override
  void visitNamedArgument(NamedArgument node) {
    if (node.name.lexeme == 'textScaler' || node.name.lexeme == 'textScaleFactor') hits.add(node);
    super.visitNamedArgument(node);
  }
}

/// A `Surface.px(...)` call: the registered exception (a SCALE-policy custom dimension no token names).
bool isPxException(MethodInvocation node) {
  final target = node.target;
  return target is SimpleIdentifier && target.name == 'Surface' && node.methodName.name == 'px';
}

/// The dimension tokens of the canon (the names lib/theme/metrics.dart gives to a derived size), for the coverage figure.
const dimensionTokens = {
  'Surface': {
    'topBar', 'namedHeader', 'chromeRow', 'workRow', 'control', 'controlHero', 'ruler', 'menuRow', 'hit', 'mark', 'glyph', 'hair',
    'focusStroke', 'labelRow', 'cellGap', 'labelGap', 'inlineGap', 'sectionGap', 'panelInset', 'controlRadius', 'faceRadius', 'faceRow', 'faceTile',
  },
  'Dn': {'nameSize', 'labelSize', 'microSize', 'numericSize'},
};

bool isDimensionToken(PrefixedIdentifier node) => dimensionTokens[node.prefix.name]?.contains(node.identifier.name) ?? false;
