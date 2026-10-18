// The pen on the live picture: points put by taps, one dragged out into a curve, back on the first point to close it,
// and the next tap beginning another shape; the path the document holds is read back and the window shot.
//   flutter drive --profile -d macos --driver=test_driver/integration_test.dart --target=integration_test/pen_test.dart
import 'dart:io';
import 'dart:ui' show ImageByteFormat;

import 'package:flutter/rendering.dart' show OffsetLayer;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/workspace/stage_view.dart';
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/workspace.dart' show Workspace;
import 'package:lab_book/workspace/ws.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized().framePolicy = LiveTestWidgetsFlutterBindingFramePolicy.fullyLive;

  testWidgets('the pen draws a path shape where the hand goes', (t) async {
    t.binding.defaultBinaryMessenger.setMockMethodCallHandler(SystemChannels.menu, (_) async => null);
    final doc = '${Directory.systemTemp.createTempSync('pen').path}/pen.motolii';
    await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => Workspace(document: doc)));
    await t.pumpAndSettle();
    final ws = WsScope.read(t.element(find.byType(WsTimeline))), m = ws.inside!;
    final out = Directory('$motoliiDir/../reports/pen')..createSync(recursive: true);
    Future<void> wait(int ms) => t.runAsync(() => Future<void>.delayed(Duration(milliseconds: ms)));
    Future<void> shoot(String name) async {
      await t.pump();
      // ignore: invalid_use_of_protected_member
      final root = t.binding.renderViews.first.layer! as OffsetLayer;
      final png = await (await root.toImage(Offset.zero & t.view.physicalSize)).toByteData(format: ImageByteFormat.png);
      File('${out.path}/$name.png').writeAsBytesSync(png!.buffer.asUint8List());
    }

    await t.tap(find.byKey(const ValueKey('ws-tool-4')));
    await t.pump();
    final mid = t.getCenter(find.byType(StageView));
    Future<void> tap(Offset at) async {
      await t.tapAt(mid + at);
      await wait(120);
    }

    Future<void> drag(Offset at, Offset handle) async {
      final hand = await t.startGesture(mid + at);
      for (var i = 1; i <= 8; i++) {
        await hand.moveTo(mid + at + handle * (i / 8));
        await t.pump(const Duration(milliseconds: 16));
      }
      await hand.up();
      await wait(120);
    }

    final before = m.things.length;
    await tap(const Offset(0, 90));
    await drag(const Offset(-90, -10), const Offset(0, -50));
    await tap(const Offset(0, -80));
    await drag(const Offset(90, -10), const Offset(0, 50));
    await tap(const Offset(0, 90)); // (back on the first: closed)
    await tap(const Offset(-160, 120)); // (and the next begins another)
    await tap(const Offset(160, 120));
    await wait(400);
    ws.reread();
    final made = m.things.sublist(before);
    final paths = [for (final n in made) m.word(n, 'path')];
    // ignore: avoid_print
    print('PEN things ${made.length}: ${paths.join(' | ')}');
    await shoot('pen');
    expect(made.length, 2);
    expect(paths[0].startsWith('M') && paths[0].endsWith('Z') && paths[0].contains('S'), true);
    expect(paths[1].contains('L'), true);
  });
}
