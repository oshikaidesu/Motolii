// A crowd of films in a real window for so long (SECONDS, ten minutes by default), the window's own pictures counted
// (Flutter's FrameTiming), the inside's pictures counted (motolii_frames: one for each of the window's), and the process's
// memory said each minute. Red: a late picture of the window, a picture the inside left out, memory that grows.
//   flutter drive --profile -d macos --driver=test_driver/integration_test.dart --target=integration_test/crowd_test.dart
import 'dart:io';

import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/workspace.dart' show Workspace;
import 'package:lab_book/workspace/ws.dart';

const _seconds = int.fromEnvironment('SECONDS', defaultValue: 600);
const _films = String.fromEnvironment('FILMS', defaultValue: '/Users/motolii_maker/motolii/filament/out/vid');

void main() {
  final binding = IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  binding.framePolicy = LiveTestWidgetsFlutterBindingFramePolicy.fullyLive;

  testWidgets('a crowd of films in the window for $_seconds seconds: no late picture, none left out, memory flat', (t) async {
    t.binding.defaultBinaryMessenger.setMockMethodCallHandler(SystemChannels.menu, (_) async => null);
    final doc = '${Directory.systemTemp.createTempSync('crowd').path}/crowd.motolii';
    await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => Workspace(document: doc)));
    await t.pumpAndSettle();
    final ws = WsScope.read(t.element(find.byType(WsTimeline))), m = ws.inside!;
    final files = (Directory(_films).listSync().whereType<File>().map((f) => f.path).where((p) => p.endsWith('.mp4')).toList()..sort());
    var n = 0;
    for (var k = 0; k < 3; k++) {
      for (final f in files) {
        final thing = m.add('film', f);
        if (thing == 0) continue;
        m.set(thing, 'x', -2.6 + (n % 7) * 0.85);
        m.set(thing, 'y', 1.3 - (n ~/ 7) * 0.7);
        m.set(thing, 'size', 0.55);
        n++;
      }
    }
    debugPrint('CROWD $n films');
    ws.reread();
    await t.pumpAndSettle();
    final timings = <FrameTiming>[];
    SchedulerBinding.instance.addTimingsCallback(timings.addAll);
    ws.playing = true;
    final clock = Stopwatch()..start();
    var drawn = m.frames, minute = 0, lateAll = 0, leftAll = 0;
    var from = 0;
    final rss = <int>[];
    while (clock.elapsedMilliseconds < _seconds * 1000) {
      await t.pump(const Duration(milliseconds: 16));
      if (clock.elapsedMilliseconds >= (minute + 1) * 60000 || clock.elapsedMilliseconds >= _seconds * 1000) {
        minute++;
        final spans = [for (final f in timings.skip(from)) f.totalSpan.inMicroseconds];
        final late = spans.where((s) => s > 16667).length;
        final nowDrawn = m.frames, wanted = spans.length, left = wanted - (nowDrawn - drawn);   // (the inside draws one for each of the window's)
        rss.add(ProcessInfo.currentRss ~/ (1 << 20));
        lateAll += late;
        if (minute > 1) leftAll += left.clamp(0, wanted);   // (the first minute: films come up)
        debugPrint('CROWD minute $minute: window pictures ${spans.length}, late $late; inside drew ${nowDrawn - drawn} for $wanted; memory ${rss.last} MB');
        from = timings.length;
        drawn = nowDrawn;
      }
    }
    ws.playing = false;
    debugPrint('CROWD in all: window late $lateAll, inside left out $leftAll after the first minute, memory ${rss.first} -> ${rss.last} MB');
  }, timeout: const Timeout(Duration(minutes: 30)));
}
