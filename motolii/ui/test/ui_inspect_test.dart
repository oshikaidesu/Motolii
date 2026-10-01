// The token intelligence of tool/ui_inspect.dart: which named dimension a value is, what a widget's own arguments are made of, and
// the source rewrite that `set` and `edit` do (the hot reload then shows it). The picking and the layout numbers are Flutter's own
// Inspector and are not tested here.
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../tool/ui_inspect.dart';

void main() {
  const metrics = '''
abstract final class Surface {
  /// doc
  static double get workRow => px(20);
  static double get hit => UiScale.derive(24, ScalePolicy.minimum, floor: hitFloor);
  static const hitFloor = 18.0;
  static double get hair => UiScale.derive(1, ScalePolicy.snap);
}
abstract final class Dn {
  static double get nameSize => Surface.px(11);
  static const leading = 1.1, leadingText = leading;
}
''';

  test('the tokens of metrics.dart are read by class and name, with their value at 100 %', () {
    final t = parseTokens(metrics);
    expect(t['Surface.workRow']!.value, 20);
    expect(t['Surface.hit']!.value, 24);
    expect(t['Surface.hitFloor']!.value, 18);
    expect(t['Dn.nameSize']!.value, 11);
    expect(t['Dn.leading']!.value, 1.1);
    expect(
      t.containsKey('Dn.leadingText'),
      isFalse,
      reason: 'an alias is not a number',
    );
  });

  test('the real canon parses: the tokens the Inspector names are there', () {
    final t = parseTokens(File('lib/theme/metrics.dart').readAsStringSync());
    for (final n in [
      'Surface.workRow',
      'Surface.chromeRow',
      'Surface.control',
      'Surface.inlineGap',
      'Surface.sectionGap',
      'Dn.nameSize',
    ]) {
      expect(t.containsKey(n), isTrue, reason: n);
    }
  });

  test('a token is rewritten in its declaration only', () {
    final t = parseTokens(metrics);
    final out = rewriteToken(metrics, t['Surface.workRow']!, 19)!;
    expect(out, contains('static double get workRow => px(19);'));
    expect(out.replaceFirst('px(19)', 'px(20)'), metrics);
    expect(
      rewriteToken(metrics, t['Dn.nameSize']!, 11.5)!,
      contains('Surface.px(11.5)'),
    );
    expect(
      rewriteToken(metrics, t['Surface.hitFloor']!, 20)!,
      contains('static const hitFloor = 20;'),
    );
    expect(
      rewriteToken(metrics, t['Dn.leading']!, 1.2)!,
      contains('leading = 1.2, leadingText = leading'),
    );
  });

  test('a literal is rewritten as a whole number on its own line', () {
    const src = 'a(13, 113, x13, 13.5);\nb(13);\n';
    expect(
      rewriteLiteral(src, 1, '13', '12'),
      'a(12, 113, x13, 13.5);\nb(13);\n',
    );
    expect(
      rewriteLiteral(src, 2, '13', '12'),
      'a(13, 113, x13, 13.5);\nb(12);\n',
    );
    expect(rewriteLiteral(src, 1, '14', '12'), isNull);
    expect(rewriteLiteral(src, 9, '13', '12'), isNull);
  });

  test('a widget\'s own arguments: tokens, registered exceptions, raw numbers; not its child; brackets in comments and strings do not count', () {
    const src = '''
Widget build() => Container(
  // a number is never cut mid-digit ("10(" for 100)
  padding: EdgeInsets.only(left: Surface.panelInset, right: Surface.px(7), top: 3),
  height: Surface.workRow, key: ValueKey('a(b'),
  child: Padding(padding: EdgeInsets.all(99), child: Text('x', style: sans(Dn.nameSize))),
);
''';
    final p = provenanceAt(
      src,
      1,
      24,
    ); // the `Container(` on line 1: "Widget build() => " is 18 characters
    expect(p.tokens, containsAll(['Surface.panelInset', 'Surface.workRow']));
    expect(
      p.tokens,
      isNot(contains('Dn.nameSize')),
      reason: 'that is the child\'s',
    );
    expect(p.px, ['Surface.px(7)']);
    expect(p.literals, ['3']);
  });

  test(
    'project paths: the app\'s own files are lib/…, the framework is not',
    () {
      expect(
        projectPath(
          'file:///private/tmp/wt/tok/motolii/ui/lib/inspector/value_controls.dart',
        ),
        'lib/inspector/value_controls.dart',
      );
      expect(
        projectPath(
          'file:///Users/x/flutter/packages/flutter/lib/src/widgets/framework.dart',
        ),
        isNull,
      );
      expect(projectPath(null), isNull);
    },
  );
}
