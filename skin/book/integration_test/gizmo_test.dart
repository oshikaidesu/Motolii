// The gizmo (the picked thing's box and handles) on the live picture: a thing swaying while it plays, and one whose X is
// dragged in the inspector; the window shot at a few moments of each, to see whether the box stays on the thing.
//   flutter drive --profile -d macos --driver=test_driver/integration_test.dart --target=integration_test/gizmo_test.dart
import 'dart:io';
import 'dart:ui' show ImageByteFormat;

import 'package:flutter/rendering.dart' show OffsetLayer;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart' show NumField;
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/workspace.dart' show Workspace;
import 'package:lab_book/workspace/ws.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized().framePolicy = LiveTestWidgetsFlutterBindingFramePolicy.fullyLive;

  testWidgets('the picked thing\'s box stays on it while it moves', (t) async {
    t.binding.defaultBinaryMessenger.setMockMethodCallHandler(SystemChannels.menu, (_) async => null);
    final doc = '${Directory.systemTemp.createTempSync('gizmo').path}/gizmo.motolii';
    await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => Workspace(document: doc)));
    await t.pumpAndSettle();
    final ws = WsScope.read(t.element(find.byType(WsTimeline))), m = ws.inside!;
    final out = Directory('$motoliiDir/../reports/gizmo')..createSync(recursive: true);
    for (final f in out.listSync()) {
      f.deleteSync();
    }
    Future<void> shoot(String name) async {
      await t.pump();
      // ignore: invalid_use_of_protected_member
      final root = t.binding.renderViews.first.layer! as OffsetLayer;
      final png = await (await root.toImage(Offset.zero & t.view.physicalSize)).toByteData(format: ImageByteFormat.png);
      File('${out.path}/$name.png').writeAsBytesSync(png!.buffer.asUint8List());
    }

    Future<void> wait(int ms) => t.runAsync(() => Future<void>.delayed(Duration(milliseconds: ms)));

    final sway = m.add('shape', 'rect');
    m
      ..say(sway, 'fill', 'ff3366')
      ..set(sway, 'wave_x', 2)
      ..set(sway, 'wave_speed', 0.5);
    ws
      ..reread()
      ..select('$sway');
    await wait(300);
    await shoot('play_0');
    ws.playing = true;
    for (var k = 1; k <= 4; k++) {
      await wait(230);
      await shoot('play_$k');
    }
    ws.playing = false;

    final still = m.add('shape', 'rect');
    m.say(still, 'fill', '33aaff');
    ws
      ..reread()
      ..select('$still');
    await wait(300);
    final knob = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_x');
    await t.ensureVisible(knob.first);
    await t.pump();
    await shoot('drag_0');
    final hand = await t.startGesture(t.getCenter(knob.first));
    for (var k = 1; k <= 3; k++) {
      for (var i = 0; i < 10; i++) {
        await hand.moveBy(const Offset(8, 0));
        await t.pump(const Duration(milliseconds: 16));
      }
      await wait(150);
      await shoot('drag_$k');
    }
    await hand.up();
    await wait(300);
    await shoot('drag_let_go');
  });
}
