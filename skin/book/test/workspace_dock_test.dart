// The shelves on a real document (the inside joined, test/door.dart): each lays out at the dock's sizes; Media is one
// browser, its tree beside the contents when wide and above them when narrow, its search over where it is.
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/gestures.dart' show PointerDeviceKind, kDoubleTapTimeout;
import 'package:flutter/services.dart' show LogicalKeyboardKey;
import 'package:flutter/widgets.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/workspace/dock.dart';
import 'package:lab_book/workspace/create_preview.dart';
import 'package:lab_book/workspace/dock_parts.dart' show DockIconButton, DockSearch, dockStarsAt;
import 'package:lab_book/workspace/dock_tags.dart' show DockTags;
import 'package:lab_book/workspace/effect_tiles.dart';

import 'door.dart';

const _shelves = ['Media', 'Effects', 'Fonts', 'Colors', 'Create', 'Program'];

/// A window's state joined to a new document in a folder of its own, with a picture beside it.
Ws _joined() {
  final dir = Directory.systemTemp.createTempSync('dock');
  File('$motoliiDir/fx_sample.jpg').copySync('${dir.path}/still.jpg');   // (a picture in git: a clean clone has it)
  final m = Motolii.open('${dir.path}/dock.motolii')!;
  addTearDown(m.close);
  return Ws()
    ..join(m)
    ..documentFolder = dir.path;
}

Future<void> _shelf(WidgetTester t, Ws ws, Size size, String shelf) async {
  t.view.physicalSize = size;
  t.view.devicePixelRatio = 1;
  addTearDown(t.view.reset);
  await t.binding.setSurfaceSize(size);   // (the live binding lays out at its surface's size: flutter_test_config.dart)
  await t.pumpWidget(
    WidgetsApp(
      color: const Color(0xFF000000),
      builder: (_, _) => WsScope(
        ws: ws,
        child: Overlay(   // (a tile is carried to the stage on an overlay, as the window gives it: Draggable's own need)
          key: ValueKey('$shelf $size'),   // (a new one for each: an overlay keeps the entries it began with)
          initialEntries: [OverlayEntry(builder: (_) => Align(alignment: Alignment.topLeft, child: SizedBox(width: size.width, height: size.height, child: WsShelf(shelf, key: ValueKey(shelf)))))],
        ),
      ),
    ),
  );
  await t.pumpAndSettle();
}

void main() {
  test('an AbsentMotion effect\'s tile is drawn with the knobs its own table shows it with (tools/effect_showcase.json), split before and after where it says', () {
    final table = EffectTiles.table(motoliiDir);
    of(String name) {
      final f = '$motoliiDir/effects/am_$name.wgsl';
      return EffectTiles.showcase(f, File(f).readAsStringSync(), table);
    }
    expect(of('rays')?.knobs, {'js_am_rays_R_center_x': .5, 'js_am_rays_R_center_y': .5, 'js_am_rays_reach': .6, 'js_am_rays_threshold': .5}, reason: 'center: skslc\'s R_ before it, a vector one knob each way');
    expect([of('duotone')?.knobs['js_am_duotone_color1_x'], of('duotone')?.knobs['js_am_duotone_color1_w']], [0x2b / 255, 1.0], reason: 'a colour #2b1055: one knob each way, its alpha 1');
    expect(of('huesaturation')?.split, .4, reason: 'before at its left, up to 0.4');
    expect(EffectTiles.showcase('$motoliiDir/effects/God Rays.fs', '', table), isNull, reason: 'VIDVOX\'s: not in the table');
    final all = [for (final n in (table['effects'] as Map).keys) if (File('$motoliiDir/effects/am_$n.wgsl').existsSync()) of('$n')];
    expect([all.length > 100, all.every((s) => s != null)], [true, true], reason: 'every one of the table\'s with a file of its own read as it is');
    expect(of('chromakey')?.sweeps['js_am_chromakey_tolerance'], [0, 0, .2, .2, 0], reason: 'a moving line\'s list of numbers for one knob: its keys');
    expect(of('stretchaxis')?.moves, isTrue, reason: 'its "move" layout: seen on something going past');
  });
  testWidgets('the Effects shelf has a reload mark: lit, and saying how many are drawn, while the tiles are', (t) async {
    await _shelf(t, _joined(), const Size(1280, 800), 'Effects');
    expect(find.byKey(const ValueKey('ws-fx-reload')), findsOneWidget);
    EffectTiles.progress.value = (24, 1184);
    await t.pump();
    expect(find.text('24 of 1184'), findsOneWidget);
    expect(t.widget<DockIconButton>(find.byKey(const ValueKey('ws-fx-reload'))).on, isTrue);
    EffectTiles.progress.value = null;
    await t.pump();
    expect(find.textContaining(' ready'), findsOneWidget);
  }, skip: !insideBuilt);
  test('a tile shows what its effect does, told by the effect: the knob that stands for it goes, one seen on nothing but an edge is laid on a card going past, with no knob the line between before and after goes across, one that does nothing says so; only that keeps no pictures', () {
    final out = Directory.systemTemp.createTempSync('motolii_tile_test');
    final effects = ['Bright.fs', 'am_longshadow.wgsl', '3d Rotate.fs', 'am_reorient360.wgsl', 'Color Invert.fs'];
    final drawn = EffectTiles.draw(motoliiDir, [for (final (i, e) in effects.indexed) ('$motoliiDir/effects/$e', '${out.path}/$i.png')]);
    expect(drawn.map((d) => d.$2), [false, false, true, false, false], reason: '3d Rotate\'s vertex shader is not run here: it changes nothing, and the tile says so');
    expect([for (final (i, _) in effects.indexed) File(EffectTiles.frame('${out.path}/$i.png', EffectTiles.moments.length - 1)).existsSync()], [true, true, false, true, true], reason: 'a line of AbsentMotion\'s table with no knob moved moves one all the same');
    for (final (i, why) in [(0, 'Bright has no clock: its knob moved, the sample stands'), (3, 'the hand set a value, and moved nothing'), (4, 'Color Invert has no knob: its line moved')]) {
      final first = File(EffectTiles.frame('${out.path}/$i.png', 0)).readAsBytesSync(), last = File(EffectTiles.frame('${out.path}/$i.png', EffectTiles.moments.length - 1)).readAsBytesSync();
      expect(first.length == last.length && first.indexed.every((b) => last[b.$1] == b.$2), isFalse, reason: why);
    }
    out.deleteSync(recursive: true);
  }, skip: !insideBuilt);

  testWidgets('every shelf lays out at the reference size and when smaller', (t) async {
    final ws = _joined();
    for (final size in const [Size(240, 918), Size(240, 600)]) {
      for (final s in _shelves) {
        await _shelf(t, ws, size, s);
        expect(t.takeException(), isNull, reason: '$s at $size');
        expect(find.text(s), findsWidgets);
      }
    }
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('Media is one browser: tree beside the contents when wide, above them when narrow; search covers the location', (t) async {
    final ws = _joined();
    await _shelf(t, ws, const Size(520, 900), 'Media');
    expect(find.byKey(const ValueKey('browser:tree-grip')), findsNothing, reason: 'wide: the tree stands beside the contents');
    await _shelf(t, ws, const Size(300, 900), 'Media');
    expect(find.byKey(const ValueKey('browser:tree-grip')), findsOneWidget, reason: 'narrow: the tree above, with a grip to size it');
    final folder = find.byKey(ValueKey('browser:file:${ws.documentFolder}')); // (the document's own folder, a tile of Project: opened)
    await t.ensureVisible(folder.first);
    await t.tap(folder.first);
    await t.pumpAndSettle();
    await t.enterText(find.byType(EditableText), 'still');
    for (var i = 0; i < 10 && find.text('still.jpg').evaluate().isEmpty; i++) { // (a folder of the disk is read without waiting: its files come a moment later)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
      await t.pumpAndSettle();
    }
    expect(find.text('still.jpg'), findsWidgets, reason: 'the picture beside the document, found by its name');
    await t.tap(find.byKey(ValueKey('browser:file:${ws.documentFolder}/still.jpg')).first);   // (picked: its foot says what it is)
    for (var i = 0; i < 600 && find.textContaining('857 × 857').evaluate().isEmpty; i++) {   // (GStreamer's discoverer, on another isolate; 5 seconds were too few for a CI Mac: golf (108))
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
      await t.pump();
    }
    expect(find.textContaining('857 × 857 · JPEG'), findsOneWidget, reason: 'its size and kind, as GStreamer\'s discoverer tells them (Alight Motion\'s Media Info)');
    expect(await mediaInfoOf(ws.inside!.libPath, '$motoliiDir/check/beats.mp4'), '320 × 180 · 30 fps · 4.0 s · Quicktime', reason: 'a film: its rate and length too');
    expect(t.takeException(), isNull);
    await t.pumpWidget(const SizedBox());
  }, retry: 2, skip: !insideBuilt);   // (a slow CI Mac only: golf_noise.md (108))

  testWidgets('a tile pressed is picked as the button goes down, so the shelf\'s button makes that one (Finder picks an icon at once and opens it on a second click; golf (77))', (t) async {
    final ws = _joined(), m = ws.inside!, before = m.things.length;
    await _shelf(t, ws, const Size(520, 918), 'Create');
    await t.tap(find.text('Layers').first);   // (its chip: the recipes come first)
    await t.pumpAndSettle();
    await t.tap(find.text('Camera').first);
    await t.pump(kDoubleTapTimeout + const Duration(milliseconds: 50));
    expect(find.text('Camera layer'), findsOneWidget);
    await t.tap(find.text('Null').first);
    await t.pump();   // (no wait for a second press)
    expect(find.text('Null layer'), findsOneWidget, reason: 'the button below is about the tile pressed last');
    await t.pump(kDoubleTapTimeout + const Duration(milliseconds: 50));
    expect(m.things.length, before, reason: 'one press makes nothing');
    expect(t.takeException(), isNull);
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('Alight Motion\'s fourteen samples under the colour on the Colors shelf: white in one press (golf (80))', (t) async {
    final ws = _joined(), m = ws.inside!, rect = m.add('shape', 'rect');
    ws
      ..reread()
      ..select('$rect');
    await _shelf(t, ws, const Size(520, 918), 'Colors');
    expect(find.byKey(const ValueKey('colour-sample-000000')), findsOneWidget, reason: 'black among them');
    await t.tap(find.byKey(const ValueKey('colour-sample-ffffff')));
    for (var i = 0; i < 200 && ws.waiting; i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 10)));
      await t.pump();
    }
    expect(m.word(rect, 'fill'), 'ffffff');
    expect(t.takeException(), isNull);
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('an effect\'s colour, three numbers to the inside, takes a sample of the Colors shelf as one colour when its card aims the shelf at it (golf (83))', (t) async {
    final ws = _joined(), m = ws.inside!, rect = m.add('shape', 'rect');
    expect(m.say(rect, 'fx', '$motoliiDir/effects/Fade Color.fs'), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$rect')
      ..aim = 'js_Fade_Color_color';   // (its card's colour link pressed: workspace_seat.dart onAim)
    await _shelf(t, ws, const Size(520, 918), 'Colors');
    await t.tap(find.byKey(const ValueKey('colour-sample-ffffff')));
    for (var i = 0; i < 200 && ws.waiting; i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 10)));
      await t.pump();
    }
    expect([for (final c in ['x', 'y', 'z']) m.value(rect, 'js_Fade_Color_color_$c', 0)], [1, 1, 1], reason: 'white: the three numbers');
    expect(t.takeException(), isNull);
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('a picture picked and another picture pressed on Media: Replace shows the new one, its keys as they were (After Effects\' Replace Layer Source; P17)', (t) async {
    final ws = _joined(), m = ws.inside!;
    File('${ws.documentFolder}/still.jpg').copySync('${ws.documentFolder}/final.jpg');
    final pic = m.add('picture', '${ws.documentFolder}/still.jpg');
    expect(pic > 0 && m.key(pic, 'x', .5, 0) && m.key(pic, 'x', 1.5, 1), isTrue, reason: m.trouble);
    ws
      ..reread()
      ..select('$pic');
    await _shelf(t, ws, const Size(520, 900), 'Media');
    final folder = find.byKey(ValueKey('browser:file:${ws.documentFolder}'));
    await t.ensureVisible(folder.first);
    await t.tap(folder.first);
    await t.pumpAndSettle();
    final tile = find.byKey(ValueKey('browser:file:${ws.documentFolder}/final.jpg'));
    for (var i = 0; i < 20 && tile.evaluate().isEmpty; i++) {   // (a folder of the disk is read without waiting)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
      await t.pumpAndSettle();
    }
    await t.tap(tile.first);
    await t.pump(kDoubleTapTimeout + const Duration(milliseconds: 50));   // (a tile pressed twice places it: one press is known once the second has not come)
    await t.pumpAndSettle();
    expect(ws.selected, '$pic', reason: 'the picture still picked');
    await t.tap(find.text('Replace'));
    for (var i = 0; i < 200 && ws.waiting; i++) {   // (asked without waiting)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 10)));
      await t.pump();
    }
    expect([m.word(pic, 'file'), m.keyMoments(pic, 'x').length], ['${ws.documentFolder}/final.jpg', 2], reason: 'the same thing, its file the new one, its keys kept');
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('a file\'s effect starred: a ★ table of the starred, and kept for the next time', (t) async {
    final kept = File('${Directory.systemTemp.createTempSync('stars').path}/effect_stars.json');
    dockStarsAt(kept);
    final ws = _joined();
    await _shelf(t, ws, const Size(600, 918), 'Effects');
    final tile = find.byWidgetPredicate((w) => w.key is ValueKey<String> && (w.key as ValueKey<String>).value.startsWith('ws-screen-')).first;
    final id = (t.widget(tile).key as ValueKey<String>).value.substring('ws-screen-'.length);
    expect(find.text('★'), findsNothing, reason: 'none starred: no table of them');
    final hand = await t.createGesture(kind: PointerDeviceKind.mouse);
    await hand.addPointer(location: t.getCenter(tile));
    await t.pumpAndSettle();
    await t.tap(find.byKey(ValueKey('ws-star-$id')));
    await t.pump(const Duration(milliseconds: 400));   // (the tile under it waits for a second press first: a double tap puts it on)
    await hand.removePointer();
    await t.pumpAndSettle();
    expect(kept.readAsStringSync(), contains(id), reason: 'kept for the next time');
    await t.tap(find.text('★'));
    await t.pumpAndSettle();
    expect(find.byWidgetPredicate((w) => w.key is ValueKey<String> && (w.key as ValueKey<String>).value.startsWith('ws-screen-')), findsOneWidget, reason: 'the starred alone');
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('a file\'s effect with nothing picked comes on an adjustment layer over all else, picked (After Effects, Photoshop\'s Adjustments panel)', (t) async {
    final ws = _joined();
    await _shelf(t, ws, const Size(600, 918), 'Effects');
    final m = ws.inside!;
    final was = m.things.length;
    final tile = find.byWidgetPredicate((w) => w.key is ValueKey<String> && (w.key as ValueKey<String>).value.startsWith('ws-screen-')).first;
    final id = (t.widget(tile).key as ValueKey<String>).value.substring('ws-screen-'.length);
    await t.tap(tile);
    await t.pump(kDoubleTapTimeout + const Duration(milliseconds: 50));
    await t.pumpAndSettle();
    expect(find.text('new layer'), findsOneWidget, reason: 'as the foot says');
    await t.tap(find.text('Add'));
    for (var i = 0; i < 200 && (ws.waiting || ws.selected == null); i++) {   // (asked without waiting)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 10)));
      await t.pump();
    }
    expect(m.things.length, was + 1);
    final board = int.parse(ws.selected!);
    expect([m.word(board, 'fill'), m.word(board, 'name'), m.word(board, 'fx')], ['behind', 'Adjustment', contains(id)]);
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('an effect pressed once shows its moving pictures large above its name (Alight Motion\'s large moving sample)', (t) async {
    final ws = _joined();
    await _shelf(t, ws, const Size(600, 918), 'Effects');
    final tile = find.byWidgetPredicate((w) => w.key is ValueKey<String> && (w.key as ValueKey<String>).value.startsWith('ws-screen-') && ws.inside!.picture((w.key as ValueKey<String>).value.substring('ws-screen-'.length)).isNotEmpty).first;   // (one with its pictures drawn)
    expect(find.byKey(const ValueKey('ws-fx-preview')), findsNothing, reason: 'none pressed: no sample');
    await t.tap(tile);
    await t.pump(const Duration(milliseconds: 400));   // (one press picks: the second would put it on)
    await t.pump(const Duration(milliseconds: 600));
    expect(find.byKey(const ValueKey('ws-fx-preview')), findsOneWidget);
    expect(t.getSize(find.byKey(const ValueKey('ws-fx-preview'))).height, greaterThan(t.getSize(tile).height), reason: 'larger than its tile');
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('a shape among the recipes (AbsentMotion\'s Heart) shows its own outline on its tile', (t) async {
    final ws = _joined();
    await _shelf(t, ws, const Size(240, 918), 'Create');
    await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), 'Heart');
    for (var i = 0; i < 1200 && RecipeMotions.of('Heart') == null; i++) {   // (the previews are made in an isolate of their own: waited for in real time)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
    }
    await t.pumpAndSettle();
    expect(RecipeMotions.of('Heart')?.single.outline, startsWith('M '), reason: 'its path, as the recipe says it');
    expect(find.byType(SvgPicture), findsOneWidget, reason: 'drawn on its tile');
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('the Effects shelf\'s search reads what an effect does, in its maker\'s words (Alight Motion\'s search of the descriptions)', (t) async {
    final ws = _joined(), m = ws.inside!;
    final checker = m.effects.firstWhere((e) => e.$2.endsWith('/am_checker.wgsl'));
    expect(m.about(checker.$2), startsWith('Draws a checkerboard'), reason: 'its head\'s --about:, from AbsentMotion\'s description');
    await _shelf(t, ws, const Size(240, 918), 'Effects');
    await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), 'checkerboard');
    await t.pumpAndSettle();
    expect(find.byKey(ValueKey('ws-screen-${checker.$2}')), findsOneWidget, reason: 'Checker found by a word of what it does, not of its name');
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('an effect put on with words typed is the first for those words the next time: Alfred\'s Keyword Latching (golf_noise.md (94))', (t) async {
    DockTags.latchedAt(File('${Directory.systemTemp.createTempSync('latch').path}/latched.json'));
    final ws = _joined();
    await _shelf(t, ws, const Size(240, 918), 'Effects');
    final search = find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText));
    List<String> shown() => [
      for (final e in find.byWidgetPredicate((w) => w.key is ValueKey<String> && (w.key! as ValueKey<String>).value.startsWith('ws-screen-')).evaluate())
        (e.widget.key! as ValueKey<String>).value.substring('ws-screen-'.length),
    ];
    await t.enterText(search, 'blur');
    await t.pumpAndSettle();
    final before = shown(), last = before[1];   // (the second: one the list does not already put first)
    expect(before.length, greaterThan(2), reason: 'several blurs, A to Z');
    final tile = find.byKey(ValueKey('ws-screen-$last'));
    await t.ensureVisible(tile);
    await t.pumpAndSettle();
    await doubleTap(t, tile);   // (two presses put it on)
    await t.pumpAndSettle();
    expect(DockTags.latched('Effects', 'blur'), last, reason: 'put on with "blur" typed: latched to it');
    await t.enterText(search, '');
    await t.pumpAndSettle();
    await t.enterText(search, 'Blur ');
    await t.pumpAndSettle();
    expect(shown().first, last, reason: 'the one put on for these words, first now (the words as typed, big letters and spaces aside)');
    expect(shown().toSet(), before.toSet(), reason: 'the rest as they were');
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('every shelf finds as the effects shelf does: a word typed wrong or in Japanese brings the nearest (golf_noise.md (97))', (t) async {
    final ws = _joined();
    Future<void> search(String shelf, String words) async {
      await _shelf(t, ws, const Size(520, 918), shelf);
      await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), words);
      await t.pumpAndSettle();
    }
    await search('Create', 'hart');
    expect(find.text('Heart'), findsWidgets, reason: 'Create: a word typed wrong');
    await search('Create', '矢印');
    expect(find.textContaining('Arrow'), findsWidgets, reason: 'Create: a Japanese word, arrow');
    await search('Fonts', 'arail black');
    expect(find.textContaining('Arial Black'), findsWidgets, reason: 'Fonts: a word typed wrong');
    expect(t.takeException(), isNull);
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('the chips move sideways as any sideways list on a Mac: the wheel with Shift held, not the wheel alone (golf_noise.md (93))', (t) async {
    final ws = _joined();
    await _shelf(t, ws, const Size(240, 918), 'Effects');
    final along = find.ancestor(of: find.text('All'), matching: find.byType(Scrollable)).first, at = t.getCenter(find.text('All'));
    expect(t.state<ScrollableState>(along).position.pixels, 0);
    final mouse = TestPointer(1, PointerDeviceKind.mouse);
    await t.sendEventToBinding(mouse.hover(at));
    await t.sendEventToBinding(mouse.scroll(const Offset(0, 100000)));
    await t.pump();
    expect(t.state<ScrollableState>(along).position.pixels, 0, reason: 'the wheel alone: up and down, not this row');
    await t.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
    await t.sendEventToBinding(mouse.scroll(const Offset(0, 100000)));
    await t.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
    await t.pump();
    final p = t.state<ScrollableState>(along).position;
    expect([p.pixels, p.maxScrollExtent > 0], [p.maxScrollExtent, true], reason: 'with Shift held: to the far end (Flutter\'s pointerAxisModifiers)');
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('the Effects shelf filters by what each effect is, read from its file, nobody asked to write it: a tag chip, or the tag typed (Shadertoy\'s filters; golf_noise.md (92))', (t) async {
    final ws = _joined(), m = ws.inside!, nameOf = {for (final (n, f) in m.effects) f: n};
    expect(m.tags(m.effects.firstWhere((e) => e.$1 == 'Waveform Displace').$2), {'sound'}, reason: 'an audio input read: ${m.tags('Waveform Displace')}');
    expect(m.tags(m.effects.firstWhere((e) => e.$1 == 'Sorting Smear').$2), {'feedback', 'time'}, reason: 'a PERSISTENT pass, the clock');
    expect(m.tags(m.effects.firstWhere((e) => e.$2.endsWith('/datamosh.wgsl')).$2), containsAll(['multipass', 'feedback', 'time']), reason: 'a WGSL effect read too: its --pass@ lines, one kept, the clock read');
    expect(m.tags(m.effects.firstWhere((e) => e.$2.endsWith('/am_brightnesscontrast.wgsl')).$2), isNot(contains('time')), reason: 'a clock field declared and not read is not the clock');
    if (m.effects.any((e) => e.$1 == 'NCSVisualizer')) expect(m.tags(m.effects.firstWhere((e) => e.$1 == 'NCSVisualizer').$2), containsAll(['generator', 'sound']), reason: 'an AviUtl2 custom object (.obj2) that reads obj.getaudio');   // (a maker's own script, not in the repository: CI has none)
    await _shelf(t, ws, const Size(240, 918), 'Effects');
    List<String> shown() => [
      for (final e in find.byWidgetPredicate((w) => w.key is ValueKey<String> && (w.key! as ValueKey<String>).value.startsWith('ws-screen-')).evaluate())
        nameOf[(e.widget.key! as ValueKey<String>).value.substring('ws-screen-'.length)]!,
    ];
    final chip = find.textContaining(RegExp(r'^#sound\b'));
    await seen(t, chip);
    await t.tap(chip.first);
    await t.pumpAndSettle();
    expect(shown(), containsAll(['Waveform Displace', 'FFT Spectrogram']), reason: 'the sound chip: the effects that hear');
    expect(shown().every((n) => m.tags(m.effects.firstWhere((e) => e.$1 == n).$2).contains('sound')), isTrue, reason: 'and none else: ${shown()}');
    expect(find.textContaining(RegExp(r'^#transition\b')), findsNothing, reason: 'Transition is a kind already, read the same way: not twice');
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('the Effects shelf lists the files\' effects by their names, one run of A to Z (After Effects\' Effects & Presets), not by file', (t) async {
    final ws = _joined(), m = ws.inside!, nameOf = {for (final (n, f) in m.effects) f: n};
    await _shelf(t, ws, const Size(240, 918), 'Effects');
    await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), 'glow');
    await t.pumpAndSettle();
    final names = [
      for (final e in find.byWidgetPredicate((w) => w.key is ValueKey<String> && (w.key! as ValueKey<String>).value.startsWith('ws-screen-')).evaluate())
        nameOf[(e.widget.key! as ValueKey<String>).value.substring('ws-screen-'.length)]!,
    ];
    expect(names, containsAll(['Edge Glow', 'Glow']), reason: 'AbsentMotion\'s am_edgeglow.wgsl and VIDVOX\'s Glow.fs');
    expect(names, [...names]..sort((a, b) => a.toLowerCase().compareTo(b.toLowerCase())), reason: 'by name: before, the capitals of VIDVOX\'s files came before all of AbsentMotion\'s am_');
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

  testWidgets('a sound is a sound: its volume on its card and no place, no effect, nothing to colour (Alight Motion\'s audio layer, reports/alight_ui.md 177)', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final wav = File('${ws.documentFolder}/tone.wav');
    final data = ByteData(44 + 8000)..setUint32(0, 0x52494646)..setUint32(4, 36 + 8000, Endian.little)..setUint32(8, 0x57415645)..setUint32(12, 0x666d7420)..setUint32(16, 16, Endian.little)..setUint16(20, 1, Endian.little)..setUint16(22, 1, Endian.little)..setUint32(24, 8000, Endian.little)..setUint32(28, 16000, Endian.little)..setUint16(32, 2, Endian.little)..setUint16(34, 16, Endian.little)..setUint32(36, 0x64617461)..setUint32(40, 8000, Endian.little);
    wav.writeAsBytesSync(data.buffer.asUint8List());
    final a = m.add('', wav.path);
    expect(a, isNot(0), reason: m.trouble);
    ws
      ..reread()
      ..select('$a')
      ..shelf = 'Colors';
    await t.pumpAndSettle();
    expect(ws.layer?.kind.name, 'audio');
    for (final (word, n) in [('Sound', 2), ('Transform', 0), ('Blending', 0), ('Fill of tone.wav', 0)]) {
      expect(find.text(word).evaluate().length, n, reason: word);
    }
    expect(find.textContaining('Add Effect'), findsNothing);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('a sound file picked on the Media shelf is heard before it is placed: pressed, the Mac plays it; pressed again, it stops (Alight Motion\'s song preview)', (t) async {
    final ws = _joined();
    final wav = File('${ws.documentFolder}/tone.wav');   // (half a second of silence, as a WAV file says it: 16-bit mono at 8000)
    final data = ByteData(44 + 8000)..setUint32(0, 0x52494646)..setUint32(4, 36 + 8000, Endian.little)..setUint32(8, 0x57415645)..setUint32(12, 0x666d7420)..setUint32(16, 16, Endian.little)..setUint16(20, 1, Endian.little)..setUint16(22, 1, Endian.little)..setUint32(24, 8000, Endian.little)..setUint32(28, 16000, Endian.little)..setUint16(32, 2, Endian.little)..setUint16(34, 16, Endian.little)..setUint32(36, 0x64617461)..setUint32(40, 8000, Endian.little);
    wav.writeAsBytesSync(data.buffer.asUint8List());
    await _shelf(t, ws, const Size(520, 900), 'Media');
    final folder = find.byKey(ValueKey('browser:file:${ws.documentFolder}'));
    await t.ensureVisible(folder.first);
    await t.tap(folder.first);
    await t.pumpAndSettle();
    final tile = find.byKey(ValueKey('browser:file:${wav.path}'));
    for (var i = 0; i < 20 && tile.evaluate().isEmpty; i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
      await t.pumpAndSettle();
    }
    await t.tap(tile.first);
    final listen = find.byKey(const ValueKey('media-listen'));
    for (var i = 0; i < 100 && listen.evaluate().isEmpty; i++) {   // (known to be sound once the discoverer has read it)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
      await t.pump();
    }
    await t.tap(listen);
    for (var i = 0; i < 40 && find.descendant(of: listen, matching: find.text('■')).evaluate().isEmpty; i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 25)));
      await t.pump();
    }
    expect(find.descendant(of: listen, matching: find.text('■')), findsOneWidget, reason: 'playing');
    await t.tap(listen);
    await t.pump();
    expect(find.descendant(of: listen, matching: find.text('▶')), findsOneWidget, reason: 'stopped');
    expect(find.textContaining('Audio'), findsWidgets, reason: 'shelved as Audio, not Video (Alight Motion\'s picker keeps songs apart)');
    final m = ws.inside!, before = m.things.length;
    await t.tap(find.text('Place'));
    await t.tap(find.text('Place'));   // (the inside still reading the song: the same press)
    for (var i = 0; i < 200 && m.things.length == before; i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 25)));
      await t.pump();
    }
    await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 300)));
    expect(m.things.length, before + 1, reason: 'one song');
    expect(m.word(m.things.last, 'kind'), 'film', reason: m.trouble);
    expect(m.value(m.things.last, 'visible', 0), 0, reason: 'placed heard and not seen, as Alight Motion\'s audio layer: no wave over the picture');
    expect(t.takeException(), isNull);
    await t.pumpWidget(const SizedBox());
  }, tags: 'main', skip: !insideBuilt);

}
