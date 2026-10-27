// reports/alight_checklist.md section A (colour that only the whole stage had, now on one layer): each row's effect
// put on the left of two photos in the window, a knob set where it is plainly seen, the stage shot; the right photo is
// the same picture without it (the effect is the layer's, not the stage's). Pictures in a temporary folder, its path
// printed: ALIGHT_A=<folder>.
//   flutter drive --profile -d macos --driver=test_driver/integration_test.dart --target=integration_test/alight_a_test.dart
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

// (the row, the bundled effect the checklist names, knobs set as the document names them: a thing's script knob,
//  doc_values.h doc_screen_knob; a colour's parts _x _y _z)
const _rows = <(String, String, Map<String, double>)>[
  ('Brightness / Contrast', 'Color Controls', {'bright': .35}),
  ('Color Temperature', 'White Point Adjust', {'newWhite_x': 1, 'newWhite_y': .85, 'newWhite_z': .6}),
  ('Color Tune', 'LGG', {'gain_z': .45}),
  ('Exposure / Gamma', 'Exposure Adjust', {'inputEV': 1.5}),
  ('Highlights and Shadows', 'Color Levels', {'minLevel': .35, 'maxLevel': .65}),
  ('Lens Flare', 'Lens Flare', {}),
  ('Light Glow', 'Bloom', {'intensity': .7}),
  ('Saturation / Vibrance', 'Vibrance', {'vibrance': .6}),
];

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  testWidgets('section A: colour on one layer', (t) async {
    t.view.physicalSize = const Size(3200, 1920);
    addTearDown(t.view.resetPhysicalSize);
    final folder = Directory('${Directory.systemTemp.createTempSync('alight_a').path}/Motolii')..createSync();
    WsZoom.set(1, keep: false);
    ColorTheme.at = '${folder.path}/none.xml';
    await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => Workspace(document: '${folder.path}/Untitled.motolii')));
    for (var i = 0; i < 100 && find.byType(WsTimeline).evaluate().isEmpty; i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 20)));
      await t.pump();
    }
    await t.pumpAndSettle();
    final ws = WsScope.read(t.element(find.byType(WsTimeline))), m = ws.inside!;
    final left = m.add('picture', '$motoliiDir/fx_sample.jpg'), right = m.add('picture', '$motoliiDir/fx_sample.jpg');
    m
      ..set(left, 'x', -.5)
      ..set(right, 'x', .5);
    Future<void> shot(String png) async {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 1500)));   // (the inside draws the stage in real time)
      for (var k = 0; k < 20; k++) {
        await t.pump(const Duration(milliseconds: 50));
      }
      final root = t.binding.renderViews.first.layer! as OffsetLayer;
      final bytes = await (await root.toImage(Offset.zero & t.view.physicalSize)).toByteData(format: ImageByteFormat.png);
      File('${folder.path}/$png').writeAsBytesSync(bytes!.buffer.asUint8List());
    }
    ws.reread();
    await shot('0_none.png');
    for (final (i, (row, effect, knobs)) in _rows.indexed) {
      final file = '$motoliiDir/effects/$effect.fs';
      expect(m.say(left, 'fx', file), isTrue, reason: '$row: ${m.trouble}');
      final stem = effect.replaceAll(RegExp(r'[^A-Za-z0-9_]'), '_');
      for (final MapEntry(:key, :value) in knobs.entries) {
        expect(m.set(left, 'js_${stem}_$key', value), isTrue, reason: '$row $key: ${m.trouble}');
      }
      ws.reread();
      await shot('${i + 1}_${row.replaceAll(RegExp(r'[^A-Za-z0-9]+'), '_')}.png');
      expect(m.say(left, 'fx', 'none'), isTrue, reason: m.trouble);
    }
    // ignore: avoid_print
    print('ALIGHT_A=${folder.path}');
  });
}
