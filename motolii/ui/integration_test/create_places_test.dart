// The real app (native host included) driven like a person: find controls by their accessible names, act, and check
// what the work became. No coordinates, no drawn sizes.
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'package:motolii_stage5/live_hf/main.dart' as app;

/// A few frames: the app never settles (the Stage keeps its clock), so wait by time, not by pumpAndSettle.
Future<void> frames(WidgetTester t, [int n = 20]) async {
  for (var i = 0; i < n; i++) {
    await t.pump(const Duration(milliseconds: 50));
  }
}

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('a Create tile places its thing in the work', (t) async {
    final semantics = t.ensureSemantics();
    app.main();
    await frames(t, 60);
    final before = find.text('Rectangle').evaluate().length;
    await t.tap(find.bySemanticsLabel('Rectangle').first);
    await frames(t, 40);
    expect(find.text('Rectangle').evaluate().length, greaterThan(before), reason: 'a Rectangle row joined the Timeline');
    semantics.dispose();
  });
}
