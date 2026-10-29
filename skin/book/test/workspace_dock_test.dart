// The shelves on a real document (the inside joined, test/door.dart): each lays out at the dock's sizes; Media is one
// browser, its tree beside the contents when wide and above them when narrow, its search over where it is.
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/gestures.dart' show PointerDeviceKind;
import 'package:flutter/widgets.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/workspace/dock.dart';
import 'package:lab_book/workspace/create_preview.dart';
import 'package:lab_book/workspace/dock_parts.dart' show DockSearch, dockStarsAt;

import 'door.dart';

const _shelves = ['Media', 'Effects', 'Fonts', 'Colors', 'Create'];

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
    for (var i = 0; i < 100 && find.textContaining('857 × 857').evaluate().isEmpty; i++) {   // (GStreamer's discoverer, on another isolate)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
      await t.pump();
    }
    expect(find.textContaining('857 × 857 · JPEG'), findsOneWidget, reason: 'its size and kind, as GStreamer\'s discoverer tells them (Alight Motion\'s Media Info)');
    expect(await mediaInfoOf(ws.inside!.libPath, '$motoliiDir/check/beats.mp4'), '320 × 180 · 30 fps · 4.0 s · Quicktime', reason: 'a film: its rate and length too');
    expect(t.takeException(), isNull);
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
    expect(t.takeException(), isNull);
    await t.pumpWidget(const SizedBox());
  }, skip: !insideBuilt);

}
