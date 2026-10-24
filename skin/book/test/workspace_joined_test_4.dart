// Part of workspace_joined_test.dart (its tests, cut there by size, as they were; called from its main).
part of 'workspace_joined_test.dart';

void _tests4() {
  testWidgets('a motion is a file: the Effects shelf lists animators/ for the picked thing, a press gives it (a text then moves letter by letter), its knobs on its card, a press takes it off', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final text = m.add('text', '/System/Library/Fonts/Supplemental/Arial Black.ttf');
    expect(m.say(text, 'text', 'MOTOLII'), isTrue, reason: m.trouble);
    final spring = m.animators.firstWhere((e) => e.$2.endsWith('/spring_in.js'));
    expect(m.animators.length, 13, reason: 'Cavalry\'s thirteen');
    ws
      ..reread()
      ..select('$text')
      ..shelf = 'Effects';
    await t.pumpAndSettle();
    final row = find.byKey(ValueKey('ws-motion-${spring.$1}'));
    await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), spring.$1);
    await t.pumpAndSettle();
    await doubleTap(t, row);
    await settled(t, ws);
    expect(m.word(text, 'script'), spring.$2, reason: m.trouble);
    expect(find.descendant(of: row, matching: find.text('ON')), findsOneWidget);
    expect(m.cards(text).firstWhere((c) => c.$1 == 'Script').$2, containsAll(['js_length', 'js_delay', 'js_far']));
    await doubleTap(t, row);
    await settled(t, ws);
    expect(ws.motion, isEmpty, reason: 'taken off');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('a knob of the stage\'s own cards in its units (Bloom in hundredths): a second drag right goes on from where the first left it, up', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    m.add('solid', 'cube');
    ws..reread()..select(null);
    await t.pumpAndSettle();
    final bloom = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_bloom');
    await seen(t, bloom);
    await t.pump();
    for (final by in const [Offset(90, 0), Offset(45, 40)]) {
      final g = await t.startGesture(t.getCenter(bloom));
      for (var i = 1; i <= 10; ++i) { await g.moveBy(by / 10); await t.pump(const Duration(milliseconds: 16)); }
      await g.up();
      await t.pumpAndSettle();
    }
    expect(m.value(0, 'bloom', 0), greaterThan(0.45), reason: 'the second drag began from 45, not from the document\'s 0.45 read as hundredths');
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('a knob dragged up or down as well as across scrubs its number and does not scroll the inspector under the hand', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final text = m.add('text', '/System/Library/Fonts/Supplemental/Arial Black.ttf');
    ws..reread()..select('$text');
    await t.pumpAndSettle();
    final bloom = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_letter_spacing');
    await seen(t, bloom);
    final list = find.ancestor(of: bloom, matching: find.byType(Scrollable)).first;
    final scrolled = t.state<ScrollableState>(list).position.pixels, was = m.value(text, 'letter_spacing', 0) ?? 1;
    expect(t.state<ScrollableState>(list).position.maxScrollExtent, greaterThan(0), reason: 'the inspector is longer than its pane');
    final g = await t.startGesture(t.getCenter(bloom));
    for (var i = 1; i <= 10; ++i) {
      await g.moveBy(const Offset(9, -8));
      await t.pump(const Duration(milliseconds: 16));
    }
    await g.up();
    await t.pumpAndSettle();
    expect(t.state<ScrollableState>(list).position.pixels, scrolled, reason: 'the inspector stays where it was');
    expect(m.value(text, 'letter_spacing', 0), greaterThan(was), reason: 'the number went up');
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('with Animate on, a flat thing\'s scale dragged goes into the document from the first drag, keyed (no key asked of a value it has not, scale_z, to take the step back)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final c = m.add('shape', 'rect');
    ws..reread()..select('$c');
    ws.animating = true;
    await t.pumpAndSettle();
    final k = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_scale_x'); // (the inside's Transform card)
    await seen(t, k);
    var was = m.value(c, 'scale_x', ws.frame / Ws.fps)!;
    for (final (by, more) in const [(Offset(40, 0), true), (Offset(-80, 0), false)]) {
      final g = await t.startGesture(t.getCenter(k));
      for (var i = 1; i <= 3; ++i) {
        await g.moveBy(by / 3);
        await t.pump(const Duration(milliseconds: 16));
      }
      await g.up();
      await t.pumpAndSettle();
      for (var i = 0; i < 20 && ws.waiting; i++) {
        await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
        await t.pump();
      }
      final now = m.value(c, 'scale_x', ws.frame / Ws.fps)!;
      expect(more ? now > was : now < was, isTrue, reason: 'dragged by $by: $was to $now, from the first drag');
      expect(m.keyMoments(c, 'scale_x'), isNotEmpty, reason: 'Animate keys it');
      was = now;
    }
    ws.animating = false;
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('the stage\'s Follow names a thing by its name (a choice, None first), not by a number dragged', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    m.add('shape', 'rect');
    final cube = m.add('solid', 'cube');
    expect(m.say(0, 'script', m.effects.firstWhere((e) => e.$2.endsWith('/kaleido.wgsl')).$2), isTrue, reason: m.trouble); // (the stage's Script card: shown with an effect on it)
    ws
      ..reread()
      ..select(null);
    await t.pumpAndSettle();
    expect(find.byWidgetPredicate((w) => w is NumField && w.id == 'v_follow'), findsNothing, reason: 'not a knob of numbers');
    final follow = find.byWidgetPredicate((w) => w is Segmented && w.items.any((i) => i.endsWith('Cube'))); // (a few things: a segmented choice, None first, by their names with their numbers: '2. Cube')
    await seen(t, follow);
    await t.tap(find.descendant(of: follow, matching: find.textContaining(RegExp('cube', caseSensitive: false))));
    await settled(t, ws);
    expect(m.value(0, 'follow', 0), cube, reason: m.trouble);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('Backspace on the timeline takes the picked thing out; undo brings it back', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final ring = m.add('solid', 'ring'), cube = m.add('solid', 'cube');
    expect(m.say(cube, 'fill', 'glass'), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$cube');
    await t.pumpAndSettle();
    expect(ws.layers.map((l) => l.name), ['Cube', 'Ring'], reason: 'the top row is the nearest thing');

    await t.tap(find.byKey(ValueKey('ws-tl-label-$cube')));
    await t.pump();
    await t.sendKeyEvent(LogicalKeyboardKey.backspace);
    await t.pumpAndSettle();
    expect(m.things, [ring]);
    expect(ws.layers.map((l) => l.id), ['$ring']);

    expect(m.undo(), isTrue);
    ws.reread();
    expect(m.things, [ring, cube]);
    expect(m.word(cube, 'fill'), 'glass', reason: 'it comes back as it was: glass is its fill');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);
}
