// The window peeks at the inside as Chromium's compositor does: one look at its counters each picture (BeginFrame),
// and only what changed is read again (damage), however the change came about: a hand on the stage, play, a call.
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart' show Ctx, NumField;

import 'door.dart';

/// What the inspector shows of a value now (the number its field is given).
double? _shown(WidgetTester t, String id) => Ctx.read(t.element(find.byWidgetPredicate((w) => w is NumField && w.id == id))).doc.get(id);

/// Where the Transform card's drawing has the thing's middle now.
Offset _gizmo(WidgetTester t) => (t.widgetList<CustomPaint>(find.byType(CustomPaint)).firstWhere((p) => p.painter.runtimeType.toString() == '_CanvasPaint').painter as dynamic).g.pivot as Offset;

void main() {
  testWidgets('a thing carried on the stage: the inspector\'s numbers follow the hand while it is still down', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    ws..reread()..select('$cube');
    await t.pumpAndSettle();
    final was = _shown(t, 'v_x'), drawn = _gizmo(t);
    final stage = t.getRect(find.byKey(const ValueKey('ws-stage-comp')));
    final g = await t.startGesture(stage.center);
    for (var i = 1; i <= 10; ++i) {
      await g.moveBy(Offset(stage.width * .02, 0));
      await t.pump(const Duration(milliseconds: 16));
    }
    await settled(t, ws);   // (the hand still down: what it told the inside done, in real time)
    final carried = _shown(t, 'v_x'), went = _gizmo(t);
    await g.up();
    await t.pumpAndSettle();
    expect(carried, isNot(was), reason: 'X moved while the hand was down ($was -> $carried)');
    expect(went.dx, greaterThan(drawn.dx + 2), reason: 'the inspector\'s drawing moved right with it ($drawn -> $went)');
    expect(drawn.dx, closeTo(t.getRect(find.byWidgetPredicate((w) => w is CustomPaint && w.painter.runtimeType.toString() == '_CanvasPaint')).width / 2, 1), reason: 'drawn from the middle when opened');
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('a change the window did not make (a call straight to the inside) shows by itself', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    ws..reread()..select('$cube');
    await t.pumpAndSettle();
    final was = _shown(t, 'v_x');
    m.put(cube, 'x', 0, .5);
    for (var i = 0; i < 5; ++i) {
      t.binding.scheduleFrame(); // (as the stage's texture does when the inside draws: a test has no picture of the stage)
      await t.pump(const Duration(milliseconds: 16));
    }
    expect(_shown(t, 'v_x'), isNot(was), reason: 'X followed the inside without being told');
    await closeWorkspace(t);
  }, skip: !insideBuilt);
}
