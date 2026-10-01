// Design Mode's text half (lib/dev/design_core.dart, shared with tool/ui_inspect.dart and the in-window HUD): which named dimension a
// value in the source is, what a widget's own arguments are made of, the one-line rewrites, and the edit session (undo, BEFORE/CURRENT,
// reset, keep, discard). The clicking and the layout numbers are Flutter's own Inspector and are not tested here.
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_ui/dev/design_core.dart';

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
  static TextStyle name([Color c = N.g91]) => _t('Inter', nameSize, c, FontWeight.w500, .05);
  static const leading = 1.1, leadingText = leading;
}
''';

  test('the tokens of metrics.dart are read by class and name, with their value at 100 %', () {
    final t = parseTokens(metrics);
    expect(t['Surface.workRow']!.value, 20);
    expect(t['Surface.hit']!.value, 24);
    expect(t['Surface.hitFloor']!.value, 18);
    expect(t['Surface.hitFloor']!.isConst, isTrue);
    expect(t['Surface.workRow']!.isConst, isFalse);
    expect(t['Dn.nameSize']!.value, 11);
    expect(t['Dn.leading']!.value, 1.1);
    expect(t.containsKey('Dn.leadingText'), isFalse, reason: 'an alias is not a number');
  });

  test('a text style token points at the size it is built from', () {
    final t = parseTokens(metrics);
    expect(t['Dn.name']!.sizeToken, 'Dn.nameSize');
    expect(t['Dn.name']!.value, 11);
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
      'Dn.name',
    ]) {
      expect(t.containsKey(n), isTrue, reason: n);
    }
  });

  test('a token is rewritten in its declaration only', () {
    final t = parseTokens(metrics);
    final out = rewriteToken(metrics, t['Surface.workRow']!, 19)!;
    expect(out, contains('static double get workRow => px(19);'));
    expect(out.replaceFirst('px(19)', 'px(20)'), metrics);
    expect(rewriteToken(metrics, t['Dn.nameSize']!, 11.5)!, contains('Surface.px(11.5)'));
    expect(rewriteToken(metrics, t['Surface.hitFloor']!, 20)!, contains('static const hitFloor = 20;'));
    expect(rewriteToken(metrics, t['Dn.leading']!, 1.2)!, contains('leading = 1.2, leadingText = leading'));
  });

  test('a literal is rewritten as a whole number on its own line', () {
    const src = 'a(13, 113, x13, 13.5);\nb(13);\n';
    expect(rewriteLiteral(src, 1, '13', '12'), 'a(12, 113, x13, 13.5);\nb(13);\n');
    expect(rewriteLiteral(src, 2, '13', '12'), 'a(13, 113, x13, 13.5);\nb(12);\n');
    expect(rewriteLiteral(src, 1, '14', '12'), isNull);
    expect(rewriteLiteral(src, 9, '13', '12'), isNull);
  });

  test("a widget's own arguments: tokens, registered exceptions, raw numbers, each with its place; not its child; brackets in comments and strings do not count", () {
    const src = '''
Widget build() => Container(
  // a number is never cut mid-digit ("10(" for 100)
  padding: EdgeInsets.only(left: Surface.panelInset, right: Surface.px(7), top: 3),
  height: Surface.workRow, key: ValueKey('a(b'),
  child: Padding(padding: EdgeInsets.all(99), child: Text('x', style: sans(Dn.nameSize))),
);
''';
    final refs = refsAt(src, 1, 19); // the `Container(` on line 1: "Widget build() => " is 18 characters
    final byText = {for (final r in refs) r.text: r};
    expect(byText.keys, containsAll(['Surface.panelInset', 'Surface.px(7)', '3', 'Surface.workRow']));
    expect(byText.containsKey('Dn.nameSize'), isFalse, reason: "that is the child's");
    expect(byText['Surface.panelInset']!.kind, RefKind.token);
    expect(byText['Surface.panelInset']!.arg, 'padding');
    expect(byText['Surface.px(7)']!.kind, RefKind.exception);
    expect(byText['3']!.kind, RefKind.raw);
    for (final r in [byText['Surface.px(7)']!, byText['3']!]) {
      expect(src.substring(r.start, r.end), fmt(r.value!), reason: 'its place is the number itself');
    }
  });

  test("project paths: the app's own files are lib/…, the framework is not", () {
    expect(projectPath('file:///private/tmp/wt/tok/motolii/ui/lib/inspector/value_controls.dart'), 'lib/inspector/value_controls.dart');
    expect(projectPath('file:///Users/x/flutter/packages/flutter/lib/src/widgets/framework.dart'), isNull);
    expect(projectPath(null), isNull);
  });

  group('a design session', () {
    late Map<String, String> disk;
    late DesignSession s;
    setUp(() {
      disk = {'a.dart': 'one(4);\ntwo(8);\n', 'b.dart': 'x(1);\n'};
      s = DesignSession((f) => disk[f]!, (f, t) => disk[f] = t);
    });

    test('edits, undo and redo write the files and say which changed', () {
      expect(s.apply('a.dart', 'one(5);\ntwo(8);\n', 'one'), ['a.dart']);
      expect(s.apply('a.dart', 'one(6);\ntwo(8);\n', 'one'), ['a.dart']);
      expect(s.apply('a.dart', 'one(6);\ntwo(8);\n', 'same'), isEmpty);
      expect(s.undo(), ['a.dart']);
      expect(disk['a.dart'], 'one(5);\ntwo(8);\n');
      expect(s.redo(), ['a.dart']);
      expect(disk['a.dart'], 'one(6);\ntwo(8);\n');
      s.apply('a.dart', 'one(7);\ntwo(8);\n', 'one');
      expect(s.canRedo, isFalse, reason: 'a new edit ends the redo line');
    });

    test('BEFORE and CURRENT swap the whole session at once, and nothing is edited while BEFORE shows', () {
      s.apply('a.dart', 'one(5);\ntwo(8);\n', 'one');
      s.apply('b.dart', 'x(2);\n', 'x');
      expect(s.toggleBefore(), containsAll(['a.dart', 'b.dart']));
      expect(disk, {'a.dart': 'one(4);\ntwo(8);\n', 'b.dart': 'x(1);\n'});
      expect(s.apply('a.dart', 'one(9);\ntwo(8);\n', 'blocked'), isEmpty);
      expect(s.canUndo, isFalse);
      s.toggleBefore();
      expect(disk, {'a.dart': 'one(5);\ntwo(8);\n', 'b.dart': 'x(2);\n'});
    });

    test('one line goes back to the session start, and that is undoable', () {
      s.apply('a.dart', 'one(5);\ntwo(9);\n', 'both');
      expect(s.resetLine('a.dart', 1), ['a.dart']);
      expect(disk['a.dart'], 'one(4);\ntwo(9);\n');
      expect(s.resetLine('a.dart', 1), isEmpty, reason: 'already as it was');
      s.undo();
      expect(disk['a.dart'], 'one(5);\ntwo(9);\n');
    });

    test('the changes are the lines that differ; discard puts the start back; keep makes now the baseline', () {
      s.apply('a.dart', 'one(5);\ntwo(9);\n', 'both');
      expect(s.changes(), [('a.dart', 1, 'one(4);', 'one(5);'), ('a.dart', 2, 'two(8);', 'two(9);')]);
      expect(s.changed, isTrue);
      expect(s.discard(), ['a.dart']);
      expect(disk['a.dart'], 'one(4);\ntwo(8);\n');
      expect(s.changed, isFalse);
      s.apply('b.dart', 'x(3);\n', 'x');
      s.keep();
      expect(s.changed, isFalse);
      expect(disk['b.dart'], 'x(3);\n');
      expect(s.history, isEmpty);
    });

    test('a drag is one undo: its edits are coalesced into one', () {
      s.apply('a.dart', 'one(5);\ntwo(8);\n', 'W.width 4 → 5');
      s.apply('a.dart', 'one(6);\ntwo(8);\n', 'W.width 5 → 6');
      s.apply('a.dart', 'one(7);\ntwo(8);\n', 'W.width 6 → 7');
      s.coalesce(0);
      expect(s.history.length, 1);
      expect(s.history.single.label, 'W.width 4 → 7');
      s.undo();
      expect(disk['a.dart'], 'one(4);\ntwo(8);\n');
    });

    test('an added line shifts the rest but is the only change listed', () {
      s.apply('a.dart', 'one(4);\nflex: 2,\ntwo(8);\n', 'insert');
      expect(s.changes(), [('a.dart', 2, '', 'flex: 2,')]);
    });
  });
}
