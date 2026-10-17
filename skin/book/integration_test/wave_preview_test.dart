// The motion previews (Good Boy Ninja's Wiggle manner) on the cards whose things move by themselves: a thing set to sway,
// to fall and bounce, to drift; each card's preview shot at rest and at a few moments while a mouse is on it.
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
import 'package:lab_book/sets/inspector/inspector_look_preview.dart' show LookPreview;
import 'package:lab_book/sets/inspector/inspector_motion_preview.dart' show MotionPreview;
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/workspace.dart' show Workspace;
import 'package:lab_book/workspace/ws.dart';
import '../test/door.dart' show seen;

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized().framePolicy = LiveTestWidgetsFlutterBindingFramePolicy.fullyLive;

  testWidgets('a card of a moving thing plays its motion while touched', (t) async {
    t.binding.defaultBinaryMessenger.setMockMethodCallHandler(SystemChannels.menu, (_) async => null);
    t.view.physicalSize = Size(t.view.physicalSize.width, t.view.physicalSize.width * 1.6);
    addTearDown(t.view.resetPhysicalSize);
    final doc = '${Directory.systemTemp.createTempSync('motion').path}/motion.motolii';
    await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => Workspace(document: doc)));
    await t.pumpAndSettle();
    final ws = WsScope.read(t.element(find.byType(WsTimeline))), m = ws.inside!;
    final sway = m.add('shape', 'rect'), fall = m.add('solid', 'cube'), floor = m.add('solid', 'cube'), drift = m.add('shape', 'rect'), field = m.add('shape', 'rect');
    m
      ..set(sway, 'wave_x', 1.2)
      ..set(sway, 'wave_rotate', 25)
      ..set(sway, 'wave_speed', 1)
      ..say(fall, 'physics', 'moves')
      ..set(fall, 'y', 2)
      ..set(fall, 'bounce', 70)
      ..say(floor, 'physics', 'still')
      ..set(floor, 'y', -1.5)
      ..set(floor, 'scale_x', 600)
      ..set(drift, 'scatter_x', 2)
      ..set(drift, 'drift_x', 1)
      ..set(field, 'copies', 8)
      ..set(field, 'copy_x', 0.5)
      ..set(field, 'field_radius', 1.2)
      ..set(field, 'field_size', 1.5)
      ..set(field, 'field_rotate', 45);
    final out = Directory('$motoliiDir/../reports/wave_preview')..createSync(recursive: true);
    final dpr = t.view.devicePixelRatio, mouse = await t.createGesture(kind: PointerDeviceKind.mouse);
    await mouse.addPointer(location: Offset.zero);
    final bent = m.add('shape', 'rect');
    m
      ..say(bent, 'fill', 'ffaa22')
      ..set(bent, 'x', 4)
      ..set(bent, 'round', 30)
      ..set(bent, 'zigzag', 0.2)
      ..set(bent, 'rotate', 15);
    for (final (card, thing) in [('Wave', sway), ('Physics', fall), ('Scatter', drift), ('Field', field), ('Path', bent)]) {
      ws
        ..reread()
        ..select('$thing');
      for (var i = 0; i < 20; i++) {
        await t.pump(const Duration(milliseconds: 50));
      }
      final preview = card == 'Path' ? find.byType(LookPreview) : find.byWidgetPredicate((w) => w is MotionPreview && w.card == card);
      await seen(t, preview);
      await Scrollable.ensureVisible(t.element(preview.first), alignment: .3);
      Future<void> shoot(String name) async {
        await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 280)));
        await t.pump();
        // ignore: invalid_use_of_protected_member
        final root = t.binding.renderViews.first.layer! as OffsetLayer;
        final png = await (await root.toImage(Offset.zero & t.view.physicalSize)).toByteData(format: ImageByteFormat.png);
        File('${out.path}/${card}_$name.png').writeAsBytesSync(png!.buffer.asUint8List());
      }

      await mouse.moveTo(Offset.zero);
      await shoot('rest');
      await mouse.moveTo(t.getCenter(preview.first)); // (a hand on it: it moves)
      for (var k = 0; k < 4; k++) {
        await shoot('$k');
      }
      await mouse.moveTo(Offset.zero); // (the hand off: still again)
      if (card == 'Wave') { // (pulled across: it sways wider, the inside told so)
        final before = m.value(thing, 'wave_x', 0);
        final hand = await t.startGesture(t.getCenter(preview.first), kind: PointerDeviceKind.mouse);
        for (var i = 0; i < 10; i++) {
          await hand.moveBy(const Offset(6, 0));
          await t.pump(const Duration(milliseconds: 16));
        }
        await hand.up();
        for (var i = 0; i < 6; i++) {
          await t.pump(const Duration(milliseconds: 50));
        }
        await shoot('pulled');
        File('${out.path}/Wave_pulled.txt').writeAsStringSync('wave_x $before -> ${m.value(thing, 'wave_x', 0)}');
      }
      final r = t.getRect(preview.first);
      File('${out.path}/${card}_rect.txt').writeAsStringSync('${r.left * dpr} ${r.top * dpr} ${r.right * dpr} ${r.bottom * dpr}');
    }
  });
}
