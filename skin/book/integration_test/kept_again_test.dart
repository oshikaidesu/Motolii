// The stage's kept pictures played, the document changed, kept and played again: what was kept is thrown away once each
// time (golf_noise.md (102): `shown` thrown away twice, then a picture drawn into a target gone, and the window fell in
// Filament's beginRenderPass). In a real window: the hand keeps pictures only while it lives there. MOTOLII_BUDGET: every
// picture counted late, so each is kept.
//   flutter drive --profile -d macos --driver=test_driver/integration_test.dart --target=integration_test/kept_again_test.dart
import 'dart:ffi' hide Size;
import 'dart:io';

import 'package:ffi/ffi.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:lab_book/theme.dart';
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/workspace.dart' show Workspace;
import 'package:lab_book/workspace/ws.dart';
import 'package:lab_book/workspace/zoom.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  testWidgets('kept, played and changed three times over, the window stands', (t) async {
    final setenv = DynamicLibrary.process().lookupFunction<Int32 Function(Pointer<Utf8>, Pointer<Utf8>, Int32), int Function(Pointer<Utf8>, Pointer<Utf8>, int)>('setenv');
    final name = 'MOTOLII_BUDGET'.toNativeUtf8(), value = '0.001'.toNativeUtf8();
    setenv(name, value, 1);
    malloc
      ..free(name)
      ..free(value);
    final folder = Directory('${Directory.systemTemp.createTempSync('kept').path}/Motolii')..createSync();
    WsZoom.set(1, keep: false);
    ColorTheme.at = '${folder.path}/none.xml';
    await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => Workspace(document: '${folder.path}/Untitled.motolii')));
    for (var i = 0; i < 100 && find.byType(WsTimeline).evaluate().isEmpty; i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 20)));
      await t.pump();
    }
    await t.pumpAndSettle();
    final ws = WsScope.read(t.element(find.byType(WsTimeline))), m = ws.inside!;
    final rect = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$rect');
    for (var round = 0; round < 3; round++) {
      expect(m.set(rect, 'x', round * .1), isTrue, reason: m.trouble);   // (the document changed: what was kept goes)
      ws.reread();
      bool enough() => m.kept.isNotEmpty && m.kept.first.$1 == 0 && m.kept.first.$2 >= 60;   // (a second kept from the start: what plays is a kept picture, not one drawn)
      for (var i = 0; i < 600 && !enough(); i++) {   // (the hand rests five seconds, then keeps)
        await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
        await t.pump();
      }
      expect(enough(), isTrue, reason: 'round $round: pictures kept: ${m.kept}');
      m.at(0);
      m.play(true);
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 600)));   // (a kept picture put on the stage)
      m.play(false);
      await t.pump();
      expect(m.set(rect, 'y', round * .1), isTrue, reason: m.trouble);   // (changed after a kept picture was shown: thrown away here)
      ws.reread();
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 400)));
      await t.pump();
      expect(m.spent, isNot(contains('round threw')), reason: 'round $round');
    }
  }, timeout: const Timeout(Duration(minutes: 3)));
}
