import 'package:analyzer/dart/ast/ast.dart';
import 'package:analyzer/dart/ast/token.dart';

/// What a dimension is: the role the argument it sits in gives it. The lint reports by it (typography, spacing, radius, geometry)
/// and the codemod (`check.dart --fix`) picks the canonical Surface token by it.
enum DimensionKind { font, radius, space, extent, thickness, other }

class DimensionUse {
  const DimensionUse(this.kind, {this.name = '', this.vertical});

  /// Not in a measured position.
  static const none = DimensionUse(DimensionKind.other);

  final DimensionKind kind;

  /// The argument or constructor part that measures it (`height`, `horizontal`, `fontSize`, `circular`).
  final String name;

  /// For a space: true on the vertical axis, false on the horizontal one, null when it is both or unknown.
  final bool? vertical;

  bool get measured => kind != DimensionKind.other;

  /// The area of the CLI report.
  String get area => switch (kind) {
        DimensionKind.font => 'typography',
        DimensionKind.space => 'spacing',
        DimensionKind.radius => 'radius',
        _ => 'geometry',
      };
}

/// Argument names whose numeric value is a layout measurement, by the kind of measure.
const _fontNames = {'fontSize'};
const _radiusNames = {'radius', 'blurRadius', 'spreadRadius'};
const _thicknessNames = {'thickness', 'strokeWidth'};
const _extentNames = {
  'width', 'height', 'minWidth', 'maxWidth', 'minHeight', 'maxHeight', 'size', 'iconSize', 'dimension', 'toolbarHeight', 'leadingWidth', 'itemExtent',
};
const _verticalNames = {'vertical', 'top', 'bottom'};
const _horizontalNames = {'horizontal', 'left', 'right', 'start', 'end'};
const _spaceNames = {..._verticalNames, ..._horizontalNames, 'spacing', 'runSpacing', 'mainAxisSpacing', 'crossAxisSpacing', 'indent', 'endIndent'};

/// All the names a named argument may measure by.
Set<String> get measuredArgumentNames => {..._fontNames, ..._radiusNames, ..._thicknessNames, ..._extentNames, ..._spaceNames};

/// Constructors and helpers whose positional numbers are measurements, by kind (a font size first for the text builders).
const _positional = {
  'EdgeInsets': DimensionKind.space,
  'EdgeInsetsDirectional': DimensionKind.space,
  'BorderRadius': DimensionKind.radius,
  'Radius': DimensionKind.radius,
  'Size': DimensionKind.extent,
  'sans': DimensionKind.font,
  'mono': DimensionKind.font,
  'caps': DimensionKind.font,
};

/// The type or helper a call names: `EdgeInsets` of `EdgeInsets.all(…)`, `sans` of `sans(11)`, `sans` of `H.s(11)` and `H.m(11)`.
String? callName(AstNode? call) {
  if (call is InstanceCreationExpression) return call.constructorName.type.name.lexeme;
  if (call is MethodInvocation) {
    final target = call.target;
    if (target is SimpleIdentifier && target.name == 'H' && const {'s', 'm'}.contains(call.methodName.name)) return 'sans';
    if (target is SimpleIdentifier) return target.name;
    if (target == null) return call.methodName.name;
  }
  return null;
}

/// The top of the chain [node] belongs to for position: branches of a conditional, parentheses and `+`/`-` operands inherit it;
/// `*` and `/` operands are ratios, not pixels.
AstNode positionTop(AstNode node) {
  var parent = node.parent;
  while (parent is ParenthesizedExpression ||
      (parent is ConditionalExpression && parent.condition != node) ||
      (parent is BinaryExpression && (parent.operator.type == TokenType.PLUS || parent.operator.type == TokenType.MINUS))) {
    node = parent!;
    parent = node.parent;
  }
  return node;
}

/// What [node] measures, from where it sits.
DimensionUse dimensionUse(AstNode node) {
  final top = positionTop(node);
  final parent = top.parent;
  if (parent is NamedArgument) {
    final name = parent.name.lexeme;
    final call = parent.parent?.parent;
    if (name == 'height' && callName(call) == 'TextStyle') return DimensionUse.none; // a line-height multiplier, not pixels
    if (_fontNames.contains(name)) return DimensionUse(DimensionKind.font, name: name);
    if (_radiusNames.contains(name)) return DimensionUse(DimensionKind.radius, name: name);
    if (_thicknessNames.contains(name)) return DimensionUse(DimensionKind.thickness, name: name);
    if ((name == 'width') && const {'BorderSide', 'Border', 'BoxBorder', 'Divider'}.contains(callName(call))) {
      return DimensionUse(DimensionKind.thickness, name: name);
    }
    if (_extentNames.contains(name)) return DimensionUse(DimensionKind.extent, name: name);
    if (_spaceNames.contains(name)) {
      return DimensionUse(DimensionKind.space, name: name, vertical: _verticalNames.contains(name) ? true : (_horizontalNames.contains(name) ? false : null));
    }
    return DimensionUse.none;
  }
  // a named dimension: `static const rowH = 20.0`, a getter that returns one, a parameter that defaults to one
  if (parent is VariableDeclaration && identical(parent.initializer, top)) {
    final list = parent.parent;
    final named = list is VariableDeclarationList &&
        (list.isConst ||
            (list.isFinal && (list.parent is FieldDeclaration || list.parent is TopLevelVariableDeclaration || _dimensionNames.contains(parent.name.lexeme))));
    if (list is VariableDeclarationList && named && _lengthLiteral(list.type, node)) {
      return DimensionUse(DimensionKind.extent, name: parent.name.lexeme);
    }
    return DimensionUse.none;
  }
  if (parent is ExpressionFunctionBody) {
    final method = parent.parent;
    if (method is MethodDeclaration && method.isGetter && _lengthLiteral(method.returnType, node)) {
      return DimensionUse(DimensionKind.extent, name: method.name.lexeme);
    }
    return DimensionUse.none;
  }
  if (parent is FormalParameterDefaultClause) {
    final param = parent.parent;
    final name = param is FormalParameter ? param.name?.lexeme : null;
    if (name != null && (measuredArgumentNames.contains(name) || _dimensionParameters.contains(name)) && _lengthLiteral(param is FormalParameter ? param.type : null, node)) {
      return DimensionUse(DimensionKind.extent, name: name);
    }
    return DimensionUse.none;
  }
  if (parent is ArgumentList) {
    final call = parent.parent;
    final type = callName(call);
    final kind = _positional[type];
    if (kind == null) return DimensionUse.none;
    final index = parent.arguments.indexOf(top as Expression);
    final ctor = call is InstanceCreationExpression ? (call.constructorName.name?.name ?? '') : '';
    switch (kind) {
      case DimensionKind.space:
        bool? vertical;
        if (ctor == 'fromLTRB' || ctor == 'fromSTEB') vertical = index.isOdd ? true : false;
        return DimensionUse(kind, name: ctor, vertical: vertical);
      case DimensionKind.extent:
        return DimensionUse(kind, name: ctor == 'fromHeight' || (ctor == '' && index == 1) ? 'height' : 'width');
      default:
        return DimensionUse(kind, name: ctor);
    }
  }
  return DimensionUse.none;
}

/// Local names that hold a length (a local `final` is only a dimension by its name; a constant or a field always is).
const _dimensionNames = {'pad', 'padX', 'padY', 'gap', 'margin', 'inset', 'edge', 'cell'};

/// Parameters that default to a length without a measured name.
const _dimensionParameters = {'h', 'w', 'pad', 'labelWidth', 'minSize', 'cursorWidth'};

/// Whether [node] is a length by its declared type (`double`/`num`) or by being written as a double.
bool _lengthLiteral(TypeAnnotation? type, AstNode node) {
  final declared = type is NamedType ? type.name.lexeme : null;
  if (declared == 'double' || declared == 'num') return true;
  final literal = node is PrefixExpression ? node.operand : node;
  return declared == null && literal is DoubleLiteral;
}

/// A literal in a painter or a canvas function: its own pixels, drawn, not laid out (not migrated; `// surface-block:` names it).
/// The class (or function) that holds it, else null.
Declaration? painterScope(AstNode node) {
  for (AstNode? n = node.parent; n != null; n = n.parent) {
    if (n is ClassDeclaration) {
      final extendsName = n.extendsClause?.superclass.name.lexeme ?? '';
      if (extendsName == 'CustomPainter' || extendsName.endsWith('Painter')) return n;
    }
    if (n is FunctionDeclaration) {
      if (_hasCanvas(n.functionExpression.parameters)) return n;
    }
    if (n is MethodDeclaration) {
      if (_hasCanvas(n.parameters)) return n;
    }
  }
  return null;
}

bool _hasCanvas(FormalParameterList? parameters) {
  if (parameters == null) return false;
  for (final p in parameters.parameters) {
    final type = p.type;
    if (type is NamedType && type.name.lexeme == 'Canvas') return true;
  }
  return false;
}
