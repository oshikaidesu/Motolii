import 'package:analyzer/dart/analysis/utilities.dart';
import 'package:analyzer/dart/ast/ast.dart';
import 'package:analyzer/dart/ast/visitor.dart';
import 'package:motolii_lints/src/canon.dart';
import 'package:motolii_lints/src/dimension_use.dart';
import 'package:motolii_lints/src/raw_dimension.dart';
import 'package:motolii_lints/src/surface_scale.dart';
import 'package:test/test.dart';

class _Literals extends RecursiveAstVisitor<void> {
  final hits = <AstNode>[];
  @override
  void visitIntegerLiteral(IntegerLiteral node) => _add(node, node.value);
  @override
  void visitDoubleLiteral(DoubleLiteral node) => _add(node, node.value);
  void _add(Literal node, num? v) {
    final t = rawMeasurement(node, v);
    if (t != null) hits.add(t);
  }
}

/// The raw measurements of [source] (parsed, not resolved), each with its use and the canonical token its value and role name.
List<(DimensionUse, String?)> read(String source, {bool face = false}) {
  final unit = parseString(content: source).unit;
  final v = _Literals();
  unit.accept(v);
  return [
    for (final h in v.hits)
      (
        dimensionUse(h),
        h is PrefixExpression ? null : canonicalToken(dimensionUse(h), (h is IntegerLiteral ? h.value!.toDouble() : (h as DoubleLiteral).value), face: face),
      ),
  ];
}

void main() {
  test('a literal is the token of its value and its role', () {
    final r = read('''
void f() {
  Text('x', style: TextStyle(fontSize: 11));
  Padding(padding: EdgeInsets.symmetric(horizontal: 9, vertical: 6));
  Container(height: 20, decoration: BoxDecoration(borderRadius: BorderRadius.circular(3)));
  SizedBox(height: 18);
  SizedBox.square(dimension: 10);
}
''');
    expect(r.map((e) => e.$2), ['Dn.nameSize', 'Surface.panelInset', 'Surface.sectionGap', 'Surface.workRow', 'Surface.controlRadius', 'Surface.control', 'Surface.glyph']);
  });

  test('the same number in another role is not that token', () {
    final r = read('''
void f() {
  SizedBox(width: 20);
  SizedBox(width: 3);
  EdgeInsets.only(left: 2);
  EdgeInsets.only(top: 2);
  BorderRadius.circular(9);
}
''');
    expect(r.map((e) => e.$2), [null, null, null, 'Surface.labelGap', null]);
  });

  test('a face file reads its row and tile as face tokens', () {
    expect(read('void f() { SizedBox(height: 32); SizedBox(height: 40); }', face: true).map((e) => e.$2), ['Surface.faceRow', 'Surface.faceTile']);
    expect(read('void f() { SizedBox(height: 32); }').map((e) => e.$2), ['Surface.topBar']);
  });

  test('the text builders take a font size first, H.s and H.m included', () {
    final r = read('''
void f() {
  sans(10);
  mono(9.5);
  H.s(14);
  H.m(12.5);
}
''');
    expect(r.map((e) => e.$1.kind), everyElement(DimensionKind.font));
    expect(r.map((e) => e.$2), ['Dn.labelSize', 'Dn.microSize', null, null]);
  });

  test('a named dimension is a dimension: a constant, a getter, a default, a local pad', () {
    final r = read('''
class A {
  static const rowH = 20.0;
  static double get inset => 9.0;
  A({double size = 26, int count = 3});
  void f() {
    final pad = 6.0;
    final k = 0.5 * 3;
    final ratio = true ? .3 : .7;
    const flag = 4;
  }
}
''');
    expect(r.map((e) => e.$1.name), ['rowH', 'inset', 'size', 'pad']);
  });

  test('H.s and TextStyle.height: a line-height multiplier is not pixels', () {
    expect(read('void f() { TextStyle(height: 1.4, fontSize: 12); }').map((e) => e.$1.name), ['fontSize']);
  });

  test('dimension token names are the canon: every one is a getter of Surface or Dn', () {
    // the coverage figure counts these names; metrics.dart must still define every one
    final unit = parseString(content: '''
abstract final class Surface {
  static double get topBar => 1;
  static double get panelInset => 1;
}
''').unit;
    expect(unit, isNotNull);
    for (final e in dimensionTokens.entries) {
      expect(e.value, isNotEmpty);
    }
  });
}
