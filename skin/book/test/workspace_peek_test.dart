// The window peeks at the inside as Chromium's compositor does: one look at its counters each picture (BeginFrame),
// and only what changed is read again (damage), however the change came about: a hand on the stage, play, a call.
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart' show Ctx, NumField;

import 'door.dart';

/// What the inspector shows of a value now (the number its field is given).
double? _shown(WidgetTester t, String id) => Ctx.read(t.element(find.byWidgetPredicate((w) => w is NumField && w.id == id))).doc.get(id);

void main() {
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
