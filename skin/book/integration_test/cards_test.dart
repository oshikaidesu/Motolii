// The cards-by-hand table in a real window, used the way a hand would, shot step by step (reports/card_ui/<use>/<step>.png).
//   flutter drive --profile -d macos --driver=test_driver/integration_test.dart --target=integration_test/cards_test.dart
import 'dart:io';
import 'dart:ui' show ImageByteFormat;

import 'package:flutter/rendering.dart' show OffsetLayer;
import 'package:flutter/services.dart' show LogicalKeyboardKey, PhysicalKeyboardKey, TextInputAction;
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:lab_book/sets/inspector/cards_set.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized().framePolicy = LiveTestWidgetsFlutterBindingFramePolicy.fullyLive;

  testWidgets('cards by hand, shot', (t) async {
    t.view.physicalSize = const Size(2560, 1640);
    t.testTextInput.register(); // (typing goes through the test's own keyboard, as a hand's would)
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
          await shot('sway_again_wider');
          g = await carry(hand('ずれ'), tile('数'));
          await shot('stagger_refused_by_count');
          await g.up();
          await t.pump(const Duration(milliseconds: 200));
          await (await carry(hand('ずれ'), tile('大きさ'))).up();
          await shot('stagger_on_size');
          await (await carry(hand('ノイズ'), tile('半径'))).up();
          await shot('noise_swaps_sway');
          Offset chip(String label) => tileBox(label).topRight + const Offset(-12, 10);
          await t.tapAt(chip('半径'));
          await shot('noise_knobs_open');
          final width = t.getCenter(find.text('幅').first);
          await (await carry(width, width + const Offset(-25, 0))).up();
          await shot('noise_narrower');
          await (await carry(chip('半径'), tile('1つずつ回す'))).up();
          await shot('noise_moved_to_spin');
          g = await carry(chip('1つずつ回す'), hand('ランダム'));
          await shot('noise_carried_to_hand');
          await g.up();
          await shot('noise_back_in_hand');
          await t.tapAt(const Offset(1280 - 8 - 26 - 2 - 13, 15));
          await shot('undo_brings_noise_back');
          await t.tap(find.text('180px').first);
          await t.pump(const Duration(milliseconds: 200));
          await t.enterText(find.byType(EditableText), '240');
          await shot('typing_radius');
          await t.testTextInput.receiveAction(TextInputAction.done);
          await shot('radius_typed_240');
          await t.tap(find.text('ずれ').last);
          await shot('tapped_stagger_goes_on_radius');
          await t.tapAt(chip('大きさ'));
          await t.pump(const Duration(milliseconds: 200));
          await t.tap(find.text('中').first);
          await shot('stagger_starts_from_the_middle');
        case '裏返す':
          final row = t.getCenter(find.text('1つずつの大きさ').first);
          final g = await carry(hand('揺れ'), row);
          await shot('sway_over_a_back_row');
          await g.up();
          await shot('sway_on_a_back_row');
          await (await carry(hand('ずれ'), t.getCenter(find.text('位置 X').first))).up();
          await shot('stagger_on_position_x');
        case '順番':
          final name = t.getRect(find.text('波').first), grip = Offset(name.left - 8 - 14 - 9, name.center.dy);
          final g = await carry(grip, grip + const Offset(0, 150));
          await shot('carrying_wave_down');
          await g.up();
          await shot('copies_now_first');
        case '投げる':
          var g = await carry(hand('散らす'), const Offset(400, 400));
          await t.pump(const Duration(milliseconds: 400));
          await shot('scatter_held_over_stage');
          await g.up();
          await shot('scatter_worn');
          await (await carry(hand('波'), const Offset(1100, 600))).up();
          await shot('wave_dropped_on_inspector');
          g = await carry(t.getCenter(find.text('色').first) + const Offset(-22, 0), t.getCenter(find.text('揺れ').last));
          await shot('colour_card_carried_to_shelf');
          await g.up();
          await shot('colour_card_on_shelf');
        case '束ねる':
          var g = await t.startGesture(tile('半径'));
          await t.pump(const Duration(milliseconds: 500));
          for (var k = 1; k <= 12; k++) {
            await g.moveTo(Offset.lerp(tile('半径'), tile('大きさ'), k / 12)!);
            await t.pump(const Duration(milliseconds: 16));
          }
          await shot('radius_lifted_over_size');
          await g.up();
          await shot('tied_radius_and_size');
          final bundle = t.getCenter(find.text('束1').first) + const Offset(120, 0);
          await (await carry(bundle, bundle + const Offset(-160, 0))).up();
          await shot('bundle_scrubbed_left');
          g = await t.startGesture(tile('1つずつ回す'));
          await t.pump(const Duration(milliseconds: 500));
          for (var k = 1; k <= 12; k++) {
            await g.moveTo(Offset.lerp(tile('1つずつ回す'), t.getCenter(find.text('大きさ').last), k / 12)!);
            await t.pump(const Duration(milliseconds: 16));
          }
          await g.up();
          await shot('spin_joins_bundle');
          expect(find.text('束1').evaluate().length, greaterThanOrEqualTo(4)); // (the bundle strip and three values' marks)
          final marks = find.text('束1').evaluate().length;
          await t.sendKeyDownEvent(LogicalKeyboardKey.metaLeft, physicalKey: PhysicalKeyboardKey.metaLeft, platform: 'macos'); // (profile builds keep no key names: the physical key is given)
          await t.sendKeyEvent(LogicalKeyboardKey.keyZ, physicalKey: PhysicalKeyboardKey.keyZ, platform: 'macos');
          await t.sendKeyUpEvent(LogicalKeyboardKey.metaLeft, physicalKey: PhysicalKeyboardKey.metaLeft, platform: 'macos');
          await shot('cmd_z_unties_spin');
          expect(find.text('束1').evaluate().length, marks - 1); // (⌘Z reaches the table: the last tie is taken back)
        case '舞台に落とす':
          var g = await carry(hand('揺れ'), const Offset(420, 420));
          await t.pump(const Duration(milliseconds: 400)); // (the hand rests: the ring opens)
          await shot('sway_rests_ring_opens');
          await g.moveBy(const Offset(70, 0));
          await t.pump(const Duration(milliseconds: 400));
          await shot('sway_ring_aims_radius');
          await g.moveBy(const Offset(-70, -70));
          await t.pump(const Duration(milliseconds: 400));
          await shot('sway_ring_aims_size');
          await g.up();
          await shot('sway_on_size_from_stage');
          g = await carry(hand('ランダム'), const Offset(500, 400));
          await t.pump(const Duration(milliseconds: 400));
          await g.moveBy(const Offset(-70, 0));
          await t.pump(const Duration(milliseconds: 300));
          await shot('random_ring_aims_hue');
          await g.up();
          await shot('random_on_hue');
        case '伏せる':
          await t.tapAt(Offset(1280 - 8 - 4 - 12, t.getCenter(find.text('波').first).dy));
          await shot('wave_face_up_again');
      }
    }
  });
}
