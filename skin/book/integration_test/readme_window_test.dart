// The README's picture of the window: the example document (docs/examples/mv.sh makes it) open at 4 seconds, its ring
// picked, written as docs/media/window.jpg's source, docs/media/window.png.
//   HOME=… ../../docs/examples/mv.sh, then
//   flutter drive --profile -d macos --driver=test_driver/integration_test.dart --target=integration_test/readme_window_test.dart --dart-define=DOC=/path/to/mv.motolii
import 'dart:io';
import 'dart:ui' show ImageByteFormat;

import 'package:flutter/rendering.dart' show OffsetLayer;
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:lab_book/motolii.dart' show motoliiDir;
import 'package:lab_book/theme.dart';
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/workspace.dart' show Workspace;
import 'package:lab_book/workspace/ws.dart';
import 'package:lab_book/workspace/zoom.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  testWidgets('the window on the README example', (t) async {
    t.view.physicalSize = const Size(3200, 1920);
    addTearDown(t.view.resetPhysicalSize);
    const doc = String.fromEnvironment('DOC');
    expect(File(doc).existsSync(), isTrue, reason: 'make it with docs/examples/mv.sh');
    WsZoom.set(1, keep: false);
    ColorTheme.at = '${Directory.systemTemp.createTempSync('readme').path}/none.xml'; // (the window's own skin)
    await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => Workspace(document: doc)));
    for (var i = 0; i < 100 && find.byType(WsTimeline).evaluate().isEmpty; i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 20)));
      await t.pump();
    }
    await t.pumpAndSettle();
    final ws = WsScope.read(t.element(find.byType(WsTimeline)));
    ws
      ..frame = 4 * Ws.fps
      ..select('2'); // (the ring)
    await t.pumpAndSettle();
    await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 1500))); // (the inside draws the stage in real time)
    for (var k = 0; k < 20; k++) {
      await t.pump(const Duration(milliseconds: 50));
    }
    final root = t.binding.renderViews.first.layer! as OffsetLayer;
    final bytes = await (await root.toImage(Offset.zero & t.view.physicalSize)).toByteData(format: ImageByteFormat.png);
    File('$motoliiDir/../docs/media/window.png').writeAsBytesSync(bytes!.buffer.asUint8List());
  });
}
