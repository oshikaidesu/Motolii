// One convention for Orbit pitch: the composition's y points down (Stage and render), so +pitch puts the eye BELOW the
// target. The Stage observer drag, the Inspector face's drawing and the face's drag all say the same.
import 'dart:convert';
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_ui/inspector/camera/face.dart';
import 'package:motolii_ui/inspector/camera/model.dart';
import 'package:motolii_ui/stage/session.dart';
import 'package:motolii_ui/session/editor_session.dart';

void main() {
  test('the model eye direction follows the native formula (+pitch: y down)', () {
    final s = CameraStore()..row('camera.orbit')['value'] = [30.0, 0.0];
    expect(s.eyeDirection[1], closeTo(math.sin(30 * math.pi / 180), 1e-9));
    expect(s.eyeDirection[1], greaterThan(0));
  });

  test('the face draws +pitch with the eye below the target, +yaw to the left', () {
    final s = CameraStore()..row('camera.orbit')['value'] = [30.0, 0.0];
    final g = CameraGeom(const Size(286, 176), s);
    expect(g.eye.dy, greaterThan(g.c.dy));
    s.row('camera.orbit')['value'] = [0.0, 30.0];
    expect(CameraGeom(const Size(286, 176), s).eye.dx, lessThan(g.c.dx + CameraGeom.orbitR * .62));
  });

  testWidgets('dragging the face eye down raises pitch, as dragging down on the Stage does', (tester) async {
    final s = CameraStore();
    await tester.pumpWidget(Directionality(textDirection: TextDirection.ltr, child: Align(alignment: Alignment.topLeft, child: CameraFace(s))));
    final origin = tester.getTopLeft(find.byType(CameraFace));
    final eye = origin + CameraGeom(const Size(286, 176), s).eye;
    final gesture = await tester.startGesture(eye);
    await gesture.moveBy(const Offset(0, 10));
    expect(s.pitch, greaterThan(0));
    await gesture.up();
  });

  testWidgets('Stage orbit drag down asks native for a positive pitch', (tester) async {
    // touch.dart sends orbitBy(dy * .3, -dx * .3)
    final sent = <dynamic>[];
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(EditorSession.channel, (call) async {
      if (call.method == 'request') {
        final command = jsonDecode(call.arguments['command']);
        if (command['op'] == 'stageView') sent.add(command['orbit']);
      }
      return <String, dynamic>{};
    });
    final c = EditorSession();
    c.document.value = {'capabilities': ['stageView']};
    StageSession.of(c, 'User').orbitBy(10 * .3, 0);
    await tester.pump(const Duration(milliseconds: 50));
    expect(sent.single[0], greaterThan(0));
    c.dispose();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(EditorSession.channel, null);
  });
}
