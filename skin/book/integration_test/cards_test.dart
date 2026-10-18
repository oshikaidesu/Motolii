// The cards-by-hand table in a real window, used the way a hand would, shot step by step (reports/card_ui/<use>/<step>.png).
//   flutter drive --profile -d macos --driver=test_driver/integration_test.dart --target=integration_test/cards_test.dart
import 'dart:io';
import 'dart:ui' show ImageByteFormat;

import 'package:flutter/rendering.dart' show OffsetLayer;
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:lab_book/sets/inspector/cards_set.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized().framePolicy = LiveTestWidgetsFlutterBindingFramePolicy.fullyLive;

  testWidgets('cards by hand, shot', (t) async {
    t.view.physicalSize = const Size(2560, 1640);
    addTearDown(t.view.resetPhysicalSize);
    late Directory out;
    var n = 0;
    Future<void> shot(String name) async {
      await t.pump(const Duration(milliseconds: 350));
      final root = t.binding.renderViews.first.layer! as OffsetLayer;
      final png = await (await root.toImage(Offset.zero & t.view.physicalSize)).toByteData(format: ImageByteFormat.png);
      File('${out.path}/${(++n).toString().padLeft(2, '0')}_$name.png').writeAsBytesSync(png!.buffer.asUint8List());
    }

    Future<TestGesture> carry(Offset from, Offset to) async {
      final g = await t.startGesture(from);
      await t.pump(const Duration(milliseconds: 60));
      for (var k = 1; k <= 12; k++) {
        await g.moveTo(Offset.lerp(from, to, k / 12)!);
        await t.pump(const Duration(milliseconds: 16));
      }
      return g;
    }

    Offset hand(String word) => t.getCenter(find.text(word).last);
    Offset tile(String label) => t.getCenter(find.text(label).first);
    Rect tileBox(String label) => t.getRect(find.ancestor(of: find.text(label).first, matching: find.byType(AnimatedContainer)).first);

    for (final (i, (name, _, _)) in cardUses.indexed) {
      out = Directory('/Users/motolii_maker/motolii/reports/card_ui/$name')..createSync(recursive: true);
      n = 0;
      await t.pumpWidget(WidgetsApp(key: ValueKey(i), color: const Color(0xFF000000), builder: (c, _) => cardsSet().useCases[i].builder(c)));
      await t.pump(const Duration(milliseconds: 500));
      await shot('start');
      switch (name) {
        case '重ねる':
          var g = await carry(hand('揺れ'), tile('半径'));
          await t.pump(const Duration(milliseconds: 600));
          await shot('sway_hovering_over_radius');
          await g.up();
          await shot('sway_laid_on_radius');
          await (await carry(hand('揺れ'), tile('半径'))).up();
          await shot('sway_again_x2');
          await (await carry(hand('ずれ'), tile('大きさ'))).up();
          await shot('stagger_on_size');
          await (await carry(hand('ノイズ'), tile('半径'))).up();
          await shot('noise_swaps_sway');
          final chip = tileBox('半径').topRight + const Offset(-12, 10);
          g = await carry(chip, const Offset(300, 300));
          await shot('pulling_noise_off');
          await g.up();
          await shot('noise_off');
        case '裏返す':
          await t.tap(find.text('表').first);
          await shot('flipped_back_to_front');
          await t.tap(find.text('裏').first);
          await shot('first_card_flipped');
        case '順番':
          final g = await carry(t.getCenter(find.text('波').first), t.getCenter(find.text('波').first) + const Offset(0, 150));
          await shot('carrying_wave_down');
          await g.up();
          await shot('copies_now_first');
        case '伏せる':
          await t.tap(find.text('伏せ').at(2));
          await shot('wave_face_up_again');
      }
    }
  });
}
