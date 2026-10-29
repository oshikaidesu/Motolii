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
    expect(find.text('${18 + r} ready'), findsOneWidget, reason: 'seventeen tiles of its own since K5 (one Repeater for Radial and Grid), Glass since 10/29 (a material is made in Create), and the recipes');
    expect(t.takeException(), isNull);
    await expectSameAfterReopen(t);
    await _close(t);
  }, tags: 'main', skip: !_built);

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
    expect(inspectorCards(t), ['Camera', 'Fog', 'Projection', 'Note'], reason: 'where it is and looks, its fog (Alight Motion\'s camera has it), then its lens: orthographic, shifted');
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

  testWidgets('timing: starts and ends lined up, to the playhead, staggered, a start drawn out; keys go with a thing moved (Alight Motion\'s timing)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final a = m.add('shape', 'rect'), b = m.add('shape', 'rect');
    expect(m.set(a, 'start', 1) && m.set(a, 'duration', 2) && m.set(b, 'start', 2.5) && m.set(b, 'duration', 1) && m.key(b, 'x', 3, 0) && m.key(b, 'x', 3.5, 1), isTrue, reason: m.trouble);
    ws.reread();
    await t.pumpAndSettle();
    Future<void> tell(String how, {double at = 0}) async {
      ws.after(m.later('time_align', word: '$how $a $b', number: at));
      await settled(t, ws);
    }
    double s(int th) => m.value(th, 'start', 0)!;
    await tell('start');
    expect([s(a), s(b)], [1, 1], reason: 'to the earliest: ${m.trouble}');
    expect(m.keyMoments(b, 'x'), [closeTo(1.5, 1e-9), closeTo(2, 1e-9)], reason: 'its keys with it');
    await tell('end');
    expect([s(a) + 2, s(b) + 1], [3, 3], reason: 'ends at the latest');
    await tell('playhead', at: .5);
    expect([s(a), s(b)], [.5, .5]);
    await tell('stagger');
    expect([s(a), s(b)], [.5, 2.5], reason: 'one after the other, in the order picked');
    await tell('extend', at: 2);
    expect([s(a), m.value(a, 'duration', 0), s(b), m.value(b, 'duration', 0)], [2, closeTo(.5, 1e-9), 2, closeTo(1.5, 1e-9)], reason: 'each start to the playhead, either way, the ends staying (Final Cut Pro\'s Extend Edit)');
    await tell('trim_end', at: 3);
    expect([m.value(a, 'duration', 0), m.value(b, 'duration', 0)], [closeTo(.5, 1e-9), closeTo(1, 1e-9)], reason: 'an end to the playhead inside it; one it is past left (Final Cut Pro\'s Trim End)');
    Future<void> press(LogicalKeyboardKey k) async {   // (Final Cut Pro's keys: ⌥[ and ⌥])
      await t.sendKeyDownEvent(LogicalKeyboardKey.altLeft);
      await t.sendKeyEvent(k);
      await t.sendKeyUpEvent(LogicalKeyboardKey.altLeft);
      await settled(t, ws);
    }
    ws
      ..select('$b')
      ..frame = (2.5 * Ws.fps).round();
    await t.pumpAndSettle();
    await press(LogicalKeyboardKey.bracketLeft);
    expect([s(b), m.value(b, 'duration', 0)], [2.5, closeTo(.5, 1e-9)], reason: '⌥[ trims its start: ${m.trouble}');
    ws.frame = (2.75 * Ws.fps).round();
    await t.pumpAndSettle();
    await press(LogicalKeyboardKey.bracketRight);
    expect(m.value(b, 'duration', 0), closeTo(.25, 1e-9), reason: '⌥] trims its end');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('⌘A picks every layer, ⌘D makes one more of the picked (picked), the arrows on the top bar take it back and bring it again', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final a = m.add('shape', 'rect');
    m.add('shape', 'circle');
    ws.reread();
    await t.pumpAndSettle();
    Future<void> press(LogicalKeyboardKey k) async {
      await t.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
      await t.sendKeyEvent(k);
      await t.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
      await settled(t, ws);
    }
    ws.select('$a');
    await t.pumpAndSettle();
    await press(LogicalKeyboardKey.keyA);
    expect(ws.many, {for (final l in ws.layers) l.id}, reason: 'all the layers picked');
    expect(find.text('${ws.layers.length} picked'), findsNWidgets(2), reason: 'the bar over the stage and the strip under it say how many, not the one picked before');
    ws.select('$a');
    final before = m.things.length;
    await press(LogicalKeyboardKey.keyD);
    for (var i = 0; i < 200 && ws.selected == '$a'; i++) {   // (the new one is picked once the inside has made it)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 25)));
      await t.pump();
    }
    expect([m.things.length, m.word(int.parse(ws.selected!), 'shape')], [before + 1, 'rect'], reason: 'one more, a rect, picked: ${m.trouble}');
    await t.tap(find.byKey(const ValueKey('ws-undo')));
    await settled(t, ws);
    expect(m.things.length, before, reason: 'taken back by the arrow on the top bar');
    expect(ws.notice, 'Undone: things added or taken away', reason: 'the top bar says what came back (P22)');
    await t.tap(find.byKey(const ValueKey('ws-redo')));
    await settled(t, ws);
    expect(m.things.length, before + 1, reason: 'and brought again');
    ws.after(m.later('set', thing: a, name: 'x', number: 300));
    await settled(t, ws);
    await t.tap(find.byKey(const ValueKey('ws-undo')));
    await settled(t, ws);
    final rectName = ws.layers.firstWhere((l) => l.id == '$a').name;
    expect(ws.notice, startsWith('Undone: $rectName '), reason: 'a value taken back: its thing and the value by name, as the Edit menu names the step');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('a bar\'s right click is its own menu, as Ableton Live\'s clip menu: Split at the playhead, Delete takes it out (the window\'s own commands)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final a = m.add('shape', 'rect');
    ws
      ..reread()
      ..select(null)
      ..frame = 60;
    await t.pumpAndSettle();
    final tracks = t.getRect(find.byKey(const ValueKey('ws-tl-tracks'))), on = Offset(tracks.left + tracks.width * .3, tracks.top + 14);   // (its bar: the only row)
    Future<void> menu(String item) async {
      await t.tapAt(on, buttons: kSecondaryButton);
      await t.pumpAndSettle();
      await t.tap(find.byKey(ValueKey('ws-menu-$item')));
      await settled(t, ws);
    }
    await menu('Split');
    expect([m.things.length, ws.selected], [2, isNot('$a')], reason: 'cut at the playhead (2 s): the copy from there picked, as Edit › Split does');
    await menu('Delete');
    expect(m.things.length, 1, reason: 'the bar right-clicked taken out');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('⇧E writes the picture at the playhead beside the document (Premiere Pro\'s Export Frame)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final rect = m.add('shape', 'rect');
    expect(m.say(rect, 'fill', 'ff3366'), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..frame = 30;
    await t.pumpAndSettle();
    await t.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
    await t.sendKeyEvent(LogicalKeyboardKey.keyE);
    await t.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
    await settled(t, ws);
    final png = File('${ws.documentFolder}/${ws.document!.substring(0, ws.document!.lastIndexOf('.'))}_30.png');   // (its name, in its folder)
    expect([png.existsSync(), ws.notice], [true, 'Written: ${png.path.split('/').last}'], reason: m.trouble);
    expect(png.readAsBytesSync().sublist(1, 4), 'PNG'.codeUnits, reason: 'a PNG');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('a skewed, bent rect put straight again by Reset on its Deform card (Alight Motion\'s reset skew and transform, After Effects\' Reset on an effect)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final rect = m.add('shape', 'rect');
    expect(m.set(rect, 'skew', 20) && m.set(rect, 'skew_axis', 45) && m.set(rect, 'bend', 30), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$rect');
    await t.pumpAndSettle();
    final deform = find.byWidgetPredicate((w) => w is Sect && w.title == 'Deform');
    await seen(t, deform);
    await t.tap(find.byKey(const ValueKey('card-reset:Deform')));
    await settled(t, ws);
    expect([m.value(rect, 'skew', 0), m.value(rect, 'skew_axis', 0), m.value(rect, 'bend', 0)], [0, 0, 0], reason: 'straight again: ${m.trouble}');
    ws.after(m.later('undo'));
    await settled(t, ws);
    expect([m.value(rect, 'skew', 0), m.value(rect, 'skew_axis', 0), m.value(rect, 'bend', 0)], [20, 45, 30], reason: 'one step back');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('Save Look keeps a layer\'s look on the Effects shelf by its name; two presses give it to another (Alight Motion\'s Save as Preset)', (t) async {
    final own = Directory.systemTemp.createTempSync('own');
    final setenv = DynamicLibrary.process().lookupFunction<Int32 Function(Pointer<Utf8>, Pointer<Utf8>, Int32), int Function(Pointer<Utf8>, Pointer<Utf8>, int)>('setenv');
    final name = 'MOTOLII_OWN'.toNativeUtf8(), path = own.path.toNativeUtf8();
    setenv(name, path, 1); // (the maker's folder here: a test does not write into the real one)
    malloc
      ..free(name)
      ..free(path);
    addTearDown(() {
      final unsetenv = DynamicLibrary.process().lookupFunction<Int32 Function(Pointer<Utf8>), int Function(Pointer<Utf8>)>('unsetenv');
      final n = 'MOTOLII_OWN'.toNativeUtf8();
      unsetenv(n);
      malloc.free(n);
    });
    final ws = await _pump(t), m = ws.inside!;
    final a = m.add('shape', 'rect'), b = m.add('shape', 'circle');
    expect(m.say(a, 'name', 'Pulse') && m.say(a, 'fill', 'ff3366') && m.key(a, 'opacity', 0, 0) && m.key(a, 'opacity', 1, 1), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$a');
    await t.pumpAndSettle();
    final edit = t.widget<PlatformMenuBar>(find.byType(PlatformMenuBar)).menus.whereType<PlatformMenu>().firstWhere((e) => e.label == 'Edit');
    edit.menus.expand((e) => e is PlatformMenuItemGroup ? e.members : [e]).firstWhere((e) => e.label == 'Save Look').onSelected!();
    await settled(t, ws);
    expect([File('${own.path}/looks/Pulse.motolii').existsSync(), ws.notice], [true, 'Look kept: Pulse'], reason: m.trouble);
    ws
      ..select('$b')
      ..shelf = 'Effects';
    await t.pumpAndSettle();
    await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), 'Pulse');
    await t.pumpAndSettle();
    await doubleTap(t, find.byKey(const ValueKey('ws-look-Pulse')));
    await settled(t, ws);
    expect([m.word(b, 'fill'), m.word(b, 'shape'), m.keyMoments(b, 'opacity')], ['ff3366', 'circle', [0, 1]], reason: 'its look, still a circle: ${m.trouble}');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('Save to My Elements keeps a group whole; its tile on the Create shelf puts it in at the playhead, linked (Alight Motion\'s Elements)', (t) async {
    final own = Directory.systemTemp.createTempSync('own');
    final setenv = DynamicLibrary.process().lookupFunction<Int32 Function(Pointer<Utf8>, Pointer<Utf8>, Int32), int Function(Pointer<Utf8>, Pointer<Utf8>, int)>('setenv');
    final name = 'MOTOLII_OWN'.toNativeUtf8(), path = own.path.toNativeUtf8();
    setenv(name, path, 1);
    malloc
      ..free(name)
      ..free(path);
    addTearDown(() {
      final unsetenv = DynamicLibrary.process().lookupFunction<Int32 Function(Pointer<Utf8>), int Function(Pointer<Utf8>)>('unsetenv');
      final n = 'MOTOLII_OWN'.toNativeUtf8();
      unsetenv(n);
      malloc.free(n);
    });
    final ws = await _pump(t), m = ws.inside!;
    final g = m.add('shape', 'null'), a = m.add('shape', 'star'), b = m.add('shape', 'circle'), w = m.add('text', _font);
    expect(m.say(g, 'name', 'Badge') && m.set(a, 'parent', g.toDouble()) && m.set(b, 'parent', g.toDouble()) && m.set(w, 'parent', g.toDouble()) && m.say(w, 'text', 'Sale') && m.set(g, 'x', .6), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$g');
    await t.pumpAndSettle();
    final edit = t.widget<PlatformMenuBar>(find.byType(PlatformMenuBar)).menus.whereType<PlatformMenu>().firstWhere((e) => e.label == 'Edit');
    edit.menus.expand((e) => e is PlatformMenuItemGroup ? e.members : [e]).firstWhere((e) => e.label == 'Save to My Elements').onSelected!();
    await settled(t, ws);
    expect([File('${own.path}/elements/Badge.motolii').existsSync(), ws.notice], [true, 'Saved to My Elements: Badge'], reason: m.trouble);
    ws
      ..frame = 30
      ..shelf = 'Create';
    await t.pumpAndSettle();
    await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), 'Badge');
    await t.pumpAndSettle();
    final tile = find.descendant(of: find.byType(CreateShelf), matching: find.text('Badge')).first;
    await seen(t, tile);
    await doubleTap(t, tile);
    await settled(t, ws);
    final made = int.parse(ws.selected!);
    List<int> held() => [for (final x in m.things) if (m.value(x, 'parent', 0) == made) x];
    expect([made != g, m.word(made, 'element'), m.value(made, 'x', 0), m.value(made, 'start', 0), [for (final x in held()) m.word(x, 'shape')]],
        [true, '{own}/elements/Badge.motolii', 0, closeTo(30 / Ws.fps, 1e-6), ['star', 'circle', '']], reason: 'in the middle at the playhead, linked, with what it holds: ${m.trouble}');
    // (the element written again, its star a polygon and its words others: the one put in follows, its own place kept)
    expect(m.set(made, 'x', -.3) && m.say(a, 'shape', 'polygon') && m.say(w, 'text', 'Big Sale'), isTrue, reason: m.trouble);
    ws.select('$g');
    await t.pumpAndSettle();
    edit.menus.expand((e) => e is PlatformMenuItemGroup ? e.members : [e]).firstWhere((e) => e.label == 'Save to My Elements').onSelected!();
    await settled(t, ws);
    ws.after(m.later('follow_elements'));
    await settled(t, ws);
    expect([m.value(made, 'x', 0), [for (final x in held()) m.word(x, 'kind') == 'text' ? m.word(x, 'text') : m.word(x, 'shape')]], [-.3, ['polygon', 'circle', 'Big Sale']], reason: 'followed: ${m.trouble}');
    final words = held().firstWhere((x) => m.word(x, 'kind') == 'text');
    expect(m.say(words, 'text', 'Ours'), isTrue);   // (its own words, kept when the element changes again: an override)
    expect(m.say(b, 'shape', 'rect'), isTrue);
    edit.menus.expand((e) => e is PlatformMenuItemGroup ? e.members : [e]).firstWhere((e) => e.label == 'Save to My Elements').onSelected!();
    await settled(t, ws);
    ws.after(m.later('follow_elements'));
    await settled(t, ws);
    expect([for (final x in held()) m.word(x, 'kind') == 'text' ? m.word(x, 'text') : m.word(x, 'shape')], ['polygon', 'rect', 'Ours'], reason: 'its own words kept: ${m.trouble}');
    ws.select('$made');   // (unlinked: a group of its own, the element's changes no longer followed)
    await t.pumpAndSettle();
    edit.menus.expand((e) => e is PlatformMenuItemGroup ? e.members : [e]).firstWhere((e) => e.label == 'Unlink from Element').onSelected!();
    await settled(t, ws);
    expect(m.say(a, 'shape', 'star'), isTrue);
    ws.select('$g');
    await t.pumpAndSettle();
    edit.menus.expand((e) => e is PlatformMenuItemGroup ? e.members : [e]).firstWhere((e) => e.label == 'Save to My Elements').onSelected!();
    await settled(t, ws);
    ws.after(m.later('follow_elements'));
    await settled(t, ws);
    expect([m.word(made, 'element'), m.word(held().first, 'shape')], ['', 'polygon'], reason: 'unlinked: ${m.trouble}');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('an element put in longer than it is plays as its Re-timing says: its intro and outro kept, the middle held, stretched or looped (Alight Motion\'s Element settings)', (t) async {
    final own = Directory.systemTemp.createTempSync('own');
    final setenv = DynamicLibrary.process().lookupFunction<Int32 Function(Pointer<Utf8>, Pointer<Utf8>, Int32), int Function(Pointer<Utf8>, Pointer<Utf8>, int)>('setenv');
    final name = 'MOTOLII_OWN'.toNativeUtf8(), path = own.path.toNativeUtf8();
    setenv(name, path, 1);
    malloc
      ..free(name)
      ..free(path);
    addTearDown(() {
      final unsetenv = DynamicLibrary.process().lookupFunction<Int32 Function(Pointer<Utf8>), int Function(Pointer<Utf8>)>('unsetenv');
      final n = 'MOTOLII_OWN'.toNativeUtf8();
      unsetenv(n);
      malloc.free(n);
    });
    final ws = await _pump(t), m = ws.inside!;
    final g = m.add('shape', 'null'), a = m.add('shape', 'star');
    expect(m.say(g, 'name', 'Slide') && m.set(g, 'duration', 2) && m.set(a, 'parent', g.toDouble()) && m.key(a, 'x', 0, 0) && m.key(a, 'x', 2, 1), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$g');
    await t.pumpAndSettle();
    expect(m.cards(g).map((c) => c.$1), contains('Element'), reason: 'a group\'s Element card');
    expect(m.say(g, 'retime', 'stretch') && m.set(g, 'retime_in', .5) && m.set(g, 'retime_out', 1.5), isTrue, reason: m.trouble);
    ws.reread();
    await t.pumpAndSettle();
    final ruler = t.widgetList<CustomPaint>(find.byType(CustomPaint)).map((c) => c.painter).whereType<WtRulerPainter>().first;
    expect(ruler.retimed, [(.5 * Ws.fps).round(), (1.5 * Ws.fps).round()], reason: 'its intro\'s end and outro\'s start marked on the ruler, as Alight Motion\'s yellow marks');
    final edit = t.widget<PlatformMenuBar>(find.byType(PlatformMenuBar)).menus.whereType<PlatformMenu>().firstWhere((e) => e.label == 'Edit');
    edit.menus.expand((e) => e is PlatformMenuItemGroup ? e.members : [e]).firstWhere((e) => e.label == 'Save to My Elements').onSelected!();
    await settled(t, ws);
    ws.frame = 0;
    ws.after(m.later('element', word: '${own.path}/elements/Slide.motolii', number: 0), null, (made) => ws.select('$made'));
    await settled(t, ws);
    final made = int.parse(ws.selected!), b = m.things.firstWhere((x) => m.value(x, 'parent', 0) == made);
    expect(m.set(made, 'duration', 4), isTrue, reason: m.trouble);   // (twice as long as it was)
    Future<double> at(double s) async => (await m.shownAt([(b, 'x', s)])).single!;
    expect([await at(.25), await at(2), await at(3.75)], [closeTo(.125, 1e-6), closeTo(.5, 1e-6), closeTo(.875, 1e-6)], reason: 'stretch: the intro and outro as they were, the middle drawn out');
    expect(m.say(made, 'retime', 'freeze'), isTrue);
    expect(await at(2), closeTo(.75, 1e-6), reason: 'freeze: the middle played, then held');
    expect(m.say(made, 'retime', 'loop'), isTrue);
    expect(await at(2), closeTo(.5, 1e-6), reason: 'loop: the middle again');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('a text placed with the Text tool is typed at once: its words picked in the Text card, what is typed is its (AbsentMotion\'s addText, Figma\'s text tool)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    ws.reread();
    await t.pumpAndSettle();
    await t.tap(find.byKey(const ValueKey('ws-tool-5')));
    await t.pumpAndSettle();
    final stage = t.getRect(find.byKey(const ValueKey('ws-stage-comp')));
    await t.tapAt(stage.center + Offset(0, stage.height * .2));
    for (var i = 0; i < 200 && ws.selected == null; i++) {   // (made by the inside, picked once it is there)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 25)));
      await t.pump();
    }
    await t.pumpAndSettle();
    final text = int.parse(ws.selected!), field = find.byWidgetPredicate((w) => w is EditableText && w.focusNode.debugLabel == 'text-content');
    expect(field, findsOneWidget);
    final f = t.widget<EditableText>(field);
    expect([f.focusNode.hasFocus, f.controller.selection.start, f.controller.selection.end], [true, 0, 'Text'.length], reason: 'its words held and picked: what is typed takes their place');
    t.testTextInput.enterText('HELLO');
    await t.pump();
    f.focusNode.unfocus();   // (left: the words are the text's)
    await settled(t, ws);
    expect(m.word(text, 'text'), 'HELLO');
    expect(t.takeException(), isNull);
    await _close(t);
  }, tags: 'main', skip: !_built);

  testWidgets('the Text tool\'s row has its font, size, alignment and colour in one row, as Alight Motion\'s text bar', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final words = m.add('text', _font);
    expect(m.say(words, 'fill', 'ff3366'), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$words');
    await t.pumpAndSettle();
    await t.tap(find.byKey(const ValueKey('ws-tool-5')));
    await t.pumpAndSettle();
    final strip = find.byWidgetPredicate((w) => w.runtimeType.toString() == '_OptionsStrip');
    expect([for (final l in ['Font', 'Size', 'L', 'Fill']) find.descendant(of: strip, matching: find.text(l)).evaluate().length], [1, 1, 1, 1]);
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('Scale Assist is put on from the Effects shelf, as Alight Motion\'s effect is; its card only then', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final rect = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$rect');
    await t.pumpAndSettle();
    expect(m.cards(rect).map((c) => c.$1), isNot(contains('Scale Assist')), reason: 'not put on: no card');
    ws.shelf = 'Effects';
    await t.pumpAndSettle();
    await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), 'Scale Assist');
    await t.pumpAndSettle();
    await doubleTap(t, find.byKey(const ValueKey('ws-look-Scale Assist')));
    await settled(t, ws);
    expect([m.word(rect, 'scale_assist'), m.cards(rect).map((c) => c.$1).contains('Scale Assist')], ['both', true], reason: 'put on, its card there: ${m.trouble}');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('Save on an effect\'s card keeps that effect\'s values and keys alone on the Effects shelf, each press another (Alight Motion\'s presets of an effect)', (t) async {
    final own = Directory.systemTemp.createTempSync('own');
    final setenv = DynamicLibrary.process().lookupFunction<Int32 Function(Pointer<Utf8>, Pointer<Utf8>, Int32), int Function(Pointer<Utf8>, Pointer<Utf8>, int)>('setenv');
    final name = 'MOTOLII_OWN'.toNativeUtf8(), path = own.path.toNativeUtf8();
    setenv(name, path, 1); // (the maker's folder here: a test does not write into the real one)
    malloc
      ..free(name)
      ..free(path);
    addTearDown(() {
      final unsetenv = DynamicLibrary.process().lookupFunction<Int32 Function(Pointer<Utf8>), int Function(Pointer<Utf8>)>('unsetenv');
      final n = 'MOTOLII_OWN'.toNativeUtf8();
      unsetenv(n);
      malloc.free(n);
    });
    final ws = await _pump(t), m = ws.inside!;
    final a = m.add('shape', 'rect'), b = m.add('shape', 'circle');
    expect(m.say(a, 'fill', 'ff3366') && m.effect(a, 'Gaussian Blur', 0) && m.key(a, 'blur', 0, 0) && m.key(a, 'blur', 1, 24), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$a');
    await t.pumpAndSettle();
    for (final n in [1, 2]) {
      final more = find.byKey(const ValueKey('card-more:Gaussian Blur'));   // (its ⋮, as AbsentMotion's: Save as Preset in it)
      await seen(t, more);
      await t.tap(more);
      await t.pumpAndSettle();
      await t.tap(find.byKey(const ValueKey('card-save:Gaussian Blur')));
      for (var i = 0; i < 100 && !File('${own.path}/looks/Gaussian Blur $n.motolii').existsSync(); i++) {
        await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 20)));
        await t.pump();
      }
      expect(File('${own.path}/looks/Gaussian Blur $n.motolii').existsSync(), isTrue, reason: 'preset $n: ${m.trouble}');
    }
    ws
      ..select('$b')
      ..shelf = 'Effects';
    await t.pumpAndSettle();
    await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), 'Gaussian Blur 1');
    await t.pumpAndSettle();
    await doubleTap(t, find.byKey(const ValueKey('ws-look-Gaussian Blur 1')));
    await settled(t, ws);
    expect([m.keyMoments(b, 'blur'), m.word(b, 'fill')], [[0, 1], ''], reason: 'the blur and its keys, not the rect\'s fill: ${m.trouble}');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('Scale with the link on grows width and height together; with it off, the width alone (Alight Motion\'s size link)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final rect = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$rect');
    await t.pumpAndSettle();
    final link = find.byType(LinkMark).first, width = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_scale_x');
    await seen(t, width);
    Future<void> turn(bool on) async {
      if (t.widget<LinkMark>(link).on != on) await t.tap(link);
      await t.pumpAndSettle();
    }
    double v(String n) => m.value(rect, n, 0) ?? 1;
    await turn(true);
    await t.dragFrom(t.getCenter(width.first), const Offset(40, 0));
    await settled(t, ws);
    expect([v('scale_x'), v('scale_y')], [greaterThan(1), closeTo(v('scale_x'), 1e-6)], reason: 'both, the same: ${m.trouble}');
    final tall = v('scale_y');
    await turn(false);
    await t.dragFrom(t.getCenter(width.first), const Offset(40, 0));
    await settled(t, ws);
    expect([v('scale_x'), v('scale_y')], [greaterThan(tall), closeTo(tall, 1e-6)], reason: 'the width alone');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('Sample on the Colors shelf takes a colour off the screen (the Mac\'s eyedropper) and fills the picked thing with it', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final rect = m.add('shape', 'rect');
    t.binding.defaultBinaryMessenger.setMockMethodCallHandler(const MethodChannel('motolii/sampler'), (call) async => call.method == 'sample' ? '3a86ff' : null);   // (NSColorSampler's answer: a hand cannot point at the screen here)
    addTearDown(() => t.binding.defaultBinaryMessenger.setMockMethodCallHandler(const MethodChannel('motolii/sampler'), null));
    ws
      ..reread()
      ..select('$rect')
      ..shelf = 'Colors';
    await t.pumpAndSettle();
    await seen(t, find.byKey(const ValueKey('colour-sample')));
    await t.tap(find.byKey(const ValueKey('colour-sample')));
    await settled(t, ws);
    expect(m.word(rect, 'fill'), '3a86ff', reason: m.trouble);
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('a number scrubbed with ⇧ held moves a tenth as far, with ⌃ ten times (fine and coarse scrubbing, Ableton Live\'s keys)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final rect = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$rect');
    await t.pumpAndSettle();
    final x = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_x');
    await seen(t, x);
    Future<double> scrub([LogicalKeyboardKey? held]) async {   // (the key held once the scrub has begun, as Live has it: Shift at the press picks a span)
      final was = m.value(rect, 'x', 0) ?? 0;
      final g = await t.startGesture(t.getCenter(x.first), kind: PointerDeviceKind.mouse);
      await g.moveBy(const Offset(10, 0));   // (10 at its own rate, then 40 with the key)
      await t.pump();
      if (held != null) await t.sendKeyDownEvent(held);
      for (var i = 0; i < 4; i++) {
        await g.moveBy(const Offset(10, 0));
        await t.pump();
      }
      if (held != null) await t.sendKeyUpEvent(held);
      await g.up();
      await settled(t, ws);
      return (m.value(rect, 'x', 0) ?? 0) - was;
    }
    final plain = await scrub(), fine = await scrub(LogicalKeyboardKey.shiftLeft), coarse = await scrub(LogicalKeyboardKey.controlLeft);
    expect(plain, isNot(0), reason: m.trouble);
    expect([fine / plain, coarse / plain], [closeTo((10 + 4) / 50, .03), closeTo((10 + 400) / 50, .5)], reason: 'plain $plain, fine $fine, coarse $coarse');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('the size chip pressed once: Alight Motion\'s shapes and resolutions; one pressed is the document\'s size', (t) async {
    final ws = await _pump(t);
    Future<void> pick(String key) async {
      final was = ws.inside;
      await t.tap(find.byKey(const ValueKey('ws-top-size')));
      await t.pump(kDoubleTapTimeout);   // (one press: a second would type its numbers)
      await t.pumpAndSettle();
      expect(find.byKey(const ValueKey('ws-size-box')), findsOneWidget);
      await t.tap(find.byKey(ValueKey(key)));
      for (var i = 0; i < 100 && identical(ws.inside, was); i++) {   // (opened again at it, in real time)
        await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 20)));
        await t.pump();
      }
      await settled(t, ws);
    }
    expect(ws.inside!.size, (1920, 1080));
    await pick('size-9:16');
    expect(ws.inside!.size, (1080, 1920), reason: 'tall, its shorter side kept');
    await pick('size-720p');
    expect(ws.inside!.size, (720, 1280), reason: 'the same shape, smaller');
    expect(find.text('720 × 1280'), findsOneWidget);
    await t.tap(find.byKey(const ValueKey('ws-top-size')));   // (and its frame rate: the document's, the timeline counting in it)
    await t.pump(kDoubleTapTimeout);
    await t.pumpAndSettle();
    await t.tap(find.byKey(const ValueKey('size-24fps')));
    for (var i = 0; i < 100 && (ws.inside!.value(0, 'frame_rate', 0) ?? 0) != 24; i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 20)));
      await t.pump();
    }
    await settled(t, ws);
    expect([ws.inside!.value(0, 'frame_rate', 0), Ws.fps], [24, 24], reason: 'the document at 24 frames a second, and the timeline: ${ws.inside!.trouble}');
    await t.pumpAndSettle();
    await t.tap(find.byKey(const ValueKey('size-bg-Light Gray')));   // (and the colour behind it, AbsentMotion's six, in the same box still open)
    for (var i = 0; i < 100 && ws.inside!.word(0, 'background') != 'd8d8d8'; i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 20)));
      await t.pump();
    }
    expect(ws.inside!.word(0, 'background'), 'd8d8d8', reason: 'light gray behind it: ${ws.inside!.trouble}');
    await t.pumpAndSettle();
    expect(t.widget<Text>(find.byKey(const ValueKey('size-edited'))).data, startsWith('Total Edit Time  0h 0m'), reason: 'Alight Motion\'s Total Edit Time: the steps just made, under a minute');
    await t.tap(find.byKey(const ValueKey('size-len-2')));   // (AbsentMotion's New Layer Length: a thing put in lasts 2 s)
    await settled(t, ws);
    final made = ws.inside!.add('shape', 'rect');
    expect([ws.inside!.value(0, 'layer_length', 0), ws.inside!.value(made, 'duration', 0)], [2, 2], reason: ws.inside!.trouble);
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('the Transform card\'s knob turns the layer, as a dial (Alight Motion\'s rotation dial)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final rect = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$rect');
    await t.pumpAndSettle();
    final canvas = find.byType(TransformCanvas);
    await seen(t, canvas);
    final paint = find.descendant(of: canvas, matching: find.byType(LayoutBuilder)).first, at = t.getRect(paint);
    final doc = Ctx.of(t.element(paint)).doc, values = Map.of(doc.v);
    final knob = at.topLeft + TransformLayout(at.size, values, space: doc.space).knob;   // (where the canvas draws it: its own layout)
    await t.timedDragFrom(knob, const Offset(60, 30), const Duration(milliseconds: 300));
    await settled(t, ws);
    expect((m.value(rect, 'rotate', 0) ?? 0).abs(), greaterThan(10), reason: 'turned by the knob: ${m.trouble}');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('a film\'s sound and pictures are drawn on its bar: its loudness, read by the inside, laid where the film shows it, and what it plays (Alight Motion\'s waveform and frames on a strip)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final film = m.add('film', '$motoliiDir/check/beats.mp4');
    expect(film, isNot(0), reason: m.trouble);
    ws.reread();
    await t.pumpAndSettle();
    List<double>? wave() => [for (final e in find.byWidgetPredicate((w) => w is CustomPaint && w.painter is WtTracksPainter).evaluate()) ...(((e.widget as CustomPaint).painter as WtTracksPainter).rows)].where((r) => r.wave != null).firstOrNull?.wave;
    for (var i = 0; i < 400 && wave() == null; i++) {   // (read on another isolate: laid on the bar when it comes)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 25)));
      await t.pump();
    }
    final w = wave();
    expect(w, isNotNull, reason: 'its bar has its sound');
    expect([w!.length, w.reduce(math.max)], [ws.layers.single.outF - ws.layers.single.inF, greaterThan(-60)], reason: 'one loudness a frame, and heard');
    List<Object>? frames() => [for (final e in find.byWidgetPredicate((w) => w is CustomPaint && w.painter is WtTracksPainter).evaluate()) ...(((e.widget as CustomPaint).painter as WtTracksPainter).rows)].where((r) => r.frames != null).firstOrNull?.frames;
    for (var i = 0; i < 400 && frames() == null; i++) {   // (its pictures made small by the inside, on another isolate)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 25)));
      await t.pump();
    }
    expect(frames(), hasLength(6), reason: 'and the pictures it plays, one for each sixth of it (Alight Motion\'s strip of frames)');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('the Media shelf switches what it shows as Alight Motion\'s albums do, each with how many it has (All, Images, Video)', (t) async {
    final ws = await _pump(t);
    for (final n in ['a.jpg', 'b.jpg']) {
      File('$motoliiDir/fx_sample.jpg').copySync('${ws.documentFolder}/$n');
    }
    File('$motoliiDir/check/beats.mp4').copySync('${ws.documentFolder}/c.mp4');
    ws.shelf = 'Media';
    await t.pumpAndSettle();
    final folder = find.byKey(ValueKey('browser:file:${ws.documentFolder}'));
    await t.ensureVisible(folder.first);
    await t.tap(folder.first);
    for (var i = 0; i < 20 && find.byKey(ValueKey('browser:file:${ws.documentFolder}/c.mp4')).evaluate().isEmpty; i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
      await t.pumpAndSettle();
    }
    final chips = {for (final (n, k) in t.widget<DockChips>(find.byType(DockChips).first).chips) n: k};
    expect([chips['Images'], chips['Video'], (chips['All'] ?? 0) >= 3], [2, 1, true], reason: '$chips');
    await t.tap(find.text('Images').first);   // (only the pictures: Alight Motion's Pictures album)
    await t.pumpAndSettle();
    expect([find.byKey(ValueKey('browser:file:${ws.documentFolder}/a.jpg')).evaluate().isNotEmpty, find.byKey(ValueKey('browser:file:${ws.documentFolder}/c.mp4')).evaluate().isEmpty], [true, true]);
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('two films picked with ⌘ on the Media shelf are placed together, one after another (Alight Motion\'s adding several)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    for (final n in ['one.mp4', 'two.mp4']) {
      File('$motoliiDir/check/beats.mp4').copySync('${ws.documentFolder}/$n');
    }
    ws.shelf = 'Media';
    await t.pumpAndSettle();
    final folder = find.byKey(ValueKey('browser:file:${ws.documentFolder}'));
    await t.ensureVisible(folder.first);
    await t.tap(folder.first);
    await t.pumpAndSettle();
    Finder tile(String n) => find.byKey(ValueKey('browser:file:${ws.documentFolder}/$n'));
    for (var i = 0; i < 20 && tile('two.mp4').evaluate().isEmpty; i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
      await t.pumpAndSettle();
    }
    await t.tap(tile('one.mp4').first);
    await t.pump(kDoubleTapTimeout);
    await t.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
    await t.tap(tile('two.mp4').first);
    await t.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
    await t.pump(kDoubleTapTimeout);
    await t.pumpAndSettle();
    expect(find.text('Place 2'), findsOneWidget, reason: 'two picked: placed together');
    await t.tap(find.text('Place 2'));
    for (var i = 0; i < 200 && (m.things.length < 2 || ws.many.length < 2); i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 25)));
      await t.pump();
    }
    await settled(t, ws);   // (laid one after another once both are there)
    ws.reread();
    final (a, b) = (m.things[0], m.things[1]);
    expect([m.word(a, 'file'), m.word(b, 'file')].map((f) => f.split('/').last), ['one.mp4', 'two.mp4'], reason: m.trouble);
    expect(m.value(b, 'start', 0), closeTo((m.value(a, 'start', 0) ?? 0) + m.lasts(a), 1e-6), reason: 'the second begins as the first ends');
    expect(ws.many, {'$a', '$b'}, reason: 'both picked');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('a still picked on the Media shelf says how long it lasts before it is placed: 2 s as it comes, End to the document\'s end (Alight Motion\'s ruler under a photo)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    File('${ws.documentFolder}/dot.png').writeAsBytesSync(const [137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 13, 73, 68, 65, 84, 120, 218, 99, 252, 207, 192, 80, 15, 0, 4, 133, 1, 128, 132, 169, 140, 33, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130]);   // (one pixel, as a PNG says it)
    ws.shelf = 'Media';
    await t.pumpAndSettle();
    final folder = find.byKey(ValueKey('browser:file:${ws.documentFolder}'));
    await t.ensureVisible(folder.first);
    await t.tap(folder.first);
    await t.pumpAndSettle();
    final tile = find.byKey(ValueKey('browser:file:${ws.documentFolder}/dot.png'));
    for (var i = 0; i < 20 && tile.evaluate().isEmpty; i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
      await t.pumpAndSettle();
    }
    await t.tap(tile.first);
    await t.pump(kDoubleTapTimeout);
    await t.pumpAndSettle();
    String said() => t.widget<Text>(find.byKey(const ValueKey('media-still-seconds'))).data!;
    expect(said(), '2 s', reason: 'two seconds as it comes, as Alight Motion\'s');
    Future<int> place() async {
      final before = m.things.length;
      await t.tap(find.text('Place'));
      for (var i = 0; i < 200 && m.things.length == before; i++) {
        await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 25)));
        await t.pump();
      }
      await settled(t, ws);
      return m.things.last;
    }
    final two = await place();
    expect(m.value(two, 'duration', 0), 2, reason: 'it lasts the two seconds said: ${m.trouble}');
    for (final _ in [1, 2, 3, 4]) {
      await t.tap(find.byKey(const ValueKey('media-still-more')));
      await t.pump();
    }
    expect(said(), 'End', reason: '3, 5, 10, then the document\'s end');
    final end = await place();
    expect(m.value(end, 'duration', 0) ?? -1, lessThanOrEqualTo(0), reason: 'no length of its own: to the document\'s end');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('a film picked on the Media shelf is cut before it is placed: the band\'s left end drawn to its middle, it plays from there (Alight Motion\'s band under a video)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    File('$motoliiDir/check/beats.mp4').copySync('${ws.documentFolder}/cut.mp4');
    ws.shelf = 'Media';
    await t.pumpAndSettle();
    final folder = find.byKey(ValueKey('browser:file:${ws.documentFolder}'));
    await t.ensureVisible(folder.first);
    await t.tap(folder.first);
    await t.pumpAndSettle();
    final tile = find.byKey(ValueKey('browser:file:${ws.documentFolder}/cut.mp4'));
    for (var i = 0; i < 20 && tile.evaluate().isEmpty; i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
      await t.pumpAndSettle();
    }
    await t.tap(tile.first);
    await t.pump(kDoubleTapTimeout);
    final band = find.byKey(const ValueKey('media-film-range'));
    for (var i = 0; i < 100 && band.evaluate().isEmpty; i++) {   // (drawn once the discoverer has said its length)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
      await t.pump();
    }
    final len = await t.runAsync(() => mediaSecondsOf(m.libPath, '${ws.documentFolder}/cut.mp4')) ?? 0;
    expect(len, greaterThan(0), reason: 'its length known');
    final r = t.getRect(band);
    await t.dragFrom(r.centerLeft + const Offset(1, 0), Offset(r.width / 2 - 1, 0));
    await t.pump();
    expect(t.widget<Text>(find.byKey(const ValueKey('media-film-said'))).data, startsWith('0:'), reason: 'the part used, in its seconds');
    final before = m.things.length;
    await t.tap(find.text('Place'));
    for (var i = 0; i < 200 && (m.things.length == before || (m.value(m.things.last, 'begin', 0) ?? 0) == 0); i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 25)));
      await t.pump();
    }
    await settled(t, ws);
    final film = m.things.last;
    expect(m.value(film, 'begin', 0), closeTo(len / 2, len * .05), reason: 'it plays from the middle of the film: ${m.trouble}');
    expect(m.value(film, 'duration', 0), closeTo(len / 2, len * .05), reason: 'to its end');
    expect(m.undo(), isTrue);
    expect(m.things.length, before, reason: 'placed and cut: one step');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('Speed dragged on the Play card stretches the thing in time round where it is held: held at its end, its end stays and its keys come with it (After Effects\' Time Stretch)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final rect = m.add('shape', 'rect');
    expect(m.set(rect, 'duration', 2) && m.key(rect, 'x', 0, -1) && m.key(rect, 'x', 1, 1), isTrue, reason: m.trouble);
    ws..reread()..select('$rect');
    await t.pumpAndSettle();
    await pick(t, 'v_speed_hold', 'end');
    await settled(t, ws);
    expect(m.word(rect, 'speed_hold'), 'end');
    final knob = find.byWidgetPredicate((w) => w is NumField && w.id == 'v_speed');
    await seen(t, knob);
    await t.dragFrom(t.getCenter(knob.first), const Offset(60, 0));
    await t.pumpAndSettle();
    for (var i = 0; i < 80 && (m.value(rect, 'speed', 0) ?? 1) == 1; i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 25)));
      await t.pump();
    }
    await settled(t, ws);
    final s = m.value(rect, 'speed', 0)!;
    expect(s, isNot(1), reason: 'its speed changed: ${m.trouble}');
    final lasts = 2 / s.abs();
    expect(m.value(rect, 'duration', 0), closeTo(lasts, 1e-6), reason: 'as much shorter as it goes faster');
    expect((m.value(rect, 'start', 0) ?? 0) + lasts, closeTo(2, 1e-6), reason: 'its end held where it was');
    expect(m.keyMoments(rect, 'x').last, closeTo(2 - 1 / s.abs(), 1e-6), reason: 'its keys stretched round its end');
    expect(m.undo(), isTrue);
    expect(m.value(rect, 'duration', 0), 2, reason: 'one step back');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('a picked thing that moves shows its way on the stage, from where it begins to where it ends (Alight Motion\'s motion path)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final rect = m.add('shape', 'rect');
    expect(m.key(rect, 'x', 0, -1) && m.key(rect, 'x', 1, 1), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$rect');
    await t.pumpAndSettle();
    List<Offset>? trail() => [for (final e in find.byWidgetPredicate((w) => w is CustomPaint && w.painter?.runtimeType.toString() == '_ViewPainter').evaluate()) (((e.widget as CustomPaint).painter as dynamic).trail as List<Offset>?)].nonNulls.firstOrNull;
    for (var i = 0; i < 200 && trail() == null; i++) {   // (the inside's reckoning, on another isolate)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 25)));
      await t.pump();
    }
    final w = trail();
    expect(w, isNotNull, reason: 'its way drawn');
    expect([w!.first.dx, w.last.dx, w.first.dy], [closeTo(960 - m.point, 2), closeTo(960 + m.point, 6), closeTo(540, 1)], reason: 'from x -1 to x 1, level');
    final still = m.add('shape', 'circle');
    ws
      ..reread()
      ..select('$still');
    await t.pumpAndSettle();
    await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 300)));
    await t.pump();
    expect(trail(), isNull, reason: 'one that stays put has none');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('a star\'s own handles on the canvas: its corners rounded by one pulled in, its points more by one pulled out (Alight Motion\'s)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final star = m.add('shape', 'star');
    expect(m.set(star, 'size', .5), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$star');
    await t.pumpAndSettle();
    final stage = find.byType(StageView), origin = t.getTopLeft(stage);
    Map<String, Offset> h() => (t.state(stage) as StageHandles).handles;
    expect(h().keys, containsAll(['round', 'points']), reason: 'a star has both');
    await t.timedDragFrom(origin + h()['round']!, const Offset(30, 30), const Duration(milliseconds: 300));
    await settled(t, ws);
    expect(m.value(star, 'round', 0) ?? 0, greaterThan(.02), reason: 'rounder, pulled in: ${m.trouble}');
    final was = (m.value(star, 'points', 0) ?? 5).round();
    await t.timedDragFrom(origin + h()['points']!, const Offset(60, 0), const Duration(milliseconds: 300));
    await settled(t, ws);
    expect((m.value(star, 'points', 0) ?? 5).round(), greaterThan(was), reason: 'more points, pulled out: ${m.trouble}');
    expect(m.undo() && (m.value(star, 'points', 0) ?? 5).round() == was, isTrue, reason: 'one step back');
    final rect = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$rect');
    await t.pumpAndSettle();
    expect([h().containsKey('round'), h().containsKey('points')], [true, false], reason: 'a rect has its roundness, no points');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('a photo\'s crop bars on the canvas: its left side pulled in cuts it, what is left staying (Alight Motion\'s crop)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final photo = m.add('picture', '$motoliiDir/fx_sample.jpg');
    expect(m.set(photo, 'size', .5), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$photo');
    await t.pumpAndSettle();
    final stage = find.byType(StageView), origin = t.getTopLeft(stage);
    Map<String, Offset> h() => (t.state(stage) as StageHandles).handles;
    expect(h().keys, containsAll(['crop 0', 'crop 1', 'crop 2', 'crop 3']), reason: 'a bar on each side');
    final wide = (h()['corner 1']! - h()['corner 0']!).distance;
    await t.timedDragFrom(origin + h()['crop 0']!, Offset(wide * .3, 0), const Duration(milliseconds: 300));
    await settled(t, ws);
    expect(m.value(photo, 'crop_left', 0) ?? 0, closeTo(.3, .05), reason: 'cut in a third from the left: ${m.trouble}');
    expect(h()['crop 0']!.dx, closeTo(h()['corner 0']!.dx + wide * .3, 4), reason: 'its bar where it is cut');
    expect(m.undo() && (m.value(photo, 'crop_left', 0) ?? 0) == 0, isTrue, reason: 'one step back');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('a handle in hand alone is shown, larger; the others come back when it is let go (Alight Motion\'s handles)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final rect = m.add('shape', 'rect');
    expect(m.set(rect, 'size', .5), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$rect');
    await t.pumpAndSettle();
    final stage = t.getRect(find.byKey(const ValueKey('ws-stage-comp'))), k = 1920 / stage.width;
    final f = m.frame(rect)!, br = stage.topLeft + Offset(f[0] + f[2] / 2, f[1] + f[3] / 2) / k, tl = stage.topLeft + Offset(f[0] - f[2] / 2, f[1] - f[3] / 2) / k;
    Future<int> lit(Offset at) async {   // (the accent's pixels within a few points of it, as the window is drawn now)
      final root = t.binding.renderViews.first.layer! as OffsetLayer, ratio = t.view.devicePixelRatio, size = t.view.physicalSize;
      final bytes = (await t.runAsync(() async => (await root.toImage(Offset.zero & (size / ratio), pixelRatio: ratio)).toByteData(format: ui.ImageByteFormat.rawRgba)))!;
      final w = size.width.round(), a = WsT.accent;
      var n = 0;
      for (var y = ((at.dy - 7) * ratio).round(); y <= ((at.dy + 7) * ratio).round(); y++) {
        for (var x = ((at.dx - 7) * ratio).round(); x <= ((at.dx + 7) * ratio).round(); x++) {
          final i = 4 * (y * w + x);
          if ((bytes.getUint8(i) - (a.r * 255).round()).abs() < 12 && (bytes.getUint8(i + 1) - (a.g * 255).round()).abs() < 12 && (bytes.getUint8(i + 2) - (a.b * 255).round()).abs() < 12) n++;
        }
      }
      return n;
    }
    final before = await lit(tl);
    final g = await t.startGesture(br, kind: PointerDeviceKind.mouse);
    for (var i = 0; i < 3; i++) {   // (past the slop: carried, as a pull is)
      await g.moveBy(const Offset(8, 0));
      await t.pump();
    }
    final held = await lit(tl);
    await g.up();
    await settled(t, ws);
    final after = await lit(tl);
    expect([held < before - 15, after > before - 5], [true, true], reason: 'the far corner\'s handle gone while another is held (its frame\'s lines stay), back after: $before, $held, $after');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('a corner handle pulled sizes the thing with the side across staying; with ⌥ held, from its middle (Figma\'s and Keynote\'s handles; Alight Motion sizes from the middle only)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final rect = m.add('shape', 'rect');
    expect(m.set(rect, 'size', .5), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$rect');
    await t.pumpAndSettle();
    final stage = t.getRect(find.byKey(const ValueKey('ws-stage-comp'))), k = 1920 / stage.width;
    Future<void> pull({bool alt = false}) async {
      final f = m.frame(rect)!, corner = stage.topLeft + Offset(f[0] + f[2] / 2, f[1] + f[3] / 2) / k;   // (its bottom right corner, where its handle is)
      if (alt) await t.sendKeyDownEvent(LogicalKeyboardKey.altLeft);
      await t.timedDragFrom(corner, const Offset(40, 0), const Duration(milliseconds: 300));
      if (alt) await t.sendKeyUpEvent(LogicalKeyboardKey.altLeft);
      await settled(t, ws);
    }
    final before = m.frame(rect)!;
    await pull();
    final after = m.frame(rect)!;
    expect([after[2], after[0] - after[2] / 2], [greaterThan(before[2] + 20), closeTo(before[0] - before[2] / 2, 1)], reason: 'wider, its left side where it was: ${m.trouble}');
    await pull(alt: true);
    final mid = m.frame(rect)!;
    expect([mid[2], mid[0]], [greaterThan(after[2] + 20), closeTo(after[0], 1)], reason: 'wider again, its middle where it was');
    expect(m.set(rect, 'rotate', -30), isTrue);   // (turned: the corner across still stays)
    ws.reread();
    await t.pumpAndSettle();
    Offset at(List<double> f, double sx, double sy) {   // (a corner of its frame on the picture: its middle and half its size, turned as it is)
      final a = f[4] * math.pi / 180, x = sx * f[2] / 2, y = sy * f[3] / 2;
      return Offset(f[0] + x * math.cos(a) - y * math.sin(a), f[1] + x * math.sin(a) + y * math.cos(a));
    }
    final turned = m.frame(rect)!, across = at(turned, -1, -1), grip = stage.topLeft + at(turned, 1, 1) / k;
    await t.timedDragFrom(grip, const Offset(30, 30), const Duration(milliseconds: 300));
    await settled(t, ws);
    final now = m.frame(rect)!;
    expect([now[2] > turned[2] + 10, (at(now, -1, -1) - across).distance], [isTrue, lessThan(1.5)], reason: 'turned, larger, the corner across where it was');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('while a hand carries a thing, where it is is said by the hand, and while it sizes one, how wide and high (Figma\'s numbers by the hand)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final rect = m.add('shape', 'rect');
    expect(m.set(rect, 'size', .5), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$rect');
    await t.pumpAndSettle();
    final stage = t.getRect(find.byKey(const ValueKey('ws-stage-comp'))), k = 1920 / stage.width;
    String? says() => find.byKey(const ValueKey('ws-held-says')).evaluate().isEmpty ? null : (find.descendant(of: find.byKey(const ValueKey('ws-held-says')), matching: find.byType(Text)).evaluate().single.widget as Text).data;
    Future<String?> carry(Offset from, Offset by) async {
      final g = await t.startGesture(from, kind: PointerDeviceKind.mouse), seen = <String>{};
      for (var i = 0; i < 6; i++) {
        await g.moveBy(by / 6);
        for (var j = 0; j < 8; j++) {   // (the inside shows what is carried in its next picture)
          await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 25)));
          await t.pump();
        }
        if (says() case final s?) seen.add(s);
      }
      final said = says();
      expect(seen.length, greaterThan(1), reason: 'the numbers follow the hand: $seen');
      await g.up();
      await settled(t, ws);
      return said;
    }
    final f = m.frame(rect)!;
    expect(await carry(stage.topLeft + Offset(f[0], f[1]) / k, const Offset(60, 0)), matches(RegExp(r'^\d+, \d+$')), reason: 'where it is, while carried');
    expect(says(), isNull, reason: 'gone once let go');
    final g = m.frame(rect)!;
    expect(await carry(stage.topLeft + Offset(g[0] + g[2] / 2, g[1] + g[3] / 2) / k, const Offset(40, 0)), matches(RegExp(r'^\d+ × \d+$')), reason: 'how wide and high, while sized');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('a thing that goes with another, picked: the other\'s frame is drawn grey on the stage (Alight Motion\'s parent outline)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final mother = m.add('shape', 'rect'), child = m.add('shape', 'circle');
    expect(m.set(mother, 'x', -1) && m.set(child, 'parent', mother.toDouble()), isTrue, reason: m.trouble);
    String? drawn() => [for (final e in find.byWidgetPredicate((w) => w is CustomPaint && w.painter?.runtimeType.toString() == '_ViewPainter').evaluate()) (((e.widget as CustomPaint).painter as dynamic).parent as String?)].nonNulls.firstOrNull;
    ws
      ..reread()
      ..select('$child');
    await t.pumpAndSettle();
    expect(drawn(), '$mother', reason: 'its parent\'s frame');
    ws.select('$mother');
    await t.pumpAndSettle();
    expect(drawn(), isNull, reason: 'one with no parent: none');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('File › Package Project gathers the document and its media in a folder beside it (Alight Motion\'s project package)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final media = Directory.systemTemp.createTempSync('media');
    File('$motoliiDir/fx_sample.jpg').copySync('${media.path}/photo.jpg');
    expect(m.add('picture', '${media.path}/photo.jpg'), isNot(0), reason: m.trouble);
    ws.reread();
    await t.pumpAndSettle();
    final file = t.widget<PlatformMenuBar>(find.byType(PlatformMenuBar)).menus.whereType<PlatformMenu>().firstWhere((e) => e.label == 'File');
    file.menus.expand((e) => e is PlatformMenuItemGroup ? e.members : [e]).firstWhere((e) => e.label == 'Package Project').onSelected!();
    await t.pumpAndSettle();
    final name = ws.document!.replaceFirst(RegExp(r'\.motolii$'), ''), folder = Directory('${ws.documentFolder}/$name package');
    expect([File('${folder.path}/$name.motolii').existsSync(), File('${folder.path}/photo.jpg').existsSync(), ws.notice], [true, true, 'Packaged with 1 file: $name package'], reason: m.trouble);
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('⌘N: a new document, and its shape and resolution asked first (Alight Motion\'s new-project box)', (t) async {
    final ws = await _pump(t), first = ws.inside;
    await t.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
    await t.sendKeyEvent(LogicalKeyboardKey.keyN);
    await t.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
    for (var i = 0; i < 100 && (identical(ws.inside, first) || find.byKey(const ValueKey('ws-size-box')).evaluate().isEmpty); i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 20)));
      await t.pump();
    }
    expect([ws.document, find.byKey(const ValueKey('ws-size-box')).evaluate().length], [startsWith('untitled_'), 1], reason: 'a new one, its box open');
    final fresh = ws.inside;
    await t.tap(find.byKey(const ValueKey('size-1:1')));
    for (var i = 0; i < 100 && identical(ws.inside, fresh); i++) {   // (opened again at it)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 20)));
      await t.pump();
    }
    await settled(t, ws);
    expect(ws.inside!.size, (1080, 1080), reason: 'square, as asked');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('a card\'s Copy and Paste give its values alone to another thing (Alight Motion\'s copy of one effect)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final a = m.add('shape', 'rect'), b = m.add('shape', 'circle');
    expect(m.set(a, 'wave_x', .3) && m.set(a, 'wave_speed', 2) && m.say(a, 'fill', 'ff3366') && m.say(b, 'fill', '3a86ff'), isTrue, reason: m.trouble);
    Future<void> press(String what) async {
      final k = find.byKey(ValueKey('card-$what:Wave'));
      await seen(t, k);
      await t.tap(k);
      await settled(t, ws);
    }
    ws
      ..reread()
      ..select('$a');
    await t.pumpAndSettle();
    await press('copy');
    ws.select('$b');
    await t.pumpAndSettle();
    await press('paste');
    expect([m.value(b, 'wave_x', 0), m.value(b, 'wave_speed', 0), m.word(b, 'fill')], [closeTo(.3, 1e-9), 2, '3a86ff'], reason: 'its wave, and nothing else of the rect: ${m.trouble}');
    expect(m.undo() && (m.value(b, 'wave_x', 0) ?? 0) == 0, isTrue, reason: 'one step');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('⌘G holds the picked together by a Null in their middle: moved, they go with it; ⌘⇧G lets them go where they are (Figma\'s group)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final a = m.add('shape', 'rect'), b = m.add('shape', 'circle');
    expect(m.set(a, 'x', -1) && m.set(b, 'x', 1) && m.set(b, 'y', .4), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..pickMany(['$a', '$b']);
    await t.pumpAndSettle();
    Future<void> press(LogicalKeyboardKey k, {bool shift = false}) async {
      await t.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
      if (shift) await t.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
      await t.sendKeyEvent(k);
      if (shift) await t.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
      await t.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
      await settled(t, ws);
    }
    await press(LogicalKeyboardKey.keyG);
    final g = m.things.firstWhere((x) => x != a && x != b);
    expect([m.word(g, 'shape'), m.word(g, 'name'), m.value(a, 'parent', 0), m.value(b, 'parent', 0), ws.selected], ['null', 'Group', g, g, '$g'], reason: m.trouble);
    expect([m.value(g, 'x', 0)!, m.value(g, 'y', 0)!], [closeTo(0, .01), closeTo(.2, .01)], reason: 'in their middle');
    await press(LogicalKeyboardKey.keyC);
    await press(LogicalKeyboardKey.keyV);
    final pasted = int.parse(ws.selected!), held = m.things.where((x) => m.value(x, 'parent', 0) == pasted);
    expect([pasted != g, m.word(pasted, 'shape'), [for (final x in held) m.word(x, 'shape')]], [true, 'null', ['rect', 'circle']], reason: 'copied with what it holds, as Alight Motion copies a group: ${m.trouble}');
    await press(LogicalKeyboardKey.keyZ);
    expect([m.things.contains(pasted), m.things.length], [false, 3], reason: 'one step back');
    final fa = m.frame(a)!;
    expect(m.set(g, 'x', .5), isTrue);
    expect(m.frame(a)![0], closeTo(fa[0] + .5 * m.point, 1), reason: 'the group moved: they go with it');
    final moved = [m.frame(a)!.take(2).toList(), m.frame(b)!.take(2).toList()];
    ws
      ..reread()
      ..select('$g');
    await t.pumpAndSettle();
    await press(LogicalKeyboardKey.keyG, shift: true);
    expect([m.things.contains(g), m.value(a, 'parent', 0), m.value(b, 'parent', 0)], [false, 0, 0], reason: 'let go: ${m.trouble}');
    for (final (i, x) in [a, b].indexed) {
      expect(m.frame(x)![0], closeTo(moved[i][0], 1), reason: 'where it was');
      expect(m.frame(x)![1], closeTo(moved[i][1], 1));
    }
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('a layer\'s label colour chosen on the Note card: its row wears it, auto gives it back its own (Ableton Live\'s clip colours, Alight Motion\'s label colour)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final rect = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$rect');
    await t.pumpAndSettle();
    final own = ws.layers.firstWhere((l) => l.id == '$rect').tone;
    await seen(t, find.byWidgetPredicate((w) => w is Chooser && w.id == 'v_label'));   // (the Note card, the last)
    await pick(t, 'v_label', 'red');
    expect(m.word(rect, 'label'), 'red', reason: m.trouble);
    expect(ws.layers.firstWhere((l) => l.id == '$rect').tone, const Color(0xFFFF3636), reason: 'its row in Live\'s red');
    await pick(t, 'v_label', 'auto');
    expect(ws.layers.firstWhere((l) => l.id == '$rect').tone, own, reason: 'its own again');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('Arrange: lined up on the picture, against their own box, spread evenly; made to fit the picture; turned over (Keynote\'s Align, Alight Motion\'s Fit and Flip)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final a = m.add('shape', 'rect'), b = m.add('shape', 'rect'), c = m.add('shape', 'rect');
    expect(m.set(a, 'x', -1) && m.set(a, 'y', .3) && m.set(b, 'x', .2) && m.set(b, 'y', -.2) && m.set(c, 'x', 1.4) && m.set(a, 'size', .3) && m.set(b, 'size', .3) && m.set(c, 'size', .3), isTrue, reason: m.trouble);
    ws.reread();
    await t.pumpAndSettle();
    Future<void> tell(String call, {int thing = 0, String word = ''}) async {
      ws.after(m.later(call, thing: thing, word: word));
      await settled(t, ws);
    }
    await tell('align', word: 'top $a $b $c');
    expect([m.value(a, 'y', 0), m.value(b, 'y', 0), m.value(c, 'y', 0)].map((v) => (v ?? 0).toStringAsFixed(6)).toSet(), hasLength(1), reason: 'the tops in a line: ${m.trouble}');
    await tell('align', word: 'across $b $a $c');
    final xs = [m.value(a, 'x', 0)!, m.value(b, 'x', 0)!, m.value(c, 'x', 0)!]..sort();
    expect(xs[1] - xs[0], closeTo(xs[2] - xs[1], 1e-6), reason: 'the gaps even: $xs');
    await tell('align', word: 'centre $a');
    expect(m.value(a, 'x', 0), closeTo(0, 1e-6), reason: 'one alone: on the picture');
    await tell('fit', thing: b, word: 'fit');
    expect([m.value(b, 'x', 0), m.value(b, 'y', 0)], [closeTo(0, 1e-6), closeTo(0, 1e-6)], reason: 'its middle at the picture\'s');
    expect(m.value(b, 'size', 0)!, greaterThan(.3), reason: 'as large as fits');
    await tell('flip', thing: c, word: 'x');
    expect(m.value(c, 'scale_x', 0), -1, reason: 'turned over left to right');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);

  testWidgets('a Parent card while it has a parent: how much of its turn and size it takes, and a wheel\'s roll (Alight Motion\'s Parenting Helper)', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    final a = m.add('shape', 'rect'), b = m.add('shape', 'rect');
    ws.reread();
    await t.pumpAndSettle();
    expect(m.cards(b).map((c) => c.$1), isNot(contains('Parent')), reason: 'no parent: nothing to take of it');
    expect(m.set(b, 'parent', a.toDouble()), isTrue, reason: m.trouble);
    expect([for (final c in m.cards(b)) if (c.$1 == 'Parent') ...c.$2], ['parent_rotate', 'parent_size', 'roll', 'roll_radius']);
    expect([m.value(b, 'parent_rotate', 0) ?? 1, m.value(b, 'parent_size', 0) ?? 1], [1, 1], reason: 'all of its parent\'s, as it comes');
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
    await settled(t, ws);   // (told the inside without waiting: read once it has it)
    expect(m.word(rect, 'cap'), 'round', reason: m.trouble);
    expect(m.undo() && m.word(rect, 'cap').isEmpty, isTrue, reason: 'one step');
    final cube = m.add('solid', 'cube');
    ws
      ..reread()
      ..select('$cube');
    await t.pumpAndSettle();
    expect(find.descendant(of: find.byType(ValuesCard), matching: find.text('Surface')), findsOneWidget, reason: 'Filament\'s own surface of a solid');
    ws.shelf = 'Create';
    await t.pumpAndSettle();
    await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), 'Glass');
    await t.pumpAndSettle();
    final glass = find.descendant(of: find.byType(CreateShelf), matching: find.text('Glass')).first;
    await seen(t, glass);
    await doubleTap(t, glass);   // (a material is made in Create and put on the picked solid, as Repeater lays the picked thing out)
    await settled(t, ws);
    expect(m.word(cube, 'fill'), 'glass', reason: m.trouble);
    expect(m.cards(cube).map((c) => c.$1), ['Fill', 'Transform', 'Blending', 'Motion', 'Play', 'Glass', 'Lit', 'Physics', 'Note'], reason: 'filled with glass: the glass\'s own card, not the surface\'s');
    expect(m.cards(rect).map((c) => c.$1).contains('Matte'), isFalse, reason: 'not clipped: no matte to say how');
    expect(m.set(rect, 'clip', 1) && m.cards(rect).map((c) => c.$1).contains('Matte'), isTrue);
    expect(m.set(cube, 'ior', 2.2) && m.value(cube, 'ior', 0) == 2.2, isTrue, reason: m.trouble);
    expect(m.say(cube, 'fill', 'ff3366') && m.cards(cube).map((c) => c.$1).join(' ') == 'Fill Transform Blending Motion Play Surface Lit Physics Note', isTrue);
    ws.reread();   // (written to the inside directly: the window reads it again)
    await t.pumpAndSettle();
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
    expect(find.text('Background of the stage'), findsOneWidget, reason: 'the shelf says what it colours');
    await t.tap(find.byWidgetPredicate((w) => w.key is ValueKey<String> && (w.key as ValueKey<String>).value.startsWith('colour-box-')).first);   // (a press colours at once: Figma's, Canva's, Alight Motion's swatches)
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
    expect(m.cards(fox).firstWhere((c) => c.$1 == 'Play').$2, ['animation', 'speed', 'speed_hold', 'frame_rate']);
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
  testWidgets('⌘O lists the documents beside it as CapCut\'s desktop home lays them, told as Alight Motion\'s are: its picture, length, shape, resolution, size and rate, by day or by name; a copy made, the copy opened; File › Use Frame as Project Picture', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    expect(m.add('picture', '$motoliiDir/fx_sample.jpg'), isNot(0), reason: m.trouble);
    ws.reread();
    await t.pumpAndSettle();
    final file = t.widget<PlatformMenuBar>(find.byType(PlatformMenuBar)).menus.whereType<PlatformMenu>().firstWhere((e) => e.label == 'File');
    ws.frame = 60;
    file.menus.expand((e) => e is PlatformMenuItemGroup ? e.members : [e]).firstWhere((e) => e.label == 'Use Frame as Project Picture').onSelected!();
    expect(m.value(0, 'poster_at', 0), closeTo(1, 1e-6), reason: 'the moment its picture is drawn at');
    await t.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
    await t.sendKeyEvent(LogicalKeyboardKey.keyO);
    await t.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
    await t.pump();
    expect(find.byKey(const ValueKey('ws-home')), findsOneWidget);
    final doc = '${ws.documentFolder}/${ws.document}', facts = find.byKey(ValueKey('home-facts:$doc'));
    for (var i = 0; i < 100 && !(t.widget<Text>(facts).data ?? '').contains('fps'); i++) {   // (read on another isolate)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 20)));
      await t.pump();
    }
    expect(t.widget<Text>(facts).data, allOf(contains('16:9'), contains('1080p'), contains('60fps')), reason: 'as the document kept it as the list was opened');
    expect(find.textContaining(RegExp(r'B \| 00:08$')), findsOneWidget, reason: 'its size and length under its name, as CapCut\'s "254.3M | 02:13"');
    await t.tap(find.byKey(ValueKey('home-more:$doc')));
    await t.pump();
    await t.tap(find.byKey(ValueKey('home-duplicate:$doc')));
    await t.pump();
    final name = ws.document!.replaceFirst(RegExp(r'\.motolii$'), ''), copy = '${ws.documentFolder}/$name Copy.motolii';
    expect(File(copy).existsSync(), isTrue, reason: 'AbsentMotion\'s "Name Copy"');
    await t.tap(find.byKey(const ValueKey('home-by-name')));
    await t.pump();
    final at = t.getTopLeft(find.byKey(ValueKey('home-doc:$doc'))), copyAt = t.getTopLeft(find.byKey(ValueKey('home-doc:$copy')));
    expect(at.dy < copyAt.dy || (at.dy == copyAt.dy && at.dx < copyAt.dx), isTrue, reason: 'by name: it before its copy, in the grid\'s reading order');
    final first = ws.inside;
    await t.tap(find.byKey(ValueKey('home-doc:$copy')));
    for (var i = 0; i < 100 && identical(ws.inside, first); i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 20)));
      await t.pump();
    }
    await settled(t, ws);
    expect([ws.document, find.byKey(const ValueKey('ws-home')).evaluate().length], ['$name Copy.motolii', 0], reason: 'the copy opened, the list gone');
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);
  testWidgets('from the list: ••• › Export… opens it and asks its export settings (AbsentMotion\'s exportFromList); two picked, by ⌘ and by Select, are written one after the other beside themselves at their own settings (DaVinci Resolve\'s Render Queue), and the open one is opened again', (t) async {
    final ws = await _pump(t), m = ws.inside!;
    expect(m.add('shape', 'rect'), isNot(0), reason: m.trouble);
    m.set(0, 'render_height', 360);
    m.set(0, 'render_rate', 30);
    final folder = ws.documentFolder!, doc = '$folder/${ws.document}', other = '$folder/other.motolii';
    File(doc).copySync(other);   // (beside it, with its settings)
    Future<void> home() async {
      await t.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
      await t.sendKeyEvent(LogicalKeyboardKey.keyO);
      await t.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
      await t.pump();
    }

    await home();
    await t.tap(find.byKey(ValueKey('home-more:$other')));
    await t.pump();
    await t.tap(find.byKey(ValueKey('home-export:$other')));
    for (var i = 0; i < 100 && ws.document != 'other.motolii'; i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 20)));
      await t.pump();
    }
    await settled(t, ws);
    expect(find.byKey(const ValueKey('render-MP4')), findsOneWidget, reason: 'its export settings asked, as Alight Motion\'s export screen');
    await t.tap(find.text('Cancel').last);
    await t.pump();

    await home();
    await t.tap(find.byKey(ValueKey('home-more:$other')));
    await t.pump();
    await t.tap(find.byKey(ValueKey('home-select:$other')));
    await t.pump();
    await t.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
    await t.tap(find.byKey(ValueKey('home-doc:$doc')));
    await t.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
    await t.pump();
    expect([find.byKey(ValueKey('home-picked:$doc')), find.byKey(ValueKey('home-picked:$other'))].map((f) => f.evaluate().length), [1, 1]);
    expect(ws.document, 'other.motolii', reason: 'a press picks while any is picked, it does not open');
    await t.tap(find.byKey(const ValueKey('home-export')));
    final films = [for (final d in [doc, other]) File('${d.substring(0, d.lastIndexOf('.'))}.mp4')];
    for (var i = 0; i < 9000 && !(ws.notice ?? '').startsWith('Written: 2 of 2'); i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 20)));
      await t.pump();
    }
    await settled(t, ws);
    expect(ws.notice, startsWith('Written: 2 of 2'), reason: ws.inside?.trouble);
    expect(films.map((f) => f.existsSync() && f.lengthSync() > 1000), [true, true], reason: 'each beside its document');
    expect(ws.document, 'other.motolii', reason: 'the one open before is open again');
    for (final f in films) {
      final p = Process.runSync('ffprobe', ['-v', 'error', '-select_streams', 'v:0', '-show_entries', 'stream=height,r_frame_rate', '-of', 'csv=p=0', f.path]);
      expect((p.stdout as String).trim(), '360,30/1', reason: 'at its own settings');
      f.deleteSync();
    }
    expect(t.takeException(), isNull);
    await _close(t);
  }, skip: !_built);
}
