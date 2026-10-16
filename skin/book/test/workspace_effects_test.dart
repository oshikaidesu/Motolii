// The Effects shelf on a real document (test/door.dart): its filter band narrows what the inside does by what each
// effect declares, and the band's height is the user's.
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/workspace/dock_effects.dart' show EffectsShelf;
import 'package:lab_book/workspace/fx_catalog.dart';
import 'package:lab_book/workspace/ws.dart' show fxValues;

import 'door.dart';

Future<void> _tap(WidgetTester t, Finder f) async {
  await t.ensureVisible(f);
  await t.pumpAndSettle();
  await t.tap(f);
  await t.pumpAndSettle();
}

Future<Ws> _effects(WidgetTester t) async {
  final ws = await openWorkspace(t)..shelf = 'Effects';
  await t.pumpAndSettle();
  return ws;
}

void main() {
  testWidgets('the filter band narrows what the inside does by what each effect declares, and Clear brings it back', (t) async {
    await _effects(t);
    expect(find.byKey(const ValueKey('browser:filters')), findsNothing);
    await _tap(t, find.byKey(const ValueKey('browser:filters-toggle')));
    expect(find.byKey(const ValueKey('browser:filters')), findsOneWidget);
    final rows = [for (final f in fxCatalog) if (fxValues.containsKey(f.name)) f.name]; // (what the inside does)
    await _tap(t, find.byKey(const ValueKey('browser:filter:Time:Uses time'))); // (none of them uses time)
    expect(find.text(rows.first), findsNothing, reason: 'narrowed away');
    await _tap(t, find.byKey(const ValueKey('browser:filters-clear')));
    await t.scrollUntilVisible(find.text(rows.first), 60, scrollable: find.descendant(of: find.byType(EffectsShelf), matching: find.byType(Scrollable)).last); // (the whole-screen effects come first)
    expect(find.text(rows.first), findsWidgets, reason: 'Clear brings them back');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('the user sets the band height: drag the grip, double-click resets', (t) async {
    await _effects(t);
    await _tap(t, find.byKey(const ValueKey('browser:filters-toggle')));
    final band = find.byKey(const ValueKey('browser:filters'));
    final h0 = t.getSize(band).height;
    await t.drag(find.byKey(const ValueKey('browser:filters-grip')), const Offset(0, 120));
    await t.pumpAndSettle();
    expect(t.getSize(band).height, closeTo(h0 + 120, 1));
    await t.drag(find.byKey(const ValueKey('browser:filters-grip')), const Offset(0, -2000));
    await t.pumpAndSettle();
    expect(t.getSize(band).height, 72, reason: 'never smaller than the floor');
    await t.tap(find.byKey(const ValueKey('browser:filters-grip')));
    await t.pump(const Duration(milliseconds: 50));
    await t.tap(find.byKey(const ValueKey('browser:filters-grip')));
    await t.pumpAndSettle();
    expect(t.getSize(band).height, closeTo(h0, 1));
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);
}
