// Whole-screen effects put on and taken off the stage, a few times, with Flutter's own timeline of every widget's build
// recorded around them: what the window builds again when an effect comes or goes, and how long it takes.
//   flutter drive --profile -d macos --driver=test_driver/integration_test.dart --target=integration_test/toggle_test.dart
import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:lab_book/motolii.dart' show motoliiDir;
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/workspace.dart' show Workspace;
import 'package:lab_book/workspace/ws.dart';

void main() {
  final binding = IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  testWidgets('whole-screen effects put on and off: what is built again', (t) async {
    t.binding.defaultBinaryMessenger.setMockMethodCallHandler(SystemChannels.menu, (_) async => null);
    final doc = '${Directory.systemTemp.createTempSync('toggle').path}/toggle.motolii';
    await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => Workspace(document: doc)));
    await t.pumpAndSettle();
    final ws = WsScope.read(t.element(find.byType(WsTimeline))), m = ws.inside!;
    final font = const String.fromEnvironment('FONT').isNotEmpty ? const String.fromEnvironment('FONT') : '$motoliiDir/out/arial_black.ttf';
    for (final (k, f) in [('solid', 'cube'), ('solid', 'ring'), ('text', font), ('light', 'sun')]) {
      m.add(k, f);
    }
    m.make('Wave lines');
    final stacked = m.effects.skip(2).take(const int.fromEnvironment('STACKED', defaultValue: 6)).map((e) => e.$2).join(' ');
    if (stacked.isNotEmpty) m.say(0, 'script', stacked);   // (as the rampage leaves the stage: several effects on already)
    ws
      ..reread()
      ..select(null)
      ..shelf = 'Effects';
    await t.pumpAndSettle();
    final names = m.effects.take(2).map((e) => e.$2).toList();   // (a file's effect, by its file: dock_effects.dart)
    debugProfileBuildsEnabledUserWidgets = true;
    await binding.traceAction(() async {
      for (final n in [...names, ...names]) {
        final row = find.byKey(ValueKey('ws-screen-$n'));
        await t.ensureVisible(row);
        await t.tap(row);   // (pressed as a hand presses it)
        for (var i = 0; i < 20; i++) {
          await t.pump(const Duration(milliseconds: 16));
        }
      }
    }, reportKey: 'timeline');
    debugProfileBuildsEnabledUserWidgets = false;
  });
}
