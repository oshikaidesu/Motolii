import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/sets/inspector/inspector_direct_transform.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';

Future<void> mount(WidgetTester tester, Doc doc, {bool locked = false, bool panel = false}) async {
  await tester.pumpWidget(
    WidgetsApp(
      color: const Color(0xFF191919),
      builder: (context, _) => Center(
        child: ListenableBuilder(
          listenable: doc,
          builder: (context, _) => Ctx(
            cfg: Cfg(locked: locked),
            doc: doc,
            child: panel
                ? const InspectorPanel(directTransform: true, height: 560)
                : const SizedBox(width: 348, child: SingleChildScrollView(child: DirectTransform())),
          ),
        ),
      ),
    ),
  );
  await tester.pump();
}

Finder get canvas => find.descendant(of: find.byType(TransformCanvas), matching: find.byType(CustomPaint)).first;

/// The canvas layout for the current values, in global coordinates.
(Rect, TransformLayout) layout(WidgetTester tester, Doc doc) {
  final rect = tester.getRect(canvas);
  return (rect, TransformLayout(rect.size, Map.of(doc.v)));
}

Offset at(WidgetTester tester, Doc doc, Offset Function(TransformLayout) of) {
  final (rect, g) = layout(tester, doc);
  return rect.topLeft + of(g);
}

void main() {
  testWidgets('object drag updates the same precision fields and one undo restores it', (tester) async {
    final doc = Doc(Kind.shape);
    await mount(tester, doc);
    final start = doc.get('pos.x')!;
    final gesture = await tester.startGesture(at(tester, doc, (g) => g.world(Offset(-g.half.dx * .6, g.half.dy * .6))));
    await gesture.moveBy(const Offset(12, 5));
    await tester.pump();
    expect(doc.get('pos.x'), greaterThan(start));
    expect(doc.get('pos.y'), greaterThan(540));
    expect(find.textContaining(doc.get('pos.x')!.toStringAsFixed(1), findRichText: true), findsOneWidget);
    await gesture.up();
    await tester.pump();
    expect(doc.undoSteps, 1);
    doc.undoGesture();
    await tester.pump();
    expect(doc.get('pos.x'), start);
    expect(doc.get('pos.y'), 540);
    expect(doc.undoSteps, 0);
    await tester.pumpWidget(const SizedBox());
    doc.dispose();
  });

  testWidgets('Esc restores values and key states and ignores the rest of the drag', (tester) async {
    final doc = Doc(Kind.shape);
    doc.setKey('pos', KeyS.anim);
    await mount(tester, doc, panel: true);
    final start = doc.get('pos.x');
    final gesture = await tester.startGesture(at(tester, doc, (g) => g.world(Offset(-g.half.dx * .6, g.half.dy * .6))));
    await tester.pump();
    await gesture.moveBy(const Offset(10, 0));
    await tester.pump();
    expect(doc.keys['pos'], KeyS.at);
    await tester.sendKeyEvent(LogicalKeyboardKey.escape);
    await tester.pump();
    await gesture.moveBy(const Offset(10, 0));
    await gesture.up();
    await tester.pump();
    expect(doc.get('pos.x'), start);
    expect(doc.keys['pos'], KeyS.anim);
    expect(doc.undoSteps, 0);
    await tester.pumpWidget(const SizedBox());
    doc.dispose();
  });

  testWidgets('locked and mixed canvases never edit the document', (tester) async {
    for (final locked in [true, false]) {
      final doc = Doc(locked ? Kind.shape : Kind.several, count: 3);
      await mount(tester, doc, locked: locked);
      final start = Map.of(doc.v);
      await tester.dragFrom(tester.getRect(canvas).center, const Offset(20, 10));
      await tester.pump();
      expect(doc.v, start);
      expect(doc.undoSteps, 0);
      if (!locked) expect(doc.get('pos.x'), isNull);
      await tester.pumpWidget(const SizedBox());
      doc.dispose();
    }
  });

  testWidgets('linked edge handle scales both axes and pointer cancellation restores both', (tester) async {
    final doc = Doc(Kind.shape);
    await mount(tester, doc);
    final gesture = await tester.startGesture(at(tester, doc, (g) => g.handles[7]));
    await gesture.moveBy(const Offset(0, 10));
    await tester.pump();
    expect(doc.get('scale.x'), greaterThan(100));
    expect(doc.get('scale.x'), doc.get('scale.y'));
    await gesture.cancel();
    await tester.pump();
    expect(doc.get('scale.x'), 100);
    expect(doc.get('scale.y'), 100);
    expect(doc.undoSteps, 0);
    await tester.pumpWidget(const SizedBox());
    doc.dispose();
  });

  testWidgets('rotation knob retains turns and pivot drag moves the actual anchor', (tester) async {
    final doc = Doc(Kind.shape);
    await mount(tester, doc);
    final (rect, g) = layout(tester, doc);
    final pivot = rect.topLeft + g.pivot;
    final knob = rect.topLeft + g.knob;
    final r = (knob - pivot).distance;
    final turn = await tester.startGesture(knob);
    final a = (knob - pivot).direction + math.pi / 2;
    await turn.moveTo(pivot + Offset(math.cos(a), math.sin(a)) * r);
    await tester.pump();
    await turn.up();
    expect(doc.get('rot.z'), closeTo(540, .001));
    expect(doc.keys['rot'], KeyS.at);
    expect(doc.undoSteps, 1);
    await tester.dragFrom(at(tester, doc, (g) => g.pivot), const Offset(-10, 0));
    await tester.pump();
    expect(doc.get('anchor.x'), isNot(50));
    expect(doc.undoSteps, 2);
    doc.undoGesture();
    await tester.pump();
    expect(doc.get('anchor.x'), 50);
    expect(doc.get('rot.z'), closeTo(540, .001));
    await tester.pumpWidget(const SizedBox());
    doc.dispose();
  });

  testWidgets('hovering a zone highlights its precision row', (tester) async {
    final doc = Doc(Kind.shape);
    await mount(tester, doc);
    final mouse = await tester.createGesture(kind: PointerDeviceKind.mouse);
    await mouse.addPointer(location: Offset.zero);
    await mouse.moveTo(at(tester, doc, (g) => g.knob));
    await tester.pump();
    expect(doc.directHover, 'rot');
    await mouse.moveTo(at(tester, doc, (g) => g.handles[0]));
    await tester.pump();
    expect(doc.directHover, 'scale');
    await mouse.moveTo(Offset.zero);
    await tester.pump();
    expect(doc.directHover, isNull);
    await mouse.removePointer();
    await tester.pumpWidget(const SizedBox());
    doc.dispose();
  });

  testWidgets('precision changes repaint the canvas and retain keys and other tabs', (tester) async {
    final doc = Doc(Kind.shape);
    await mount(tester, doc, panel: true);
    final field = find.byWidgetPredicate((w) => w is NumField && w.id == 'pos.x');
    await tester.tap(field);
    await tester.pump(const Duration(milliseconds: 350));
    await tester.pump();
    await tester.enterText(find.byType(EditableText), '600');
    await tester.sendKeyEvent(LogicalKeyboardKey.enter);
    await tester.pump();
    expect(doc.get('pos.x'), 600);
    doc.set('rot.z', 480);
    await tester.pump();
    expect(find.byType(TransformCanvas), findsOneWidget);
    // one stack, no tabs: transform, material and effects sit together; Relations is gone
    expect(find.text('Relations'), findsNothing);
    expect(find.byType(RelationsBlock), findsNothing);
    expect(find.byType(FillStroke), findsOneWidget);
    expect(find.byType(EffectsStack), findsOneWidget);
    expect(find.byType(KeyMark), findsNothing);
    expect(doc.keys['pos'], isNot(KeyS.off));
    // a titled card folds from its title and keeps its values
    await tester.tap(find.text('Transform'));
    await tester.pump();
    expect(find.byType(TransformCanvas), findsNothing);
    await tester.tap(find.text('Transform'));
    await tester.pump();
    expect(find.byType(TransformCanvas), findsOneWidget);
    expect(doc.get('rot.z'), 480);
    expect(doc.keys['rot'], KeyS.at);
    await tester.pumpWidget(const SizedBox());
    doc.dispose();
  });

  testWidgets('every space shows the same parameters', (tester) async {
    final doc = Doc(Kind.shape);
    await mount(tester, doc);
    List<String> fields() => [for (final w in tester.widgetList<NumField>(find.byType(NumField))) w.id];
    final flat = fields();
    expect(flat, containsAll(['pos.z', 'rot.x', 'rot.y', 'scale.z', 'depth']));
    for (final s in [Space.d25, Space.d3]) {
      doc
        ..space = s
        ..poke();
      await tester.pump();
      expect(fields(), flat);
    }
    await tester.pumpWidget(const SizedBox());
    doc.dispose();
  });

  testWidgets('2D keeps Z: the rail edits it and the layer keeps its size', (tester) async {
    final doc = Doc(Kind.shape);
    await mount(tester, doc);
    expect(find.byWidgetPredicate((w) => w is NumField && w.id == 'pos.z'), findsOneWidget);
    final (rect, g) = layout(tester, doc);
    final corners = g.corners;
    final z = doc.get('pos.z')!;
    final gesture = await tester.startGesture(rect.topLeft + g.liftKnob);
    await gesture.moveBy(const Offset(0, -12));
    await gesture.up();
    await tester.pump();
    expect(doc.get('pos.z'), lessThan(z));
    expect(layout(tester, doc).$2.corners, corners);
    await tester.pumpWidget(const SizedBox());
    doc.dispose();
  });

  testWidgets('the anchor card is a box of 27 points: a click puts the anchor there, one undo each, and it folds to its name', (tester) async {
    final doc = Doc(Kind.shape);
    await mount(tester, doc, panel: true);
    final cube = find.byType(AnchorCube);
    await tester.ensureVisible(cube);
    await tester.pump();
    final rect = tester.getRect(cube);
    final g = AnchorGeometry(rect.size);
    Future<void> tapAt(double x, double y, double z) async {
      await tester.tapAt(rect.topLeft + g.at(x, y, z));
      await tester.pump(const Duration(milliseconds: 400));
    }

    await tapAt(0, 0, -50);
    expect((doc.get('anchor.x'), doc.get('anchor.y'), doc.get('anchor.z')), (0, 0, -50));
    await tapAt(100, 50, 50);
    expect((doc.get('anchor.x'), doc.get('anchor.y'), doc.get('anchor.z')), (100, 50, 50));
    expect(doc.undoSteps, 2);
    doc.undoGesture();
    expect((doc.get('anchor.x'), doc.get('anchor.y'), doc.get('anchor.z')), (0, 0, -50));

    // a drag turns the view and edits nothing; a double-click squares it again
    final turn = await tester.startGesture(rect.topLeft + const Offset(12, 12));
    await turn.moveBy(const Offset(60, 20));
    await turn.up();
    await tester.pump();
    expect((doc.get('anchor.x'), doc.get('anchor.y'), doc.get('anchor.z')), (0, 0, -50));
    expect(doc.undoSteps, 1);
    await tester.tapAt(rect.topLeft + const Offset(12, 12));
    await tester.pump(const Duration(milliseconds: 50));
    await tester.tapAt(rect.topLeft + const Offset(12, 12));
    await tester.pump(const Duration(milliseconds: 400));
    await tapAt(50, 50, 0);
    expect((doc.get('anchor.x'), doc.get('anchor.y'), doc.get('anchor.z')), (50, 50, 0));
    doc.undoGesture();
    await tester.pump();
    await tester.ensureVisible(find.text('Anchor'));
    await tester.pump();
    await tester.tap(find.text('Anchor'));
    await tester.pump();
    expect(cube, findsNothing);
    expect(find.text('Top left · Front'), findsOneWidget);
    await tester.tap(find.text('Centre'));
    await tester.pump();
    expect(find.text('Centre · Middle'), findsOneWidget);
    await tester.pumpWidget(const SizedBox());
    doc.dispose();
  });

  testWidgets('outside 2D the same front view adds a Z rail and tilt knobs that edit their rows', (tester) async {
    final doc = Doc(Kind.shape);
    await mount(tester, doc, panel: true);
    await tester.tap(find.byWidgetPredicate((w) => w is SpacePlate && w.space == Space.d25));
    await tester.pump();
    expect(doc.space, Space.d25);
    TransformLayout g() => TransformLayout(tester.getRect(canvas).size, Map.of(doc.v), space: doc.space);
    Offset global(Offset p) => tester.getRect(canvas).topLeft + p;
    Future<void> pull(Offset from, Offset by) async {
      final gesture = await tester.startGesture(global(from));
      await gesture.moveBy(by);
      await gesture.up();
      await tester.pump();
    }

    final z = doc.get('pos.z')!;
    await pull(g().liftKnob, const Offset(0, -12));
    expect(doc.get('pos.z'), lessThan(z));
    final rx = doc.get('rot.x')!, ry = doc.get('rot.y')!;
    await pull(g().tiltKnob(1), const Offset(0, -6));
    expect(doc.get('rot.x'), greaterThan(rx));
    await pull(g().tiltKnob(2), const Offset(6, 0));
    expect(doc.get('rot.y'), greaterThan(ry));
    final sx = doc.get('scale.x')!;
    final out = g().handles[5] - g().pivot;
    await pull(g().handles[5], out / out.distance * 8);
    expect(doc.get('scale.x'), isNot(sx));
    expect(doc.undoSteps, 4);
    await tester.pumpWidget(const SizedBox());
    doc.dispose();
  });

  testWidgets('Shift-click picks a rectangle of fields; typing sets them all, a scrub moves them together, Esc lets go', (tester) async {
    final doc = Doc(Kind.shape);
    await mount(tester, doc, panel: true);
    Finder field(String id) => find.byWidgetPredicate((w) => w is NumField && w.id == id);
    await tester.tap(field('pos.x'));
    await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
    await tester.tap(field('scale.y'));
    await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
    await tester.pump(const Duration(milliseconds: 400));
    expect(doc.picked, {'pos.x', 'pos.y', 'scale.x', 'scale.y'});

    final before = {for (final id in doc.picked) id: doc.get(id)!};
    final g = await tester.startGesture(tester.getCenter(field('pos.y')));
    await g.moveBy(const Offset(30, 0));
    await g.up();
    await tester.pump(const Duration(milliseconds: 400));
    final moved = doc.get('pos.y')! - before['pos.y']!;
    expect(moved, isNot(0));
    for (final id in doc.picked) {
      expect(doc.get(id)! - before[id]!, closeTo(moved, 1e-6));
    }
    expect(doc.get('pos.z'), 0);

    final ef = field('scale.x');
    await tester.tap(ef);
    await tester.pump(const Duration(milliseconds: 400));
    await tester.pump();
    await tester.enterText(find.byType(EditableText), '75');
    await tester.sendKeyEvent(LogicalKeyboardKey.enter);
    await tester.pump();
    for (final id in doc.picked) {
      expect(doc.get(id), 75);
    }

    await tester.sendKeyEvent(LogicalKeyboardKey.escape);
    await tester.pump();
    expect(doc.picked.length, lessThan(2));
    await tester.pumpWidget(const SizedBox());
    doc.dispose();
  });

  testWidgets('holding an effect head and dragging it reorders the stack', (tester) async {
    final doc = Doc(Kind.shape);
    await mount(tester, doc, panel: true);
    expect(doc.fxOrder, ['blur', 'kaleido']);
    final kal = find.text('Kaleido');
    await tester.ensureVisible(kal);
    await tester.pumpAndSettle();
    final from = tester.getCenter(kal), to = tester.getTopLeft(find.text('Blur')) + const Offset(4, 2);
    final g = await tester.startGesture(from);
    await tester.pump(const Duration(milliseconds: 300));
    await g.moveTo(Offset(from.dx, (from.dy + to.dy) / 2));
    await tester.pump();
    await g.moveTo(Offset(from.dx, to.dy));
    await tester.pump();
    await g.up();
    await tester.pump();
    expect(doc.fxOrder, ['kaleido', 'blur']);
    await tester.pumpWidget(const SizedBox());
    doc.dispose();
  });

  testWidgets('an unknown shader gets controls that move the way their values mean', (tester) async {
    final doc = Doc(Kind.shape);
    doc.fxOrder
      ..clear()
      ..add('kaleido');
    for (final t in ['Transform', 'Layer', 'Fill and stroke', 'Blend']) {
      doc.flag('fold.$t', true);
    }
    doc.flag('anchor.open', false);
    await mount(tester, doc, panel: true);

    final pad = find.byWidgetPredicate((w) => w.runtimeType.toString() == '_PadGlyph').first;
    await tester.ensureVisible(pad);
    await tester.pumpAndSettle();
    final p0 = await tester.startGesture(tester.getCenter(pad));
    await p0.moveBy(const Offset(20, 10));
    await p0.up();
    await tester.pump();
    expect(doc.get('kaleido_center.x'), greaterThan(.5), reason: 'sideways moves X');
    expect(doc.get('kaleido_center.y'), greaterThan(.5), reason: 'down moves Y down the canvas');
    doc.undoGesture();
    expect(doc.get('kaleido_center.x'), .5);
    expect(doc.get('kaleido_center.y'), .5);

    final drift = find.byWidgetPredicate((w) => w is NumField && w.caption == 'Drift Y');
    final before = doc.get('kaleido_driftY')!;
    final g = await tester.startGesture(tester.getCenter(drift));
    await g.moveBy(const Offset(0, -40));
    await tester.pump();
    expect(doc.get('kaleido_driftY'), before, reason: 'every number scrubs sideways, a Y too');
    await g.moveBy(const Offset(40, 0));
    await g.up();
    await tester.pump(const Duration(milliseconds: 400));
    expect(doc.get('kaleido_driftY'), greaterThan(before));

    final cols = tester.getTopLeft(find.byWidgetPredicate((w) => w is NumField && w.caption == 'Twist')).dy;
    expect(tester.getTopLeft(find.byWidgetPredicate((w) => w is NumField && w.caption == 'Segments')).dy, cols, reason: 'blocks share a row');

    await tester.tap(find.text('+'));
    await tester.pump();
    expect(doc.get('kaleido_segments'), 7);

    await tester.pumpWidget(const SizedBox());
    doc.dispose();
  });

  testWidgets('hover and slide sideways: right adds, left takes away, up and down leave the value alone', (tester) async {
    final doc = Doc(Kind.shape);
    await mount(tester, doc, panel: true);
    final field = find.descendant(of: find.byType(LayerTiles), matching: find.byType(NumField)).first;
    await tester.ensureVisible(field);
    await tester.pump();
    final at = tester.getCenter(field), start = doc.get('op')!;
    final mouse = TestPointer(1, PointerDeviceKind.mouse)..hover(at);
    Future<void> scroll(Offset d) async {
      await tester.sendEventToBinding(mouse.scroll(d));
      await tester.pump();
    }

    // opacity scrubs at .5 per px; a slide of the same length moves it the same amount
    await scroll(const Offset(20, 0));
    expect(doc.get('op'), start + 10);
    await scroll(const Offset(-40, 0));
    expect(doc.get('op'), start - 10);
    await scroll(const Offset(0, 120));
    expect(doc.get('op'), start - 10, reason: 'an up/down slide scrolls the panel instead');

    final pad = TestPointer(2, PointerDeviceKind.trackpad);
    await tester.sendEventToBinding(pad.panZoomStart(at));
    await tester.sendEventToBinding(pad.panZoomUpdate(at, pan: const Offset(20, 0)));
    await tester.sendEventToBinding(pad.panZoomEnd());
    await tester.pump();
    expect(doc.get('op'), start);

    await tester.pumpWidget(const SizedBox());
    doc.dispose();
  });

  testWidgets('a chooser list floats over the panel and never pushes the cards below', (tester) async {
    final doc = Doc(Kind.shape);
    await mount(tester, doc, panel: true);
    final parent = find.descendant(of: find.byType(LayerTiles), matching: find.byType(Chooser));
    await tester.ensureVisible(parent);
    await tester.pump();
    final below = find.text('Fill and stroke');
    final y = tester.getTopLeft(below).dy;

    await tester.tap(parent);
    await tester.pump();
    expect(find.text('Search'), findsOneWidget);
    expect(tester.getTopLeft(below).dy, y);

    final option = find.text('Backdrop');
    expect(option, findsOneWidget);
    await tester.tap(option);
    await tester.pump();
    expect(doc.s2['parent'], 'Backdrop');
    expect(find.text('Search'), findsNothing);
    expect(tester.getTopLeft(below).dy, y);

    await tester.pumpWidget(const SizedBox());
    doc.dispose();
  });
}
