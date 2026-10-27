// The README's "put an effect from the net in": an ISF file Motolii does not bundle (ASCII Art, from Vidvox's ISF-Files,
// which filament/check/isf_all.sh fetches) put in the maker's own effects folder, seen on the Effects shelf at once and
// put on a photo; a picture of the shelf and of the stage after (docs/media/drop_step1..2.png). Put in with the window open: the shelf sees it, and its
// tile is drawn on the balloon by itself (lib/workspace/effect_tiles.dart).
//   flutter drive --profile -d macos --driver=test_driver/integration_test.dart --target=integration_test/drop_effect_test.dart
import 'dart:ffi' hide Size;
import 'dart:io';
import 'dart:ui' show ImageByteFormat;

import 'package:ffi/ffi.dart';
import 'package:flutter/rendering.dart' show OffsetLayer;
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:lab_book/motolii.dart' show motoliiDir;
import 'package:lab_book/theme.dart';
import 'package:lab_book/workspace/dock_parts.dart' show DockFooter, DockSearch;
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/workspace.dart' show Workspace;
import 'package:lab_book/workspace/ws.dart';
import 'package:lab_book/workspace/zoom.dart';

import '../test/door.dart' show seen, settled;

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  testWidgets('an effect from the net, put in the own folder, is on the shelf and on a photo', (t) async {
    t.view.physicalSize = const Size(3200, 1920);
    addTearDown(t.view.resetPhysicalSize);
    final from = File('$motoliiDir/third_party/ISF-Files/ISF/ASCII Art.fs');
    expect(from.existsSync(), isTrue, reason: 'fetch ISF-Files with filament/check/isf_all.sh');
    final own = Directory.systemTemp.createTempSync('own');
    Directory('${own.path}/effects').createSync();
    final setenv = DynamicLibrary.process().lookupFunction<Int32 Function(Pointer<Utf8>, Pointer<Utf8>, Int32), int Function(Pointer<Utf8>, Pointer<Utf8>, int)>('setenv');
    final name = 'MOTOLII_OWN'.toNativeUtf8(), path = own.path.toNativeUtf8();
    setenv(name, path, 1); // (the maker's folder here: a test does not write into the real one)
    malloc
      ..free(name)
      ..free(path);
    final folder = Directory('${Directory.systemTemp.createTempSync('drop').path}/Motolii')..createSync();
    WsZoom.set(1, keep: false);
    ColorTheme.at = '${folder.path}/none.xml'; // (the window's own skin)
    await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => Workspace(document: '${folder.path}/Untitled.motolii')));
    for (var i = 0; i < 100 && find.byType(WsTimeline).evaluate().isEmpty; i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 20)));
      await t.pump();
    }
    await t.pumpAndSettle();
    final ws = WsScope.read(t.element(find.byType(WsTimeline))), m = ws.inside!;
    final out = Directory('$motoliiDir/../docs/media')..createSync(recursive: true);
    Future<void> shot(String png) async {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 1500))); // (the inside draws the stage in real time)
      for (var k = 0; k < 20; k++) {
        await t.pump(const Duration(milliseconds: 50));
      }
      final root = t.binding.renderViews.first.layer! as OffsetLayer;
      final bytes = await (await root.toImage(Offset.zero & t.view.physicalSize)).toByteData(format: ImageByteFormat.png);
      File('${out.path}/$png').writeAsBytesSync(bytes!.buffer.asUint8List());
    }

    from.copySync('${own.path}/effects/ASCII Art.fs'); // (what a maker does in Finder, with the window open)
    final photo = m.add('picture', '$motoliiDir/fx_sample.jpg');
    m.set(photo, 'size', 2);
    for (var i = 0; i < 200 && m.picture('ASCII Art').isEmpty; i++) {   // (seen by the folder's watch, read again, its picture drawn: effect_tiles.dart)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
      await t.pump();
    }
    expect(File(m.picture('ASCII Art')).existsSync(), isTrue, reason: 'its tile drawn where the inside says (~/Library/Caches/Motolii/tiles)');
    ws
      ..reread()
      ..select('$photo')
      ..shelf = 'Effects';
    await t.pumpAndSettle();
    await t.enterText(find.descendant(of: find.byType(DockSearch), matching: find.byType(EditableText)), 'ASCII');
    await t.pumpAndSettle();
    final row = find.byKey(const ValueKey('ws-screen-ASCII Art'));
    await seen(t, row);
    await t.tap(row);
    await t.pump(const Duration(milliseconds: 400));
    await t.pumpAndSettle();
    await shot('drop_step1.png'); // (on the shelf, picked)
    await t.tap(find.descendant(of: find.byType(DockFooter), matching: find.text('Add')));
    await settled(t, ws);
    for (var i = 0; i < 100 && m.word(photo, 'fx').isEmpty; i++) {   // (put on without waiting)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
      await t.pump();
    }
    expect(m.word(photo, 'fx'), endsWith('/ASCII Art.fs'), reason: m.trouble);
    await shot('drop_step2.png'); // (on the photo, in the window's own picture)
    final film = '${folder.path}/drop.mp4';
    expect(await t.runAsync(() => m.write(film)), isTrue, reason: m.trouble); // (written from the window's own document: drawn right)
    // ignore: avoid_print
    print('written: $film');
    List<String> bare() => [for (final (n, f) in m.effects) if (m.picture(f).isEmpty || m.picture(f) != m.tileHome(f)) n];   // (each drawn here: moving)
    for (var i = 0; i < 6000 && bare().isNotEmpty; i++) {   // (every effect on the shelf with a tile: the ones none was drawn for, drawn by themselves)
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 200)));
      await t.pump();
    }
    expect(bare(), isEmpty, reason: 'every effect on the shelf has a tile');
  });
}
