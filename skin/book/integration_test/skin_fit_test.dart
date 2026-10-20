// Every word of the window fits in every skin: in its own look and in skins that push it (larger parts, larger type, a wide
// font, more air), no word is broken or left out and nothing overflows.
//   flutter drive --profile -d macos --driver=test_driver/integration_test.dart --target=integration_test/skin_fit_test.dart
import 'dart:io';

import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:lab_book/theme.dart';
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/workspace.dart' show Workspace;
import 'package:lab_book/workspace/ws.dart';
import 'package:lab_book/workspace/zoom.dart';

const _skins = {
  'own': '',
  'size 1.3': '<Size Value="1.3"/>',
  'size 0.85': '<Size Value="0.85"/>',
  'type 1.25': '<TextScale Value="1.25"/>',
  'wide font': '<FontSans Value="Monaco"/><FontMono Value="Monaco"/>',
  'air 1.5': '<Spacing Value="1.5"/>',
  'not numbers': '<Size Value="NaN"/><Spacing Value="Infinity"/><TextScale Value="big"/><Corner Value="-Infinity"/>',
  'size -3': '<Size Value="-3"/>',
  'type 0': '<TextScale Value="0"/>',
  'size 40': '<Size Value="40"/>',
  'type 40': '<TextScale Value="40"/>',
  'air 2': '<Spacing Value="2"/>',
  'air 40': '<Spacing Value="40"/>',
  'corner -2': '<Corner Value="-2"/>',
  'bad colour': '<Background Value="#zzzzzz"/>',
  'no font': '<FontSans Value="No Such Font"/>',
  'no parts': '<Parts Value="/nowhere"/>',
  'all at once': '<Size Value="1.25"/><TextScale Value="1.2"/><Spacing Value="1.3"/><FontSans Value="Monaco"/><FontMono Value="Monaco"/>',
};

/// The words cut: a word broken across lines, or lines left out.
List<String> _cut(RenderObject root) {
  final out = <String>[];
  void walk(RenderObject r) {
    if (r is RenderParagraph && r.hasSize) {
      final text = r.text.toPlainText();
      // (cut: a word broken, its box narrower than the widest word; or lines left out, an ellipsis. Two lines of whole words fit)
      final word = r.getMinIntrinsicWidth(double.infinity);
      if (text.trim().isNotEmpty && (r.size.width + .5 < word || r.didExceedMaxLines)) {
        out.add('"$text" ${r.size.width.round()}/${r.getMaxIntrinsicWidth(double.infinity).round()}');
      }
    }
    r.visitChildren(walk);
  }

  walk(root);
  return out;
}

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  testWidgets('every word fits in every skin', (t) async {
    t.view.physicalSize = const Size(3200, 1920);
    addTearDown(t.view.resetPhysicalSize);
    t.binding.defaultBinaryMessenger.setMockMethodCallHandler(SystemChannels.menu, (_) async => null);
    WsZoom.set(1, keep: false);
    final dir = Directory.systemTemp.createTempSync('fit');
    ColorTheme.at = '${dir.path}/none.xml';
    await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => Workspace(document: '${dir.path}/fit.motolii')));
    await t.pumpAndSettle();
    final ws = WsScope.read(t.element(find.byType(WsTimeline))), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    final things = [
      cube,
      for (final (k, w) in [('light', 'sun'), ('shape', 'circle'), ('text', '/System/Library/Fonts/Supplemental/Arial Black.ttf'), ('camera', 'camera')]) m.add(k, w),
    ];
    m.key(cube, 'x', 0, -200);
    m.key(cube, 'x', 2, 200);
    ws
      ..reread()
      ..select('$cube');
    await t.pumpAndSettle();

    final semantics = t.ensureSemantics(); // (the shelves' tabs found by their names)
    final report = <String>[];
    final was = FlutterError.onError;
    for (final MapEntry(key: name, value: roles) in _skins.entries) {
      final overflows = <String>[];
      FlutterError.onError = (e) => overflows.add(e.exceptionAsString().split('\n').first); // (counted, not thrown: every skin is looked at)
      File('${dir.path}/$name.xml').writeAsStringSync('<Motolii><Theme>$roles</Theme></Motolii>');
      ColorTheme.watch(() => t.binding.reassembleApplication(), '${dir.path}/$name.xml');
      t.binding.reassembleApplication();
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 400))); // (a font read from the system)
      for (var k = 0; k < 10; k++) {
        await t.pump(const Duration(milliseconds: 50));
      }
      // (every shelf in front, and each kind of thing picked, then nothing: the window's states, each looked at)
      final cut = <String>{};
      for (final shelf in const ['Media', 'Create', 'Effects', 'Fonts', 'Colors']) {
        if (find.bySemanticsLabel(shelf).evaluate().isEmpty) { // (a shelf's tab pushed out of the window: the skin broke it)
          FlutterError.onError = was;
          fail('$name: the $shelf tab is not in the window');
        }
        await t.tap(find.bySemanticsLabel(shelf));
        await t.pump();
        expect(ws.shelf, shelf); // (that shelf is in front)
        for (final thing in [...things, null]) {
          thing == null ? ws.select(null) : ws.select('$thing');
          for (var k = 0; k < 4; k++) {
            await t.pump(const Duration(milliseconds: 50));
          }
          cut.addAll([for (final c in _cut(t.binding.renderViews.first)) '$c  ($shelf, ${thing ?? 'nothing'} picked)']);
        }
      }
      FlutterError.onError = was;
      report.add('$name: ${cut.length} cut, ${overflows.length} overflow');
      for (final c in {...cut, ...overflows}) {
        report.add('  $c');
      }
    }
    // ignore: avoid_print
    print('fit:\n${report.join('\n')}');
    File('/Users/motolii_maker/motolii/reports/skins/fit.txt').writeAsStringSync('${report.join('\n')}\n'); // (the same, kept)
    semantics.dispose();
    expect(report.where((l) => !l.startsWith(' ') && !l.endsWith(': 0 cut, 0 overflow')), isEmpty);
  });
}
