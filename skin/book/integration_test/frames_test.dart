// The window's own pictures counted (Flutter's FrameTiming), in a real window (flutter drive --profile): a heavy
// document (fifteen recipes) played, a knob dragged to and fro, and the playhead moved back and forth, twenty seconds
// each. Red: any picture of the window that took longer than a sixtieth of a second.
//   flutter drive --profile -d macos --driver=test_driver/integration_test.dart --target=integration_test/frames_test.dart
import 'dart:io';
import 'dart:ui' show FramePhase;

import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart' show NumField;
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/workspace.dart' show Workspace;
import 'package:lab_book/workspace/ws.dart';

const _seconds = int.fromEnvironment('SECONDS', defaultValue: 20);

void main() {
  final binding = IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  binding.framePolicy = LiveTestWidgetsFlutterBindingFramePolicy.fullyLive;

  testWidgets('the window at 60 a second: playing, a knob dragged, the playhead moved, over a heavy document', (t) async {
    t.binding.defaultBinaryMessenger.setMockMethodCallHandler(SystemChannels.menu, (_) async => null);
    final doc = '${Directory.systemTemp.createTempSync('frames').path}/frames.motolii';
    await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => Workspace(document: doc)));
    await t.pumpAndSettle();
    final ws = WsScope.read(t.element(find.byType(WsTimeline))), m = ws.inside!;
    final made = [
      for (final (name, what) in m.recipes.where((r) => !r.$2.contains('Canvalry')).take(15)) (name, m.make(name), what),
    ];
    debugPrint('FRAMES the document: ${made.where((r) => r.$2 != 0).length} recipes (${made.map((r) => r.$1).join(', ')})');
    ws
      ..reread()
      ..select(const bool.fromEnvironment('NONE') ? null : '${made.firstWhere((r) => r.$2 != 0).$2}'); // (NONE: nothing picked, the stage's cards: to see what the picked thing's inspector costs)
    await t.pumpAndSettle();

    final timings = <FrameTiming>[];
    void took(List<FrameTiming> ts) => timings.addAll(ts);
    SchedulerBinding.instance.addTimingsCallback(took);
    final report = <String>[];
    var late = 0;
    Future<void> phase(String what, Future<void> Function(int frame) step) async {
      debugPrint('INSIDE before $what: ${ws.inside!.spent}');
      Ws.spent.clear();
      final from = timings.length, clock = Stopwatch()..start();
      for (var f = 0; clock.elapsedMilliseconds < _seconds * 1000; f++) {
        await step(f);
        await t.pump(const Duration(milliseconds: 16));
      }
      await t.pump(const Duration(milliseconds: 100));
      final spans = [for (final f in timings.skip(from)) f.totalSpan.inMicroseconds / 1000.0]..sort();
      final over = spans.where((s) => s > 1000 / 60).length;
      late += over;
      report.add('$what: ${spans.length} pictures, late $over, median ${spans.isEmpty ? 0 : spans[spans.length ~/ 2].toStringAsFixed(2)} ms, '
          '99th ${spans.isEmpty ? 0 : spans[(spans.length * 0.99).floor().clamp(0, spans.length - 1)].toStringAsFixed(2)} ms, most ${spans.isEmpty ? 0 : spans.last.toStringAsFixed(2)} ms');
      final slow = [for (final f in timings.skip(from)) if (f.totalSpan.inMicroseconds > 16667) f];
      double mid(Iterable<double> xs) { final l = xs.toList()..sort(); return l.isEmpty ? 0 : l[l.length ~/ 2]; }
      report.last += '; the late ones: build ${mid(slow.map((f) => f.buildDuration.inMicroseconds / 1000)).toStringAsFixed(1)}, raster ${mid(slow.map((f) => f.rasterDuration.inMicroseconds / 1000)).toStringAsFixed(1)}, '
          'before building ${mid(slow.map((f) => (f.timestampInMicroseconds(FramePhase.buildStart) - f.timestampInMicroseconds(FramePhase.vsyncStart)) / 1000)).toStringAsFixed(1)} ms (medians)';
      debugPrint('FRAMES ${report.last}');
      debugPrint('INSIDE after $what: ${ws.inside!.spent}');
      debugPrint('BUILDING during $what: ${(Ws.spent.entries.toList()..sort((a, b) => b.value.compareTo(a.value))).map((e) => '${e.key} ${e.value.inMilliseconds} ms').join(', ')}; layer ${ws.layer?.name} keys ${ws.layer?.keys.length}');
    }

    ws.playing = true;
    await phase('playing', (_) async {});
    ws.playing = false;

    final knob = find.byWidgetPredicate((w) => w is NumField && w.id == (const bool.fromEnvironment('NONE') ? 'v_exposure' : 'pos.x'));
    await t.ensureVisible(knob);
    await t.pump();
    final g = await t.startGesture(t.getCenter(knob));
    await phase('a knob dragged', (f) async => g.moveBy(Offset((f ~/ 60).isEven ? 3 : -3, 0)));
    await g.up();

    await phase('the playhead moved', (f) async => ws.frame = (f * 3) % (Ws.fps * 8));

    SchedulerBinding.instance.removeTimingsCallback(took);
    debugPrint('FRAMES all: late $late');
    expect(late, 0, reason: report.join('\n'));
  });
}
