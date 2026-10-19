// The window in its own skin and in each skin folder it is given (--dart-define=SKINS=/a,/b; skins/shoot.sh says them),
// each shot as preview.png in its folder (and, told OWN, the own skin as reports/skins/own.png): a skin changes how the window looks and
// nothing it does.
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
  testWidgets('the window in each skin', (t) async {
    t.view.physicalSize = const Size(3200, 1920);
    addTearDown(t.view.resetPhysicalSize);
    t.binding.defaultBinaryMessenger.setMockMethodCallHandler(SystemChannels.menu, (_) async => null);
    final out = Directory('/Users/motolii_maker/motolii/reports/skins')..createSync(recursive: true);
    final doc = '${Directory.systemTemp.createTempSync('skin').path}/skin.motolii';
    WsZoom.set(1, keep: false); // (shot at 100%, whatever this Mac's window.xml keeps)
    await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => Workspace(document: doc)));
    await t.pumpAndSettle();
    final m = WsScope.read(t.element(find.byType(WsTimeline))).inside!;
    for (final (k, f) in [('solid', 'cube'), ('solid', 'ring'), ('light', 'sun')]) {
      m.add(k, f);
    }
    Future<void> shot(String theme, String png) async {
      ColorTheme.watch(() => t.binding.reassembleApplication(), theme);
      t.binding.reassembleApplication();
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 600))); // (the skin's pictures and fonts read from disk)
      for (var k = 0; k < 20; k++) {
        await t.pump(const Duration(milliseconds: 50));
      }
      final root = t.binding.renderViews.first.layer! as OffsetLayer;
      final bytes = await (await root.toImage(Offset.zero & t.view.physicalSize)).toByteData(format: ImageByteFormat.png);
      File(png).writeAsBytesSync(bytes!.buffer.asUint8List());
      // ignore: avoid_print
      print('shot: $png');
    }

    if (const bool.fromEnvironment('OWN')) await shot('${Directory.systemTemp.createTempSync('skin').path}/none.xml', '${out.path}/own.png'); // (no file: the window's own skin)
    for (final dir in const String.fromEnvironment('SKINS').split(',').where((d) => d.isNotEmpty)) {
      await shot('$dir/theme.xml', '$dir/preview.png');
      final parts = Directory(ColorTheme.place(ColorTheme.word('Parts', '')));
      final svgs = ColorTheme.word('Parts', '').isEmpty || !parts.existsSync() ? 0 : parts.listSync().where((f) => f.path.endsWith('.svg')).length;
      expect(SkinParts.count, svgs, reason: '$dir: every picture in its parts folder read');
    }
    WsZoom.set(1.25, keep: false); // (the whole window 125%: not kept, the home folder's window.xml stays as it is)
    await t.pump();
    expect(find.text('125%'), findsOneWidget);
    WsZoom.set(1, keep: false);
  });
}
