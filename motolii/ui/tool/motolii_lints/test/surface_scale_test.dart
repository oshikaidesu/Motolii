import 'dart:io';

import 'package:analyzer/dart/analysis/utilities.dart';
import 'package:analyzer/dart/ast/ast.dart';
import 'package:analyzer/dart/ast/visitor.dart';
import 'package:motolii_lints/src/raw_dimension.dart';
import 'package:motolii_lints/src/surface_scale.dart';
import 'package:test/test.dart';

int scaling(String body) => scalingViolations(parseString(content: 'void f() { $body }').unit).length;

class _Px extends RecursiveAstVisitor<void> {
  int n = 0;
  @override
  void visitMethodInvocation(MethodInvocation node) {
    if (isPxException(node)) n++;
    super.visitMethodInvocation(node);
  }
}

void main() {
  test('Transform.scale, ScaleTransition, a text scaler and a scaled viewport are violations', () {
    expect(scaling('Transform.scale(scale: 2, child: x);'), 1);
    expect(scaling('ScaleTransition(scale: a, child: x);'), 1);
    expect(scaling('MediaQuery(data: d.copyWith(textScaler: TextScaler.linear(2)), child: x);'), 2);
    expect(scaling('EditorScaledViewport(scale: 2, child: x);'), 1);
  });

  test('a plain widget tree is clean', () {
    expect(scaling('Transform.translate(offset: o, child: Container());'), 0);
    expect(scaling('Transform.rotate(angle: 1, child: x);'), 0);
  });

  test('Surface.px is the registered exception, and only it', () {
    final v = _Px();
    parseString(content: 'void f() { Surface.px(3); Surface.hair; px(3); Other.px(2); }').unit.accept(v);
    expect(v.n, 1);
  });

  test('a scale excuse needs its reason', () {
    const a = 'x(); // surface: a pulse, not the UI Scale\nTransform.scale();';
    const b = 'x(); // surface:\nTransform.scale();';
    expect(excused(a, a.indexOf('Transform')), isTrue);
    expect(excused(b, b.indexOf('Transform')), isFalse);
  });

  test('a surface-block above a declaration excuses what is inside it', () {
    const src = '''
// surface-block: painter geometry: drawn in the canvas's own pixels
class P { void paint() { box(width: 5); } }
class Q { void f() { box(width: 5); } }
''';
    final unit = parseString(content: src).unit;
    final hits = <AstNode>[];
    unit.accept(_W(hits));
    expect(hits, hasLength(2));
    expect(excusedNode(src, hits[0]), isTrue);
    expect(excusedNode(src, hits[1]), isFalse);
  });

  test('metrics.dart defines every dimension token the coverage figure counts', () {
    final text = File('../../lib/theme/metrics.dart').readAsStringSync();
    for (final e in dimensionTokens.entries) {
      for (final name in e.value) {
        expect(RegExp('static double get $name\\b').hasMatch(text), isTrue, reason: '${e.key}.$name');
      }
    }
  });
}

class _W extends RecursiveAstVisitor<void> {
  _W(this.hits);
  final List<AstNode> hits;
  @override
  void visitIntegerLiteral(IntegerLiteral node) => hits.add(node);
}
