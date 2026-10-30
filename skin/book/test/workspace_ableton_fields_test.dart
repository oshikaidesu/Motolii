// The window's number fields as Ableton Live 12's (its manual, 42.4 and 42.6): a digit typed on a field starts its
// number, Delete puts it back to its default. Driven on the real inside, as a hand would.
import 'package:flutter/gestures.dart' show kSecondaryButton;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart' show Ctx, NumField;

import 'door.dart';

void main() {
  testWidgets('a click picks a field and no more, a digit typed on it starts the number, Delete puts it back to its default', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final a = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$a');
    await t.pumpAndSettle();
    final x = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_x');
    await seen(t, x);
    await t.tap(x);   // (the field holds the keys)
    await t.pumpAndSettle();
    await t.sendKeyEvent(LogicalKeyboardKey.digit4);
    await t.pump();
    final typing = find.descendant(of: x, matching: find.byType(EditableText));
    expect(typing, findsOneWidget, reason: 'a digit starts the number');
    expect(t.widget<EditableText>(typing).controller.text, '4', reason: 'begun with what was typed, not the old number');
    await t.sendKeyEvent(LogicalKeyboardKey.escape);
    await t.pumpAndSettle();
    expect(m.set(a, 'x', 100), isTrue, reason: m.trouble);
    ws.reread();
    await t.pumpAndSettle();
    final was = m.value(a, 'x', ws.frame / 60);
    await t.tap(x);
    await t.pumpAndSettle();
    expect(find.descendant(of: x, matching: find.byType(EditableText)), findsNothing, reason: 'a click picks the field, no more (Live)');
    await t.sendKeyEvent(LogicalKeyboardKey.delete);
    await t.pumpAndSettle();
    expect(m.value(a, 'x', ws.frame / 60), isNot(was), reason: 'Delete: back to its default');
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('a sum typed in a field is worked out, one begun with * or / from the value as it is; what is no sum keeps the field open (After Effects, Figma, Blender)', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final a = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$a');
    await t.pumpAndSettle();
    expect(m.workOut('1/3'), closeTo(1 / 3, 1e-12), reason: 'not whole numbers\' division');
    expect(m.workOut('(1+2)*3 - 1e1'), closeTo(-1, 1e-12));
    expect(m.workOut('select 1'), isNull);
    expect(m.workOut('1+'), isNull);
    expect(m.workOut('1/0'), isNull, reason: 'SQLite: no number');
    final x = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_x');
    final typing = find.descendant(of: x, matching: find.byType(EditableText));
    Future<void> type(String said) async {
      await seen(t, x);
      await t.tap(x);
      await t.pumpAndSettle();
      await t.sendKeyEvent(LogicalKeyboardKey.enter);
      await t.pumpAndSettle();
      t.widget<EditableText>(typing).controller.text = said;
      await t.sendKeyEvent(LogicalKeyboardKey.enter);
      await settled(t, ws);
    }
    await type('(100+20)/3');
    double? shown() => Ctx.read(t.element(x)).doc.get('v_x');
    expect(shown(), closeTo(40, .06));
    await type('*2');
    expect(shown(), closeTo(80, .1), reason: 'from the value as it is');
    await type('1+');
    expect(typing, findsOneWidget, reason: 'no sum: the field stays open');
    await t.sendKeyEvent(LogicalKeyboardKey.escape);
    await t.pumpAndSettle();
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, tags: 'main', skip: !insideBuilt);

  testWidgets('a right click on any number offers Reset, Copy, Paste and its keys, on a row or an effect\'s knob alike (After Effects, Blender; all by the mouse alone)', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final a = m.add('shape', 'rect');
    expect(m.effect(a, 'Gaussian Blur', 0), isTrue, reason: m.trouble);
    expect(m.set(a, 'x', 100), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$a');
    await t.pumpAndSettle();
    final x = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_x');
    await seen(t, x);
    Future<void> menu(Finder f) async {
      await t.tap(f, buttons: kSecondaryButton);
      await t.pumpAndSettle();
    }
    await menu(x);
    expect(find.byKey(const ValueKey('num-key:v_x')), findsOneWidget, reason: 'its key here');
    await t.tap(find.byKey(const ValueKey('num-key:v_x')));
    await settled(t, ws);
    expect(m.keyMoments(a, 'x'), hasLength(1), reason: m.trouble);
    await menu(x);
    await t.tap(find.byKey(const ValueKey('num-unanimate:v_x')));
    await settled(t, ws);
    expect(m.keyMoments(a, 'x'), isEmpty);
    await menu(x);
    await t.tap(find.byKey(const ValueKey('num-reset:v_x')));
    await settled(t, ws);
    expect(m.value(a, 'x', 0), isNot(100), reason: 'one value back to what it was, by the mouse alone');
    final blur = find.byWidgetPredicate((w) => w is NumField && w.id.endsWith('_blur'));
    await seen(t, blur);
    await menu(blur.first);
    expect(find.text('Reset'), findsWidgets, reason: 'an effect\'s knob has the same menu: it had none');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, tags: 'main', skip: !insideBuilt);

  testWidgets('typing a number: Tab keeps it and opens the next field, Shift-Tab the one before; the arrows move what is typed; nothing typed is as Esc (After Effects, Figma, Blender)', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final a = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$a');
    await t.pumpAndSettle();
    Finder field(String id) => find.byWidgetPredicate((w) => w is NumField && w.id == id);
    Finder typing(String id) => find.descendant(of: field(id), matching: find.byType(EditableText));
    await seen(t, field('v_x'));
    await t.tap(field('v_x'));
    await t.pumpAndSettle();
    await t.sendKeyEvent(LogicalKeyboardKey.digit4);
    await t.sendKeyEvent(LogicalKeyboardKey.digit2);
    await t.pump();
    await t.sendKeyEvent(LogicalKeyboardKey.tab);
    await t.pumpAndSettle();
    await settled(t, ws);
    expect(typing('v_x'), findsNothing, reason: 'kept');
    expect(typing('v_y'), findsOneWidget, reason: 'the next field open to be typed in');
    t.widget<EditableText>(typing('v_y')).controller.text = '7';
    await t.sendKeyEvent(LogicalKeyboardKey.arrowUp);
    await t.pump();
    expect(double.parse(t.widget<EditableText>(typing('v_y')).controller.text), closeTo(8, 1e-9), reason: 'the number typed so far moved a step');
    await t.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
    await t.sendKeyEvent(LogicalKeyboardKey.tab);
    await t.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
    await t.pumpAndSettle();
    await settled(t, ws);
    expect(typing('v_x'), findsOneWidget, reason: 'the field before it');
    t.widget<EditableText>(typing('v_x')).controller.clear();
    await t.sendKeyEvent(LogicalKeyboardKey.enter);
    await t.pumpAndSettle();
    expect(typing('v_x'), findsNothing, reason: 'nothing typed: closed, no "Number required"');
    expect(find.text('Number required'), findsNothing);
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, tags: 'main', skip: !insideBuilt);
}
