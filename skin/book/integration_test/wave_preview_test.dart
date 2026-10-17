// The Wave card's preview (Good Boy Ninja's Wiggle manner): a card with a wave, its inspector shot at rest and then at a
// few moments while a mouse is on the preview, to see the face sway as the stage will only while touched.
//   flutter drive --profile -d macos --driver=test_driver/integration_test.dart --target=integration_test/wave_preview_test.dart
import 'dart:io';
import 'dart:ui' show ImageByteFormat;

import 'package:flutter/gestures.dart' show PointerDeviceKind;
import 'package:flutter/rendering.dart' show OffsetLayer;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/sets/inspector/inspector_wave.dart' show WavePreview;
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/workspace.dart' show Workspace;
import 'package:lab_book/workspace/ws.dart';
import '../test/door.dart' show seen;

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized().framePolicy = LiveTestWidgetsFlutterBindingFramePolicy.fullyLive;

  testWidgets('the wave card plays the sway by itself', (t) async {
    t.binding.defaultBinaryMessenger.setMockMethodCallHandler(SystemChannels.menu, (_) async => null);
    t.view.physicalSize = Size(t.view.physicalSize.width, t.view.physicalSize.width * 1.6);
    addTearDown(t.view.resetPhysicalSize);
    final doc = '${Directory.systemTemp.createTempSync('wave').path}/wave.motolii';
    await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => Workspace(document: doc)));
    await t.pumpAndSettle();
    final ws = WsScope.read(t.element(find.byType(WsTimeline))), m = ws.inside!;
    final card = m.add('shape', 'rect');
    m
      ..say(card, 'fill', 'ff3366')
      ..set(card, 'wave_x', 1.2)
      ..set(card, 'wave_rotate', 25)
      ..set(card, 'wave_speed', 1);
    ws
      ..reread()
      ..select('$card');
    for (var i = 0; i < 20; i++) {
      await t.pump(const Duration(milliseconds: 50));
    }
    final preview = find.byType(WavePreview);
    await seen(t, preview);
    await Scrollable.ensureVisible(t.element(preview.first), alignment: .3);
    final out = Directory('$motoliiDir/../reports/wave_preview')..createSync(recursive: true);
    Future<void> shoot(String name) async {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 250)));
      await t.pump();
      // ignore: invalid_use_of_protected_member
      final root = t.binding.renderViews.first.layer! as OffsetLayer;
      final png = await (await root.toImage(Offset.zero & t.view.physicalSize)).toByteData(format: ImageByteFormat.png);
      File('${out.path}/$name.png').writeAsBytesSync(png!.buffer.asUint8List());
    }

    await shoot('rest');
    final mouse = await t.createGesture(kind: PointerDeviceKind.mouse);
    await mouse.addPointer(location: Offset.zero);
    await mouse.moveTo(t.getCenter(preview.first)); // (a hand on it: it sways)
    for (var k = 0; k < 4; k++) {
      await shoot('$k');
    }
    await mouse.moveTo(Offset.zero); // (the hand off: still again)
    await shoot('off');
    final r = t.getRect(preview.first), dpr = t.view.devicePixelRatio;
    File('${out.path}/rect.txt').writeAsStringSync('${r.left * dpr} ${r.top * dpr} ${r.right * dpr} ${r.bottom * dpr}');
  });
}
