// Surface audit: where the Inspector's area goes (content, control, text, face, and what is left as Housing), measured on the real
// app's render tree for a chosen layer. A measuring instrument, not a check: it only runs with `SURFACE_AUDIT=1` in the environment and
// prints a block that begins `SURFACE`.
import 'dart:io' show Platform;
import 'dart:math' as math;

import 'package:flutter/rendering.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'package:motolii_ui/inspector/inspector_panel.dart' show ParamCell;
import 'package:motolii_ui/inspector/inspector_seat.dart';
import 'package:motolii_ui/timeline/timeline_view.dart';
import 'package:motolii_ui/main.dart' as app;

Future<void> frames(WidgetTester t, [int n = 20]) async {
  for (var i = 0; i < n; i++) {
    await t.pump(const Duration(milliseconds: 50));
  }
}

const _controls = {'ValueToy', 'ValuesToy', 'PairToy', 'ToggleToy', 'ChoiceToy', 'TextToy', 'ReferenceToy', 'RouteToy', 'ActionChip', 'SearchField'};

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  final on = Platform.environment['SURFACE_AUDIT'] != null;

  testWidgets('where the Inspector\'s area goes', skip: !on, (t) async {
    app.main();
    await frames(t, 60);
    final c = t.widget<LiveTimeline>(find.byType(LiveTimeline)).c;
    final shape = c.layers.firstWhere((l) => l['name'] == 'Rounded Rectangle');
    await c.command('select', {'ids': [shape['id']]});
    await frames(t, 40);

    final seatEl = t.element(find.byType(RightSeat));
    final seat = seatEl.renderObject! as RenderBox;
    final origin = seat.localToGlobal(Offset.zero);
    final w = seat.size.width.floor();
    final h = math.min(seat.size.height, 700.0).floor();
    final grid = List<int>.filled(w * h, 0); // 0 empty, 1 text, 2 control, 3 face (the gizmo)

    Rect? rectOf(Element e) {
      final r = e.renderObject;
      if (r is! RenderBox || !r.attached || !r.hasSize) return null;
      return (r.localToGlobal(Offset.zero) - origin) & r.size;
    }

    void paint(Rect r, int k) {
      final x0 = r.left.floor().clamp(0, w), x1 = r.right.ceil().clamp(0, w);
      final y0 = r.top.floor().clamp(0, h), y1 = r.bottom.ceil().clamp(0, h);
      for (var y = y0; y < y1; y++) {
        for (var x = x0; x < x1; x++) {
          grid[y * w + x] = k;
        }
      }
    }

    final texts = <Rect>[], ctrls = <Rect>[], faces = <Rect>[];
    final cellRects = <Rect>[], labelH = <double>[], toyH = <double>[];
    final cards = <Rect>[];
    int sections = 0;
    void walk(Element e, bool inControl) {
      final name = e.widget.runtimeType.toString();
      final r = rectOf(e);
      var inC = inControl;
      if (!inC && _controls.contains(name) && r != null) {
        ctrls.add(r);
        inC = true;
      }
      if (name == 'TransformGizmo' && r != null) faces.add(r);
      if (e.renderObject is RenderParagraph && !inC && r != null) texts.add(r);
      final k = e.widget.key;
      if (k is ValueKey<String>) {
        if (k.value.startsWith('effect-card:') && r != null) cards.add(r);
        if (k.value.startsWith('tone-')) sections++;
      }
      e.visitChildren((c) => walk(c, inC));
    }

    walk(seatEl, false);

    for (final e in find.byType(ParamCell).evaluate()) {
      final r = rectOf(e);
      if (r == null || r.top >= h) continue;
      cellRects.add(r);
      // the cell's Column: [label row, gap, toy, ...]
      e.visitChildren((col) {
        final kids = <Rect>[];
        col.visitChildren((k) {
          final kr = rectOf(k);
          if (kr != null) kids.add(kr);
        });
        if (kids.length >= 3) {
          labelH.add(kids[0].height);
          toyH.add(kids[2].height);
        }
      });
    }

    for (final r in texts) { paint(r, 1); }
    for (final r in ctrls) { paint(r, 2); }
    for (final r in faces) { paint(r, 3); }
    final count = List<int>.filled(4, 0);
    for (final v in grid) { count[v]++; }
    final area = (w * h).toDouble();
    String pct(int n) => '${(100 * n / area).toStringAsFixed(1)}%';
    double med(List<double> v) {
      if (v.isEmpty) return 0;
      final s = [...v]..sort();
      return s[s.length ~/ 2];
    }

    final visibleCtrls = ctrls.where((r) => r.top < h && r.bottom > 0).length;
    final meaningful = ctrls.where((r) => r.bottom <= h + 0.5 && r.top >= 0).length;
    // ignore: avoid_print
    print('SURFACE region ${w}x$h  (seat ${seat.size.width.toStringAsFixed(0)}x${seat.size.height.toStringAsFixed(0)})');
    // ignore: avoid_print
    print('SURFACE area  control ${pct(count[2])}  text-outside-control ${pct(count[1])}  face(gizmo) ${pct(count[3])}  empty(housing,gaps,padding) ${pct(count[0])}');
    // ignore: avoid_print
    print('SURFACE controls fully visible $meaningful (touching region $visibleCtrls), param cells ${cellRects.length}, effect cards ${cards.length}, section headers $sections');
    // ignore: avoid_print
    print('SURFACE param cell: median height ${med([for (final r in cellRects) r.height]).toStringAsFixed(1)}  label row ${med(labelH).toStringAsFixed(1)}  control ${med(toyH).toStringAsFixed(1)}');
    final cardH = [for (final r in cards) r.height];
    // ignore: avoid_print
    print('SURFACE effect cards: heights ${cardH.map((v) => v.toStringAsFixed(0)).toList()}  cell heights inside: ${[for (final r in cellRects) r.height.toStringAsFixed(0)]}');
    if (faces.isNotEmpty) {
      // ignore: avoid_print
      print('SURFACE gizmo: ${faces.first.width.toStringAsFixed(0)}x${faces.first.height.toStringAsFixed(0)} = ${(100 * faces.first.width * faces.first.height / area).toStringAsFixed(1)}% of the region');
    }
    final rows = <String>[];
    for (var y = 0; y < h; y += 1) {
      var ink = 0;
      for (var x = 0; x < w; x++) {
        if (grid[y * w + x] != 0) ink++;
      }
      rows.add(ink == 0 ? '.' : (ink < w * .5 ? '-' : '#'));
    }
    // ignore: avoid_print
    print('SURFACE rows (per y: . empty row, - partly used, # mostly used) ${rows.join()}');
    final empty = rows.where((r) => r == '.').length;
    // ignore: avoid_print
    print('SURFACE full-width empty rows: $empty of $h px (${(100 * empty / h).toStringAsFixed(1)}%)');
  });
}
