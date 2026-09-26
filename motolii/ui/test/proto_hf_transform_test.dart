// The Transform Instrument, driven with real pointers and keys. The gizmo is for gesture, the Values are for precision;
// both write through one store, with Classic's semantics: relative scrubs across the selection, absolute typed numbers,
// locked layers refuse, scale kept in shape by default, Anchor and Parent on the shown layer, Space on the selection.
import 'dart:math' as math;
import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_stage5/proto_hf/desk/common.dart' show kBlue, kMint, kPink, kViolet;
import 'package:motolii_stage5/proto_hf/insp/transform.dart';
import 'package:motolii_stage5/proto_hf/insp/transform_gizmo.dart';
import 'package:motolii_stage5/proto_hf/insp/transform_model.dart';

Widget host(Widget child, double w, double h) => WidgetsApp(
      color: const Color(0xFF000000),
      pageRouteBuilder: <T>(settings, builder) => PageRouteBuilder<T>(settings: settings, pageBuilder: (c, _, __) => builder(c)),
      home: Align(alignment: Alignment.topLeft, child: SizedBox(width: w, height: h, child: child)),
    );

List<TLayer> layers() => [
      TLayer(1, 'Logo', position: [320, 180, 0], rotation: 32),
      TLayer(2, 'Title', position: [-120, 40, 60], scale: [1.4, .8, 1], rotation: 12, rotX: 20, rotY: -8, depth: 120, anchor: [0, 1], projection: '2.5D', parent: 1),
      TLayer(3, 'Shape 3', position: [40, -60, 0], rotation: 90, locked: true),
    ];

Future<TransformStore> open(WidgetTester t, {List<int>? sel, int? active, double w = 310, double h = 640, TMode mode = TMode.move}) async {
  t.view.physicalSize = const Size(900, 900);
  t.view.devicePixelRatio = 1;
  addTearDown(t.view.reset);
  final s = TransformStore(layers(), selection: sel, active: active);
  await t.pumpWidget(host(TransformInstrument(s, initialMode: mode), w, h));
  await t.pump();
  return s;
}

Finder k(String key) => find.byKey(ValueKey(key));
Future<void> realGap(WidgetTester t) async { await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 380))); }

/// Where things are in the gizmo, from the same numbers the painter uses.
class G {
  G(this.t, this.s);
  final WidgetTester t;
  final TransformStore s;
  Rect get rect => t.getRect(k('gizmo'));
  Offset get c => rect.center;
  TLayer get l => s.active;
  Offset get half => Offset(28 * l.scale[0].abs().clamp(.35, 2.2), 20 * l.scale[1].abs().clamp(.35, 2.2));
  double get rad => l.rotation * math.pi / 180;
  Offset rot(Offset v, double a) => Offset(v.dx * math.cos(a) - v.dy * math.sin(a), v.dx * math.sin(a) + v.dy * math.cos(a));
  Offset screen(Offset local) => c + rot(local, rad);
  double get ringR => (half.distance + 20).clamp(56.0, math.max(56.0, math.min(76.0, rect.shortestSide / 2 - 4)));
  Offset corner(int i) => screen([Offset(-half.dx, -half.dy), Offset(half.dx, -half.dy), Offset(half.dx, half.dy), Offset(-half.dx, half.dy)][i]);
  Offset edgeX() => screen(Offset(half.dx, 0));
  Offset anchor(double fx, double fy) => screen(Offset((fx - .5) * 2 * half.dx, (fy - .5) * 2 * half.dy));
  Offset tipX() => c + Offset(ringR, 0);
  Offset tipY() => c + Offset(0, ringR);
  Offset ringAt(double deg) => c + Offset(math.cos(deg * math.pi / 180), math.sin(deg * math.pi / 180)) * ringR;
}

Color? modeButton(WidgetTester t, String m) => ((t.widget<Container>(find.descendant(of: k('mode-$m'), matching: find.byType(Container)).first)).decoration as BoxDecoration).color;
dynamic pos(TransformStore s, [int layer = 1]) => s.layers.firstWhere((l) => l.id == layer).position;
TLayer L(TransformStore s, int id) => s.layers.firstWhere((l) => l.id == id);

void main() {
  setUpAll(() async {
    final inter = FontLoader('Inter')
      ..addFont(rootBundle.load('assets/fonts/Inter-Regular.ttf'))
      ..addFont(rootBundle.load('assets/fonts/Inter-Medium.ttf'))
      ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.ttf'));
    await inter.load();
  });

  group('lays out as one surface at wide, narrow and 2.5D', () {
    for (final (label, w, h, active) in [('wide 2D', 310.0, 640.0, 1), ('wide 2.5D', 310.0, 640.0, 2), ('narrow', 170.0, 560.0, 1), ('narrower', 150.0, 300.0, 1)]) {
      testWidgets(label, (t) async {
        await open(t, w: w, h: h, active: active);
        expect(t.takeException(), isNull);
        expect(k('gizmo'), findsOneWidget);
        expect(k('row-position'), findsOneWidget);
      });
    }
  });

  group('move', () {
    testWidgets('dragging the body moves X and Y together, as one logical commit', (t) async {
      final s = await open(t);
      final g = G(t, s);
      await t.dragFrom(g.c + const Offset(4, 3), const Offset(30, -20));
      await t.pump();
      expect(pos(s)[0], closeTo(350, 1));
      expect(pos(s)[1], closeTo(160, 1));
      expect(s.commits, 1);
      expect(s.previews, greaterThan(0));
    });

    testWidgets('the axis handles constrain the move to one axis', (t) async {
      final s = await open(t);
      final g = G(t, s);
      await t.dragFrom(g.tipX(), const Offset(30, 25));
      await t.pump();
      expect(pos(s)[0], closeTo(350, 1));
      expect(pos(s)[1], 180);
      await t.dragFrom(g.tipY(), const Offset(25, -20));
      await t.pump();
      expect(pos(s)[0], closeTo(350, 1));
      expect(pos(s)[1], closeTo(160, 1));
    });

    testWidgets('Shift makes the gesture fine, like every other Value', (t) async {
      final s = await open(t);
      final g = G(t, s);
      await t.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
      await t.dragFrom(g.c + const Offset(4, 3), const Offset(40, 0));
      await t.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
      await t.pump();
      expect(pos(s)[0], closeTo(324, .6));
    });

    testWidgets('Z appears outside 2D and moves depth of position with its own handle', (t) async {
      final s = await open(t, active: 2);
      final g = G(t, s);
      await t.dragFrom(g.c + Offset(-g.ringR, -g.ringR * .5), const Offset(0, -20));
      await t.pump();
      expect(pos(s, 2)[2], closeTo(80, 1));
      expect(k('toy-position.z'), findsOneWidget);
    });

    testWidgets('exact position: click, type, Enter; the number is absolute', (t) async {
      final s = await open(t);
      await t.tap(k('toy-position-0'));
      await t.pump();
      await t.enterText(find.descendant(of: k('toy-position-0'), matching: find.byType(EditableText)), '250');
      await t.testTextInput.receiveAction(TextInputAction.done);
      await t.pump(const Duration(seconds: 1));
      expect(pos(s)[0], 250);
      expect(pos(s)[1], 180);
      expect(s.commits, 1);
    });
  });

  group('scale', () {
    testWidgets('kept in shape by default: one number, and the corner scales both axes together', (t) async {
      final s = await open(t);
      expect(k('toy-scale-0'), findsOneWidget);
      expect(k('toy-scale-1'), findsNothing); // Classic: locked and even shows one well
      final g = G(t, s);
      final from = g.corner(1);
      await t.dragFrom(from, (from - g.c) * .5); // out to 1.5 times the distance
      await t.pump();
      expect(L(s, 1).scale[0], closeTo(1.5, .05));
      expect(L(s, 1).scale[1], closeTo(L(s, 1).scale[0], 1e-9));
      expect(s.commits, 1);
    });

    testWidgets('unlinking gives the axes back: an edge scales one axis only, and the numbers show both', (t) async {
      final s = await open(t);
      await t.tap(k('link-scale'));
      await t.pump();
      expect(s.linked.contains('scale'), isFalse);
      expect(k('toy-scale-1'), findsOneWidget);
      final g = G(t, s);
      await t.dragFrom(g.edgeX(), const Offset(24, 0));
      await t.pump();
      expect(L(s, 1).scale[0], greaterThan(1.3));
      expect(L(s, 1).scale[1], 1.0);
      expect(s.commits, 1);
    });

    testWidgets('the number scrubs the linked pair by ratio; a typed percent is exact', (t) async {
      final s = await open(t);
      await t.drag(k('toy-scale-0'), const Offset(30, 0));
      await t.pump(const Duration(seconds: 1));
      final x = L(s, 1).scale[0];
      expect(x, greaterThan(1));
      expect(L(s, 1).scale[1], closeTo(x, 1e-9));
      expect(s.commits, 1);
      await realGap(t);
      await t.tap(k('toy-scale-0'));
      await t.pump();
      await t.enterText(find.descendant(of: k('toy-scale-0'), matching: find.byType(EditableText)), '200');
      await t.testTextInput.receiveAction(TextInputAction.done);
      await t.pump(const Duration(seconds: 1));
      expect(L(s, 1).scale[0], closeTo(2, 1e-9));
      expect(L(s, 1).scale[1], closeTo(2, 1e-9));
    });

    testWidgets('an uneven scale shows both axes even while linked, and linked edits keep their ratio', (t) async {
      final s = await open(t, active: 2);
      expect(k('toy-scale-1'), findsOneWidget);
      final ratio = L(s, 2).scale[0] / L(s, 2).scale[1];
      await t.drag(k('toy-scale-0'), const Offset(20, 0));
      await t.pump(const Duration(seconds: 1));
      expect(L(s, 2).scale[0] / L(s, 2).scale[1], closeTo(ratio, 1e-6));
    });
  });

  group('rotate', () {
    testWidgets('turning the ring turns the body and keeps counting past a full turn', (t) async {
      final s = await open(t);
      final g = G(t, s);
      final r = g.ringR;
      final gest = await t.startGesture(g.ringAt(45));
      for (var deg = 55; deg <= 445; deg += 10) {
        await gest.moveTo(g.c + Offset(math.cos(deg * math.pi / 180), math.sin(deg * math.pi / 180)) * r);
      }
      await gest.up();
      await t.pump();
      expect(L(s, 1).rotation, closeTo(32 + 400, 6)); // not wrapped: 432, not 72
      expect(s.commits, 1);
    });

    testWidgets('exact degrees keep turns: 720 stays 720; reset returns to rest', (t) async {
      final s = await open(t);
      await t.tap(k('toy-rotation'));
      await t.pump();
      await t.enterText(find.descendant(of: k('toy-rotation'), matching: find.byType(EditableText)), '720');
      await t.testTextInput.receiveAction(TextInputAction.done);
      await t.pump(const Duration(seconds: 1));
      expect(L(s, 1).rotation, 720);
      await t.tap(k('reset-rotation'));
      await t.pump();
      expect(L(s, 1).rotation, 0);
    });

    testWidgets('outside 2D the X and Y axes join the family and the ring turns the chosen one', (t) async {
      final s = await open(t, active: 2, mode: TMode.rotate);
      expect(k('toy-rotation.x'), findsOneWidget);
      expect(k('toy-rotation.y'), findsOneWidget);
      await t.tap(k('rot-axis-1'));
      await t.pump();
      final g = G(t, s);
      final r = g.ringR;
      final gest = await t.startGesture(g.ringAt(45));
      for (var deg = 55; deg <= 135; deg += 10) {
        await gest.moveTo(g.c + Offset(math.cos(deg * math.pi / 180), math.sin(deg * math.pi / 180)) * r);
      }
      await gest.up();
      await t.pump();
      expect(L(s, 2).rotX, closeTo(20 + 90, 5));
      expect(L(s, 2).rotation, 12);
    });
  });

  group('anchor', () {
    testWidgets('the nine places: a click chooses, the current one is named', (t) async {
      final s = await open(t, mode: TMode.anchor);
      expect(find.text('Center'), findsOneWidget);
      final g = G(t, s);
      await t.tapAt(g.anchor(0, 1));
      await t.pump();
      expect(L(s, 1).anchor, [0.0, 1.0]);
      expect(find.text('Bottom left'), findsOneWidget);
      expect(s.commits, 1);
      // the always-present mini grid does the same in one click
      await t.tap(k('anchor-cell-20'));
      await t.pump();
      expect(L(s, 1).anchor, [1.0, 0.0]);
      expect(find.text('Top right'), findsOneWidget);
    });

    testWidgets('hovering a place shows the pivot on the Stage; leaving withdraws it', (t) async {
      final s = await open(t, mode: TMode.anchor);
      final g = G(t, s);
      final mouse = await t.createGesture(kind: PointerDeviceKind.mouse);
      await mouse.addPointer(location: const Offset(890, 890));
      addTearDown(mouse.removePointer);
      expect(s.anchorPreview.value, isNull);
      await mouse.moveTo(g.anchor(1, 1));
      await t.pump();
      expect(s.anchorPreview.value, [1.0, 1.0]);
      expect(L(s, 1).anchor, [.5, .5]); // hovering chooses nothing
      await mouse.moveTo(const Offset(890, 890));
      await t.pump();
      expect(s.anchorPreview.value, isNull);
      // and from the mini grid
      await mouse.moveTo(t.getCenter(k('anchor-cell-00')));
      await t.pump();
      expect(s.anchorPreview.value, [0.0, 0.0]);
      await mouse.moveTo(const Offset(890, 890));
      await t.pump();
      expect(s.anchorPreview.value, isNull);
    });
  });

  group('modes', () {
    testWidgets('Move is the working mode: scale corners and the rotate ring work from it without switching', (t) async {
      final s = await open(t);
      expect(modeButton(t, 'move'), kMint);
      final g = G(t, s);
      await t.dragFrom(g.corner(2), (g.corner(2) - g.c) * .3);
      await t.pump();
      expect(L(s, 1).scale[0], greaterThan(1.2));
      final g2 = G(t, s);
      final gest = await t.startGesture(g2.ringAt(45));
      await gest.moveTo(g2.ringAt(135));
      await gest.up();
      await t.pump();
      expect(L(s, 1).rotation, closeTo(32 + 90, 6));
      expect(modeButton(t, 'move'), kMint); // still Move
    });

    testWidgets('a solo mode is one click, one key, or the row glyph; touching numbers never changes the mode', (t) async {
      await open(t);
      await t.tap(k('mode-scale'));
      await t.pump();
      expect(modeButton(t, 'scale'), kBlue);
      await t.tap(k('gizmo'));
      await t.sendKeyEvent(LogicalKeyboardKey.digit3);
      await t.pump();
      expect(modeButton(t, 'rotate'), kPink);
      await t.sendKeyEvent(LogicalKeyboardKey.tab);
      await t.pump();
      expect(modeButton(t, 'anchor'), kViolet);
      await t.drag(k('toy-position-0'), const Offset(10, 0)); // touching numbers leaves the mode alone
      await t.tap(k('link-scale'));
      await t.pump(const Duration(seconds: 1));
      expect(modeButton(t, 'anchor'), kViolet);
      await t.tap(k('glyph-position')); // the row's glyph is a mode switch
      await t.pump();
      expect(modeButton(t, 'move'), kMint);
    });

    testWidgets('revealing a property shows the mode that owns it and focuses its number', (t) async {
      final s = await open(t);
      s.focusProperty('scale');
      await t.pump();
      await t.pump();
      expect(modeButton(t, 'scale'), kBlue);
      expect(FocusManager.instance.primaryFocus?.debugLabel, 'tf:scale:0');
      s.focusProperty('rotation');
      await t.pump();
      await t.pump();
      expect(modeButton(t, 'rotate'), kPink);
      expect(FocusManager.instance.primaryFocus?.debugLabel, 'tf:rotation');
    });
  });

  group('selection: relative scrubs, absolute typing, mixed, lock', () {
    testWidgets('a gizmo drag moves every selected layer by the same amount; each keeps its offset', (t) async {
      final s = await open(t, sel: [1, 2], active: 1);
      final g = G(t, s);
      await t.dragFrom(g.c + const Offset(4, 3), const Offset(30, 10));
      await t.pump();
      expect(pos(s, 1)[0], closeTo(350, 1));
      expect(pos(s, 2)[0], closeTo(-90, 1));
      expect(pos(s, 2)[1], closeTo(50, 1));
      expect(pos(s, 1)[0] - pos(s, 2)[0], closeTo(440, 1)); // the offset between them is kept
      expect(s.commits, 1);
    });

    testWidgets('a typed number is absolute for all of them', (t) async {
      final s = await open(t, sel: [1, 2], active: 1);
      await t.tap(k('toy-position-0'));
      await t.pump();
      await t.enterText(find.descendant(of: k('toy-position-0'), matching: find.byType(EditableText)), '100');
      await t.testTextInput.receiveAction(TextInputAction.done);
      await t.pump(const Duration(seconds: 1));
      expect(pos(s, 1)[0], 100);
      expect(pos(s, 2)[0], 100);
      expect(pos(s, 2)[1], 40); // the axis you did not touch is left alone
    });

    testWidgets('numbers that differ across the selection read as mixed; agreeing ones do not', (t) async {
      final s = await open(t, sel: [1, 2], active: 1);
      expect(find.text('—'), findsWidgets);
      expect(s.mixed('position', 0), isTrue);
      final agree = TransformStore([TLayer(1, 'A', position: [5, 5, 0]), TLayer(2, 'B', position: [5, 9, 0])], selection: [1, 2]);
      expect(agree.mixed('position', 0), isFalse);
      expect(agree.mixed('position', 1), isTrue);
    });

    testWidgets('a locked layer in the selection is left out of every write', (t) async {
      final s = await open(t, sel: [1, 3], active: 1);
      final g = G(t, s);
      await t.dragFrom(g.c + const Offset(4, 3), const Offset(30, 0));
      await t.pump();
      expect(pos(s, 1)[0], closeTo(350, 1));
      expect(pos(s, 3)[0], 40);
    });
  });

  group('marks: key, animate, reset', () {
    testWidgets('the key lamp keys and unkeys the shown layer; Animate turns a touched value into a key', (t) async {
      final s = await open(t);
      expect(L(s, 1).keyedNow, isEmpty);
      await t.tap(k('key-position'));
      await t.pump();
      expect(L(s, 1).keyedNow, contains('position'));
      await t.tap(k('key-position'));
      await t.pump();
      expect(L(s, 1).keyedNow, isNot(contains('position')));
      await t.tap(k('tf-animate'));
      await t.pump();
      final g = G(t, s);
      await t.dragFrom(g.c + const Offset(4, 3), const Offset(20, 0));
      await t.pump();
      expect(L(s, 1).animated, contains('position'));
      expect(L(s, 1).keyedNow, contains('position'));
    });

    testWidgets('reset appears only when there is something to go back to, and puts back the rest value', (t) async {
      final s = await open(t);
      expect(k('reset-position'), findsOneWidget); // 320, 180 is not the rest
      await t.tap(k('reset-position'));
      await t.pump();
      expect(pos(s)[0], 0);
      expect(pos(s)[1], 0);
      expect(k('reset-position'), findsNothing);
    });
  });

  group('around the instrument: Space, Parent, Depth', () {
    testWidgets('Space applies to every selected layer; Depth exists only outside 2D', (t) async {
      final s = await open(t, sel: [1, 3], active: 1);
      expect(k('row-depth'), findsNothing);
      await t.tap(k('space-3D'));
      await t.pump();
      expect(L(s, 1).projection, '3D');
      expect(L(s, 3).projection, '3D'); // the selection, even the locked one, as Classic's setAttrs on selectedIds
      expect(k('row-depth'), findsOneWidget);
      expect(k('toy-position.z'), findsOneWidget);
      await t.tap(k('space-2D'));
      await t.pump();
      expect(k('row-depth'), findsNothing);
      expect(k('toy-position.z'), findsNothing);
    });

    testWidgets('Depth is an exact Value with a way to the Depth Desk; the route replaces nothing', (t) async {
      final s = await open(t, active: 2);
      await t.drag(k('toy-depth'), const Offset(20, 0));
      await t.pump(const Duration(seconds: 1));
      expect(L(s, 2).depth, isNot(120));
      await t.tap(k('route-depth'));
      await t.pump();
      expect(s.routes, ['Depth']);
      expect(s.routeFrom, ['depth']);
    });

    testWidgets('Parent picks among the other layers or None, for the shown layer only', (t) async {
      final s = await open(t, sel: [1, 2], active: 2);
      expect(find.text('Logo'), findsWidgets);
      await t.tap(k('parent-next'));
      await t.pump();
      expect(L(s, 2).parent, 3);
      expect(L(s, 1).parent, isNull);
      await t.tap(k('parent-next'));
      await t.pump();
      expect(L(s, 2).parent, isNull);
    });
  });

  group('locked layer: editable = false disables gizmo and Values together (cross-cutting, kept)', () {
    testWidgets('nothing changes and nothing is counted', (t) async {
      final s = await open(t, active: 3);
      final g = G(t, s);
      await t.dragFrom(g.c + const Offset(4, 3), const Offset(30, 0));
      await t.drag(k('toy-position-0'), const Offset(30, 0));
      await t.tapAt(g.anchor(1, 1));
      await t.tap(k('space-3D'));
      await t.tap(k('parent-next'));
      await t.pump(const Duration(seconds: 1));
      expect(pos(s, 3)[0], 40);
      expect(L(s, 3).projection, '2D');
      expect(L(s, 3).parent, isNull);
      expect(s.commits, 0);
      expect(find.text('Locked'), findsOneWidget);
    });
  });

  group('narrow keeps the instrument and reaches everything', () {
    testWidgets('the gizmo, the modes and the values are all there, and the gizmo still works', (t) async {
      final s = await open(t, w: 170, h: 560);
      final g = G(t, s);
      expect(g.rect.width, greaterThan(140)); // not shrunk to a mark
      await t.dragFrom(g.c + const Offset(4, 3), const Offset(20, 0));
      await t.pump();
      expect(pos(s)[0], closeTo(340, 1));
      await t.tap(k('mode-rotate'));
      await t.pump();
      expect(modeButton(t, 'rotate'), kPink);
    });

    testWidgets('a very short panel scrolls; the gizmo is first and every row is reachable', (t) async {
      await open(t, active: 2, w: 150, h: 300);
      await t.ensureVisible(k('row-depth'));
      await t.pump();
      expect(k('row-depth'), findsOneWidget);
      expect(t.takeException(), isNull);
    });
  });

  group('the Classic seven-step loop, counted', () {
    testWidgets('Position, uniform Scale, unlink and edge Scale, Rotation, Anchor, exact Position: eight actions, no mode clicks', (t) async {
      final s = await open(t);
      var actions = 0;
      final g0 = G(t, s);
      await t.dragFrom(g0.c + const Offset(4, 3), const Offset(20, 10)); actions++; // 1 Position
      final g1 = G(t, s);
      await t.dragFrom(g1.corner(1), (g1.corner(1) - g1.c) * .3); actions++; // 2 Scale uniform
      await t.tap(k('link-scale')); actions++; // 3 unlink
      await t.pump();
      final g2 = G(t, s);
      await t.dragFrom(g2.edgeX(), const Offset(14, 0)); actions++; // 4 X only
      final g3 = G(t, s);
      final gest = await t.startGesture(g3.ringAt(45));
      await gest.moveTo(g3.ringAt(135));
      await gest.up();
      actions++; // 5 Rotation
      await t.tap(k('anchor-cell-00')); actions++; // 6 Anchor
      await t.pump();
      await t.tap(k('toy-position-0')); actions++; // 7 exact Position: click...
      await t.pump();
      await t.enterText(find.descendant(of: k('toy-position-0'), matching: find.byType(EditableText)), '100');
      await t.testTextInput.receiveAction(TextInputAction.done);
      actions++; // ...and type
      await t.pump(const Duration(seconds: 1));
      expect(actions, 8);
      expect(pos(s)[0], 100);
      expect(L(s, 1).scale[1], greaterThan(1));
      expect(L(s, 1).anchor, [0.0, 0.0]);
      expect(s.commits, 6); // six logical edits, none of them a mode change
    });
  });
}
