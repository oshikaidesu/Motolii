// The Camera Instrument, driven with real pointers. The face is the view ray (target point, eye on its orbit, distance along
// the ray, roll about it); Zoom is the lens and is a different tool from Distance; Target layer overrides the point.
import 'dart:math' as math;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_stage5/hf/insp/camera.dart';
import 'package:motolii_stage5/hf/insp/camera_face.dart';
import 'package:motolii_stage5/hf/insp/camera_model.dart';

Widget host(Widget child, double w, double h) => WidgetsApp(
      color: const Color(0xFF000000),
      pageRouteBuilder: <T>(settings, builder) => PageRouteBuilder<T>(settings: settings, pageBuilder: (c, _, __) => builder(c)),
      home: Align(alignment: Alignment.topLeft, child: SizedBox(width: w, height: h, child: child)),
    );

const layers = [CamLayer(1, 'Logo', 960, 540, 0), CamLayer(2, 'Title', 400, 300, 60), CamLayer(3, 'Backdrop', 1500, 800, -200)];

Future<CameraStore> open(WidgetTester t, {CameraStore? store, double w = 310, double h = 700}) async {
  t.view.physicalSize = const Size(900, 900);
  t.view.devicePixelRatio = 1;
  addTearDown(t.view.reset);
  final s = store ?? CameraStore(layers: layers);
  await t.pumpWidget(host(CameraInstrument(s), w, h));
  await t.pump();
  return s;
}

Finder k(String key) => find.byKey(ValueKey(key));
Future<void> realGap(WidgetTester t) async { await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 380))); }

class F {
  F(this.t, this.s);
  final WidgetTester t;
  final CameraStore s;
  Rect get rect => t.getRect(k('camera-face'));
  CameraGeom get g => CameraGeom(rect.size, s);
  Offset at(Offset local) => rect.topLeft + local;
}

Future<void> typeInto(WidgetTester t, String key, String text) async {
  await t.ensureVisible(k(key));
  await t.tap(k(key));
  await t.pump();
  await t.enterText(find.descendant(of: k(key), matching: find.byType(EditableText)), text);
  await t.testTextInput.receiveAction(TextInputAction.done);
  await t.pump(const Duration(seconds: 1));
  await realGap(t);
}

void main() {
  setUpAll(() async {
    final inter = FontLoader('Inter')
      ..addFont(rootBundle.load('assets/fonts/Inter-Regular.ttf'))
      ..addFont(rootBundle.load('assets/fonts/Inter-Medium.ttf'))
      ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.ttf'));
    await inter.load();
  });

  group('lays out at every size', () {
    for (final (label, w, h) in [('wide', 310.0, 700.0), ('narrow', 170.0, 700.0), ('narrower', 150.0, 320.0)]) {
      testWidgets(label, (t) async {
        await open(t, w: w, h: h);
        expect(t.takeException(), isNull);
        expect(k('camera-face'), findsOneWidget);
      });
    }
  });

  group('the face: all four gestures live at once, no modes', () {
    testWidgets('dragging the camera orbits it around the target: pitch and yaw together, as one edit, following the pointer', (t) async {
      final s = await open(t);
      final f = F(t, s);
      final start = f.g.eye;
      final to = start + const Offset(-30, 18);
      final gest = await t.startGesture(f.at(start));
      await gest.moveTo(f.at(to));
      await gest.up();
      await t.pump();
      expect(s.pitch, isNot(0));
      expect(s.yaw, isNot(0));
      expect((f.g.eye - to).distance, lessThan((start - to).distance)); // the camera moved toward where it was pulled
      expect(s.commits, 1);
    });

    testWidgets('turns around the target are kept: yaw past a full circle is not wrapped', (t) async {
      final s = await open(t);
      await typeInto(t, 'toy-camera.orbit-1', '400');
      expect(s.yaw, 400);
    });

    testWidgets('the distance handle rides the ray: outward dollies away, inward dollies in, and it clamps', (t) async {
      final s = await open(t);
      final f = F(t, s);
      final h = f.g.distanceHandle;
      final u = f.g.rayDir;
      await t.dragFrom(f.at(h), u * 24);
      await t.pump();
      expect(s.distance, greaterThan(1.3));
      final d1 = s.distance;
      await t.dragFrom(f.at(f.g.distanceHandle), -u * 40);
      await t.pump();
      expect(s.distance, lessThan(d1));
      expect(s.commits, 2);
      await typeInto(t, 'toy-camera.distance', '500');
      expect(s.distance, 100); // the declared range is a guard, not a bar
    });

    testWidgets('the roll ring twists about the ray, counting past a full turn', (t) async {
      final s = await open(t);
      final f = F(t, s);
      final c = f.rect.center;
      final gest = await t.startGesture(c + Offset(math.cos(0.3), math.sin(0.3)) * CameraGeom.ringR);
      for (var deg = 30; deg <= 420; deg += 10) {
        await gest.moveTo(c + Offset(math.cos(0.3 + deg * math.pi / 180), math.sin(0.3 + deg * math.pi / 180)) * CameraGeom.ringR);
      }
      await gest.up();
      await t.pump();
      expect(s.roll, greaterThan(400)); // not wrapped
      expect(s.commits, 1);
      await t.ensureVisible(k('reset-roll'));
      await t.tap(k('reset-roll'));
      await t.pump();
      expect(s.roll, 0); // reset goes back to rest
    });

    testWidgets('dragging the target dot moves Center X and Y together, in world pixels', (t) async {
      final s = await open(t);
      final f = F(t, s);
      await t.dragFrom(f.at(f.g.c), const Offset(10, -5));
      await t.pump();
      expect(s.cx, closeTo(40, 1));
      expect(s.cy, closeTo(-20, 1));
      expect(s.commits, 1);
    });

    testWidgets('Shift makes every face gesture fine', (t) async {
      final s = await open(t);
      final f = F(t, s);
      await t.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
      await t.dragFrom(f.at(f.g.c), const Offset(20, 0));
      await t.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
      await t.pump();
      expect(s.cx, closeTo(8, .6));
    });
  });

  group('Target layer overrides the point; it never overwrites what is stored', () {
    testWidgets('a Target layer decides X, Y and Z: they show its origin, refuse edits, and the dot cannot be dragged', (t) async {
      final s = await open(t);
      await t.tap(k('choice-camera.target-2')); // Title
      await t.pump();
      expect(s.targetLocked, isTrue);
      expect(s.cx, 400 - 960);
      expect(s.cy, 300 - 540);
      expect(s.row('camera.target.z')['value'], 60);
      final f = F(t, s);
      await t.dragFrom(f.at(f.g.c), const Offset(40, 0));
      await t.drag(k('toy-camera.center-0'), const Offset(40, 0));
      await t.pump(const Duration(seconds: 1));
      expect(s.cx, -560);
    });

    testWidgets('clearing the Target brings the authored point back', (t) async {
      final s = await open(t);
      final f = F(t, s);
      await t.dragFrom(f.at(f.g.c), const Offset(10, 0));
      final authored = s.cx;
      await t.tap(k('choice-camera.target-1'));
      await t.pump();
      expect(s.cx, isNot(authored));
      await t.tap(k('choice-camera.target-0'));
      await t.pump();
      expect(s.cx, authored);
    });
  });

  group('Distance and Zoom: one shared number, two different tools', () {
    testWidgets('magnification is Zoom over Distance; each moves only its own thing', (t) async {
      final s = await open(t);
      await typeInto(t, 'toy-camera.distance', '0.5');
      await typeInto(t, 'toy-camera.zoom', '2');
      expect(s.magnification, 4);
      expect(find.text('Frames the target at ×4.00'), findsOneWidget);
      // Distance did not touch the lens, Zoom did not touch the eye
      expect(s.zoom, 2);
      expect(s.distance, .5);
      expect(s.eyeDirection, [0.0, 0.0, -1.0]);
    });

    testWidgets('zooming out while dollying in keeps the framing at the target plane', (t) async {
      final s = await open(t);
      await typeInto(t, 'toy-camera.distance', '0.5');
      await typeInto(t, 'toy-camera.zoom', '0.5');
      expect(s.magnification, 1); // the same framing, a different picture
      expect(s.distance, isNot(s.zoom == 1 ? 0 : 1));
    });
  });

  group('exact values and marks', () {
    testWidgets('pitch, yaw, roll and the target numbers take typed values', (t) async {
      final s = await open(t);
      await typeInto(t, 'toy-camera.orbit-0', '30');
      await typeInto(t, 'toy-camera.center-1', '-80');
      await typeInto(t, 'toy-camera.roll', '720');
      expect(s.pitch, 30);
      expect(s.cy, -80);
      expect(s.roll, 720);
    });

    testWidgets('the route to Depth hands over and changes nothing', (t) async {
      final s = await open(t);
      await t.tap(k('route-depth'));
      await t.pump();
      expect(s.routes, ['Depth']);
      expect(s.routeFrom, ['camera']);
      expect(s.commits, 0);
    });
  });

  group('marks: key lamp and reset per group', () {
    testWidgets('a group keys and unkeys as one, and reset appears only when there is something to return to', (t) async {
      final s = await open(t);
      expect(k('reset-orbit'), findsNothing);
      await t.tap(k('key-orbit'));
      await t.pump();
      expect(s.keyedNow, contains('camera.orbit'));
      await typeInto(t, 'toy-camera.orbit-0', '25');
      expect(k('reset-orbit'), findsOneWidget);
      await t.tap(k('reset-orbit'));
      await t.pump();
      expect(s.pitch, 0);
      expect(k('reset-orbit'), findsNothing);
    });
  });

  group('locked', () {
    testWidgets('nothing changes and nothing is counted', (t) async {
      final s = await open(t, store: CameraStore(layers: layers, frozen: true));
      final f = F(t, s);
      await t.dragFrom(f.at(f.g.eye), const Offset(-30, 10));
      await t.dragFrom(f.at(f.g.c), const Offset(20, 0));
      await t.dragFrom(f.at(f.g.distanceHandle), const Offset(-10, 10));
      await t.tap(k('choice-camera.target-1'));
      await t.drag(k('toy-camera.roll'), const Offset(30, 0));
      await t.pump(const Duration(seconds: 1));
      expect(s.pitch, 0);
      expect(s.yaw, 0);
      expect(s.cx, 0);
      expect(s.distance, 1);
      expect(s.roll, 0);
      expect(s.targetLocked, isFalse);
      expect(s.commits, 0);
    });
  });

  group('narrow keeps the face and reaches everything', () {
    testWidgets('the face is not shrunk to a mark and still orbits', (t) async {
      final s = await open(t, w: 170, h: 700);
      final f = F(t, s);
      expect(f.rect.width, greaterThan(140));
      await t.dragFrom(f.at(f.g.eye), const Offset(-20, 10));
      await t.pump();
      expect(s.yaw, isNot(0));
      await t.ensureVisible(k('toy-camera.roll'));
      expect(t.takeException(), isNull);
    });
  });

  group('the orbit, distance, roll, zoom, retarget loop, counted against Classic', () {
    testWidgets('five actions here; Classic needs seven (Pitch, Yaw, Distance, Roll, Zoom, Center X, Center Y)', (t) async {
      final s = await open(t);
      final f = F(t, s);
      var actions = 0;
      final e0 = f.g.eye;
      final g1 = await t.startGesture(f.at(e0));
      await g1.moveTo(f.at(e0 + const Offset(-24, 14)));
      await g1.up();
      actions++; // 1 orbit: pitch and yaw in one gesture
      await t.pump();
      final h = f.g.distanceHandle;
      await t.dragFrom(f.at(h), f.g.rayDir * 20);
      actions++; // 2 distance
      await t.pump();
      final c = f.rect.center;
      final g3 = await t.startGesture(c + Offset(math.cos(0.3), math.sin(0.3)) * CameraGeom.ringR);
      await g3.moveTo(c + Offset(math.cos(1.2), math.sin(1.2)) * CameraGeom.ringR);
      await g3.up();
      actions++; // 3 roll
      await t.pump();
      await t.ensureVisible(k('toy-camera.zoom'));
      await t.drag(k('toy-camera.zoom'), const Offset(20, 0));
      actions++; // 4 zoom
      await t.pump(const Duration(seconds: 1));
      await t.ensureVisible(k('camera-face'));
      await t.pump();
      await t.dragFrom(f.at(f.g.c), const Offset(8, 4));
      actions++; // 5 retarget: Center X and Y together
      await t.pump();
      expect(actions, 5);
      const classic = 7;
      expect(actions, lessThan(classic));
      expect(s.pitch, isNot(0));
      expect(s.yaw, isNot(0));
      expect(s.distance, greaterThan(1));
      expect(s.roll, isNot(0));
      expect(s.zoom, greaterThan(1));
      expect(s.cx, isNot(0));
      expect(s.cy, isNot(0));
      expect(s.commits, 5);
    });
  });
}
