// The window rampaged, in a real window (flutter drive --profile): every control it shows now (the cards' knobs,
// switches, segments, menus, links, folds, and the stage's handles) worked over and over in a seeded order. Red:
// a control that moves nothing (neither the document nor the window), a card whose numbers disagree with the
// document after a hand let go, and any picture of the window that took longer than a sixtieth of a second.
//   flutter drive --profile -d macos --driver=test_driver/integration_test.dart --target=integration_test/rampage_test.dart
import 'dart:developer' show Timeline;
import 'dart:io';
import 'dart:math' as math;
import 'dart:ui' show FramePhase, FrameTiming, ImageByteFormat;

import 'package:flutter/rendering.dart' show OffsetLayer;
import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart' show debugProfileBuildsEnabledUserWidgets;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/parts/controls.dart' show Segmented;
import 'package:lab_book/sets/inspector/inspector_parts.dart'
    show Chooser, Ctx, InspectorPanel, LinkMark, NumField, OnOff, RowOff;
import 'package:lab_book/workspace/stage_view.dart';
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/workspace.dart' show Workspace, wsTransform;
import 'package:lab_book/workspace/ws.dart';

import '../test/door.dart' show expectSameAfterReopen, openedDocument;

const _font = '$motoliiDir/out/arial_black.ttf';
const _seed = int.fromEnvironment('SEED', defaultValue: 7),
    _acts = int.fromEnvironment('ACTS', defaultValue: 400);

/// The whole document as values, thing by thing (the stage's own is thing 0).
const _judge = bool.fromEnvironment('JUDGE', defaultValue: true); // (off: the hand alone, nothing read to judge it: to see what the judging itself costs)
Map<String, Object?> _all(Motolii m, double at) => !_judge ? const {} : {
  for (final t in [0, ...m.things])
    for (final n in m.values(t)) '$t:$n': m.range(t, n) != null ? m.value(t, n, at) : m.word(t, n), // (a word as a word: read as a number every word was 0, the stage's effects unseen)
};

/// Whether a control is greyed by its row (a row that is off takes no hand).
bool _off(Element e) {
  var off = false;
  e.visitAncestorElements(
    (a) => !(off =
        (a.widget is IgnorePointer && (a.widget as IgnorePointer).ignoring) || (a.widget is RowOff && (a.widget as RowOff).off)), // (greyed as the field itself sees it: a row off takes no hand either)
  );
  return off;
}

/// A control brought into the middle of what shows it, as a hand looks at what it works (at an edge, a header that
/// stays put lay over it, and the hand could not reach it).
Future<void> _intoView(WidgetTester t, Finder f) => Scrollable.ensureVisible(t.element(f), alignment: .5);

void main() {
  final binding = IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  binding.framePolicy = LiveTestWidgetsFlutterBindingFramePolicy.fullyLive;
  WidgetController.hitTestWarningShouldBeFatal = true;

  testWidgets(
    'the window rampaged: everything moves, the cards agree with the document, 60 a second',
    (t) async {
      final timings = <FrameTiming>[];
      void took(List<FrameTiming> ts) => timings.addAll(ts);
      t.binding.defaultBinaryMessenger.setMockMethodCallHandler(
        SystemChannels.menu,
        (_) async => null,
      );
      final doc =
          '${Directory.systemTemp.createTempSync('rampage').path}/rampage.motolii';
      await t.pumpWidget(
        WidgetsApp(
          color: const Color(0xFF000000),
          builder: (c, _) => Workspace(document: doc),
        ),
      );
      await t.pumpAndSettle();
      final ws = WsScope.read(t.element(find.byType(WsTimeline))),
          m = ws.inside!;
      final things = [
        m.add('solid', 'cube'),
        m.add('solid', 'ring'),
        m.add('text', _font),
        m.add('light', 'sun'),
        m.make('Wave lines'),
      ]..removeWhere((n) => n == 0);
      m.say(things[1], 'fill', 'glass');
      ws
        ..reread()
        ..shelf = const bool.fromEnvironment('SHELF', defaultValue: true) ? 'Effects' : 'Media'; // (the shelf's buttons are hands too: the whole-screen effects go on and off the stage)
      await t.pumpAndSettle();
      SchedulerBinding.instance.addTimingsCallback(took);

      final r = math.Random(_seed),
          reds = <String>{},
          worked = <String, int>{},
          slow = <String, double>{},
          lateAfter = <String, int>{};
      final judging = <(int, int)>[], dragging = <(int, int)>[], acted = <(int, String)>[]; // (acted: when each act began, and what it was: a late picture told with the act before it) // (the test's own reading of the whole document: pictures begun then are not the window's)
      double at() => ws.frame / Ws.fps;
      Object state() {
        // (the window's own state: what its inspector holds, folds and flags and words)
        final d = Ctx.read(t.element(find.byType(InspectorPanel))).doc;
        return '${d.v}|${d.b}|${d.s2}|${d.animateAll}|${d.link}|${d.kind}';
      }

      // A drag as a hand makes it, in steps: before it is let go, the document must already show it (the preview the
      // stage draws while a hand is on a knob or a handle); the moment that happens is told back.
      Future<bool> dragSeen(Offset from, Offset by) async {
        final start = _all(m, at()), g = await t.startGesture(from), dragFrom = Timeline.now;
        var seen = false;
        for (var i = 1; i <= 3; i++) {
          await g.moveTo(from + by * (i / 3));
          await t.pump(const Duration(milliseconds: 16));
          final j = Timeline.now;
          final mid = _all(m, at());
          judging.add((j, Timeline.now));
          seen =
              seen ||
              mid.length != start.length ||
              mid.entries.any((e) => start[e.key] != e.value);
        }
        dragging.add((dragFrom, Timeline.now));
        await g.up();
        return seen;
      }

      Future<void> settle() async {
        for (var i = 0; i < 6; i++) {
          await t.pump(const Duration(milliseconds: 16));
        }
      }

      const trace = bool.fromEnvironment('TRACE');
      if (trace) debugProfileBuildsEnabledUserWidgets = true;
      Future<void> rampage() async {
        for (var act = 0; act < _acts; act++) {
          // (a list left floating by the hand before takes the next press, as it should: closed first, as a hand would)
          for (var k = 0; k < 3 && find.byType(CompositedTransformFollower).evaluate().isNotEmpty; k++) {
            await t.tapAt(const Offset(2, 2));
            await settle();
            debugPrint('CLOSED a floating list left open before act $act');
          }
          if (act % 25 == 0) {
            // (now and then another thing picked, or none)
            final pick = r.nextInt(things.length + 1);
            final seenPick = timings.length;
            ws.select(pick == things.length ? null : '${things[pick]}');
            await settle();
            lateAfter['pick'] = (lateAfter['pick'] ?? 0) + timings.skip(seenPick).where((f) => f.totalSpan.inMicroseconds > 16667).length;
          }
          final panel = find.byType(InspectorPanel), l0 = Timeline.now; // (the test's own walk of the window for what it may press: not the window's time)
          final ledger = <(String, Finder)>[
            for (final e
                in find
                    .descendant(of: panel, matching: find.byType(NumField))
                    .evaluate())
              if ((e.widget as NumField).enabled && !_off(e))
                (
                  'knob ${(e.widget as NumField).id}',
                  find.descendant(
                    of: panel,
                    matching: find.byWidgetPredicate(
                      (w) => w is NumField && w.id == (e.widget as NumField).id,
                    ),
                  ),
                ),
            for (final e
                in find
                    .descendant(of: panel, matching: find.byType(OnOff))
                    .evaluate())
              if ((e.widget as OnOff).enabled && !_off(e))
                ('switch #${e.hashCode % 997}', find.byWidget(e.widget)),
            for (final e
                in find
                    .descendant(of: panel, matching: find.byType(Segmented))
                    .evaluate())
              if (!_off(e))
                (
                  'segments ${(e.widget as Segmented).items.join('/')}',
                  find.descendant(
                    of: panel,
                    matching: find.byWidgetPredicate(
                      (w) =>
                          w is Segmented &&
                          w.items.join('/') ==
                              (e.widget as Segmented).items.join('/'),
                    ),
                  ),
                ),
            for (final e
                in find
                    .descendant(of: panel, matching: find.byType(Chooser))
                    .evaluate())
              if ((e.widget as Chooser).enabled && !_off(e))
                (
                  'menu ${(e.widget as Chooser).id}',
                  find.descendant(
                    of: panel,
                    matching: find.byWidgetPredicate(
                      (w) => w is Chooser && w.id == (e.widget as Chooser).id,
                    ),
                  ),
                ),
            for (final e
                in find
                    .descendant(of: panel, matching: find.byType(LinkMark))
                    .evaluate())
              if (!_off(e)) ('link', find.byWidget(e.widget)),
            for (final e in find.byWidgetPredicate((w) => w.key is ValueKey<String> && (w.key as ValueKey<String>).value.startsWith('ws-screen-')).evaluate())
              ('screen ${(e.widget.key as ValueKey<String>).value.substring(10)}', find.byKey(e.widget.key!)),
            if (find.byType(StageView).evaluate().isNotEmpty)
              for (final MapEntry(key: name, value: at) in (t.state(
                find.byType(StageView),
              ) as StageHandles).handles.entries)
                if ((Offset.zero & t.getSize(find.byType(StageView))).deflate(4).contains(at)) // (a hand takes only what it sees: a thing fallen off the picture has its handles there too)
                  ('handle $name', find.byType(StageView)),
          ];
          judging.add((l0, Timeline.now));
          if (ledger.isEmpty) continue;
          final (what, where) = ledger[r.nextInt(ledger.length)];
          acted.add((Timeline.now, what));
          worked[what.split(' ').take(2).join(' ')] =
              (worked[what.split(' ').take(2).join(' ')] ?? 0) + 1;
          var j0 = Timeline.now;
          final before = _all(m, at()), was = state(), seen = timings.length;
          var under = ''; // (what the hand's first touch landed on: told with a red)
          judging.add((j0, Timeline.now));
          try {
            if (what.startsWith('knob') || what.startsWith('handle')) {
              // (one way, and if that moved nothing (a value at its end), the other way: then the preview is judged)
              Offset from;
              if (what.startsWith('knob')) {
                await _intoView(t, where.first);
              await t.pump(); // (ensureVisible only scrolls: the place is laid out again at the next picture)
                from = t.getCenter(where.first);
              } else {
                final at0 = (t.state(
                  where.first,
                ) as StageHandles).handles[what.substring(7)];
                if (at0 == null) continue;
                from = t.getTopLeft(where.first) + at0;
              }
              final sign = r.nextBool() ? 1.0 : -1.0, by = Offset(sign * (30 + r.nextDouble() * 30), (r.nextBool() ? 1 : -1) * (30 + r.nextDouble() * 30)); // (far enough to move a whole step)
              under = [for (final e in t.hitTestOnBinding(from).path.take(6)) e.target.runtimeType.toString().replaceAll('Render', '')].join('<') + ' sel ${ws.selected}';
              var seen = await dragSeen(from, by);
              await settle();
              if (_all(
                    m,
                    at(),
                  ).entries.every((e) => before[e.key] == e.value)) { // (the document unmoved: a value at its end, pushed past it; the window may show the hand a moment longer, that is not a move)
                seen = await dragSeen(
                  what.startsWith('knob') ? t.getCenter(where.first) : from,
                  -by * 2,
                );
                await settle();
              }
              final done = _all(m, at());
              if (!seen && done.entries.any((e) => before[e.key] != e.value))
                reds.add(
                  '$what: nothing moves until the hand lets go (no preview)',
                );
            } else if (what.startsWith('segments')) {
              await _intoView(t, where.first);
              await t.pump(); // (ensureVisible only scrolls: the place is laid out again at the next picture)
              final seg = t.widget(where.first) as Segmented,
                  n = seg.items.length;
              final other = (seg.index + 1 + r.nextInt(math.max(1, n - 1))) % n;
              final word = find.descendant(
                of: where.first,
                matching: find.byWidgetPredicate(
                  (w) =>
                      w is Text &&
                      (w.data ?? '').toLowerCase() ==
                          seg.items[other].toLowerCase(),
                ),
              );
              if (word.evaluate().isNotEmpty) {
                await t.tap(word.first);
              } else {
                final box = t.getRect(where.first);
                await t.tapAt(
                  Offset(
                    box.left + box.width * (other + 0.5) / n,
                    box.center.dy,
                  ),
                );
              }
            } else if (what.startsWith('menu')) {
              await _intoView(t, where.first);
              await t.pump(); // (ensureVisible only scrolls: the place is laid out again at the next picture)
              final chooser = t.widget(where.first) as Chooser;
              await t.tap(where.first);
              await settle();
              final pick = chooser.options[r.nextInt(chooser.options.length)];
              final option = find.text(pick);
              if (option.evaluate().length > 1) {
                // (the word in the list that floats over the rest: the same word may be a row of the timeline beneath it)
                final words = [for (var i = option.evaluate().length - 1; i >= 0; i--) option.at(i)];
                final top = words.firstWhere((w) {
                  final box = t.renderObject(w);
                  return t.hitTestOnBinding(t.getCenter(w)).path.any((e) => e.target == box);
                }, orElse: () => option.last);
                await t.ensureVisible(top); // (a long list scrolls within: the word found is brought into view first)
                await t.pump();
                await t.tap(top);
              } else {
                await t.tap(where.first); // (nothing to pick: the menu closed again, so it does not cover the next hand)
              }
            } else {
              await _intoView(t, where.first);
              await t.pump(); // (ensureVisible only scrolls: the place is laid out again at the next picture)
              await t.tap(where.first);
            }
          } catch (e, stack) {
            final at = stack.toString().split('\n').where((l) => l.contains('package:lab_book')).take(2).join(' < ');
            reds.add(
              '$what: the hand could not work it (${e.toString().split('\n').first}) at $at',
            );
          }
          await settle();
          j0 = Timeline.now;
          final after = _all(m, at()), now = state();
          for (final f in timings.skip(seen)) {
            // (the slowest picture after each kind of hand)
            final ms = f.totalSpan.inMicroseconds / 1000.0,
                key = what.split(' ').take(2).join(' ');
            if (ms > 1000 / 60 && ms > (slow[key] ?? 0)) slow[key] = ms;
            if (ms > 1000 / 60) lateAfter[key.split(' ').first] = (lateAfter[key.split(' ').first] ?? 0) + 1;
          }
          final moved = {
            for (final k in {...before.keys, ...after.keys})
              if (before[k] != after[k]) k,
          };
          if (moved.isEmpty && was == now && !what.startsWith('menu'))
            reds.add('$what: moves nothing');
          if (moved.isEmpty && was == now && !what.startsWith('menu')) debugPrint('RED-UNDER $what: ${what.startsWith('knob') && where.evaluate().isNotEmpty ? 'wide ${t.getSize(where.first).width.toStringAsFixed(0)} editing ${find.descendant(of: where.first, matching: find.byType(EditableText)).evaluate().isNotEmpty} held ${Ctx.read(t.element(where.first)).doc.get((t.widget(where.first) as NumField).id)} worked ${worked[what.split(' ').take(2).join(' ')]}' : ''} $under | trouble "${m.trouble}" notice "${ws.notice}" animating ${ws.animating} keys ${what.startsWith('knob v_') ? m.keyMoments(int.tryParse(ws.selected ?? '') ?? 0, what.substring(7)).length : -1} value ${what.startsWith('knob v_') ? m.word(int.tryParse(ws.selected ?? '') ?? 0, what.substring(7)) : ''}');
          // (the cards agree with the document: each number the inspector holds is what the document makes of it now)
          final told = wsTransform(ws, ws.selected),
              held = Ctx.read(t.element(find.byType(InspectorPanel))).doc.v;
          for (final MapEntry(:key, :value) in told.entries) {
            if (held[key] case final shown?
                when (shown - value).abs() > 0.051 && !key.startsWith('cam.'))
              reds.add(
                '$what: the card shows $key $shown, the document $value',
              );
          }
          judging.add((j0, Timeline.now));
          if (act % 40 == 39) {
            // (now and then five steps taken back and done again: the document as it was)
            final u0 = Timeline.now; // (the test's own check, at once and waited for: not a hand's, not the window's time)
            final snap = _all(m, at());
            var back = 0;
            while (back < 5 && m.undo()) {
              back++;
            }
            for (var i = 0; i < back; i++) {
              m.redo();
            }
            final again = _all(m, at());
            if (again.length != snap.length ||
                again.entries.any((e) => snap[e.key] != e.value))
              reds.add('five undone and redone: the document is not as it was');
            ws.reread();
            judging.add((u0, Timeline.now));
            await settle();
          }
        }
      }

      if (trace) {
        await binding.traceAction(rampage, reportKey: 'timeline');
      } else {
        await rampage();
      }
      SchedulerBinding.instance.removeTimingsCallback(took);
      // ignore: avoid_print
      print('INSIDE ${m.spent}');
      for (final MapEntry(:key, :value)
          in (Ws.spent.entries.toList()
            ..sort((a, b) => b.value.compareTo(a.value)))) {
        // ignore: avoid_print
        print('SPENT $key ${value.inMilliseconds} ms');
      }

      bool judged(FrameTiming f) {
        final v = f.timestampInMicroseconds(FramePhase.vsyncStart);
        return judging.any((w) => v >= w.$1 - 17000 && v <= w.$2);
      }

      // ignore: avoid_print
      print(
        'JUDGED-OUT ${timings.where(judged).length} pictures begun while the test read the document',
      );
      timings.removeWhere(judged);
      if (acted.isNotEmpty) timings.removeWhere((f) => f.timestampInMicroseconds(FramePhase.vsyncStart) < acted.first.$1); // (the window opening, before the first hand: not a hand's)
    bool underHand(FrameTiming f) { final v = f.timestampInMicroseconds(FramePhase.vsyncStart); return dragging.any((w) => v >= w.$1 && v <= w.$2); }
    final lateUnder = timings.where((f) => f.totalSpan.inMicroseconds > 16667 && underHand(f)).length, lateElse = timings.where((f) => f.totalSpan.inMicroseconds > 16667 && !underHand(f)).length;
    // ignore: avoid_print
    print('LATE-WHEN $lateUnder while a hand drags, $lateElse otherwise (letting go, picking, menus, taps)');
      final spans = [
        for (final f in timings) f.totalSpan.inMicroseconds / 1000.0,
      ]..sort();
      double pc(double p) => spans.isEmpty
          ? 0
          : spans[math.min(spans.length - 1, (p * spans.length).floor())];
      final late = spans.where((s) => s > 1000 / 60).length;
      // (of the late pictures, how much was the window's own building (Dart, and its calls to the inside) and how much its drawing)
      final lateOnes = [
        for (final f in timings)
          if (f.totalSpan.inMicroseconds > 16667) f,
      ];
      double mid(Iterable<double> xs) {
        final l = xs.toList()..sort();
        return l.isEmpty ? 0 : l[l.length ~/ 2];
      }

      // ignore: avoid_print
      print(
        'LATE ${lateOnes.length}: build median ${mid(lateOnes.map((f) => f.buildDuration.inMicroseconds / 1000)).toStringAsFixed(1)} ms, raster median ${mid(lateOnes.map((f) => f.rasterDuration.inMicroseconds / 1000)).toStringAsFixed(1)} ms, waiting median ${mid(lateOnes.map((f) => (f.totalSpan - f.buildDuration - f.rasterDuration).inMicroseconds / 1000)).toStringAsFixed(1)} ms (before building ${mid(lateOnes.map((f) => f.vsyncOverhead.inMicroseconds / 1000)).toStringAsFixed(1)}, between building and drawing ${mid(lateOnes.map((f) => (f.timestampInMicroseconds(FramePhase.rasterStart) - f.timestampInMicroseconds(FramePhase.buildFinish)) / 1000)).toStringAsFixed(1)})',
      );
      final report = {
        'acts': _acts,
        'seed': _seed,
        'pictures': spans.length,
        'late (over 16.7 ms)': late,
        'median ms': pc(0.5),
        '90th ms': pc(0.9),
        '99th ms': pc(0.99),
        'most ms': spans.isEmpty ? 0 : spans.last,
        'worked': worked,
        'red': reds.toList()..sort(),
      };
      // (the window as it is at the end, as a picture: its outermost painted layer, made a PNG by Flutter itself)
      // ignore: invalid_use_of_protected_member
      final root = t.binding.renderViews.first.layer! as OffsetLayer;
      final shot = await (await root.toImage(Offset.zero & t.view.physicalSize)).toByteData(format: ImageByteFormat.png);
      if (shot != null) File('$motoliiDir/../reports/skin_run/window_after_rampage.png').writeAsBytesSync(shot.buffer.asUint8List()); // (seen by whoever reads the report)
      binding.reportData = {...?binding.reportData, ...report};
      for (final red in reds.toList()..sort()) {
        // ignore: avoid_print
        print('RED $red');
      }
      for (final MapEntry(:key, :value)
          in (slow.entries.toList()
            ..sort((a, b) => b.value.compareTo(a.value)))) {
        // ignore: avoid_print
        print('SLOW $key ${value.toStringAsFixed(1)} ms');
      }
      // ignore: avoid_print
      print('LATE-AFTER $lateAfter');
      for (final f in lateOnes) {
        final v = f.timestampInMicroseconds(FramePhase.vsyncStart), act = acted.lastWhere((a) => a.$1 <= v, orElse: () => (0, 'before the first act'));
        // ignore: avoid_print
        print('LATE-ONE ${(f.totalSpan.inMicroseconds / 1000).toStringAsFixed(1)} ms ${((v - act.$1) / 1000).toStringAsFixed(0)} ms into ${act.$2} (build ${(f.buildDuration.inMicroseconds / 1000).toStringAsFixed(1)}, before building ${(f.vsyncOverhead.inMicroseconds / 1000).toStringAsFixed(1)})');
      }
      // ignore: avoid_print
      print(
        'RAMPAGE ${report['pictures']} pictures, late ${report['late (over 16.7 ms)']}, median ${pc(0.5).toStringAsFixed(2)} ms, 99th ${pc(0.99).toStringAsFixed(2)} ms, most ${(spans.isEmpty ? 0 : spans.last).toStringAsFixed(2)} ms; ${reds.length} red',
      );
      if (trace) return; // (a run that only traces where the time goes: its report written whatever it found)
      openedDocument = doc;
      await expectSameAfterReopen(t); // (the window is the document's alone: what the rampage left, opened again on the same document, shows the same)
    expect(reds, isEmpty);
      expect(late, 0, reason: 'pictures over a sixtieth of a second');
    },
    semanticsEnabled: false, // (a user's window builds no accessibility tree unless asked: the test should not either)
  );
}
