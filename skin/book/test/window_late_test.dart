// The window late: the hand done with a whole step before the window has heard its first answer, as a busy frame makes
// it on a slow Mac (the window's thread held here, so it is so each time).
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'door.dart';

void main() {
  testWidgets('a thing made and picked stays picked when the window hears of it late: the end of its step comes with what the step made', (t) async {
    final copy = '${Directory.systemTemp.createTempSync('window_late').path}/try.motolii';
    File('test/data/relation_try.motolii').copySync(copy);
    final ws = await openWorkspace(t, document: copy);
    await settled(t, ws);
    final m = ws.inside!;
    var made = 0;
    m.later('together', number: 1);
    ws.after(m.later('add', name: 'behaviour', word: 'group'), null, (b) {
      made = b;
      ws.select('$b');
    });
    sleep(const Duration(milliseconds: 300));   // (its answer told, not yet heard)
    m
      ..later('set', thing: -1, name: 'strength', number: 1)
      ..later('relate', thing: -1, word: '1 2 3');
    ws.after(m.later('together'), (_, _) => ws.reread());
    sleep(const Duration(milliseconds: 300));   // (the step done, nothing of it heard)
    await settled(t, ws);
    expect(m.things, contains(made));
    expect(ws.selected, '$made');
  });
}
