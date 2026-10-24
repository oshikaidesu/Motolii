// The window's own pictures counted (Flutter's FrameTiming), in a real window (flutter drive --profile): a heavy
// document (fifteen recipes) played, a knob dragged to and fro, and the playhead moved back and forth, twenty seconds
// each. Red: any picture of the window that took longer than a sixtieth of a second.
//   flutter drive --profile -d macos --driver=test_driver/integration_test.dart --target=integration_test/frames_test.dart
import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:ui' show FramePhase;

import 'package:flutter/rendering.dart' show debugProfileLayoutsEnabled;
import 'package:flutter/widgets.dart' show debugProfileBuildsEnabled;
import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart' show NumField;
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/workspace.dart' show Workspace;
import 'package:lab_book/workspace/stage_art.dart' show WsArt;
import 'package:lab_book/workspace/ws.dart';

const _seconds = int.fromEnvironment('SECONDS', defaultValue: 20);

void main() {
  final binding = IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  binding.framePolicy = LiveTestWidgetsFlutterBindingFramePolicy.fullyLive;

  testWidgets('the window at 60 a second: playing, a knob dragged, the playhead moved, over a heavy document', (t) async {
    t.binding.defaultBinaryMessenger.setMockMethodCallHandler(SystemChannels.menu, (_) async => null);
    final doc = '${Directory.systemTemp.createTempSync('frames').path}/frames.motolii';
    if (const String.fromEnvironment('DOC') case final from when from.isNotEmpty) File(from).copySync(doc);   // (DOC=path: a copy of that document, as it is, instead of the recipes)
    await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => Workspace(document: doc)));
    await t.pumpAndSettle();
    final ws = WsScope.read(t.element(find.byType(WsTimeline))), m = ws.inside!;
    final made = const String.fromEnvironment('DOC').isNotEmpty ? [for (final t in m.things) ('', t, '')] : [
      for (final (name, what) in m.recipes.where((r) => !r.$2.contains('Canvalry')).take(15)) (name, m.make(name), what),
    ];
    debugPrint('FRAMES the document: ${made.where((r) => r.$2 != 0).length} recipes (${made.map((r) => r.$1).join(', ')})');
    if (const bool.fromEnvironment('FX')) { // (FX: every recipe given ThorVG's blur, shadow and glow: effects piled on)
      for (final (_, thing, _) in made) {
        if (thing != 0) for (final fx in const ['Gaussian Blur', 'Drop Shadow', 'Glow']) m.effect(thing, fx, 0);
      }
      debugPrint('FRAMES effects on every recipe: blur, shadow, glow');
    }
    ws
      ..reread()
      ..select(const bool.fromEnvironment('NONE') ? null : '${made.firstWhere((r) => r.$2 != 0).$2}'); // (NONE: nothing picked, the stage's cards: to see what the picked thing's inspector costs)
    await t.pumpAndSettle();

    final timings = <FrameTiming>[];
    void took(List<FrameTiming> ts) => timings.addAll(ts);
    SchedulerBinding.instance.addTimingsCallback(took);
    final report = <String>[], broke = <String>[];   // (broke: where the window's thread waited on the inside)
    var late = 0;
    Future<void> phase(String what, Future<void> Function(int frame) step) async {
      debugPrint('INSIDE before $what: ${ws.inside!.spent}');
      Ws.spent.clear();
      final from = timings.length, drawn = ws.inside!.frames, reads = ws.reads, clock = Stopwatch()..start();
      ws.inside!.watch(true);   // (the window's thread may not wait on the inside: what did is kept)
      var steps = 0;
      for (var f = 0; clock.elapsedMilliseconds < _seconds * 1000; f++, steps++) {
        await step(f);
        await SchedulerBinding.instance.endOfFrame;   // (one move a picture of the window: t.pump's wait took two, the hand moved 29 a second)
      }
      final stage = (ws.inside!.frames - drawn) / (clock.elapsedMicroseconds / 1e6);   // (the stage's own pictures a second: what the thing moves at)
      await t.pump(const Duration(milliseconds: 100));
      ws.inside!.watch(false);
      if (ws.inside!.broken case final b when b.isNotEmpty) broke.add('$what:\n$b');
      final spans = [for (final f in timings.skip(from)) f.totalSpan.inMicroseconds / 1000.0]..sort();
      final over = spans.where((s) => s > 1000 / 60).length;
      debugPrint('FRAMES $what: the late ones at ${[for (final (i, f) in timings.skip(from).indexed) if (f.totalSpan.inMicroseconds > 1000000 / 60) '#$i ${(f.totalSpan.inMicroseconds / 1000).toStringAsFixed(0)} ms (build ${(f.buildDuration.inMicroseconds / 1000).toStringAsFixed(0)}, waited ${((f.timestampInMicroseconds(FramePhase.buildStart) - f.timestampInMicroseconds(FramePhase.vsyncStart)) / 1000).toStringAsFixed(0)})'].join(', ')}');
      late += over;
      if (stage < 59) late++;
      report.add('$what: the document read again ${ws.reads - reads} times, the hand moved ${(steps / (clock.elapsedMicroseconds / 1e6)).toStringAsFixed(1)} a second, the stage drew ${stage.toStringAsFixed(1)} a second; ${spans.length} pictures, late $over, median ${spans.isEmpty ? 0 : spans[spans.length ~/ 2].toStringAsFixed(2)} ms, '
          '99th ${spans.isEmpty ? 0 : spans[(spans.length * 0.99).floor().clamp(0, spans.length - 1)].toStringAsFixed(2)} ms, most ${spans.isEmpty ? 0 : spans.last.toStringAsFixed(2)} ms');
      final slow = [for (final f in timings.skip(from)) if (f.totalSpan.inMicroseconds > 16667) f];
      double mid(Iterable<double> xs) { final l = xs.toList()..sort(); return l.isEmpty ? 0 : l[l.length ~/ 2]; }
      report.last += '; the late ones: build ${mid(slow.map((f) => f.buildDuration.inMicroseconds / 1000)).toStringAsFixed(1)}, raster ${mid(slow.map((f) => f.rasterDuration.inMicroseconds / 1000)).toStringAsFixed(1)}, '
          'before building ${mid(slow.map((f) => (f.timestampInMicroseconds(FramePhase.buildStart) - f.timestampInMicroseconds(FramePhase.vsyncStart)) / 1000)).toStringAsFixed(1)} ms (medians)';
      debugPrint('FRAMES ${report.last}');
      debugPrint('INSIDE after $what: ${ws.inside!.spent}');
      debugPrint('BUILDING during $what: ${(Ws.spent.entries.toList()..sort((a, b) => b.value.compareTo(a.value))).map((e) => '${e.key} ${e.value.inMilliseconds} ms').join(', ')}; layer ${ws.layer?.name} keys ${ws.layer?.keys.length}');
    }

    if (const int.fromEnvironment('KEEP') > 0) { // (KEEP=n: the hand rests n seconds first, so the pictures that would be late are kept on disk: stage_kept.h)
      for (var i = 0; i < const int.fromEnvironment('KEEP') * 10; i++) {
        await t.pump(const Duration(milliseconds: 100));
        await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 100)));
      }
      final kept = Directory('${Platform.environment['HOME']}/Library/Caches/Motolii/frames').listSync().where((f) => f.path.contains('/stage_')).length;
      debugPrint('FRAMES kept on disk after resting: $kept pictures (all documents)');
    }
    if (const bool.fromEnvironment('REBUILDS')) {   // (what is built again in the first pictures of playing, each picture's beginnings)
      final now = <Element>{};
      final started = <String>[];
      SchedulerBinding.instance.addPersistentFrameCallback((_) => now.clear());
      debugOnRebuildDirtyWidget = (e, _) {
        var under = false;
        e.visitAncestorElements((a) => !(under = now.contains(a)));
        if (!under) started.add('${e.widget.runtimeType}');
        if (e is! InheritedElement) now.add(e);
      };
      ws.playing = true;
      for (var i = 0; i < 15; i++) {
        started.add('|${i + 1}');
        await SchedulerBinding.instance.endOfFrame;
      }
      ws.playing = false;
      debugOnRebuildDirtyWidget = null;
      debugPrint('REBUILT FROM the first pictures of playing: ${started.join(' ')}');
      await t.pump(const Duration(milliseconds: 200));
    }
    ws.playing = true;
    if (const bool.fromEnvironment('TRACE_PLAYING')) { // (TRACE_PLAYING: Flutter's own timeline of the playing, every widget's build and the collector: heavy, a gigabyte and more)
      debugProfileLayoutsEnabled = true;
      debugProfileBuildsEnabled = true;
      await binding.traceAction(() => phase('playing', (_) async {}), reportKey: 'timeline', streams: const ['Dart', 'Embedder', 'GC']);
      File('/tmp/motolii_playing_timeline.json').writeAsStringSync(jsonEncode(binding.reportData!['timeline'])); // (kept even when the test fails)
      debugProfileLayoutsEnabled = false;
      debugProfileBuildsEnabled = false;
    } else {
      await phase('playing', (_) async {});
    }
    ws.playing = false;

    final knob = find.byWidgetPredicate((w) => w is NumField && w.id == (const bool.fromEnvironment('NONE') ? 'v_exposure' : 'v_x'));
    await t.ensureVisible(knob);
    await t.pump();
    final g = await t.startGesture(t.getCenter(knob));
    Future<void> traced(String what, Future<void> Function(int) step) async { // (TRACE: Flutter's own timeline of it, kept in /tmp)
      if (!const bool.fromEnvironment('TRACE')) return phase(what, step);
      debugProfileBuildsEnabled = true;
      debugProfileBuildsEnabledUserWidgets = true;   // (only the window's own widgets named: light enough to leave the frames as they are)
      await binding.traceAction(() => phase(what, step), reportKey: 'timeline', streams: const ['Dart', 'Embedder', 'GC']);
      debugProfileBuildsEnabled = false;
      File('/tmp/motolii_${what.replaceAll(' ', '_')}_timeline.json').writeAsStringSync(jsonEncode(binding.reportData!['timeline']));
    }

    final moves = <double>[];
    await traced('a knob dragged', (f) async {
      if (f == 400 && const bool.fromEnvironment('SAMPLE')) unawaited(Process.start('sample', ['$pid', '2', '1', '-file', '/tmp/motolii_knob_sample.txt']));   // (SAMPLE: the main thread's stacks where a late picture comes, every millisecond)
      final w = Stopwatch()..start();
      await g.moveBy(Offset((f ~/ 60).isEven ? 3 : -3, 0));
      moves.add(w.elapsedMicroseconds / 1000);
    });
    moves.sort();
    debugPrint('FRAMES a move of the hand took: median ${moves[moves.length ~/ 2].toStringAsFixed(2)} ms, 90th ${moves[(moves.length * .9).floor()].toStringAsFixed(2)} ms, most ${moves.last.toStringAsFixed(2)} ms');
    await g.up();

    final rebuilt = <String, int>{};   // (REBUILDS, a debug run: which widgets are built again while only the playhead moves, Flutter's own hook)
    final roots = <String, int>{}, now = <Element>{};   // (and where each began: one built again whose parents were not)
    if (const bool.fromEnvironment('REBUILDS')) {
      SchedulerBinding.instance.addPersistentFrameCallback((_) => now.clear());
      debugOnRebuildDirtyWidget = (e, _) {
        rebuilt[e.widget.runtimeType.toString()] = (rebuilt[e.widget.runtimeType.toString()] ?? 0) + 1;
        var under = false;
        e.visitAncestorElements((a) => !(under = now.contains(a)));
        if (!under) roots['${e.widget.runtimeType} in ${e.findAncestorWidgetOfExactType<Workspace>() == null ? '?' : (e.debugGetDiagnosticChain().skip(1).map((a) => a.widget.runtimeType.toString()).firstWhere((n) => !n.startsWith('_') && n != 'Builder', orElse: () => '?'))}'] = (roots['${e.widget.runtimeType} in ${e.findAncestorWidgetOfExactType<Workspace>() == null ? '?' : (e.debugGetDiagnosticChain().skip(1).map((a) => a.widget.runtimeType.toString()).firstWhere((n) => !n.startsWith('_') && n != 'Builder', orElse: () => '?'))}'] ?? 0) + 1;
        if (e is! InheritedElement) now.add(e);   // (a scope telling what changed is no beginning: those who read it are)
      };
    }
    var moved = 0;
    await traced('the playhead moved', (f) async => (ws.frame = (f * 3) % (Ws.fps * 8), moved++));
    if (const bool.fromEnvironment('REBUILDS')) {   // (and play pressed and pressed again: what is built again for it)
      final before = rebuilt.values.fold(0, (a, b) => a + b), rootsBefore = Map.of(roots);
      for (var i = 0; i < 10; i++) {
        ws.playing = !ws.playing;
        await SchedulerBinding.instance.endOfFrame;
      }
      ws.playing = false;
      debugPrint('REBUILDS play pressed: ${((rebuilt.values.fold(0, (a, b) => a + b) - before) / 10).toStringAsFixed(1)} widgets each time');
      debugPrint('REBUILT FROM play pressed: ${[for (final e in roots.entries) if (e.value != (rootsBefore[e.key] ?? 0)) '${e.key} ${((e.value - (rootsBefore[e.key] ?? 0)) / 10).toStringAsFixed(1)}'].join(', ')}');
    }
    debugOnRebuildDirtyWidget = null;
    if (roots.isNotEmpty) debugPrint('REBUILT FROM a move of the playhead: ${(roots.entries.toList()..sort((a, b) => b.value.compareTo(a.value))).take(30).map((e) => '${e.key} ${(e.value / moved).toStringAsFixed(1)}').join(', ')}');
    if (rebuilt.isNotEmpty) debugPrint('REBUILDS a move of the playhead: ${(rebuilt.entries.toList()..sort((a, b) => b.value.compareTo(a.value))).take(40).map((e) => '${e.key} ${(e.value / moved).toStringAsFixed(1)}').join(', ')}');

    // (a thing carried on the stage, to and fro: the inspector's numbers and drawing follow the hand by the window's peek)
    if (m.frame(int.tryParse(ws.selected ?? '') ?? -1) case final box?) {
      final stage = t.getRect(find.byKey(const ValueKey('ws-stage-comp')));
      final h = await t.startGesture(stage.topLeft + Offset(box[0] / WsArt.comp.width * stage.width, box[1] / WsArt.comp.height * stage.height));
      await traced('a thing carried on the stage', (f) => h.moveBy(Offset((f ~/ 60).isEven ? 3 : -3, (f ~/ 90).isEven ? 1 : -1)));
      await h.up();
    }

    if (const bool.fromEnvironment('LETGO')) { // (LETGO: a knob taken, moved and let go again and again, Flutter's own timeline of it with each widget's build, for reading)
      Future<void> letGos() async {
        for (var i = 0; i < 15; i++) {
          final h = await t.startGesture(t.getCenter(knob));
          for (var k = 0; k < 5; k++) {
            await h.moveBy(const Offset(4, 0));
            await t.pump(const Duration(milliseconds: 16));
          }
          await h.up();
          for (var k = 0; k < 20; k++) {
            await t.pump(const Duration(milliseconds: 16));
          }
        }
      }
      Ws.spent.clear();
      if (const bool.fromEnvironment('TRACE')) {
        debugProfileBuildsEnabled = true;
        debugProfileBuildsEnabledUserWidgets = true; // (only the window's own widgets named: Flutter's own are many and slow the reading)
        await binding.traceAction(letGos, reportKey: 'timeline', streams: const ['Dart', 'Embedder']);
        debugProfileBuildsEnabled = false;
        File('/tmp/motolii_let_go_timeline.json').writeAsStringSync(jsonEncode(binding.reportData!['timeline']));
      } else {
        final from = timings.length; // (the timings are being gathered already: see addTimingsCallback above)
        await letGos();
        await t.pump(const Duration(milliseconds: 100));
        final spans = [for (final f in timings.skip(from)) f.totalSpan.inMicroseconds / 1000.0]..sort();
        debugPrint('FRAMES let go fifteen times: ${spans.length} pictures, late ${spans.where((s) => s > 1000 / 60).length}, most ${spans.isEmpty ? 0 : spans.last.toStringAsFixed(1)} ms, the late ones ${[for (final s in spans) if (s > 1000 / 60) s.toStringAsFixed(1)]}');
      }
      debugPrint('BUILDING during let go: ${(Ws.spent.entries.toList()..sort((a, b) => b.value.compareTo(a.value))).map((e) => '${e.key} ${e.value.inMilliseconds} ms').join(', ')}');
    }

    if (const bool.fromEnvironment('SCREEN')) { // (SCREEN: a whole-screen effect put on the stage and taken off again ten times, nothing picked: the stage's cards change shape each time)
      ws.select(null);
      await t.pumpAndSettle();
      final kaleido = m.effects.firstWhere((e) => e.$2.endsWith('/kaleido.wgsl')).$2;
      Future<void> toggles() async {
        for (var i = 0; i < 20; i++) {
          ws.screenToggle(kaleido);
          for (var k = 0; k < 20; k++) {
            await t.pump(const Duration(milliseconds: 16));
          }
        }
      }
      final from = timings.length;
      if (const bool.fromEnvironment('TRACE')) {
        await binding.traceAction(toggles, reportKey: 'timeline', streams: const ['Dart', 'Embedder']);
      } else {
        await toggles();
      }
      await t.pump(const Duration(milliseconds: 100));
      final spans = [for (final f in timings.skip(from)) f.totalSpan.inMicroseconds / 1000.0]..sort();
      debugPrint('FRAMES an effect on and off ten times: ${spans.length} pictures, late ${spans.where((s) => s > 1000 / 60).length}, the late ones ${[for (final s in spans) if (s > 1000 / 60) s.toStringAsFixed(1)]}');
    }
    SchedulerBinding.instance.removeTimingsCallback(took);
    debugPrint('FRAMES all: late $late');
    debugPrint('WAITED ON THE INSIDE: ${broke.isEmpty ? 'never' : broke.join('\n')}');
    expect(broke, isEmpty, reason: broke.join('\n'));   // (the rule: the window never waits on the inside)
    if (!const bool.fromEnvironment('TRACE')) expect(late, 0, reason: report.join('\n')); // (a traced run is for reading: its timeline is written only if it passes)
  });
}
