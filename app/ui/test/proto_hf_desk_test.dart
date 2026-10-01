// Desk prototypes: the direct manipulation each face promises, driven with real pointer events,
// plus a pixel check that blend previews actually differ per operation.
import 'dart:ui' as ui;
import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_stage5/hf/desk/blend.dart';
import 'package:motolii_stage5/hf/desk/depth.dart';
import 'package:motolii_stage5/hf/desk/ease.dart';
import 'package:motolii_stage5/hf/desk/history.dart';
import 'package:motolii_stage5/hf/desk/notes.dart';

Widget host(Widget child, double w, double h) => WidgetsApp(
      color: const Color(0xFF000000),
      pageRouteBuilder: <T>(settings, builder) => PageRouteBuilder<T>(settings: settings, pageBuilder: (c, _, __) => builder(c)),
      home: Align(alignment: Alignment.topLeft, child: SizedBox(width: w, height: h, child: child)),
    );

Future<void> show(WidgetTester t, Widget child, double w, double h) async {
  addTearDown(() => t.pump(const Duration(seconds: 1)));
  t.view.physicalSize = const Size(900, 900);
  t.view.devicePixelRatio = 1;
  addTearDown(t.view.reset);
  await t.pumpWidget(host(child, w, h));
  await t.pump();
}

void main() {
  setUpAll(() async {
    final inter = FontLoader('Inter')
      ..addFont(rootBundle.load('assets/fonts/Inter-Regular.ttf'))
      ..addFont(rootBundle.load('assets/fonts/Inter-Medium.ttf'))
      ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.ttf'));
    await inter.load();
  });

  group('every Desk lays out at wide, strip and tall without error', () {
    final desks = <String, Widget Function()>{
      'Ease': () => const EaseDesk(),
      'Depth': () => const DepthDesk(),
      'Blend': () => const BlendDesk(),
      'History': () => const HistoryDesk(),
      'Notes': () => const NotesDesk(),
    };
    for (final e in desks.entries) {
      for (final (label, w, h) in [('wide', 310.0, 640.0), ('strip', 310.0, 120.0), ('tall', 150.0, 300.0)]) {
        testWidgets('${e.key} $label', (t) async {
          await show(t, e.value(), w, h);
          expect(t.takeException(), isNull);
        });
      }
    }
  });

  group('Ease', () {
    testWidgets('dragging a handle moves the numbers and makes the interval a custom bezier', (t) async {
      await show(t, const EaseDesk(), 310, 640);
      final rect = t.getRect(find.byKey(const ValueKey('ease-plot')));
      expect(find.text('0.25'), findsOneWidget);
      final h1 = rect.topLeft + easeAt(rect.size, .25, .10);
      await t.dragFrom(h1, const Offset(60, -50));
      await t.pump();
      expect(find.text('0.25'), findsNothing);
      expect(find.text('0.10'), findsNothing);
      expect(t.takeException(), isNull);
    });

    testWidgets('interval selection shows that interval\'s own curve values, All is a mixed state', (t) async {
      await show(t, const EaseDesk(), 310, 640);
      await t.ensureVisible(find.byKey(const ValueKey('ease-seg-1')));
      await t.tap(find.byKey(const ValueKey('ease-seg-1')));
      await t.pump();
      expect(find.text('0.42'), findsOneWidget); // second interval is the Ease preset
      await t.ensureVisible(find.byKey(const ValueKey('ease-all')));
      await t.tap(find.byKey(const ValueKey('ease-all')));
      await t.pump();
      expect(find.text('—'), findsNWidgets(4));
    });

    testWidgets('choosing Spring drops handles: a drag on the plot changes nothing', (t) async {
      await show(t, const EaseDesk(), 310, 640);
      await t.ensureVisible(find.byKey(const ValueKey('ease-preset-3')));
      await t.tap(find.byKey(const ValueKey('ease-preset-3')));
      await t.pump();
      await t.ensureVisible(find.byKey(const ValueKey('ease-plot')));
      await t.pump();
      final rect = t.getRect(find.byKey(const ValueKey('ease-plot')));
      final before = find.text('0.25').evaluate().length;
      await t.dragFrom(rect.topLeft + easeAt(rect.size, .25, .1), const Offset(40, -40));
      await t.pump();
      expect(find.text('0.25').evaluate().length, before);
    });

    testWidgets('Sequence ghosts toggle without disturbing the plot', (t) async {
      await show(t, const EaseDesk(), 310, 640);
      await t.ensureVisible(find.byKey(const ValueKey('ease-seq')));
      await t.tap(find.byKey(const ValueKey('ease-seq')));
      await t.pump();
      expect(t.takeException(), isNull);
    });

    testWidgets('hovering a preset peeks it on the plot and leaving restores; Enter applies from the keyboard', (t) async {
      await show(t, const EaseDesk(), 310, 640);
      await t.ensureVisible(find.byKey(const ValueKey('ease-preset-3')));
      await t.pump();
      final mouse = await t.createGesture(kind: PointerDeviceKind.mouse);
      await mouse.addPointer(location: const Offset(890, 890));
      addTearDown(mouse.removePointer);
      await mouse.moveTo(t.getCenter(find.byKey(const ValueKey('ease-preset-3'))));
      await t.pump();
      expect(t.takeException(), isNull);
      await mouse.moveTo(const Offset(890, 890));
      await t.pump();
      // keyboard: focus the row by clicking Linear, arrow to the next, Enter applies it
      await t.tap(find.byKey(const ValueKey('ease-preset-0')));
      await t.sendKeyEvent(LogicalKeyboardKey.arrowRight);
      await t.sendKeyEvent(LogicalKeyboardKey.enter);
      await t.pump();
      await t.ensureVisible(find.byKey(const ValueKey('ease-seg-0')));
      expect(find.text('0.42'), findsOneWidget); // Ease preset values are now in the numbers
    });

    testWidgets('Copy curve saves the current curve; saved applies it; Clear empties the list', (t) async {
      await show(t, const EaseDesk(), 310, 640);
      await t.ensureVisible(find.byKey(const ValueKey('ease-copy')));
      expect(find.byKey(const ValueKey('ease-saved-1')), findsOneWidget);
      await t.tap(find.byKey(const ValueKey('ease-copy')));
      await t.pump();
      expect(find.byKey(const ValueKey('ease-saved-2')), findsOneWidget);
      await t.tap(find.byKey(const ValueKey('ease-saved-0')));
      await t.pump();
      expect(find.text('0.70'), findsOneWidget);
      await t.tap(find.byKey(const ValueKey('ease-clear')));
      await t.pump();
      expect(find.byKey(const ValueKey('ease-saved-0')), findsNothing);
    });

    testWidgets('audition plays and stops', (t) async {
      await show(t, const EaseDesk(), 310, 640);
      await t.ensureVisible(find.byKey(const ValueKey('ease-play')));
      await t.tap(find.byKey(const ValueKey('ease-play')));
      await t.pump(const Duration(milliseconds: 300));
      await t.tap(find.byKey(const ValueKey('ease-play')));
      await t.pump();
      expect(t.takeException(), isNull);
    });

    testWidgets('narrow Ease keeps the curve first and every control reachable by scrolling', (t) async {
      await show(t, const EaseDesk(), 310, 120);
      await t.ensureVisible(find.byKey(const ValueKey('ease-copy')));
      await t.pump();
      expect(find.byKey(const ValueKey('ease-copy')), findsOneWidget);
      expect(t.takeException(), isNull);
    });
  });

  group('primary loop stays in one viewport', () {
    testWidgets('Ease: curve, presets, intervals and play share the viewport, and using them never moves the curve', (t) async {
      await show(t, const EaseDesk(), 310, 640);
      final plot = t.getRect(find.byKey(const ValueKey('ease-plot')));
      for (final k in ['ease-preset-0', 'ease-preset-4', 'ease-seg-0', 'ease-seg-2', 'ease-all', 'ease-play', 'ease-prev', 'ease-next']) {
        final r = t.getRect(find.byKey(ValueKey(k)));
        expect(r.bottom, lessThanOrEqualTo(640), reason: '$k is below the panel viewport');
        expect(r.top, greaterThan(plot.bottom - 1), reason: '$k should sit right under the curve');
      }
      // three rounds of the loop: pick an interval, try presets, play
      for (var round = 0; round < 3; round++) {
        await t.tap(find.byKey(ValueKey('ease-seg-${round % 3}')));
        await t.tap(find.byKey(ValueKey('ease-preset-${(round + 1) % 5}')));
        await t.tap(find.byKey(const ValueKey('ease-next')));
        await t.tap(find.byKey(const ValueKey('ease-play')));
        await t.pump(const Duration(milliseconds: 200));
        await t.tap(find.byKey(const ValueKey('ease-play')));
        await t.pump();
        expect(t.getRect(find.byKey(const ValueKey('ease-plot'))), plot, reason: 'the curve moved while looping');
      }
      // secondary precision still scrolls
      await t.ensureVisible(find.byKey(const ValueKey('ease-copy')));
      expect(t.takeException(), isNull);
    });

    testWidgets('Ease keeps its loop in the viewport at the tall narrow size too', (t) async {
      await show(t, const EaseDesk(), 150, 300);
      final plot = t.getRect(find.byKey(const ValueKey('ease-plot')));
      for (final k in ['ease-preset-0', 'ease-preset-4', 'ease-seg-0', 'ease-play']) {
        expect(t.getRect(find.byKey(ValueKey(k))).bottom, lessThanOrEqualTo(300), reason: k);
      }
      await t.tap(find.byKey(const ValueKey('ease-preset-3')));
      await t.pump();
      expect(t.getRect(find.byKey(const ValueKey('ease-plot'))), plot);
    });

    testWidgets('Blend: the result stays put while the mode list scrolls', (t) async {
      await show(t, const BlendDesk(), 310, 640);
      final before = t.getRect(find.byKey(const ValueKey('blend-result')));
      await t.ensureVisible(find.byKey(const ValueKey('blend-stage')));
      await t.pump();
      await t.tap(find.byKey(const ValueKey('blend-mark-18')));
      await t.pump();
      expect(t.getRect(find.byKey(const ValueKey('blend-result'))), before);
      expect(find.text('Silhouette'), findsWidgets);
    });

    testWidgets('History: Undo / Redo keep the current step in view in a short panel', (t) async {
      await show(t, const HistoryDesk(), 310, 300);
      for (var i = 0; i < 8; i++) {
        await t.tap(find.byKey(const ValueKey('history-undo')));
        await t.pump();
        await t.pump();
        await t.pump(const Duration(milliseconds: 250));
        final y = t.getCenter(find.byKey(const ValueKey('row-current'))).dy;
        expect(y, inInclusiveRange(58, 300 - 56), reason: 'current step left the visible rail');
      }
      for (var i = 0; i < 11; i++) {
        await t.tap(find.byKey(const ValueKey('history-redo')));
        await t.pump();
        await t.pump();
        await t.pump(const Duration(milliseconds: 250));
        final y = t.getCenter(find.byKey(const ValueKey('row-current'))).dy;
        expect(y, inInclusiveRange(58, 300 - 56), reason: 'current step left the visible rail');
      }
    });

    testWidgets('Notes: the canvas has no vertical scroll of its own', (t) async {
      await show(t, const NotesDesk(), 310, 640);
      expect(find.byType(Scrollable), findsNothing);
    });
  });

  group('Depth', () {
    testWidgets('dragging the camera in Top moves it and the readout follows', (t) async {
      await show(t, const DepthDesk(), 310, 640);
      final rect = t.getRect(find.byKey(const ValueKey('depth-diagram')));
      final g = DepthGeom(rect.size, DView.top, depthDefaultCam(), depthDefaultLayers(), pad: 26);
      expect(find.text('620'), findsOneWidget);
      await t.dragFrom(rect.topLeft + g.camPx(), const Offset(0, -120));
      await t.pump();
      expect(find.text('620'), findsNothing);
    });

    testWidgets('Side is a real projection: dragging the selected layer along depth changes its readout', (t) async {
      await show(t, const DepthDesk(), 310, 640);
      await t.tap(find.text('Side'));
      await t.pump();
      final rect = t.getRect(find.byKey(const ValueKey('depth-diagram')));
      final g = DepthGeom(rect.size, DView.side, depthDefaultCam(), depthDefaultLayers(), pad: 26);
      expect(find.text('660'), findsOneWidget);
      await t.dragFrom(rect.topLeft + g.layerPx(1), const Offset(30, 0));
      await t.pump();
      expect(find.text('660'), findsNothing);
    });

    testWidgets('Front shows the same scene and a layer drag there does not change its depth', (t) async {
      await show(t, const DepthDesk(), 310, 640);
      await t.tap(find.text('Front'));
      await t.pump();
      final rect = t.getRect(find.byKey(const ValueKey('depth-diagram')));
      final g = DepthGeom(rect.size, DView.front, depthDefaultCam(), depthDefaultLayers(), pad: 26);
      await t.dragFrom(rect.topLeft + g.layerPx(1), const Offset(20, 10));
      await t.pump();
      expect(find.text('660'), findsOneWidget);
    });

    testWidgets('tapping a layer selects it and the readout follows', (t) async {
      await show(t, const DepthDesk(), 310, 640);
      final rect = t.getRect(find.byKey(const ValueKey('depth-diagram')));
      final g = DepthGeom(rect.size, DView.top, depthDefaultCam(), depthDefaultLayers(), pad: 26);
      await t.tapAt(rect.topLeft + g.layerPx(2));
      await t.pump();
      expect(find.text('Layer 3'), findsOneWidget);
      expect(find.text('410'), findsOneWidget);
    });

    testWidgets('Depth owns no camera properties: no sliders, no dropdowns; a pointer to the Inspector instead', (t) async {
      await show(t, const DepthDesk(), 310, 640);
      expect(find.text('Field of View'), findsNothing);
      expect(find.text('Focus Layer'), findsNothing);
      expect(find.text('Depth of Field'), findsNothing);
      expect(find.textContaining('Inspector'), findsOneWidget);
    });
  });

  group('Blend', () {
    testWidgets('hover previews a mode, leaving restores it, click keeps it', (t) async {
      await show(t, const BlendDesk(), 310, 640);
      final mouse = await t.createGesture(kind: PointerDeviceKind.mouse);
      await mouse.addPointer(location: const Offset(890, 890));
      addTearDown(mouse.removePointer);
      expect(find.text('Screen'), findsOneWidget); // tile label only
      await mouse.moveTo(t.getCenter(find.byKey(const ValueKey('blend-mark-5'))));
      await t.pump();
      expect(find.text('Screen'), findsNWidgets(2)); // also as the result caption
      await mouse.moveTo(const Offset(890, 890));
      await t.pump();
      expect(find.text('Screen'), findsOneWidget);
      await t.tap(find.byKey(const ValueKey('blend-mark-5')));
      await mouse.moveTo(const Offset(890, 890));
      await t.pump();
      expect(find.text('Screen'), findsNWidgets(2));
    });

    testWidgets('Preview on Stage off: hover no longer changes the result', (t) async {
      await show(t, const BlendDesk(), 310, 640);
      await t.ensureVisible(find.byKey(const ValueKey('blend-stage')));
      await t.tap(find.byKey(const ValueKey('blend-stage')));
      await t.pump();
      await t.ensureVisible(find.byKey(const ValueKey('blend-mark-5')));
      final mouse = await t.createGesture(kind: PointerDeviceKind.mouse);
      await mouse.addPointer(location: const Offset(890, 890));
      addTearDown(mouse.removePointer);
      await mouse.moveTo(t.getCenter(find.byKey(const ValueKey('blend-mark-5'))));
      await t.pump();
      expect(find.text('Screen'), findsOneWidget);
    });

    testWidgets('targets that disagree are a mixed selection; picking a mode sets all of them', (t) async {
      await show(t, const BlendDesk(), 310, 640);
      await t.tap(find.byKey(const ValueKey('blend-target-1'))); // Layer 3 (Screen) joins Layer 2 (Multiply)
      await t.pump();
      expect(find.text('Targets differ'), findsOneWidget);
      await t.tap(find.byKey(const ValueKey('blend-mark-8'))); // Overlay
      await t.pump();
      expect(find.text('Targets differ'), findsNothing);
    });

    testWidgets('Blend owns no layer properties: no Opacity control here', (t) async {
      await show(t, const BlendDesk(), 310, 640);
      expect(find.text('Opacity'), findsNothing);
    });

    // The overlap of two fixed shapes is the whole message: it must differ per operation.
    testWidgets('each operation gives its own overlap on the same two shapes', (t) async {
      final overlap = <String, List<int>>{};
      await t.runAsync(() async {
        for (final name in ['Normal', 'Darken', 'Multiply', 'Lighten', 'Screen', 'Color Dodge', 'Overlay', 'Hard Light', 'Difference', 'Exclusion', 'Hue', 'Luminosity']) {
          final i = blendModes.indexWhere((m) => m.name == name);
          final rec = ui.PictureRecorder();
          paintResult(Canvas(rec), const Rect.fromLTWH(0, 0, 96, 64), i);
          final img = await rec.endRecording().toImage(96, 64);
          final bytes = (await img.toByteData())!.buffer.asUint8List();
          final o = (32 * 96 + 48) * 4; // the centre of the overlap
          overlap[name] = [bytes[o], bytes[o + 1], bytes[o + 2]];
        }
      });
      double lum(String n) => .3 * overlap[n]![0] + .59 * overlap[n]![1] + .11 * overlap[n]![2];
      expect(lum('Multiply'), lessThan(lum('Normal')));
      expect(lum('Screen'), greaterThan(lum('Normal')));
      final names = overlap.keys.toList();
      for (var a = 0; a < names.length; a++) {
        for (var b = a + 1; b < names.length; b++) {
          final d = [for (var k = 0; k < 3; k++) (overlap[names[a]]![k] - overlap[names[b]]![k]).abs()].reduce((x, y) => x + y);
          expect(d, greaterThan(12), reason: '${names[a]} vs ${names[b]} overlap looks the same (differs by $d)');
        }
      }
    });
  });

  group('History', () {
    testWidgets('clicking a step goes to it; Undo and Redo step by one', (t) async {
      await show(t, const HistoryDesk(), 310, 640);
      final y8 = t.getCenter(find.byKey(const ValueKey('row-current'))).dy;
      final y3 = t.getCenter(find.byKey(const ValueKey('row-3'))).dy;
      await t.tap(find.byKey(const ValueKey('row-3')));
      await t.pump();
      expect(t.getCenter(find.byKey(const ValueKey('row-current'))).dy, closeTo(y3, .5));
      expect(find.byKey(const ValueKey('row-8')), findsOneWidget); // row 8 is now in the redo tail
      await t.tap(find.byKey(const ValueKey('history-undo')));
      await t.pump();
      expect(t.getCenter(find.byKey(const ValueKey('row-current'))).dy, closeTo(y3 - 38, .5));
      await t.tap(find.byKey(const ValueKey('history-redo')));
      await t.tap(find.byKey(const ValueKey('history-redo')));
      await t.pump();
      expect(t.getCenter(find.byKey(const ValueKey('row-current'))).dy, closeTo(y3 + 38, .5));
      expect(y8, greaterThan(y3));
    });

    testWidgets('the strip rail is clickable too', (t) async {
      await show(t, const HistoryDesk(), 310, 120);
      await t.tapAt(t.getTopLeft(find.byKey(const ValueKey('history-strip'))) + const Offset(20, 20));
      await t.pump();
      expect(t.takeException(), isNull);
    });
  });

  group('Notes', () {
    // Double-tap recognizers hold a short timer; let it finish before the test ends.
    tearDown(() {});
    testWidgets('a block moves with the pointer at working scale', (t) async {
      await show(t, const NotesDesk(), 310, 640);
      final before = t.getTopLeft(find.byKey(const ValueKey('blk-1')));
      await t.drag(find.byKey(const ValueKey('blk-1')), const Offset(-30, 60));
      await t.pump();
      final after = t.getTopLeft(find.byKey(const ValueKey('blk-1')));
      expect((after - before).dx, closeTo(-30, 1));
      expect((after - before).dy, closeTo(60, 1));
      await t.pump(const Duration(seconds: 1));
    });

    testWidgets('tap selects, the handle resizes, tapping empty canvas deselects', (t) async {
      await show(t, const NotesDesk(), 310, 640);
      await t.tap(find.byKey(const ValueKey('blk-3')));
      await t.pump();
      expect(find.byKey(const ValueKey('resize')), findsOneWidget);
      final w0 = t.getSize(find.byKey(const ValueKey('blk-3'))).width;
      await t.drag(find.byKey(const ValueKey('resize')), const Offset(60, 0));
      await t.pump();
      expect(t.getSize(find.byKey(const ValueKey('blk-3'))).width, closeTo(w0 + 60, 1.5));
      await t.tapAt(t.getTopLeft(find.byKey(const ValueKey('notes-canvas'))) + const Offset(12, 380));
      await t.pump();
      expect(find.byKey(const ValueKey('resize')), findsNothing);
    });

    testWidgets('double tap edits a note in place', (t) async {
      await show(t, const NotesDesk(), 310, 640);
      await t.tap(find.byKey(const ValueKey('blk-1')));
      await t.pump(const Duration(milliseconds: 60));
      await t.tap(find.byKey(const ValueKey('blk-1')));
      await t.pump();
      await t.pump(const Duration(milliseconds: 400));
      expect(find.byKey(const ValueKey('note-edit')), findsOneWidget);
      await t.enterText(find.byKey(const ValueKey('note-edit')), 'Hold on the pose');
      await t.pump();
      expect(find.text('Hold on the pose'), findsOneWidget);
    });

    testWidgets('Delete removes the selected block', (t) async {
      await show(t, const NotesDesk(), 310, 640);
      await t.tap(find.byKey(const ValueKey('blk-4')));
      await t.pump(const Duration(milliseconds: 400));
      expect(find.text('Camera 1'), findsOneWidget);
      await t.sendKeyEvent(LogicalKeyboardKey.delete);
      await t.pump();
      expect(find.text('Camera 1'), findsNothing);
      await t.pump(const Duration(seconds: 1));
    });

    testWidgets('zoom buttons and canvas pan work', (t) async {
      await show(t, const NotesDesk(), 310, 640);
      await t.tap(find.byKey(const ValueKey('zoom-in')));
      await t.pump();
      expect(find.text('125%'), findsOneWidget);
      final before = t.getTopLeft(find.byKey(const ValueKey('blk-2')));
      await t.dragFrom(t.getTopLeft(find.byKey(const ValueKey('notes-canvas'))) + const Offset(290, 440), const Offset(-40, -20));
      await t.pump();
      final after = t.getTopLeft(find.byKey(const ValueKey('blk-2')));
      expect((after - before).dx, closeTo(-40, 1));
    });

    testWidgets('the narrow face is the same canvas: blocks are still there and still move', (t) async {
      await show(t, const NotesDesk(), 310, 120);
      expect(find.byKey(const ValueKey('notes-canvas-fit')), findsOneWidget);
      final before = t.getTopLeft(find.byKey(const ValueKey('blk-0')));
      await t.drag(find.byKey(const ValueKey('blk-0')), const Offset(60, 0));
      await t.pump();
      final after = t.getTopLeft(find.byKey(const ValueKey('blk-0')));
      expect((after - before).dx, closeTo(60, 1.5));
      await t.pump(const Duration(seconds: 1));
    });
  });
}
