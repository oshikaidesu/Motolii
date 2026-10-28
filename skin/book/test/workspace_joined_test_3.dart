// Part of workspace_joined_test.dart (its tests, cut there by size, as they were; called from its main).
part of 'workspace_joined_test.dart';

void _tests3() {
  testWidgets('solo, lock and loop are the document\'s and the inside\'s, not the window\'s', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final a = m.add('shape', 'rect'), b = m.add('shape', 'circle');
    ws.reread();
    await t.pumpAndSettle();
    expect(m.set(a, 'solo', 1), isTrue, reason: m.trouble);
    expect([m.frame(a), m.frame(b)], [isNotNull, isNull], reason: 'while one is solo only it is seen');
    expect(m.set(a, 'solo', 0) && m.frame(b) != null, isTrue);
    expect(m.set(b, 'lock', 1), isTrue);
    expect(m.set(b, 'x', 1), isFalse);
    expect(m.trouble, 'it is locked');
    expect(m.remove(b), isFalse, reason: 'nor taken away');
    expect(m.set(b, 'lock', 0) && m.set(b, 'x', 1), isTrue, reason: 'unlocked, it is changed again');
    expect(m.looping, isTrue, reason: 'play goes round, as a document is opened');
    m.looping = false;
    expect(m.looping, isFalse);
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('the strip means the document: Snap sticks to the picture\'s middle, Group picks the parent, a Mask rectangle cuts the picked thing; the pivot is the anchor', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final a = m.add('shape', 'circle'), b = m.add('shape', 'rect');
    expect(m.set(a, 'x', -1.5) && m.set(b, 'x', 1.5) && m.set(b, 'anchor_x', 1), isTrue, reason: m.trouble);
    ws.reread();
    await t.pumpAndSettle();
    final fb = m.frame(b)!;
    expect(fb[5], closeTo(fb[0] + fb[2] / 2, 1), reason: 'the point it turns about is its anchor: its right edge');
    final stage = t.getRect(find.byKey(const ValueKey('ws-stage-comp'))), k = 1920 / stage.width;
    Offset at(int thing) => stage.topLeft + Offset(m.frame(thing)![0], m.frame(thing)![1]) / k;
    await t.tap(find.byKey(const ValueKey('ws-tool-1')));
    await t.pumpAndSettle();
    await t.dragFrom(at(a), stage.center + const Offset(3, -2) - at(a));
    await t.pumpAndSettle();
    expect(m.frame(a)![0], closeTo(960, .5), reason: 'Snap: its middle on the picture\'s, though let go three points off');
    expect(m.frame(a)![1], closeTo(540, .5));
    expect(m.set(a, 'parent', b.toDouble()), isTrue, reason: m.trouble);
    ws.reread();
    await t.tap(find.byKey(const ValueKey('ws-tool-0')));
    await t.pumpAndSettle();
    await t.tap(find.text('Group'));
    await t.pumpAndSettle();
    await t.tapAt(at(a));
    await t.pumpAndSettle();
    expect(ws.selected, '$b', reason: 'Group: what goes with another is picked as that one');
    ws.select('$a');
    await t.tap(find.byKey(const ValueKey('ws-tool-2')));
    await t.pumpAndSettle();
    await t.tap(find.text('Mask'));
    await t.pumpAndSettle();
    await t.dragFrom(at(a) - Offset(stage.width * .03, stage.height * .03), Offset(stage.width * .06, stage.height * .06));
    await settled(t, ws);
    for (var i = 0; i < 200 && ws.selected == '$a'; i++) {   // (the mask made by the inside, picked once it is there)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 25)));
      await t.pump();
    }
    final cut = int.parse(ws.selected!), all = m.things;
    expect(all.indexOf(cut), all.indexOf(a) - 1, reason: 'the mask lies just behind the thing it cuts: ${m.trouble}');
    expect([m.value(a, 'clip', 0), m.value(cut, 'opacity', 0)], [1, 0], reason: 'it is clipped by it, which is not seen itself');
    expect(m.undo() && m.things.length == 2 && m.value(a, 'clip', 0) == 0, isTrue, reason: 'one step');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('the shelves\' headers count what the inside has: effects it does, colours the document uses, tiles that make', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final a = m.add('shape', 'rect');
    expect(m.say(a, 'fill', 'ff3366'), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..shelf = 'Colors';
    await t.pumpAndSettle();
    expect(find.text('1 used here'), findsOneWidget);
    ws.shelf = 'Effects';
    await settled(t, ws);
    final fx = m.thingEffects.length + m.effects.length;
    expect(fx, greaterThan(3));
    expect(find.text('$fx ready'), findsOneWidget);
    ws.shelf = 'Create';
    await t.pumpAndSettle();
    final r = ws.inside!.recipes.length; // (a recipe is a file: the shelf grows by what is in recipes/)
    expect(find.text('${17 + r} ready'), findsOneWidget, reason: 'seventeen tiles of its own since K5 (one Repeater for Radial and Grid), and the recipes');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('scale is three numbers of the document, each its own; a flat thing has no scale z (its thickness to come), a solid has', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final a = m.add('shape', 'rect'), cube = m.add('solid', 'cube');
    ws
      ..reread()
      ..select('$a');
    await t.pumpAndSettle();
    Finder knob(String id) => find.byWidgetPredicate((w) => w is NumField && w.id == id);
    expect(knob('v_scale_z'), findsNothing, reason: 'a flat thing: no depth to scale yet');
    expect(knob('depth'), findsNothing, reason: 'no Depth knob: a thing\'s thickness is its scale z');
    ws.select('$cube');
    await t.pumpAndSettle();
    await seen(t, knob('v_scale_z')); // (the inside's Transform card: each axis its own number)
    await t.pumpAndSettle();
    await t.drag(knob('v_scale_z'), const Offset(60, 0));
    await t.pumpAndSettle();
    expect([m.value(cube, 'scale_x', 0), m.value(cube, 'scale_y', 0)], [m.value(cube, 'scale_z', 0), m.value(cube, 'scale_z', 0)], reason: 'linked, as After Effects\' scale is at first: all three together');
    expect(m.value(cube, 'scale_z', 0), isNot(1), reason: m.trouble);
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('a camera picked shows its own card (where it looks, how far, its zoom), not a thing\'s scale, opacity or depth', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final cam = m.add('camera', 'camera');
    ws
      ..reread()
      ..select('$cam');
    await t.pumpAndSettle();
    final ids = {for (final e in find.byType(NumField).evaluate()) (e.widget as NumField).id};
    expect(ids, containsAll(['v_x', 'v_distance', 'v_zoom']), reason: 'the inside\'s Camera card, as After Effects\' camera');
    expect(ids.intersection({'v_scale_x', 'v_opacity', 'depth'}), isEmpty, reason: 'a camera has none of these');
    expect(inspectorCards(t), ['Camera', 'Projection', 'Note'], reason: 'where it is and looks, then its lens: orthographic, shifted');
    expect(m.say(cam, 'projection', 'orthographic') && m.key(cam, 'shift_y', 0, 0.2), isTrue, reason: m.trouble);
    final dist = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_distance');
    await seen(t, dist);
    await t.timedDrag(dist, const Offset(60, 0), const Duration(milliseconds: 300));
    await t.pumpAndSettle();
    expect(m.value(cam, 'distance', 0), isNot(anyOf(isNull, 1.0)), reason: 'the distance is the document\'s: ${ws.notice}');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('the inspector\'s own: a card folds and stays folded; Ghost the document has not is not shown; the text box is the picked text\'s', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    const font = '/System/Library/Fonts/Supplemental/Arial Unicode.ttf';
    final a = m.add('text', font), b = m.add('text', font);
    expect(m.say(a, 'text', 'first') && m.say(b, 'text', 'second'), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$a');
    await t.pumpAndSettle();
    final box = find.descendant(of: find.byType(TextGroup), matching: find.byType(EditableText));
    expect((t.widget(box) as EditableText).controller.text, 'first');
    ws.select('$b');
    await t.pumpAndSettle();
    expect((t.widget(box) as EditableText).controller.text, 'second', reason: 'another text picked: its own words');
    await t.tap(box);
    await t.pumpAndSettle();
    FocusManager.instance.primaryFocus?.unfocus();
    await t.pumpAndSettle();
    expect([m.word(a, 'text'), m.word(b, 'text')], ['first', 'second'], reason: 'in and out of the box writes nothing stale');
    expect(find.byWidgetPredicate((w) => w is NumField && w.id == 'v_x'), findsOneWidget); // (the inside's Transform card)
    await t.tap(find.text('Transform').first);
    await t.pumpAndSettle();
    expect(find.byWidgetPredicate((w) => w is NumField && w.id == 'v_x'), findsNothing, reason: 'folded, and it stays so');
    expect(find.byWidgetPredicate((w) => w is PropRow && w.id == 'ghost'), findsNothing, reason: 'Ghost the document has not is not shown');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('the inspector\'s effects are the document\'s on the thing: a card each, its fields the numbers the inside has for it', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final a = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$a');
    await t.pumpAndSettle();
    ws
      ..fxToggle('Drop Shadow')
      ..fxToggle('Glow');   // (sent without waiting: put on as the window looks again)
    await settled(t, ws);
    expect(inspectorCards(t), containsAll(['Drop Shadow', 'Glow']), reason: 'an effect on it is one of its cards');
    final far = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_shadow_distance'), was = m.value(a, 'shadow_distance', 0);
    await seen(t, far);
    final ids = {for (final e in find.byType(NumField).evaluate()) (e.widget as NumField).id};
    expect(ids, containsAll(['v_shadow_distance', 'v_shadow_angle', 'v_shadow_blur', 'v_shadow_opacity']));
    await t.timedDrag(far, const Offset(40, 0), const Duration(milliseconds: 300));
    await settled(t, ws);   // (the hand let go, not waited for by the window)
    expect(m.value(a, 'shadow_distance', 0), isNot(was), reason: 'the distance is the document\'s: ${ws.notice}');
    expect(m.undo(), isTrue);
    expect(m.value(a, 'shadow_distance', 0), was, reason: 'one step');
    ws.fxToggle('Drop Shadow'); // (taken off as the shelf's Remove takes it)
    await t.pumpAndSettle();
    expect([ws.fxOf('$a'), m.word(a, 'shadow')], [['Glow'], ''], reason: 'taken off the document');
    expect(inspectorCards(t).contains('Drop Shadow'), isFalse);
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('Saved says when the document\'s file was last written: a change is written as it is made', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    m.add('shape', 'rect');
    ws.reread();
    await t.pumpAndSettle();
    final at = File('${ws.documentFolder}/${ws.document}').lastModifiedSync();
    expect(DateTime.now().difference(at).inSeconds, lessThan(5), reason: 'the change is in the file already');
    expect(find.text('Saved ${at.hour.toString().padLeft(2, '0')}:${at.minute.toString().padLeft(2, '0')}'), findsOneWidget);
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('Cmd+E cuts the picked film at the playhead: it ends there, and the part after begins there, as far on in the film', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final film = m.add('', '$motoliiDir/out/vid/check_film.mp4');
    expect(m.set(film, 'start', .5) && m.set(film, 'duration', 3) && m.set(film, 'begin', .25), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$film')
      ..frame = 2 * Ws.fps;
    await t.pumpAndSettle();
    await t.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
    await t.sendKeyEvent(LogicalKeyboardKey.keyE);
    await t.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
    await settled(t, ws);
    for (var i = 0; i < 200 && ws.selected == '$film'; i++) {   // (the part after is picked once the inside has made it)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 25)));
      await t.pump();
    }
    final after = int.parse(ws.selected!);
    expect(after, isNot(film), reason: m.trouble);
    expect([m.value(film, 'start', 0), m.value(film, 'duration', 0)], [.5, closeTo(1.5, 1e-9)], reason: 'the first ends at the playhead');
    expect([m.value(after, 'start', 0), m.value(after, 'duration', 0), m.value(after, 'begin', 0)], [closeTo(2, 1e-9), closeTo(1.5, 1e-9), closeTo(1.75, 1e-9)], reason: 'the part after: from the playhead to where it ended, 1.5 s further on in the film');
    expect(m.word(after, 'file'), m.word(film, 'file'));
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('Frame rate on the Play card: a thing\'s moving values seen only at its own frames (Alight Motion\'s Time Quantization)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final rect = m.add('shape', 'rect');
    expect(m.key(rect, 'x', 0, 0) && m.key(rect, 'x', 1, 1), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$rect');
    await t.pumpAndSettle();
    expect([for (final c in m.cards(rect)) if (c.$1 == 'Play') ...c.$2], contains('frame_rate'), reason: 'its card: ${m.cards(rect)}');
    expect((await m.shownAt([(rect, 'x', .45)])).single, closeTo(.45, 1e-9), reason: 'as smooth as the stage');
    ws.after(m.later('set', thing: rect, name: 'frame_rate', number: 4));
    await settled(t, ws);
    expect(await m.shownAt([for (final at in [.3, .45, .5, .99]) (rect, 'x', at)]), [closeTo(.25, 1e-9), closeTo(.25, 1e-9), closeTo(.5, 1e-9), closeTo(.75, 1e-9)], reason: 'four frames a second');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('a film\'s Volume on its Sound card; Extract Sound: a hidden copy heard, the film itself silent (Alight Motion\'s Extract Audio)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final film = m.add('', '$motoliiDir/out/vid/check_film.mp4');
    ws
      ..reread()
      ..select('$film');
    await t.pumpAndSettle();
    expect([for (final c in m.cards(film)) if (c.$1 == 'Sound') ...c.$2], contains('volume'), reason: 'its card: ${m.cards(film)}');
    expect(m.value(film, 'volume', 0) ?? 1, 1, reason: 'as loud as it comes');
    ws.after(m.later('extract_sound', thing: film), null, (made) => ws.select('$made'));
    await settled(t, ws);
    for (var i = 0; i < 200 && ws.selected == '$film'; i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 25)));
      await t.pump();
    }
    final sound = int.parse(ws.selected!);
    expect(sound, isNot(film), reason: m.trouble);
    expect([m.word(sound, 'file'), m.value(sound, 'visible', 0), m.value(film, 'volume', 0)], [m.word(film, 'file'), 0, 0], reason: 'the copy hidden and heard, the film silent');
    expect(m.word(sound, 'name'), endsWith('(sound)'));
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('Cmd+C then Cmd+V pastes the picked thing whole at the playhead, its keys moved with it; Cmd+Alt+C and Cmd+Alt+V carry its look', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final rect = m.add('shape', 'rect'), other = m.add('shape', 'circle');
    expect(m.set(rect, 'start', 1) && m.set(rect, 'x', .3) && m.key(rect, 'rotate', 1, 0) && m.key(rect, 'rotate', 2, 90) && m.say(rect, 'fill', 'ff3366'), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$rect')
      ..frame = 3 * Ws.fps;
    await t.pumpAndSettle();
    Future<void> press(LogicalKeyboardKey k, {bool alt = false}) async {
      await t.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
      if (alt) await t.sendKeyDownEvent(LogicalKeyboardKey.altLeft);
      await t.sendKeyEvent(k);
      if (alt) await t.sendKeyUpEvent(LogicalKeyboardKey.altLeft);
      await t.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
      await settled(t, ws);
    }
    await press(LogicalKeyboardKey.keyC);
    await press(LogicalKeyboardKey.keyV);
    for (var i = 0; i < 200 && ws.selected == '$rect'; i++) {   // (the pasted one is picked once the inside has made it)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 25)));
      await t.pump();
    }
    final pasted = int.parse(ws.selected!);
    expect(pasted, isNot(rect), reason: m.trouble);
    expect([m.word(pasted, 'kind'), m.word(pasted, 'fill'), m.value(pasted, 'x', 0), m.value(pasted, 'start', 0)], ['shape', m.word(rect, 'fill'), closeTo(.3, 1e-9), closeTo(3, 1e-9)], reason: 'whole, beginning at the playhead');
    expect(m.keyMoments(pasted, 'rotate'), [closeTo(3, 1e-9), closeTo(4, 1e-9)], reason: 'its keys moved by as much as it was');
    ws.select('$rect');
    await t.pumpAndSettle();
    await press(LogicalKeyboardKey.keyC, alt: true);
    ws.select('$other');
    await t.pumpAndSettle();
    await press(LogicalKeyboardKey.keyV, alt: true);
    for (var i = 0; i < 200 && m.word(other, 'fill') != m.word(rect, 'fill'); i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 25)));
      await t.pump();
    }
    expect(m.word(other, 'fill'), m.word(rect, 'fill'), reason: 'its look given: ${m.trouble}');
    expect(m.word(other, 'shape'), 'circle', reason: 'what it is stays');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('the inspector says what a thing is in the document\'s word, and shows only the cards of values it has', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final film = m.add('', '$motoliiDir/out/vid/check_film.mp4'), cube = m.add('solid', 'cube');
    ws
      ..reread()
      ..select('$film');
    await t.pumpAndSettle();
    expect(find.descendant(of: find.byType(InspHeader), matching: find.text('Film')), findsOneWidget);
    expect(find.byType(FillStroke), findsNothing, reason: 'a film has no fill: ${m.values(film)}');
    ws.select('$cube');
    await t.pumpAndSettle();
    expect(find.descendant(of: find.byType(InspHeader), matching: find.text('Solid')), findsOneWidget);
    expect(find.byType(FillStroke), findsOneWidget, reason: 'a solid\'s fill is a colour or glass');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('the inside\'s own cards: a text\'s Type (ThorVG\'s spacing and slant), a shape\'s Line (its ends, corners, dashes), each from the document', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final text = m.add('text', _font), rect = m.add('shape', 'rect');
    expect(m.say(text, 'text', 'MOTOLII') && m.say(rect, 'stroke', 'ff3366') && m.set(rect, 'stroke_width', 12), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$text');
    await t.pumpAndSettle();
    await seen(t, find.descendant(of: find.byType(ValuesCard), matching: find.text('Type')));
    expect(find.descendant(of: find.byType(ValuesCard), matching: find.text('Type')), findsOneWidget);
    expect(m.cards(text).map((c) => c.$1).contains('Gradient'), isFalse, reason: 'one colour: no ramp to turn');
    expect(m.say(text, 'fill', 'ff3366 1b2fd8') && m.cards(text).map((c) => c.$1).contains('Gradient'), isTrue, reason: 'two: its ramp');
    expect(m.say(text, 'fill', 'ffffff'), isTrue);
    final wide = m.frame(text)![2], letters = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_letter_spacing');
    expect(letters, findsOneWidget);
    await seen(t, letters);
    await t.timedDrag(letters, const Offset(80, 0), const Duration(milliseconds: 300));
    await settled(t, ws);   // (the hand let go, not waited for by the window: read once the inside has taken it)
    expect(m.value(text, 'letter_spacing', 0), greaterThan(1), reason: 'the letters further apart: ${ws.notice}');
    expect(m.frame(text)![2], greaterThan(wide), reason: 'and the words wider on the stage');
    ws.select('$rect');
    await t.pumpAndSettle();
    await seen(t, find.descendant(of: find.byType(ValuesCard), matching: find.text('Line')));
    expect(find.descendant(of: find.byType(ValuesCard), matching: find.text('Line')), findsOneWidget);
    final round = find.descendant(of: find.byType(ValuesCard), matching: find.text('ROUND')).first; // (the ends: butt, round, square, as the sample's segments write them)
    await seen(t, round);
    await t.tap(round);
    await t.pumpAndSettle();
    expect(m.word(rect, 'cap'), 'round', reason: m.trouble);
    expect(m.undo() && m.word(rect, 'cap').isEmpty, isTrue, reason: 'one step');
    final cube = m.add('solid', 'cube');
    ws
      ..reread()
      ..select('$cube');
    await t.pumpAndSettle();
    expect(find.descendant(of: find.byType(ValuesCard), matching: find.text('Surface')), findsOneWidget, reason: 'Filament\'s own surface of a solid');
    expect(m.say(cube, 'fill', 'glass'), isTrue, reason: m.trouble);
    expect(m.cards(cube).map((c) => c.$1), ['Fill', 'Transform', 'Blending', 'Motion', 'Play', 'Glass', 'Lit', 'Physics', 'Note'], reason: 'filled with glass: the glass\'s own card, not the surface\'s');
    expect(m.cards(rect).map((c) => c.$1).contains('Matte'), isFalse, reason: 'not clipped: no matte to say how');
    expect(m.set(rect, 'clip', 1) && m.cards(rect).map((c) => c.$1).contains('Matte'), isTrue);
    expect(m.set(cube, 'ior', 2.2) && m.value(cube, 'ior', 0) == 2.2, isTrue, reason: m.trouble);
    expect(m.say(cube, 'fill', 'ff3366') && m.cards(cube).map((c) => c.$1).join(' ') == 'Fill Transform Blending Motion Play Surface Lit Physics Note', isTrue);
    expect(find.byWidgetPredicate((w) => w is NumField && w.id == 'v_metallic'), findsOneWidget);
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('nothing picked, the inspector is the whole picture\'s: Filament\'s grade, glow and room, the stage\'s own values', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    m.add('solid', 'cube');
    ws
      ..reread()
      ..select(null);
    await t.pumpAndSettle();
    final titles = inspectorCards(t);
    expect(titles, ['Grade', 'Lens', 'Room', 'Physics']);
    final bloom = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_bloom');
    await seen(t, bloom);
    await t.timedDrag(bloom, const Offset(60, 0), const Duration(milliseconds: 300));
    await t.pumpAndSettle();
    expect(m.value(0, 'bloom', 0), greaterThan(0), reason: 'the stage glows: ${ws.notice}');
    expect(m.undo() && m.value(0, 'bloom', 0) == 0, isTrue, reason: 'one step');
    // (a colour on a card: its link aims the Colors shelf at it; a swatch picked there is set to it)
    await seen(t, find.byKey(const ValueKey('link:colors:v_background')));
    await t.tap(find.byKey(const ValueKey('link:colors:v_background')));
    await t.pumpAndSettle();
    expect(ws.aim, 'background');
    await t.tap(find.descendant(of: find.byWidgetPredicate((w) => w.runtimeType.toString() == '_Strip'), matching: find.byType(DockHover)).first);
    await t.pumpAndSettle();
    await t.tap(find.descendant(of: find.byType(DockFooter), matching: find.text('Set')));
    await settled(t, ws);   // (set by the inside, not waited for by the window)
    expect(m.word(0, 'background'), hasLength(6), reason: 'the stage\'s colour behind everything: ${m.trouble}');
    expect(m.say(0, 'tone_mapper', 'agx') && m.word(0, 'tone_mapper') == 'agx', isTrue, reason: m.trouble);
    expect(m.say(0, 'tone_mapper', 'sepia'), isFalse, reason: 'one of Filament\'s own');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('a light is a thing of the document, Filament\'s own: made from the Create shelf, its Light card, its colour its fill', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    m.add('solid', 'cube');
    ws
      ..reread()
      ..shelf = 'Create';
    await t.pumpAndSettle();
    await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), 'Spot');
    await t.pumpAndSettle();
    final tile = find.descendant(of: find.byType(CreateShelf), matching: find.text('Spot')).first;
    await seen(t, tile);
    await t.tap(tile);
    await t.pump(const Duration(milliseconds: 400)); // (two presses would make it: one is known when the second does not come)
    await t.pumpAndSettle();
    await t.tap(find.descendant(of: find.byType(DockFooter), matching: find.text('Create')));
    await t.pumpAndSettle();
    final spot = int.parse(ws.selected!);
    expect([m.word(spot, 'kind'), m.word(spot, 'file')], ['light', 'spot'], reason: m.trouble);
    expect(find.descendant(of: find.byType(InspHeader), matching: find.text('Light')), findsOneWidget);
    expect(inspectorCards(t), ['Fill', 'Transform', 'Light', 'Note'], reason: 'its colour, where it is and how it is turned, as After Effects\' lights have, then its own');
    expect(m.set(spot, 'shadows', 1) && m.say(spot, 'fill', 'ffb36b') && m.set(spot, 'cone', 20), isTrue, reason: m.trouble);
    expect(m.set(spot, 'shadows', 2), isFalse, reason: 'shadows is 0 or 1');
    expect(m.key(spot, 'intensity', 0, 0) && m.key(spot, 'intensity', 1, 3), isTrue, reason: 'its strength moves: ${m.trouble}');
    expect(m.add('light', 'lamp'), 0, reason: 'a light is a sun, a point or a spot');
    // (Animate on: a card's number dragged is a key at the playhead, as every change then is; the field shows it)
    expect(m.unkey(spot, 'intensity', 0) && m.unkey(spot, 'intensity', 1), isTrue, reason: m.trouble);
    ws
      ..animating = true
      ..reread();
    await t.pumpAndSettle();
    final strong = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_intensity');
    await seen(t, strong);
    await t.timedDrag(strong, const Offset(40, 0), const Duration(milliseconds: 300));
    await t.pumpAndSettle();
    expect(m.keyMoments(spot, 'intensity'), [0], reason: 'a key where the playhead is: ${ws.notice}');
    expect(wsKeyMarks(ws, '$spot')['v_intensity'], KeyS.at);
    ws.animating = false;
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('an After Effects file plays a named stretch of its own (ThorVG\'s Lottie markers), chosen on its Play card', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final pic = m.add('picture', '$motoliiDir/third_party/thorvg_example/res/lottie/swinging.json');
    expect(pic, isNot(0), reason: m.trouble);
    ws
      ..reread()
      ..select('$pic');
    await t.pumpAndSettle();
    expect(m.words(pic, 'marker'), ['all', 'Start', 'End'], reason: 'its file\'s own names');
    expect(inspectorCards(t), contains('Play'));
    expect(m.say(pic, 'marker', 'Start') && m.word(pic, 'marker') == 'Start', isTrue, reason: m.trouble);
    // (a 3D file plays one of its own movements, at its speed)
    final fox = m.add('solid', '$motoliiDir/third_party/filament_src/third_party/models/Fox/Fox.glb');
    ws.reread();
    await t.pumpAndSettle();
    expect(m.words(fox, 'animation'), ['Survey', 'Walk', 'Run'], reason: 'its file\'s own movements');
    expect(m.say(fox, 'animation', 'Run') && m.set(fox, 'speed', 2), isTrue, reason: m.trouble);
    expect(m.cards(fox).firstWhere((c) => c.$1 == 'Play').$2, ['animation', 'speed', 'frame_rate']);
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('a recipe is a picture in one step (as Cavalry builds it from a shape, a duplicator and a behaviour): the Create shelf makes it, its knobs on its cards', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    expect(m.recipes.map((r) => r.$1), containsAll(['Wave lines', 'Iris', 'Dot wave', 'Tunnel', 'Burst']));
    ws
      ..reread()
      ..shelf = 'Create';
    await t.pumpAndSettle();
    await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), 'Iris');
    await t.pumpAndSettle();
    final tile = find.descendant(of: find.byType(CreateShelf), matching: find.text('Iris')).first;
    await seen(t, tile);
    await t.tap(tile);
    await t.pump(const Duration(milliseconds: 400)); // (a tile that makes on two presses knows one only when the second is not coming)
    await t.pumpAndSettle();
    await t.tap(find.descendant(of: find.byType(DockFooter), matching: find.text('Create')));
    await t.pumpAndSettle();
    final iris = int.parse(ws.selected!);
    expect([m.value(iris, 'copies', 0), m.word(iris, 'copy_fill')], [120, 'ff3366 ffd166 1b2fd8 22c55e ff3366'], reason: m.trouble);
    expect(inspectorCards(t), containsAll(['Copies', 'Wave']));
    expect(m.undo() && m.things.isEmpty, isTrue, reason: 'one step');
    // (two presses on the tile make it at once, without the foot's button)
    await settled(t, ws);   // (the undo's picture taken first: the tile pressed where it then is)
    await doubleTap(t, tile);
    await settled(t, ws);   // (made by the inside, not waited for by the window)
    for (var i = 0; i < 100 && m.things.isEmpty; i++) {   // (its picture of the document, told once the inside has made it: as drop_effect_test waits for what the window does not)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
      await t.pump();
    }
    expect(m.things.length, 1, reason: 'made by two presses: ${m.trouble}');
    expect(m.word(m.things.single, 'name'), 'Iris');
    expect(m.make('Nope'), 0);
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('a recipe is a file: one put in recipes/ is on the shelf at once, its knobs on a Script card, a drag on one is the document\'s', (t) async {
    final file = File('$motoliiDir/recipes/zz_window_test.js');   // (put in before the window opens: the folders are read once as it opens, BRIDGE.md rule 8)
    addTearDown(() => file.existsSync() ? file.deleteSync() : null);
    file.writeAsStringSync('// Window test: dots in a ring\n// make shape circle\n// set copies 24\n// --track@reach:Reach,0,3,1,0.01\n\n'
        'function each(i, n, t, k) { const a = 2 * Math.PI * i / n; return { x: k.reach * Math.cos(a), y: k.reach * Math.sin(a) }; }\n');
    final ws = await _pump(t), m = ws.inside!;
    expect(m.recipes.map((r) => r.$1), contains('Window test'), reason: 'on the shelf without a build');
    final dots = m.make('Window test');
    expect(dots, isNot(0), reason: m.trouble);
    ws
      ..reread()
      ..select('$dots');
    await t.pumpAndSettle();
    expect(inspectorCards(t), contains('Script'));
    final knob = find.byWidgetPredicate((w) => w is NumField && w.id.contains('js_reach'));
    await seen(t, knob);
    expect(knob, findsOneWidget, reason: 'ids: ${[for (final e in find.byType(NumField).evaluate()) (e.widget as NumField).id]}');
    await seen(t, knob);
    await t.timedDrag(knob, const Offset(60, 0), const Duration(milliseconds: 300));
    await t.pumpAndSettle();
    expect(m.value(dots, 'js_reach', 0), isNot(1.0), reason: 'the knob is the document\'s: ${ws.notice}');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('with a thing picked, an effect of the list goes on it (as the camera sees it, its fx), not on the whole stage; a press takes it off', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    final half = m.effects.firstWhere((e) => e.$2.endsWith('/halftone.wgsl'));
    ws
      ..reread()
      ..select('$cube')
      ..shelf = 'Effects';
    await t.pumpAndSettle();
    final row = find.byKey(ValueKey('ws-screen-${half.$2}'));   // (a file's effect, by its file: dock_effects.dart)
    await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), half.$1);
    await t.pumpAndSettle();
    await doubleTap(t, row);
    await settled(t, ws);
    expect(m.word(cube, 'fx'), half.$2, reason: m.trouble);
    expect(m.word(0, 'script'), anyOf('', 'none'), reason: 'the whole stage is left as it was');
    expect(find.descendant(of: row, matching: find.text('ON 1')), findsOneWidget);
    await doubleTap(t, row);
    await settled(t, ws);
    expect(ws.screenFx, isEmpty, reason: 'taken off');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);
}
