// The window in its own skin and in a skin file (skins/labs.xml), shot side by side (reports/skins/<skin>.png): a skin changes
// colours, corners, type and shade, and nothing the window does.
//   flutter drive --profile -d macos --driver=test_driver/integration_test.dart --target=integration_test/skin_test.dart
import 'dart:io';
import 'dart:ui' show ImageByteFormat;

import 'package:flutter/rendering.dart' show OffsetLayer;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:lab_book/parts/skin_parts.dart';
import 'package:lab_book/theme.dart';
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/workspace.dart' show Workspace;
import 'package:lab_book/workspace/ws.dart';
import 'package:lab_book/workspace/zoom.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  testWidgets('the window in two skins', (t) async {
    t.view.physicalSize = const Size(3200, 1920);
    addTearDown(t.view.resetPhysicalSize);
    t.binding.defaultBinaryMessenger.setMockMethodCallHandler(SystemChannels.menu, (_) async => null);
    final out = Directory('/Users/motolii_maker/motolii/reports/skins')..createSync(recursive: true);
    final doc = '${Directory.systemTemp.createTempSync('skin').path}/skin.motolii';
    await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => Workspace(document: doc)));
    await t.pumpAndSettle();
    final m = WsScope.read(t.element(find.byType(WsTimeline))).inside!;
    for (final (k, f) in [('solid', 'cube'), ('solid', 'ring'), ('light', 'sun')]) {
      m.add(k, f);
    }
    Future<void> shot(String name) async {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 600))); // (the skin's pictures read from disk)
      for (var k = 0; k < 20; k++) {
        await t.pump(const Duration(milliseconds: 50));
      }
      final root = t.binding.renderViews.first.layer! as OffsetLayer;
      final png = await (await root.toImage(Offset.zero & t.view.physicalSize)).toByteData(format: ImageByteFormat.png);
      File('${out.path}/$name.png').writeAsBytesSync(png!.buffer.asUint8List());
    }

    final own = '${Directory.systemTemp.createTempSync('skin').path}/none.xml'; // (no file: the window's own skin, whatever the home folder holds)
    ColorTheme.watch(() => t.binding.reassembleApplication(), own);
    t.binding.reassembleApplication();
    await shot('own');
    ColorTheme.watch(() => t.binding.reassembleApplication(), '/Users/motolii_maker/motolii/skin/book/skins/labs.xml');
    t.binding.reassembleApplication();
    await shot('labs');
    WsZoom.set(1.25, keep: false); // (the whole window 125%: not kept, the home folder's window.xml stays as it is)
    await shot('labs_zoom125');
    expect(find.text('125%'), findsOneWidget);
    WsZoom.set(1, keep: false);
    expect(ColorTheme.number('Corner', 1), 3.5); // (the skin's numbers were read)
    expect(SkinParts.count, 8); // (and its drawn parts, from the folder it names beside it)
  });
}
