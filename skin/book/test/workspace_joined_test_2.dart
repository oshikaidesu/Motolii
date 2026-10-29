// Part of workspace_joined_test.dart (its tests, cut there by size, as they were; called from its main).
part of 'workspace_joined_test.dart';

void _tests2() {
  testWidgets('round 3 of the judges: a thing put aside is not held; Animate all travels from where it was; a value with keys changed not being keyed gets a key there', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    expect(m.set(cube, 'visible', 0), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select(null);
    await t.pumpAndSettle();
    expect(m.frame(cube), isNull, reason: 'what is not seen has no frame');
    final stage = t.getRect(find.byKey(const ValueKey('ws-stage-comp')));
    await t.dragFrom(stage.center, Offset(stage.width * .2, 0));
    await t.pumpAndSettle();
    expect(ws.selected, isNull, reason: 'the hand went through it');
    expect(m.value(cube, 'x', 0), 0);
    expect(m.set(cube, 'visible', 1), isTrue);

    // (Animate all turned on at 0, the thing carried at 2: it is where it was at 0 and travels to where it was let go)
    ws.frame = 0 * Ws.fps;   // (the playhead is the window's, told to the inside by it: BRIDGE.md rule 9)
    await settled(t, ws);
    m.keying(true);
    ws.frame = 2 * Ws.fps;   // (the playhead is the window's, told to the inside by it: BRIDGE.md rule 9)
    await settled(t, ws);
    expect(m.hold(cube) && m.carry('x', 3, 0) && m.letGo(true), isTrue, reason: m.trouble);
    expect(await m.shownAt([(cube, 'x', 0), (cube, 'x', 1), (cube, 'x', 2)]), [0, 1.5, 3],   // (other moments: asked, BRIDGE.md rule 7)
        reason: 'from where it was to where it was let go');
    expect(m.undo(), isTrue, reason: 'both keys are one step');
    expect(m.keyed(cube, 'x', 2), 0);
    expect(m.redo(), isTrue);
    m.keying(false);

    // (not being keyed: a value with keys gets a key where it is changed, the others as they were (After Effects))
    ws.frame = 1 * Ws.fps;   // (the playhead is the window's, told to the inside by it: BRIDGE.md rule 9)
    await settled(t, ws);
    final was = [m.value(cube, 'x', 0), m.value(cube, 'x', 2)];
    expect(m.put(cube, 'x', 1, 2.5), isTrue, reason: m.trouble);
    expect([m.value(cube, 'x', 0), m.value(cube, 'x', 1), m.value(cube, 'x', 2)], [was[0], 2.5, was[1]]);
    expect(m.keyed(cube, 'x', 1), 2, reason: 'a key was made there');

    // (the frame of a thing turned about Y is the box of what it covers: a cube seen edge-on is wider than face-on)
    final faceOn = m.frame(cube)!;
    expect(m.set(cube, 'rotate_y', 45), isTrue, reason: m.trouble);
    final edgeOn = m.frame(cube)!;
    expect(edgeOn[2], greaterThan(faceOn[2] * 1.2));
    expect(edgeOn[4], 0);
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('the Fonts shelf offers the machine\'s font files: the picked text is set in the one applied, as one step, and it is kept among the recent', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final text = m.add('text', _font);
    final recent = File('${Directory.systemTemp.createTempSync('fonts').path}/font_recent.json');
    fontRecentAt(recent);   // (not the maker's own list)
    await t.runAsync(() => fontsRead(ws)); // (the shelf reads the folders without waiting; here the reading is done first)
    ws
      ..reread()
      ..select('$text')
      ..shelf = 'Fonts';
    await t.pumpAndSettle();
    final georgia = find.textContaining('Andale Mono');   // (its row says its tags beside: 'Andale Mono  ·  mono')
    await t.scrollUntilVisible(georgia, 300, scrollable: find.descendant(of: find.byType(FontsShelf), matching: find.byType(Scrollable)).last, maxScrolls: 200);
    expect(georgia, findsWidgets, reason: 'a font file of the system is on the shelf');
    await t.pumpAndSettle();
    await doubleTap(t, georgia.first);   // (one press picks, two put it on: f052e7e)
    await settled(t, ws);
    expect(m.word(text, 'file'), endsWith('/Andale Mono.ttf'), reason: m.trouble);
    final chips = [for (final e in find.descendant(of: find.byType(FontsShelf), matching: find.byType(Text)).evaluate()) (e.widget as Text).data];
    expect(chips[chips.indexOf('Used') + 1], '1', reason: 'the shelf counts the one in use (its Used tag)');
    expect([chips[chips.indexOf('Recent') + 1], recent.readAsStringSync()], ['1', '["Andale Mono"]'], reason: 'and keeps it among the fonts set lately, for the next time');
    expect([m.fontTags('/System/Library/Fonts/Supplemental/AppleGothic.ttf'), m.fontTags('/System/Library/Fonts/Supplemental/Georgia.ttf')], [contains('korean'), containsAll(['serif', 'russian', 'greek'])], reason: 'its kind, and the letters beyond Latin it has, as CoreText says them');
    expect(m.undo(), isTrue);
    expect(m.word(text, 'file'), _font);
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('Stroke in the inspector is the edge of a flat shape or a text: a colour, None, and a width', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final rect = m.add('shape', 'rect'), cube = m.add('solid', 'cube');
    expect(m.say(rect, 'fill', 'ff3366'), isTrue);
    ws
      ..reread()
      ..select('$rect');
    await t.pumpAndSettle();
    final link = find.byKey(const ValueKey('link:colors:v_stroke'));
    await seen(t, link);
    await t.tap(link);
    await t.pumpAndSettle();
    expect(ws.aim, 'stroke', reason: 'the Colors shelf is aimed at its edge, as at a fill');
    expect(m.say(rect, 'stroke', 'ff3366'), isTrue, reason: m.trouble);
    ws.reread();
    await t.pumpAndSettle();
    expect(find.descendant(of: find.byType(FillStroke), matching: find.text('#FF3366')), findsWidgets, reason: 'the edge shown as the document says it');
    expect(m.word(rect, 'stroke'), 'ff3366', reason: m.trouble);
    expect(m.set(rect, 'stroke_width', 6), isTrue, reason: m.trouble);
    expect(wsTransform(ws, '$rect')['v_stroke_width'], 6, reason: 'the width is shown as the document says it');
    expect(m.say(rect, 'stroke', ''), isTrue, reason: 'None: no edge');
    expect(m.value(cube, 'stroke_width', 0), isNull, reason: 'a solid thing has no edge to say');
    // (a value the thing has not, changed in the inspector: the top bar says why it was not taken)
    ws.select('$cube');
    await settled(t, ws);   // (what the window told the inside on the way, answered: the words below are this change's own)
    expect(find.byWidgetPredicate((w) => w is NumField && w.id == 'op'), findsNothing, reason: 'a solid has no opacity: no knob for it');
    expect(m.set(rect, 'opacity', .5), isTrue);
    expect(m.say(rect, 'stroke', 'grey'), isFalse);
    expect(m.trouble, contains('six digits'));
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('the Anchor points into the thing: it turns and grows about that point, and it is where it stands', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final rect = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$rect');
    await t.pumpAndSettle();
    final before = m.frame(rect)!;
    expect([before[0], before[1]], [closeTo(960, 1), closeTo(540, 1)]);
    await seen(t, find.byType(AnchorCube)); // (the Anchor card's box: 27 points, one click each)
    final box = t.getRect(find.byType(AnchorCube));
    await t.tapAt(box.topLeft + AnchorGeometry(box.size).at(0, 0, 0)); // (its top left corner)
    await settled(t, ws);
    expect([m.value(rect, 'anchor_x', 0), m.value(rect, 'anchor_y', 0), m.value(rect, 'anchor_z', 0)], [-1, 1, 0], reason: 'its top left corner: ${m.trouble}');
    final after = m.frame(rect)!;
    expect(after[0], greaterThan(before[0] + 10), reason: 'its top left corner is where it stands now: it lies right of there');
    expect(after[1], greaterThan(before[1] + 10), reason: 'and below');
    expect(m.undo() && m.value(rect, 'anchor_x', 0) == 0 && m.value(rect, 'anchor_y', 0) == 0, isTrue, reason: 'one step');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('Parent: a thing goes with another without moving when it is given; moves with it after; no ring of parents', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final rect = m.add('shape', 'rect'), cube = m.add('solid', 'cube');
    expect(m.set(rect, 'x', -1) && m.set(rect, 'size', 2) && m.set(rect, 'rotate', 30) && m.set(cube, 'x', 1) && m.set(cube, 'y', .5), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$cube');
    await t.pumpAndSettle();
    final before = m.frame(cube)!;
    final parent = find.byWidgetPredicate((w) => w is Chooser && w.id == 'v_parent');
    await seen(t, parent);
    await pick(t, 'v_parent', t.widget<Chooser>(parent).options.firstWhere((o) => o.endsWith('Rect'))); // (the inside's Transform card: Parent, another thing, named with its number: '2. Rect')
    await settled(t, ws);   // (the hand let go, not waited for by the window: where it stands is said once the inside has taken it)
    expect(m.value(cube, 'parent', 0), rect, reason: m.trouble);
    final given = m.frame(cube)!;
    expect([given[0], given[1]], [closeTo(before[0], 1), closeTo(before[1], 1)], reason: 'it did not move when it was given a parent');
    expect(m.set(rect, 'x', 0), isTrue);
    expect(m.frame(cube)![0], greaterThan(given[0] + 50), reason: 'it goes with its parent');
    expect(m.set(rect, 'parent', cube.toDouble()), isFalse, reason: 'no ring of parents');
    expect(m.set(cube, 'parent', 0), isTrue);
    expect(m.frame(cube)![0], closeTo(m.frame(cube)![0], 1));
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('a text of two lines, its size and its alignment are the inside\'s Text card', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final text = m.add('text', _font);
    ws
      ..reread()
      ..select('$text');
    await t.pumpAndSettle();
    final box = find.descendant(of: find.byType(TextGroup), matching: find.byType(EditableText));
    await seen(t, box);
    await t.tap(box);
    await t.enterText(box, 'MOTOLII\nmakes moves');
    FocusManager.instance.primaryFocus?.unfocus();
    await t.pumpAndSettle();
    expect(m.word(text, 'text'), 'MOTOLII\nmakes moves', reason: m.trouble);
    expect(ws.layer?.name, 'MOTOLII makes moves', reason: 'listed on one line');
    expect(wsTransform(ws, '$text')['v_size'], closeTo(.5 * m.point, .1), reason: 'its size in points of the picture');
    final centre = find.descendant(of: find.byWidgetPredicate((w) => w is PropRow && w.id == 'v_align'), matching: find.byWidgetPredicate((w) => w is Text && w.data?.toLowerCase() == 'centre')); // (buttons say it in capitals)
    await seen(t, centre);
    await t.tap(centre);
    await settled(t, ws);
    expect(m.value(text, 'align', 0), .5, reason: 'Centre is ThorVG\'s half way: ${m.trouble}');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('the Effects shelf: Gaussian Blur and Drop Shadow are ThorVG\'s own, put on and taken off; the rest are not on the shelf', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final rect = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$rect')
      ..shelf = 'Effects';
    await t.pumpAndSettle();
    Future<void> pick(String name) async {
      await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), name); // (the shelf's own search: the row is in sight)
      await t.pumpAndSettle();
      final row = find.byElementPredicate((e) => e.widget is Text && (e.widget as Text).data == name && e.findAncestorWidgetOfExactType<DockSearch>() == null && e.findAncestorWidgetOfExactType<DockFooter>() == null).first; // (the tile: not the search box's words, nor the foot that names the one picked)
      await seen(t, row);
      await doubleTap(t, row);   // (two presses put it on, or take it off: f052e7e)
      await settled(t, ws);
    }

    await pick('Gaussian Blur');
    expect(m.value(rect, 'blur', 0), 12, reason: m.trouble);
    expect(ws.fxOf('$rect'), ['Gaussian Blur']);
    expect(wsTransform(ws, '$rect')['v_blur'], 12, reason: 'the Blur card says the document\'s blur');
    await pick('Gaussian Blur');
    expect(m.value(rect, 'blur', 0), 0);

    await pick('Drop Shadow');
    expect(m.word(rect, 'shadow'), '000000', reason: m.trouble);
    expect(m.undo() && m.word(rect, 'shadow').isEmpty, isTrue, reason: 'one step');

    await pick('Duotone');
    expect(m.value(rect, 'tint', 0), 1, reason: m.trouble);
    await pick('Gradient Map');
    expect(m.value(rect, 'tritone', 0), 1, reason: m.trouble);
    expect(ws.fxOf('$rect'), ['Duotone', 'Gradient Map']);

    await pick('Glow');
    expect(m.word(rect, 'glow'), 'on', reason: m.trouble);   // (Filament's bloom on its own picture, its strength glow_opacity: 72e9aea)

    await pick('Trim Paths');
    expect(m.keyMoments(rect, 'trim'), [0, 1], reason: 'drawn on in a second, from the playhead');
    expect([m.value(rect, 'trim', 0), m.value(rect, 'trim', 1)], [1, 0]);
    await pick('Trim Paths');
    expect([m.keyMoments(rect, 'trim'), m.value(rect, 'trim', 0)], [isEmpty, 0], reason: 'taken off with its keys');

    const none = 'Extrude'; // (one After Effects has and the inside does not)
    await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), none);
    await t.pumpAndSettle();
    expect(find.text(none), findsOneWidget, reason: 'only the search box\'s own words: $none, which the inside does not do, is not on the shelf');
    await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), '');
    await t.pumpAndSettle();
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('Space: a flat thing is 2D (by the order) or 2.5D (in the world, by depth); a solid thing is 3D', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final ring = m.add('solid', 'ring'), rect = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$rect');
    await t.pumpAndSettle();
    final plate = find.byWidgetPredicate((w) => w is GlyphPlate && w.label == '2.5D'); // (the Transform card's three plates)
    await seen(t, plate);
    await t.tap(plate);
    await settled(t, ws);
    expect(m.word(rect, 'space'), '2.5d', reason: m.trouble);
    ws.reread();
    await t.pumpAndSettle();
    final z = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_z');   // (in the world: its depth, scrubbed as any number: Alight Motion's Z strip)
    await seen(t, z);
    await t.timedDragFrom(t.getCenter(z.first), const Offset(60, 0), const Duration(milliseconds: 300));
    await settled(t, ws);
    expect(m.value(rect, 'z', 0), isNot(0), reason: 'moved in depth: ${m.trouble}');
    expect(m.undo(), isTrue, reason: 'the depth back');
    expect(m.undo() && m.word(rect, 'space').isEmpty, isTrue, reason: 'one step');
    ws.select('$ring');
    await t.pumpAndSettle();
    expect(find.byKey(const ValueKey('choice:v_space')), findsNothing, reason: 'a solid thing is 3D, and only that');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('Create: Polygon, Star, Line and Null are ThorVG shapes of the document; a Null is a thing to go with', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final made_ = <String, int>{};
    for (final (shape, name) in const [('polygon', 'Polygon'), ('star', 'Star'), ('line', 'Line'), ('null', 'Null')]) {
      final made = made_[shape] = m.add('shape', shape);
      expect(made, isNot(0), reason: m.trouble);
      expect(m.word(made, 'name'), name);
      expect(m.frame(made), isNotNull, reason: 'it can be held, even if nothing of it is seen');
    }
    final camera = m.add('camera', 'camera');
    expect(camera, isNot(0), reason: m.trouble);
    expect([m.word(camera, 'name'), m.frame(camera)], ['Camera', null], reason: 'a camera is not seen: nothing to hold on the stage');
    expect(m.set(camera, 'rotate_y', 30) && m.set(camera, 'zoom', 2) && m.set(camera, 'distance', 1.5), isTrue, reason: m.trouble);
    expect(m.set(camera, 'size', 2), isFalse, reason: 'a camera has no size');
    expect(m.set(camera, 'aperture', 1.4) && m.set(camera, 'focus', 6) && m.set(camera, 'blur_level', 3), isTrue, reason: m.trouble);
    ws.reread();
    expect(ws.layers.first.kind, WsKind.camera);
    final seen = wsTransform(ws, '$camera');
    expect([seen['v_rotate_y'], seen['v_zoom'], seen['v_distance']], [30, 200, closeTo(1.5 * 6 * m.point, .1)], reason: 'the Camera card says the document');
    expect(m.set(made_['star']!, 'points', 8) && m.set(made_['star']!, 'inner', .7), isTrue, reason: m.trouble);   // (by what each is, not where it lies in the list: the list is the top row first)
    expect(m.set(made_['polygon']!, 'points', 0), isFalse, reason: 'points from 1 (one value for every shape since 7ec94ea: an arc\'s degrees too)');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('Create: the Repeater lays the picked thing out in copies, as one step (one tile, its numbers continuous: K5)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final star = m.add('shape', 'star');
    ws
      ..reread()
      ..select('$star')
      ..shelf = 'Create';
    await t.pumpAndSettle();
    await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), 'Repeat');
    await t.pumpAndSettle();
    final tile = find.descendant(of: find.byType(CreateShelf), matching: find.text('Repeater')).first;
    await seen(t, tile);
    await doubleTap(t, tile);   // (two presses put it on: f052e7e)
    await settled(t, ws);
    expect([m.value(star, 'copies', 0), m.value(star, 'copy_x', 0), m.value(star, 'copy_size', 0)], [5, 1.3, .92], reason: m.trouble);
    expect(m.undo() && m.value(star, 'copies', 0) == 1, isTrue, reason: 'one step');
    expect(m.set(star, 'copies', 1001), isFalse, reason: 'a thousand at most');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('the Depth desk is the camera\'s lens: its focal length is the zoom, a scrub of the aperture is one step', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    final camera = m.add('camera', 'camera');
    ws
      ..reread()
      ..select('$camera')
      ..front('Camera');   // (a pane of its own since c1d8253: the camera seen from above, whatever is picked)
    await t.pumpAndSettle();
    expect(find.text('45 mm'), findsWidgets, reason: 'zoom 1 on Filament\'s 24 mm film');
    expect(find.text('On ${m.word(cube, 'name')}'), findsOneWidget, reason: 'the focus is on what lies where it looks: the document says so, not the window');
    final stop = find.widgetWithText(DeskScrub, 'Aperture');
    await seen(t, stop);
    await t.drag(stop, const Offset(-80, 0));
    await t.pumpAndSettle();
    final aperture = m.value(camera, 'aperture', 0)!;
    expect(aperture, inExclusiveRange(1.4, 22), reason: 'the f-number the hand left');
    expect(m.undo() && m.value(camera, 'aperture', 0) == 0, isTrue, reason: 'one step');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('Colors: a saved gradient fills a flat thing as a ramp of all its colours; Used here is the document\'s fills', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final rect = m.add('shape', 'rect');
    expect(m.say(rect, 'fill', '2a4bd8'), isTrue);
    ws
      ..reread()
      ..select('$rect')
      ..shelf = 'Colors';
    await t.pumpAndSettle();
    await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), 'Hot pink');   // (found by its name)
    await t.pumpAndSettle();
    final tile = find.byWidgetPredicate((w) => w.runtimeType.toString() == '_Grad').first; // (a starter gradient: Hot pink)
    await seen(t, tile);
    await doubleTap(t, tile);   // (two presses put it on: f052e7e)
    await settled(t, ws);
    expect(m.word(rect, 'fill').split(' '), allOf(hasLength(11), containsAllInOrder(['e974ab', 'f08a5d', 'efcb4e'])), reason: 'its three colours in order, mixed between as CAM16 mixes them (f052e7e): ${m.trouble}');
    expect(m.say(m.add('solid', 'cube'), 'fill', 'e974ab f08a5d'), isFalse, reason: 'a solid is filled with one colour');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('no decoration: the document\'s name on the stage and the timeline, its own marks and work area on the ruler', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    m.add('shape', 'rect');
    ws.reread();
    await t.pumpAndSettle();
    expect(find.text('Hit'), findsNothing, reason: 'the sample\'s marks are not the document\'s');
    final name = ws.document!.replaceFirst('.motolii', '');
    expect(find.text(name), findsWidgets, reason: 'the timeline says the document');
    expect(m.work, (0.0, m.seconds), reason: 'not said: all of it');
    expect(m.setWork(1, 3) && m.work == (1.0, 3.0), isTrue, reason: m.trouble);
    expect(m.undo() && m.work == (0.0, m.seconds), isTrue, reason: 'one step');
    expect(m.setWork(3, 1), isFalse);
    expect(m.mark(2, 'Drop') && m.mark(.5, 'Hit'), isTrue, reason: m.trouble);
    expect(m.marks, [(.5, 'Hit'), (2.0, 'Drop')]);
    ws.reread();
    await t.pumpAndSettle();
    expect(m.mark(2, '') && m.marks.length == 1, isTrue, reason: 'a mark said with nothing is taken away');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('Animate all in the inspector is the inside\'s keying; a new kind picked keeps it (the top bar\'s Edit/Animate band went: f052e7e)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final rect = m.add('shape', 'rect'), cube = m.add('solid', 'cube');
    ws
      ..reread()
      ..select('$rect');
    await t.pumpAndSettle();
    Finder animateAll() => find.descendant(of: find.ancestor(of: find.text('Animate all'), matching: find.byType(Row)).first, matching: find.byType(OnOff));
    await t.tap(find.descendant(of: animateAll(), matching: find.text('Off')));
    await settled(t, ws);
    expect(m.keyingNow, isTrue, reason: 'Animate all keys every change; its word is pressed as its switch (AbsentMotion\'s SwitchRow)');
    ws.select('$cube');
    await t.pumpAndSettle();
    expect(m.keyingNow, isTrue, reason: 'another kind picked does not turn it off');
    expect(t.widget<OnOff>(animateAll()).on, isTrue, reason: 'and its switch says so');
    await t.tap(animateAll());
    await settled(t, ws);
    expect(m.keyingNow, isFalse);
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('guides are the document\'s: pulled out of the top ruler, a level one where it is let go; back over the ruler, gone', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    m.add('shape', 'rect');
    ws.reread();
    await t.pumpAndSettle();
    expect(m.guides, isEmpty, reason: 'the sample\'s guides are not the document\'s');
    final stage = t.getRect(find.byKey(const ValueKey('ws-stage-comp')));
    final view = t.getRect(find.ancestor(of: find.byKey(const ValueKey('ws-stage-comp')), matching: find.byType(LayoutBuilder)).first);
    await t.dragFrom(Offset(stage.center.dx, view.top + 5), Offset(0, stage.center.dy - view.top - 5));
    await t.pumpAndSettle();
    expect(m.guides.length, 1, reason: m.trouble);
    expect(m.guides.single.$1, 'y');
    expect(m.guides.single.$2, closeTo(540, 30), reason: 'about the middle of the picture');
    expect(m.undo() && m.guides.isEmpty, isTrue, reason: 'one step');
    expect(m.guide('x', 150, true) && m.guide('x', 150, false) && m.guides.isEmpty, isTrue);
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('the tools draw things of the document: a rectangle as wide and high as the box drawn, a line from point to point', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    ws.reread();
    await t.pumpAndSettle();
    final stage = t.getRect(find.byKey(const ValueKey('ws-stage-comp')));
    await t.tap(find.byKey(const ValueKey('ws-tool-2'))); // (the Rectangle tool)
    await t.pumpAndSettle();
    await t.dragFrom(stage.center - Offset(stage.width * .2, stage.height * .1), Offset(stage.width * .4, stage.height * .2));
    await t.pumpAndSettle();
    final rect = int.parse(ws.selected!);
    expect([m.word(rect, 'kind'), m.word(rect, 'shape')], ['shape', 'rect'], reason: m.trouble);
    final box = m.frame(rect)!, k = 1920 / stage.width;
    expect(box[2], closeTo(stage.width * .4 * k, 12), reason: 'as wide as drawn');
    expect(box[3], closeTo(stage.height * .2 * k, 12), reason: 'and as high');
    expect(m.undo() && m.things.isEmpty, isTrue, reason: 'one step');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);

  testWidgets('the Brush draws a path by hand: its stroke kept, its outline AbsentMotion\'s brush round it, a closed path in the ink', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    ws
      ..reread()
      ..select(null);
    await t.pumpAndSettle();
    final stage = t.getRect(find.byKey(const ValueKey('ws-stage-comp'))), before = m.things.length;
    await t.tap(find.byKey(const ValueKey('ws-tool-6')));
    await t.pumpAndSettle();
    final g = await t.startGesture(stage.center - Offset(stage.width * .2, 0));
    for (var i = 0; i < 20; i++) {   // (a wave across the stage)
      await g.moveBy(Offset(stage.width * .02, (i % 10 < 5 ? 1 : -1) * stage.height * .02));
      await t.pump();
    }
    await g.up();
    await settled(t, ws);
    for (var i = 0; i < 200 && ws.selected == null; i++) {   // (made by the inside, picked once it is there)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 25)));
      await t.pump();
    }
    final made = int.parse(ws.selected!);
    expect(m.things.length, before + 1, reason: m.trouble);
    expect([m.word(made, 'kind'), m.word(made, 'shape')], ['shape', 'path']);
    final path = m.word(made, 'path');
    expect(path.startsWith('M') && path.endsWith('Z'), isTrue, reason: 'a closed outline: ${path.length > 80 ? path.substring(0, 80) : path}');
    final drawing = m.word(made, 'drawing').split(' ');
    expect(drawing.first == '1' && drawing.length > 8 && (drawing.length - 2) % 3 == 0, isTrue, reason: 'the stroke kept as drawn, a brush: how wide, then each point and its pressure');
    expect(m.word(made, 'fill'), ws.ink.isEmpty ? 'ffffff' : ws.ink, reason: 'drawn in the ink, white with none');
    await t.tap(find.text('Eraser').last);   // (Alight Motion's eraser: a stroke that takes from the picked drawing)
    await t.pumpAndSettle();
    final cut = await t.startGesture(stage.center - Offset(stage.width * .1, stage.height * .2));
    for (var i = 0; i < 20; i++) {   // (down across the wave)
      await cut.moveBy(Offset(0, stage.height * .02));
      await t.pump();
    }
    await cut.up();
    await settled(t, ws);
    for (var i = 0; i < 200 && !m.word(made, 'drawing').contains(';'); i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 25)));
      await t.pump();
    }
    expect([m.things.length, m.word(made, 'drawing').contains('; 3 '), m.word(made, 'path') != path, m.word(made, 'path').startsWith('M')], [before + 1, true, true, true],
        reason: 'erased from it, no new thing: ${m.trouble}');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('the Pen puts a point at each press and closes on the first: a path shape of straight stretches (Alight Motion\'s vector drawing)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    ws
      ..reread()
      ..select(null);
    await t.pumpAndSettle();
    final stage = t.getRect(find.byKey(const ValueKey('ws-stage-comp'))), before = m.things.length;
    await t.tap(find.byKey(const ValueKey('ws-tool-4')));   // (Pen, P)
    await t.pumpAndSettle();
    final c = stage.center, d = stage.height * .15;
    for (final p in [c + Offset(-d, d), c + Offset(0, -d), c + Offset(d, d), c + Offset(-d, d)]) {   // (round, and on the first again)
      await t.tapAt(p);
      await settled(t, ws);
    }
    for (var i = 0; i < 200 && m.things.length == before; i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 25)));
      await t.pump();
    }
    final made = m.things.last, path = m.word(made, 'path');
    expect([m.things.length, m.word(made, 'shape')], [before + 1, 'path'], reason: m.trouble);
    expect([' L'.allMatches(path).length, path.endsWith(' Z')], [2, isTrue], reason: 'three points, closed: $path');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('the inks and the strip mean the document: a colour filled with nothing picked is what is drawn next; the Move tool along X keeps y', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    ws
      ..reread()
      ..select(null)
      ..shelf = 'Colors';
    await t.pumpAndSettle();
    await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), 'Lime dusk');   // (found by its name)
    await t.pumpAndSettle();
    final tile = find.byWidgetPredicate((w) => w.runtimeType.toString() == '_Grad').first;
    await seen(t, tile);
    await doubleTap(t, tile);   // (two presses put it on: f052e7e)
    await settled(t, ws);
    final ink = ws.ink.split(' ');
    expect([ink.first, ink.last], ['b3c66b', '4781e5'], reason: 'nothing picked: the ink');
    final stage = t.getRect(find.byKey(const ValueKey('ws-stage-comp')));
    await t.tap(find.byKey(const ValueKey('ws-tool-2')));
    await t.pumpAndSettle();
    await t.dragFrom(stage.center, Offset(stage.width * .1, stage.height * .1));
    await settled(t, ws);
    for (var i = 0; i < 200 && ws.selected == null; i++) {   // (made by the inside, picked once it is there)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 25)));
      await t.pump();
    }
    final rect = int.parse(ws.selected!);
    expect(m.word(rect, 'fill'), ws.ink, reason: 'drawn in the ink');
    await t.tap(find.byKey(const ValueKey('ws-tool-1')));
    await t.pumpAndSettle();
    await t.tap(find.text('X').first); // (the strip's, before the inspector's)
    await t.pumpAndSettle();
    final y = m.value(rect, 'y', 0);
    await t.dragFrom(stage.center + Offset(stage.width * .05, stage.height * .05), Offset(stage.width * .1, stage.height * .1));
    await t.pumpAndSettle();
    expect(m.value(rect, 'y', 0), y, reason: 'carried along X only');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, skip: !_built);
}
