// The window's number fields as Ableton Live 12's (its manual, 42.4 and 42.6): a digit typed on a field starts its
// number, Delete puts it back to its default. Driven on the real inside, as a hand would.
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart' show NumField;

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
}
