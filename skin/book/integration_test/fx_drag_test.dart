// An effect's card dragged by its ≡ with the mouse in a real window, its engine drawing: a blur let go on a halftone's
// card lays after it (golf_noise.md (101); the window test does it with a finger, a computer-use drag never reached
// Flutter). Shot before and after (build/fx_drag_1..2.png).
//   flutter drive --profile -d macos --driver=test_driver/integration_test.dart --target=integration_test/fx_drag_test.dart
import 'dart:io';
import 'dart:ui' show ImageByteFormat;

import 'package:flutter/gestures.dart' show PointerDeviceKind;
import 'package:flutter/rendering.dart' show OffsetLayer;
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:lab_book/motolii.dart' show motoliiDir;
import 'package:lab_book/sets/inspector/inspector_parts.dart' show ValuesCard;
import 'package:lab_book/theme.dart';
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/workspace.dart' show Workspace;
import 'package:lab_book/workspace/ws.dart';
import 'package:lab_book/workspace/zoom.dart';

import '../test/door.dart' show seen, settled;

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  testWidgets('a blur dragged by its ≡ with the mouse onto a halftone\'s card lays after it', (t) async {
    t.view.physicalSize = const Size(3200, 1920);
    addTearDown(t.view.resetPhysicalSize);
    final folder = Directory('${Directory.systemTemp.createTempSync('fxdrag').path}/Motolii')..createSync();
    WsZoom.set(1, keep: false);
    ColorTheme.at = '${folder.path}/none.xml'; // (the window's own skin)
    await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => Workspace(document: '${folder.path}/Untitled.motolii')));
    for (var i = 0; i < 100 && find.byType(WsTimeline).evaluate().isEmpty; i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 20)));
      await t.pump();
    }
    await t.pumpAndSettle();
    final ws = WsScope.read(t.element(find.byType(WsTimeline))), m = ws.inside!;
    Future<void> shot(String png) async {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 800)));
      await t.pump(const Duration(milliseconds: 50));
      final root = t.binding.renderViews.first.layer! as OffsetLayer;
      final bytes = await (await root.toImage(Offset.zero & t.view.physicalSize)).toByteData(format: ImageByteFormat.png);
      File('$motoliiDir/../skin/book/build/$png').writeAsBytesSync(bytes!.buffer.asUint8List());
    }

    final halftone = '$motoliiDir/effects/halftone.wgsl';
    final rect = m.add('shape', 'rect');
    expect(m.effect(rect, 'Gaussian Blur', 0), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$rect');
    expect(ws.screenToggle(halftone), isTrue);
    await settled(t, ws);
    expect(m.word(rect, 'fx'), 'blur $halftone');
    final grip = find.byKey(const ValueKey('card-drag:Gaussian Blur'));
    await seen(t, grip);
    await shot('fx_drag_1.png');
    final onto = t.getCenter(find.byWidgetPredicate((w) => w is ValuesCard && w.card.name == 'Halftone'));
    final hand = await t.startGesture(t.getCenter(grip), kind: PointerDeviceKind.mouse);
    for (var k = 1; k <= 12; k++) {   // (moved as a hand moves, a frame at a time)
      await hand.moveTo(Offset.lerp(t.getCenter(grip), onto, k / 12)!);
      await t.pump(const Duration(milliseconds: 16));
    }
    await hand.up();
    await settled(t, ws);
    expect(m.word(rect, 'fx'), '$halftone blur', reason: 'the blur after the halftone: ${m.trouble}');
    await seen(t, find.byKey(const ValueKey('card-drag:Gaussian Blur')));
    await shot('fx_drag_2.png');
  });
}
