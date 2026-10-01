// Design Mode's keys, through the real key path: F2, undo / redo, and the keep / discard question at exit (Enter, Esc), against files in a
// temporary source root (no dev session is signalled: the state directory is empty).
import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_ui/dev/design_mode.dart';

void main() {
  late Directory dir;
  late DesignController c;

  setUp(() {
    dir = Directory.systemTemp.createTempSync('design_mode_keys');
    Directory('${dir.path}/lib/theme').createSync(recursive: true);
    for (final f in ['neutral', 'identity', 'metrics']) {
      File('${dir.path}/lib/theme/$f.dart').writeAsStringSync('abstract final class T {}\n');
    }
    Directory('${dir.path}/state').createSync();
    File('${dir.path}/lib/a.dart').writeAsStringSync('one(4);\n');
    c = DesignController(root: dir.path, state: Directory('${dir.path}/state'));
  });
  tearDown(() => dir.deleteSync(recursive: true));

  Future<void> pump(WidgetTester t) => t.pumpWidget(
    DesignMode(
      controller: c,
      child: Focus(autofocus: true, child: const SizedBox()),
    ),
  );
  String file() => File('${dir.path}/lib/a.dart').readAsStringSync();

  testWidgets('F2 turns it on and off; z undoes, shift+z redoes', (t) async {
    await pump(t);
    expect(c.on, isFalse);
    await t.sendKeyEvent(LogicalKeyboardKey.f2);
    expect(c.on, isTrue);
    c.session.apply('lib/a.dart', 'one(5);\n', 'x');
    await t.sendKeyEvent(LogicalKeyboardKey.keyZ);
    expect(file(), 'one(4);\n');
    await t.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
    await t.sendKeyEvent(LogicalKeyboardKey.keyZ);
    await t.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
    expect(file(), 'one(5);\n');
    c.session.discard();
    await t.sendKeyEvent(LogicalKeyboardKey.f2);
    expect(c.on, isFalse);
  });

  testWidgets('with changes, F2 asks; Esc discards', (t) async {
    await pump(t);
    await t.sendKeyEvent(LogicalKeyboardKey.f2);
    c.session.apply('lib/a.dart', 'one(9);\n', 'x');
    await t.sendKeyEvent(LogicalKeyboardKey.f2);
    expect(c.exiting, isTrue);
    expect(c.on, isTrue);
    await t.sendKeyEvent(LogicalKeyboardKey.escape);
    expect(c.on, isFalse);
    expect(file(), 'one(4);\n');
  });

  testWidgets('with changes, F2 asks; Enter keeps; a second F2 in the question goes back to work', (t) async {
    await pump(t);
    await t.sendKeyEvent(LogicalKeyboardKey.f2);
    c.session.apply('lib/a.dart', 'one(9);\n', 'x');
    await t.sendKeyEvent(LogicalKeyboardKey.f2);
    await t.sendKeyEvent(LogicalKeyboardKey.f2);
    expect(c.exiting, isFalse);
    expect(c.on, isTrue);
    await t.sendKeyEvent(LogicalKeyboardKey.f2);
    await t.sendKeyEvent(LogicalKeyboardKey.enter);
    expect(c.on, isFalse);
    expect(file(), 'one(9);\n');
  });
}
