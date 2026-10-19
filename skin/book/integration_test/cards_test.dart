// The cards-by-hand table in a real window, used the way a hand would, shot step by step (reports/card_ui/<use>/<step>.png).
//   flutter drive --profile -d macos --driver=test_driver/integration_test.dart --target=integration_test/cards_test.dart
import 'dart:io';
import 'dart:ui' show ImageByteFormat;

import 'package:flutter/gestures.dart' show PointerDeviceKind;
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
        case 'Lay':
          var g = await carry(hand('Oscillator'), tile('Radius'));
          await t.pump(const Duration(milliseconds: 600));
          await shot('oscillator_hovering_over_radius');
          await g.up();
          await shot('oscillator_laid_on_radius');
          await (await carry(hand('Oscillator'), tile('Radius'))).up();
          await shot('oscillator_again_wider');
          g = await carry(hand('Stagger'), tile('Count'));
          await shot('stagger_refused_by_count');
          await g.up();
          await t.pump(const Duration(milliseconds: 200));
          await (await carry(hand('Stagger'), tile('Size'))).up();
          await shot('stagger_on_size');
          await (await carry(hand('Noise'), tile('Radius'))).up();
          await shot('noise_swaps_oscillator');
          Offset chip(String label) => tileBox(label).topRight + const Offset(-12, 10);
          await t.tapAt(chip('Radius'));
          await shot('noise_knobs_open');
          final width = t.getCenter(find.text('Amount').first);
          await (await carry(width, width + const Offset(-25, 0))).up();
          await shot('noise_narrower');
          await (await carry(chip('Radius'), tile('Turn each'))).up();
          await shot('noise_moved_to_turn_each');
          g = await carry(chip('Turn each'), hand('Random'));
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
          await t.tap(find.text('Stagger').last);
          await shot('tapped_stagger_goes_on_radius');
          await t.tapAt(chip('Size'));
          await t.pump(const Duration(milliseconds: 200));
          await t.tap(find.text('MID').first);
          await shot('stagger_starts_from_the_middle');
        case 'Flip':
          final row = t.getCenter(find.text('Copy scale').first);
          final g = await carry(hand('Oscillator'), row);
          await shot('oscillator_over_a_back_row');
          await g.up();
          await shot('oscillator_on_a_back_row');
          await (await carry(hand('Stagger'), t.getCenter(find.text('Position X').first))).up();
          await shot('stagger_on_position_x');
        case 'Order':
          final name = t.getRect(find.text('Wave').first), grip = Offset(name.left - 8 - 14 - 9, name.center.dy);
          final g = await carry(grip, grip + const Offset(0, 150));
          await shot('carrying_wave_down');
          await g.up();
          await shot('copies_now_first');
        case 'Throw':
          var g = await carry(hand('Scatter'), const Offset(400, 400));
          await t.pump(const Duration(milliseconds: 400));
          await shot('scatter_held_over_stage');
          await g.up();
          await shot('scatter_worn');
          await (await carry(hand('Wave'), const Offset(1100, 600))).up();
          await shot('wave_dropped_on_inspector');
          g = await carry(t.getCenter(find.text('Fill').first) + const Offset(-22, 0), t.getCenter(find.text('Oscillator').last));
          await shot('fill_card_carried_to_shelf');
          await g.up();
          await shot('fill_card_on_shelf');
        case 'Group':
          var g = await t.startGesture(tile('Radius'), kind: PointerDeviceKind.mouse); // (a mouse, trembling a pixel while it holds: still a hold)
          await t.pump(const Duration(milliseconds: 120));
          await g.moveBy(const Offset(1, 0));
          await t.pump(const Duration(milliseconds: 400));
          for (var k = 1; k <= 12; k++) {
            await g.moveTo(Offset.lerp(tile('Radius'), tile('Size'), k / 12)!);
            await t.pump(const Duration(milliseconds: 16));
          }
          await shot('radius_lifted_over_size');
          await g.up();
          await shot('grouped_radius_and_size');
          final group = t.getCenter(find.text('G1').first) + const Offset(120, 0);
          await (await carry(group, group + const Offset(-160, 0))).up();
          await shot('group_scrubbed_left');
          g = await t.startGesture(tile('Turn each'));
          await t.pump(const Duration(milliseconds: 500));
          for (var k = 1; k <= 12; k++) {
            await g.moveTo(Offset.lerp(tile('Turn each'), t.getCenter(find.text('Size').last), k / 12)!);
            await t.pump(const Duration(milliseconds: 16));
          }
          await g.up();
          await shot('turn_each_joins_group');
          expect(find.text('G1').evaluate().length, greaterThanOrEqualTo(4)); // (the group strip and three values' marks)
          final marks = find.text('G1').evaluate().length;
          await t.sendKeyDownEvent(LogicalKeyboardKey.metaLeft, physicalKey: PhysicalKeyboardKey.metaLeft, platform: 'macos'); // (profile builds keep no key names: the physical key is given)
          await t.sendKeyEvent(LogicalKeyboardKey.keyZ, physicalKey: PhysicalKeyboardKey.keyZ, platform: 'macos');
          await t.sendKeyUpEvent(LogicalKeyboardKey.metaLeft, physicalKey: PhysicalKeyboardKey.metaLeft, platform: 'macos');
          await shot('cmd_z_ungroups_turn_each');
          expect(find.text('G1').evaluate().length, marks - 1); // (⌘Z reaches the table: the last tie is taken back)
        case 'Stage ring':
          var g = await carry(hand('Oscillator'), const Offset(420, 420));
          await t.pump(const Duration(milliseconds: 400)); // (the hand rests: the ring opens)
          await shot('oscillator_rests_ring_opens');
          await g.moveBy(const Offset(70, 0));
          await t.pump(const Duration(milliseconds: 400));
          await shot('oscillator_ring_aims_radius');
          await g.moveBy(const Offset(-70, -70));
          await t.pump(const Duration(milliseconds: 400));
          await shot('oscillator_ring_aims_size');
          await g.up();
          await shot('oscillator_on_size_from_stage');
          g = await carry(hand('Random'), const Offset(500, 400));
          await t.pump(const Duration(milliseconds: 400));
          await g.moveBy(const Offset(-70, 0));
          await t.pump(const Duration(milliseconds: 300));
          await shot('random_ring_aims_hue');
          await g.up();
          await shot('random_on_hue');
        case 'Face down':
          await t.tapAt(Offset(1280 - 8 - 4 - 12, t.getCenter(find.text('Wave').first).dy));
          await shot('wave_face_up_again');
        case 'Link':
          // the ring's radius breathes; carried onto three squares (their Rotation picked on the ring that opens there), all wear one tag
          Offset at(double x, double y) => Offset(460 + x * 820 / 720, 410 + y * 820 / 720); // (a place on the stage, as the things are placed)
          Offset onStage(String word) => find.text(word).evaluate().map((e) => t.getCenter(find.byElementPredicate((x) => x == e))).firstWhere((p) => p.dx < 900);
          Future<void> cmdZ() async {
            await t.sendKeyDownEvent(LogicalKeyboardKey.metaLeft, physicalKey: PhysicalKeyboardKey.metaLeft, platform: 'macos');
            await t.sendKeyEvent(LogicalKeyboardKey.keyZ, physicalKey: PhysicalKeyboardKey.keyZ, platform: 'macos');
            await t.sendKeyUpEvent(LogicalKeyboardKey.metaLeft, physicalKey: PhysicalKeyboardKey.metaLeft, platform: 'macos');
          }
          await (await carry(hand('Oscillator'), tile('Radius'))).up();
          await shot('ring_radius_breathes');
          for (final y in [-220.0, 0.0, 220.0]) {
            final g = await carry(tile('Radius'), at(190, y));
            await t.pump(const Duration(milliseconds: 100));
            await g.up();
            await t.pump(const Duration(milliseconds: 300));
            if (y == -220) await shot('radius_let_go_on_square_1_ring_waits');
            await t.tapAt(onStage('Rotation')); // (the ring opened on the square: its Rotation picked)
            await t.pump(const Duration(milliseconds: 200));
          }
          await shot('three_squares_wear_the_ring_radius_tag');
          expect(find.text('A'), findsWidgets); // (the ring's radius wears tag A)
          await t.tap(find.text('A').first);
          await shot('tag_a_wearers_listed');
          expect(find.textContaining(' · Rotation'), findsNWidgets(3));
          await t.tapAt(t.getRect(find.text('Square 2 · Rotation')).centerRight + const Offset(13, 0));
          await shot('square_2_takes_the_tag_off');
          expect(find.textContaining(' · Rotation'), findsNWidgets(2));
          await cmdZ();
          await shot('cmd_z_puts_it_back');
          expect(find.textContaining(' · Rotation'), findsNWidgets(3));
          final g = await carry(tile('Rotation'), at(340, -220)); // (carried onto square 4 and rested there: its ring opens; let go with nothing aimed)
          await t.pump(const Duration(milliseconds: 600));
          await shot('rotation_rests_on_square_4_ring_opens');
          await g.up();
          await shot('square_4_rotation_wears_b');
          expect(find.text('B'), findsWidgets); // (a rest before letting go still puts the tag on: the same-named value)
          await t.tapAt(at(190, 0));
          await shot('square_2_picked_its_rotation_wears_a');
      }
    }
  });
}
