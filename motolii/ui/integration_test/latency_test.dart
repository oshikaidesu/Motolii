// Where the time goes between a hand and a pixel, per boundary, for the operations a person holds: dragging on the Stage,
// scrubbing an Inspector number, scrubbing the playhead (the real app and its host; results are printed as LAT lines).
import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'dart:ui' show FrameTiming;

import 'package:flutter/gestures.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'package:motolii_stage5/controls/panel/drag.dart' show EditorPreviewQueue;
import 'package:motolii_stage5/timeline/face.dart';
import 'package:motolii_stage5/app/main.dart' as app;
import 'package:motolii_stage5/stage/panel.dart' show StagePanel;
import 'package:motolii_stage5/stage/session.dart' show StageSession;
import 'package:motolii_stage5/session/latency_probe.dart';

Future<void> frames(WidgetTester t, [int n = 20]) async {
  for (var i = 0; i < n; i++) {
    await t.pump(const Duration(milliseconds: 50));
  }
}

double _median(List<double> v) {
  if (v.isEmpty) return double.nan;
  final s = [...v]..sort();
  return s[s.length ~/ 2];
}

double _p95(List<double> v) {
  if (v.isEmpty) return double.nan;
  final s = [...v]..sort();
  return s[(s.length * .95).floor().clamp(0, s.length - 1)];
}

/// Per command: pointer -> cmd -> req -> reply -> render-req -> render-reply -> next frame, and the last pointer to the next frame.
String summarize(String label, String op) {
  final ev = LatencyProbe.events;
  final pointers = [for (final e in ev) if (e.name == 'pointer') e];
  final rows = <String, List<double>>{'queue (cmd->req)': [], 'roundtrip (req->reply)': [], 'gap (reply->render-req)': [], 'render (req->reply)': [], 'to frame (accepted->next)': [], 'command total (cmd->next frame)': [], 'total frames (cmd->next)': []};
  for (var i = 0; i < ev.length; i++) {
    if (ev[i].name != 'cmd:$op') continue;
    final cmd = ev[i];
    T(String n, int from) {
      for (var j = from; j < ev.length; j++) {
        if (ev[j].name == n) return ev[j];
      }
      return null;
    }
    final req = T('req:$op', i), reply = T('reply:$op', i);
    final rr = T('render-req', i), rp = T('render-reply', i), acc = T('render-accepted', i), nf = T('next-frame', i);
    if (req == null || reply == null || nf == null) continue;
    rows['queue (cmd->req)']!.add((req.us - cmd.us) / 1000);
    rows['roundtrip (req->reply)']!.add((reply.us - req.us) / 1000);
    if (rr != null) rows['gap (reply->render-req)']!.add((rr.us - reply.us) / 1000);
    if (rr != null && rp != null) rows['render (req->reply)']!.add((rp.us - rr.us) / 1000);
    if (acc != null) rows['to frame (accepted->next)']!.add((nf.us - acc.us) / 1000);
    rows['command total (cmd->next frame)']!.add((nf.us - cmd.us) / 1000);
    rows['total frames (cmd->next)']!.add((nf.frame - cmd.frame).toDouble());
  }
  // pointer -> visible: each input is shown by the first command issued at or after it (the queue keeps only the latest), at
  // the first frame after that command's render was taken in
  final p2v = <double>[], p2vFrames = <double>[], p2a = <double>[];
  for (var i = 0; i < ev.length; i++) {
    if (ev[i].name != 'pointer') continue;
    int? cmd;
    for (var j = i; j < ev.length; j++) {
      if (ev[j].name == 'cmd:$op') {
        cmd = j;
        break;
      }
    }
    if (cmd == null) continue;
    int? acc;
    for (var j = cmd; j < ev.length; j++) {
      if (ev[j].name == 'render-accepted') {
        acc = j;
        break;
      }
    }
    if (acc == null) continue;
    p2a.add((ev[acc].us - ev[i].us) / 1000);
    for (var j = acc; j < ev.length; j++) {
      if (ev[j].name == 'next-frame') {
        p2v.add((ev[j].us - ev[i].us) / 1000);
        p2vFrames.add((ev[j].frame - ev[i].frame).toDouble());
        break;
      }
    }
  }
  final out = StringBuffer('LAT $label: ${pointers.length} pointer/inputs, ${rows['queue (cmd->req)']!.length} commands\n');
  rows['POINTER -> RENDER TAKEN IN (ms)'] = p2a;
  rows['POINTER -> VISIBLE (ms)'] = p2v;
  rows['POINTER -> VISIBLE (frames)'] = p2vFrames;
  rows.forEach((k, v) => out.writeln('LAT   ${k.padRight(34)} median ${_median(v).toStringAsFixed(1).padLeft(6)}  p95 ${_p95(v).toStringAsFixed(1).padLeft(6)}'));
  return out.toString();
}

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('latency per boundary', (t) async {
    app.main();
    await frames(t, 80);
    final c = t.widget<LiveTimeline>(find.byType(LiveTimeline)).c;
    final report = StringBuffer();
    // a larger document: LATENCY_LAYERS more rectangles, to see what grows with the document
    const extra = int.fromEnvironment('LATENCY_LAYERS', defaultValue: 0);
    for (var i = 0; i < extra; i++) {
      await c.command('create', {'kind': 'rectangle'});
    }
    if (extra > 0) {
      await frames(t, 10);
      report.writeln('LAT document: ${c.layers.length} layers, layers JSON ${(jsonEncode(c.state['layers']).length / 1024).toStringAsFixed(0)} KB');
    }

    // 1. dragging on the Stage
    LatencyProbe.enable();
    LatencyProbe.counts.clear();
    final nativeMs = <double>[];
    c.rendered.addListener(() {
      final v = c.state['renderMs'];
      if (v is num) nativeMs.add(v.toDouble());
    });
    final stage = find.byType(StagePanel);
    final centre = t.getCenter(stage.first);
    final g = await t.startGesture(centre, kind: PointerDeviceKind.mouse);
    await t.pump(const Duration(milliseconds: 40));
    LatencyProbe.events.clear();
    for (var i = 0; i < 40; i++) {
      await g.moveBy(const Offset(3, 2));
      await t.pump(const Duration(milliseconds: 16));
    }
    await g.up();
    await frames(t, 12);
    report.write(summarize('stage drag (stageGesture)', 'stageGesture'));
    report.writeln('LAT slices woken by the Stage drag: ${[for (final e in LatencyProbe.counts.entries) '${e.key.substring(6)}=${e.value}'].join(', ')}');
    LatencyProbe.counts.clear();
    report.writeln('LAT   native render_into_surface (renderMs)  median ${_median(nativeMs).toStringAsFixed(1)}  p95 ${_p95(nativeMs).toStringAsFixed(1)}  (${nativeMs.length} renders, all three tests)');
    await c.command('undo');
    await frames(t, 6);

    // 2. an Inspector number scrubbed (the same call the field makes for each tick)
    final layers = c.layers.where((l) => l['kind'] != 'Camera' && l['kind'] != 'Group').toList();
    await c.command('select', {'ids': [layers.first['id']]});
    await frames(t, 8);
    LatencyProbe.events.clear();
    // the field's own queue: latest wins, and the next value waits for the last one's render
    final queue = EditorPreviewQueue<int>((i) async {
      await c.command('previewProperties', {
        'edits': [
          {'layer': layers.first['id'], 'property': 'position', 'value': [900.0 + i * 3, 500.0 + i], 'spread': 'offset'}
        ]
      });
    });
    for (var i = 0; i < 120; i++) {
      LatencyProbe.mark('pointer');
      queue.add(i);
      await t.pump(const Duration(milliseconds: 8)); // a 120 Hz hand
    }
    await queue.drained;
    await frames(t, 12);
    report.write(summarize('inspector scrub (previewProperties)', 'previewProperties'));
    await c.command('cancelPreview');
    await frames(t, 4);

    // frame cost while an Inspector number is scrubbed: how long the Flutter side itself spends per frame
    {
      final builds = <double>[], rasters = <double>[];
      void onTimings(List<FrameTiming> ts) {
        for (final f in ts) {
          builds.add(f.buildDuration.inMicroseconds / 1000);
          rasters.add(f.rasterDuration.inMicroseconds / 1000);
        }
      }

      SchedulerBinding.instance.addTimingsCallback(onTimings);
      LatencyProbe.counts.clear();
      for (var i = 0; i < 60; i++) {
        c.commandDirect('previewProperties', {
          'edits': [
            {'layer': layers.first['id'], 'property': 'position', 'value': [900.0 + i * 3, 500.0 + i], 'spread': 'offset'}
          ]
        }, 'x');
        await t.pump(const Duration(milliseconds: 16));
      }
      await frames(t, 6);
      SchedulerBinding.instance.removeTimingsCallback(onTimings);
      await c.commandDirect('cancelPreview');
      final woken = LatencyProbe.counts.entries.toList()..sort((a, b) => b.value.compareTo(a.value));
      report.writeln('LAT slices woken by a 60-step Inspector scrub: ${[for (final e in woken) '${e.key.substring(6)}=${e.value}'].join(', ')}');
      report.writeln('LAT Flutter frame cost while scrubbing (debug Dart): build median ${_median(builds).toStringAsFixed(1)} p95 ${_p95(builds).toStringAsFixed(1)} ms; raster median ${_median(rasters).toStringAsFixed(1)} p95 ${_p95(rasters).toStringAsFixed(1)} ms; ${builds.length} frames');
    }

    // 3. the playhead scrubbed
    LatencyProbe.events.clear();
    LatencyProbe.counts.clear();
    for (var i = 0; i < 40; i++) {
      LatencyProbe.mark('pointer');
      c.command('seek', {'frame': 10 + i * 2});
      await t.pump(const Duration(milliseconds: 16));
    }
    await frames(t, 12);
    report.write(summarize('playhead scrub (seek)', 'seek'));
    report.writeln('LAT slices woken by the playhead scrub: ${[for (final e in LatencyProbe.counts.entries) '${e.key.substring(6)}=${e.value}'].join(', ')}');

    // 4. panning the Stage (what a hand-pan does: the view moves, the window native draws must follow)
    final session = StageSession.of(c, 'User');
    final size = t.getSize(stage.first);
    LatencyProbe.events.clear();
    for (var i = 0; i < 40; i++) {
      LatencyProbe.mark('pointer');
      session.panBy(const Offset(4, 0), size);
      await t.pump(const Duration(milliseconds: 16));
    }
    await frames(t, 12);
    report.write(summarize('stage pan (stageWindow)', 'stageWindow'));

    // 5. the Inspector queue fed by a 125 Hz hand in real time (a timer, not the frame clock): does a value wait behind more than
    // the one render that is already running?
    LatencyProbe.events.clear();
    var n = 0;
    final queue2 = EditorPreviewQueue<int>((i) async {
      await c.command('previewProperties', {
        'edits': [
          {'layer': layers.first['id'], 'property': 'position', 'value': [900.0 + i * 2, 500.0 + i], 'spread': 'offset'}
        ]
      });
    });
    final hand = Timer.periodic(const Duration(milliseconds: 8), (_) {
      LatencyProbe.mark('pointer');
      queue2.add(n++);
    });
    await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 1500)));
    hand.cancel();
    await queue2.drained;
    await frames(t, 12);
    report.write(summarize('inspector queue, 125 Hz real-time hand', 'previewProperties'));
    report.writeln('LAT   inputs ${LatencyProbe.events.where((e) => e.name == 'pointer').length}, commands sent ${LatencyProbe.events.where((e) => e.name == 'cmd:previewProperties').length} (the rest were replaced by newer values)');
    await c.command('cancelPreview');

    // 6. the live Inspector's own path: a preview per tick straight to the session (`command`, the old way) and through the
    // latest-wins path (`commandDirect`), at a 125 Hz hand in real time; how far behind the newest input is the last picture?
    for (final direct in [false, true]) {
      LatencyProbe.events.clear();
      var m = 0;
      final id = layers.first['id'];
      final hand2 = Timer.periodic(const Duration(milliseconds: 8), (_) {
        LatencyProbe.mark('pointer');
        final args = {
          'edits': [
            {'layer': id, 'property': 'position', 'value': [900.0 + m * 2, 500.0 + m], 'spread': 'offset'}
          ]
        };
        m++;
        if (direct) {
          c.commandDirect('previewProperties', args, '$id:position');
        } else {
          c.command('previewProperties', args);
        }
      });
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 1500)));
      hand2.cancel();
      final stopped = LatencyProbe.events.where((e) => e.name == 'pointer').last.us;
      final sw = Stopwatch()..start();
      await c.commandDirect('commitPreview');
      final settle = (LatencyProbe.events.where((e) => e.name == 'render-accepted').last.us - stopped) / 1000;
      await frames(t, 8);
      final sent = LatencyProbe.events.where((e) => e.name == 'cmd:previewProperties').length;
      report.writeln('LAT live Inspector path, ${direct ? 'latest-wins (commandDirect)' : 'every tick queued (command)'}: $m inputs; $sent sent; after the hand stopped the last picture was taken in ${settle.toStringAsFixed(0)} ms');
      await c.command('undo');
      await frames(t, 6);
    }

    // 7. a long hand-pan at 125 Hz in real time: how far behind is the picture when the hand stops?
    {
      LatencyProbe.events.clear();
      final hand3 = Timer.periodic(const Duration(milliseconds: 8), (_) {
        LatencyProbe.mark('pointer');
        session.panBy(const Offset(1, 0), size);
      });
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 1500)));
      hand3.cancel();
      final stopped = LatencyProbe.events.where((e) => e.name == 'pointer').last.us;
      final inputs = LatencyProbe.events.where((e) => e.name == 'pointer').length;
      await frames(t, 20);
      final accepted = LatencyProbe.events.where((e) => e.name == 'render-accepted' && e.us > stopped).toList();
      final sent = LatencyProbe.events.where((e) => e.name == 'cmd:stageWindow').length;
      final last = LatencyProbe.events.where((e) => e.name == 'render-accepted').last.us;
      report.writeln('LAT stage pan, 125 Hz hand for 1.5 s: $inputs inputs; $sent window requests; the last picture was taken in ${((last - stopped) / 1000).toStringAsFixed(0)} ms after the hand stopped (${accepted.length} renders after it stopped)');
    }

    // ignore: avoid_print
    print(report);
    File('/tmp/latency_report.txt').writeAsStringSync(report.toString());
  });
}
