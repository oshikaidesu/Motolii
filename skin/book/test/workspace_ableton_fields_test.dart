// The window's number fields as Ableton Live 12's (its manual, 42.4 and 42.6): a digit typed on a field starts its
// number, Delete puts it back to its default. Driven on the real inside, as a hand would.
import 'package:flutter/gestures.dart' show PointerDeviceKind, kSecondaryButton;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/motolii.dart' show motoliiDir;
import 'package:lab_book/workspace/dock_parts.dart' show DockSearch;
import 'package:lab_book/workspace/keys.dart' show WsKeys;
import 'package:lab_book/workspace/timeline.dart' show WsTimeline;
import 'package:lab_book/sets/inspector/inspector_parts.dart' show Chooser, Ctx, InspectorPanel, NumField, ValuesCard;

import 'door.dart';

void main() {
  testWidgets('a click picks a field and no more, a digit typed on it starts the number, Delete puts it back to its default', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final a = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$a');
    await t.pumpAndSettle();
    final x = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_x');
    await seen(t, x);
    await t.tap(x);   // (the field holds the keys)
    await t.pumpAndSettle();
    await t.sendKeyEvent(LogicalKeyboardKey.digit4);
    await t.pump();
    final typing = find.descendant(of: x, matching: find.byType(EditableText));
    expect(typing, findsOneWidget, reason: 'a digit starts the number');
    expect(t.widget<EditableText>(typing).controller.text, '4', reason: 'begun with what was typed, not the old number');
    await t.sendKeyEvent(LogicalKeyboardKey.escape);
    await t.pumpAndSettle();
    expect(m.set(a, 'x', 100), isTrue, reason: m.trouble);
    ws.reread();
    await t.pumpAndSettle();
    final was = m.value(a, 'x', ws.frame / 60);
    await t.tap(x);
    await t.pumpAndSettle();
    expect(find.descendant(of: x, matching: find.byType(EditableText)), findsNothing, reason: 'a click picks the field, no more (Live)');
    await t.sendKeyEvent(LogicalKeyboardKey.delete);
    await t.pumpAndSettle();
    expect(m.value(a, 'x', ws.frame / 60), isNot(was), reason: 'Delete: back to its default');
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('what is typed before the editor has the keys goes on the number; Cmd+A on a field held is all of its number, not every thing (After Effects, Figma)', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final a = m.add('shape', 'rect');
    m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$a');
    await t.pumpAndSettle();
    final x = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_x');
    await seen(t, x);
    await t.tap(x);
    await t.pumpAndSettle();
    for (final (k, c) in [(LogicalKeyboardKey.digit1, '1'), (LogicalKeyboardKey.digit0, '0'), (LogicalKeyboardKey.digit5, '5')]) {
      await simulateKeyDownEvent(k, character: c);   // (no frame between them)
      await simulateKeyUpEvent(k);
    }
    await t.pump();
    final typing = find.descendant(of: x, matching: find.byType(EditableText));
    expect(t.widget<EditableText>(typing).controller.text, '105', reason: 'the 2nd and 3rd keys came before the frame that gives the editor the keys');
    await t.sendKeyEvent(LogicalKeyboardKey.escape);
    await t.pumpAndSettle();
    await t.tap(x);
    await t.pumpAndSettle();
    t.widget<WsKeys>(find.byType(WsKeys)).commands['edit.selectAll']!();
    await t.pump();
    await t.pump();
    final words = t.widget<EditableText>(typing).controller;
    expect([ws.many.length, words.selection.start, words.selection.end], [0, 0, words.text.length], reason: 'the things stay as they were, the number is all picked: a digit takes its place');
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('the time pressed from another field takes the keys: all of it picked, what is typed takes its place (golf_noise.md (111))', (t) async {
    await openWorkspace(t);
    final search = find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText));
    await t.tap(search, kind: PointerDeviceKind.mouse);   // (a mouse: a press outside a field takes the keys from it, a finger's does not on the Mac's own default)
    await t.pump();
    await t.pump();
    await t.tap(find.byKey(const ValueKey('ws-tl-time')), kind: PointerDeviceKind.mouse);
    for (var i = 0; i < 4; i++) {
      await t.pump();
    }
    final words = t.widget<EditableText>(find.descendant(of: find.byKey(const ValueKey('ws-tl-time-words')), matching: find.byType(EditableText)));
    expect([words.focusNode.hasPrimaryFocus, words.controller.selection.start, words.controller.selection.end], [true, 0, words.controller.text.length]);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('a text\'s script comes off by the bin on its card, as an effect does (Alight Motion\'s Delete)', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final a = m.add('text', '$motoliiDir/out/arial_black.ttf');
    expect(m.say(a, 'script', '$motoliiDir/animators/bounce_in.js'), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$a');
    await t.pumpAndSettle();
    final bin = find.byKey(const ValueKey('card-remove:Script'));
    await seen(t, bin);
    await t.tap(bin);
    await settled(t, ws);
    expect([m.word(a, 'script'), bin.evaluate().length], ['', 0], reason: 'its card goes with it');
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('a sum typed in a field is worked out, one begun with * or / from the value as it is; what is no sum keeps the field open (After Effects, Figma, Blender)', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final a = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$a');
    await t.pumpAndSettle();
    expect(m.workOut('1/3'), closeTo(1 / 3, 1e-12), reason: 'not whole numbers\' division');
    expect(m.workOut('(1+2)*3 - 1e1'), closeTo(-1, 1e-12));
    expect(m.workOut('select 1'), isNull);
    expect(m.workOut('1+'), isNull);
    expect(m.workOut('1/0'), isNull, reason: 'SQLite: no number');
    final x = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_x');
    final typing = find.descendant(of: x, matching: find.byType(EditableText));
    Future<void> type(String said) async {
      await seen(t, x);
      await t.tap(x);
      await t.pumpAndSettle();
      await t.sendKeyEvent(LogicalKeyboardKey.enter);
      await t.pumpAndSettle();
      t.widget<EditableText>(typing).controller.text = said;
      await t.sendKeyEvent(LogicalKeyboardKey.enter);
      await settled(t, ws);
    }
    await type('(100+20)/3');
    double? shown() => Ctx.read(t.element(x)).doc.get('v_x');
    expect(shown(), closeTo(40, .06));
    await type('*2');
    expect(shown(), closeTo(80, .1), reason: 'from the value as it is');
    await type('1+');
    expect(typing, findsOneWidget, reason: 'no sum: the field stays open');
    await t.sendKeyEvent(LogicalKeyboardKey.escape);
    await t.pumpAndSettle();
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, tags: 'main', skip: !insideBuilt);

  testWidgets('an effect\'s number goes past its knob when typed, never past 0, and a scrub stays where it was typed (After Effects\' slider and valid ranges)', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final a = m.add('shape', 'rect');
    expect(m.say(a, 'fx', '$motoliiDir/effects/am_circularripple.wgsl'), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$a');
    await t.pumpAndSettle();
    const radius = 'js_am_circularripple_radius';
    final x = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_$radius');
    final typing = find.descendant(of: x, matching: find.byType(EditableText));
    Future<void> type(String said) async {
      await seen(t, x);
      await t.tap(x);
      await t.pumpAndSettle();
      await t.sendKeyEvent(LogicalKeyboardKey.enter);
      await t.pumpAndSettle();
      t.widget<EditableText>(typing).controller.text = said;
      await t.sendKeyEvent(LogicalKeyboardKey.enter);
      await settled(t, ws);
    }
    await type('16');
    expect(m.value(a, radius, 0), 16, reason: 'its author\'s 0.8 is where the knob ends, not all it takes');
    await type('-3');
    expect(m.value(a, radius, 0), 0, reason: 'a range from 0 holds at 0');
    await type('4');
    final at = t.getCenter(x), hand = await t.startGesture(at);
    await hand.moveBy(const Offset(12, 0));
    await hand.moveBy(const Offset(6, 0));
    await t.pump();
    await hand.up();
    await settled(t, ws);
    expect(m.value(a, radius, 0), greaterThanOrEqualTo(4), reason: 'a scrub does not throw it back to the knob\'s end');
    expect(m.set(a, radius, -1), isFalse);
    await closeWorkspace(t);
  }, tags: 'main', skip: !insideBuilt);

  testWidgets('a right click on any number offers Reset, Copy, Paste and its keys, on a row or an effect\'s knob alike (After Effects, Blender; all by the mouse alone)', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final a = m.add('shape', 'rect');
    expect(m.effect(a, 'Gaussian Blur', 0), isTrue, reason: m.trouble);
    expect(m.set(a, 'x', 100), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$a');
    await t.pumpAndSettle();
    final x = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_x');
    await seen(t, x);
    Future<void> menu(Finder f) async {
      await t.tap(f, buttons: kSecondaryButton);
      await t.pumpAndSettle();
    }
    await menu(x);
    expect(find.byKey(const ValueKey('num-key:v_x')), findsOneWidget, reason: 'its key here');
    await t.tap(find.byKey(const ValueKey('num-key:v_x')));
    await settled(t, ws);
    expect(m.keyMoments(a, 'x'), hasLength(1), reason: m.trouble);
    await menu(x);
    await t.tap(find.byKey(const ValueKey('num-unanimate:v_x')));
    await settled(t, ws);
    expect(m.keyMoments(a, 'x'), isEmpty);
    await menu(x);
    await t.tap(find.byKey(const ValueKey('num-reset:v_x')));
    await settled(t, ws);
    expect(m.value(a, 'x', 0), isNot(100), reason: 'one value back to what it was, by the mouse alone');
    final label = find.text('Scale');
    await seen(t, label);
    final tall = t.getSize(find.byType(InspectorPanel)).height, below = t.getTopLeft(find.text('Rotation'));
    await menu(label.first);
    expect(find.byKey(const ValueKey('row-key:v_scale')), findsOneWidget, reason: 'a row\'s label has its menu');
    expect([t.getSize(find.byType(InspectorPanel)).height, t.getTopLeft(find.text('Rotation'))], [tall, below], reason: 'over the panel, the rows where they were (10/30: it was laid in among them)');
    await t.tapAt(const Offset(5, 5));
    await t.pumpAndSettle();
    expect(find.byKey(const ValueKey('row-key:v_scale')), findsNothing, reason: 'a press elsewhere closes it');
    expect(m.set(a, 'x', 100) && m.set(a, 'rotate', 30), isTrue, reason: m.trouble);
    ws.reread();
    await t.pumpAndSettle();
    final whole = find.byKey(const ValueKey('card-reset:Transform'));
    await seen(t, whole);
    await t.tap(whole);
    await settled(t, ws);
    expect([m.value(a, 'x', 0), m.value(a, 'rotate', 0)], [isNot(100), 0], reason: 'the Transform card\'s Reset, as the other cards\': ${m.trouble}');
    final blur = find.byWidgetPredicate((w) => w is NumField && w.id.endsWith('_blur'));
    await seen(t, blur);
    await menu(blur.first);
    expect(find.text('Reset'), findsWidgets, reason: 'an effect\'s knob has the same menu: it had none');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, tags: 'main', skip: !insideBuilt);

  testWidgets('a list\'s keys: Down opens it, the arrows go along it, Return takes one, Esc leaves it, a letter goes to the first that starts so (W3C ARIA Authoring Practices, Select-Only Combobox)', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final a = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$a');
    await t.pumpAndSettle();
    final list = find.byWidgetPredicate((w) => w is Chooser && w.id == 'v_blend');
    await seen(t, list);
    await t.tap(list);
    await t.pumpAndSettle();
    await t.sendKeyEvent(LogicalKeyboardKey.arrowDown);
    await t.pumpAndSettle();
    await t.sendKeyEvent(LogicalKeyboardKey.enter);
    await settled(t, ws);
    final second = m.word(a, 'blend');
    expect(second, isNotEmpty, reason: 'the next one down: ${m.trouble}');
    await t.tap(list);
    await t.pumpAndSettle();
    await t.sendKeyEvent(LogicalKeyboardKey.keyS);
    await t.pumpAndSettle();
    await t.sendKeyEvent(LogicalKeyboardKey.enter);
    await settled(t, ws);
    expect(m.word(a, 'blend'), startsWith('s'), reason: 'the first that starts so');
    final chosen = m.word(a, 'blend');
    await t.tap(list);
    await t.pumpAndSettle();
    await t.sendKeyEvent(LogicalKeyboardKey.arrowDown);
    await t.pumpAndSettle();
    await t.sendKeyEvent(LogicalKeyboardKey.escape);
    await settled(t, ws);
    expect(m.word(a, 'blend'), chosen, reason: 'Esc leaves it as it was');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, tags: 'main', skip: !insideBuilt);

  testWidgets('Option on a card\'s head folds all the cards, and opens them again (the Mac\'s Option on a disclosure triangle, Finder\'s list)', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final a = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$a');
    await t.pumpAndSettle();
    final doc = Ctx.read(t.element(find.byType(InspectorPanel))).doc;
    bool folded(String card) => doc.b['fold.$card'] ?? false;
    final blend = find.text('Transform');
    await seen(t, blend);
    await t.sendKeyDownEvent(LogicalKeyboardKey.altLeft);
    await t.tap(blend.first);
    await t.sendKeyUpEvent(LogicalKeyboardKey.altLeft);
    await t.pumpAndSettle();
    expect(doc.cards.map((c) => c.name).where(folded).length, doc.cards.length, reason: 'all of them');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, tags: 'main', skip: !insideBuilt);

  testWidgets('Add Effect in the inspector opens the Effects shelf with its search taking the keys: Wave Warp a word away (Alight Motion\'s Add Effect)', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final a = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$a')
      ..shelf = 'Media';
    await t.pumpAndSettle();
    final add = find.byKey(const ValueKey('add-effect'));
    await seen(t, add);
    await t.tap(add);
    await t.pumpAndSettle();
    expect(ws.shelf, 'Effects');
    await t.pump();
    final search = find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText));
    expect(t.widget<EditableText>(search).focusNode.hasFocus, isTrue, reason: 'ready to be typed in');
    await t.enterText(search, 'wave warp');
    await settled(t, ws);
    expect(find.text('Wave Warp'), findsWidgets);
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, tags: 'main', skip: !insideBuilt);

  testWidgets('Copy all under a thing\'s effects, Paste on another: its whole stack goes over, in order, its knobs with it (Alight Motion\'s Copy Effects, Paste Effects)', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final a = m.add('shape', 'rect'), b = m.add('shape', 'circle');
    final halftone = '$motoliiDir/effects/halftone.wgsl';
    expect(m.effect(a, 'Gaussian Blur', 0) && m.set(a, 'blur', 33), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$a');
    expect(ws.screenToggle(halftone), isTrue);
    await settled(t, ws);
    Future<void> press(String key) async {
      final k = find.byKey(ValueKey(key));
      await seen(t, k);
      await t.tap(k);
      await settled(t, ws);
    }
    await press('effects-copy');
    ws.select('$b');
    await t.pumpAndSettle();
    await press('effects-paste');
    expect(m.word(b, 'fx'), m.word(a, 'fx'), reason: 'the stack, in order: ${m.trouble}');
    expect(m.value(b, 'blur', 0), 33, reason: 'its knobs with it');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, tags: 'main', skip: !insideBuilt);

  testWidgets('the Transform card\'s ⋮ flips and fits, a row\'s right click resets that row alone (Alight Motion\'s ⋯ on Move & Transform; they lay in the menu bar alone)', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final a = m.add('shape', 'rect');
    expect(m.set(a, 'rotate', 30) && m.set(a, 'x', 1), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$a');
    await t.pumpAndSettle();
    final more = find.byKey(const ValueKey('card-more:Transform'));
    await seen(t, more);
    await t.tap(more);
    await t.pumpAndSettle();
    await t.tap(find.byKey(const ValueKey('card-flipX:Transform')));
    await settled(t, ws);
    expect(m.value(a, 'scale_x', 0), -1, reason: 'flipped: ${m.trouble}');
    await t.tap(more);
    await t.pumpAndSettle();
    await t.tap(find.byKey(const ValueKey('card-stretch:Transform')));
    await settled(t, ws);
    expect(m.value(a, 'scale_x', 0)!.abs(), isNot(1), reason: 'stretched to the picture: ${m.trouble}');
    final label = find.text('Rotation');
    await seen(t, label);
    await t.tap(label.first, buttons: kSecondaryButton);
    await t.pumpAndSettle();
    await t.tap(find.byKey(const ValueKey('row-reset:v_rotate')));
    await settled(t, ws);
    expect([m.value(a, 'rotate', 0), m.value(a, 'x', 0)], [0, isNot(0)], reason: 'that row alone');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, tags: 'main', skip: !insideBuilt);

  testWidgets('a menu shuts when anything else is touched, by either button, and on Esc; another right click opens the next in its place', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final a = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$a');
    await t.pumpAndSettle();
    final x = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_x'), y = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_z');
    final number = find.byKey(const ValueKey('num-reset:v_x')), other = find.byKey(const ValueKey('num-reset:v_z')), stage = find.byKey(const ValueKey('ws-stage-comp'));
    await seen(t, x);
    Future<void> open() async {
      await t.tap(x, buttons: kSecondaryButton);
      await t.pumpAndSettle();
      expect(number, findsOneWidget);
    }
    await open();
    await t.tapAt(t.getCenter(stage));
    await t.pumpAndSettle();
    expect(number, findsNothing, reason: 'a press on the stage');
    await open();
    await t.tapAt(t.getCenter(stage), buttons: kSecondaryButton);
    await t.pumpAndSettle();
    expect(number, findsNothing, reason: 'a right press elsewhere');
    await open();
    await t.sendKeyEvent(LogicalKeyboardKey.escape);
    await t.pumpAndSettle();
    expect(number, findsNothing, reason: 'Esc');
    await open();
    await t.tap(y, buttons: kSecondaryButton);
    await t.pumpAndSettle();
    expect([number.evaluate().length, other.evaluate().length], [0, 1], reason: 'the next one, in its place');
    await t.tapAt(t.getCenter(stage));
    await t.pumpAndSettle();
    final ruler = find.byType(WsTimeline), at = t.getTopLeft(ruler) + const Offset(400, 34);
    await t.tapAt(at, buttons: kSecondaryButton);
    await t.pumpAndSettle();
    final menu = find.byKey(const ValueKey('ws-menu'));
    expect(menu, findsOneWidget, reason: 'the ruler\'s own');
    await t.tapAt(t.getCenter(stage));
    await t.pumpAndSettle();
    expect(menu, findsNothing, reason: 'the window\'s menu shuts on a press elsewhere');
    await t.tapAt(at, buttons: kSecondaryButton);
    await t.pumpAndSettle();
    await t.tapAt(t.getCenter(stage), buttons: kSecondaryButton);
    await t.pumpAndSettle();
    expect(menu, findsNothing, reason: 'and on a right press elsewhere');
    await t.tapAt(at, buttons: kSecondaryButton);
    await t.pumpAndSettle();
    await t.sendKeyEvent(LogicalKeyboardKey.escape);
    await t.pumpAndSettle();
    expect(menu, findsNothing, reason: 'and on Esc');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, tags: 'main', skip: !insideBuilt);

  testWidgets('nothing picked, the inspector shows nothing unasked: the stage\'s own cards when Stage settings is pressed (After Effects\' Effect Controls, Alight Motion, Apple Motion\'s Project)', (t) async {
    final ws = await openWorkspace(t);
    ws.select(null);
    await t.pumpAndSettle();
    expect(find.text('Nothing selected'), findsWidgets);
    expect(find.byType(ValuesCard), findsNothing, reason: 'no card unasked');
    await t.tap(find.byKey(const ValueKey('stage-settings')));
    await t.pumpAndSettle();
    expect(find.byType(ValuesCard), findsWidgets, reason: 'asked for');
    await t.tap(find.byKey(const ValueKey('stage-settings')));
    await t.pumpAndSettle();
    expect(find.byType(ValuesCard), findsNothing, reason: 'and put away');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, tags: 'main', skip: !insideBuilt);

  testWidgets('typing a number: Tab keeps it and opens the next field, Shift-Tab the one before; the arrows move what is typed; nothing typed is as Esc (After Effects, Figma, Blender)', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final a = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$a');
    await t.pumpAndSettle();
    Finder field(String id) => find.byWidgetPredicate((w) => w is NumField && w.id == id);
    Finder typing(String id) => find.descendant(of: field(id), matching: find.byType(EditableText));
    await seen(t, field('v_x'));
    await t.tap(field('v_x'));
    await t.pumpAndSettle();
    await t.sendKeyEvent(LogicalKeyboardKey.digit4);
    await t.sendKeyEvent(LogicalKeyboardKey.digit2);
    await t.pump();
    await t.sendKeyEvent(LogicalKeyboardKey.tab);
    await t.pumpAndSettle();
    await settled(t, ws);
    expect(typing('v_x'), findsNothing, reason: 'kept');
    expect(typing('v_y'), findsOneWidget, reason: 'the next field open to be typed in');
    t.widget<EditableText>(typing('v_y')).controller.text = '7';
    await t.sendKeyEvent(LogicalKeyboardKey.arrowUp);
    await t.pump();
    expect(double.parse(t.widget<EditableText>(typing('v_y')).controller.text), closeTo(8, 1e-9), reason: 'the number typed so far moved a step');
    await t.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
    await t.sendKeyEvent(LogicalKeyboardKey.tab);
    await t.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
    await t.pumpAndSettle();
    await settled(t, ws);
    expect(typing('v_x'), findsOneWidget, reason: 'the field before it');
    t.widget<EditableText>(typing('v_x')).controller.clear();
    await t.sendKeyEvent(LogicalKeyboardKey.enter);
    await t.pumpAndSettle();
    expect(typing('v_x'), findsNothing, reason: 'nothing typed: closed, no "Number required"');
    expect(find.text('Number required'), findsNothing);
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, tags: 'main', skip: !insideBuilt);
}
