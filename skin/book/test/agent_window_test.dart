// A hand for a tester who only sees pictures, on the whole window joined to a real document (a stage of nine things):
// lines appended to $AGENT_DIR/inbox.txt are done as a mouse would, after each the window is shot to $AGENT_DIR/NNN.png
// and "NNN <line>" added to $AGENT_DIR/done.txt. Coordinates are the shot's pixels (the window is 1600 x 960).
//   tap X Y | shifttap X Y | doubletap X Y
//   drag X1 Y1 X2 Y2           press, move there in steps, let go
//   path X1 Y1 X2 Y2 X3 Y3 ... press at the first, go through the rest, let go (drawing round)
//   key cmd+z | cmd+shift+z | escape | delete
//   look                       only a shot
//   end                        the window closed, the test over
// With AGENT_TAB=<a way's name> the window opens on a copy of the relation try document with that way of the Relations
// tab beside the inspector (its path in $AGENT_DIR/document.txt).
// With AGENT_BENCH=<a document> the relation bench is opened on a copy of it instead (its path in $AGENT_DIR/document.txt).
// Without AGENT_DIR it is skipped. Run:  AGENT_DIR=/some/dir flutter test test/agent_window_test.dart
// The stage's picture is the window's stand-in (boxes where the inside says things lie), the words in Arial.
import 'dart:io';
import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:flutter/gestures.dart' show PointerDeviceKind;
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/sets/relation_bench.dart';
import 'package:lab_book/sets/relation_tabs.dart';
import 'package:lab_book/workspace/dock_glyphs.dart';
import 'package:lab_book/workspace/panes.dart' show WsPane;
import 'package:lab_book/workspace/stage.dart' show WsStage;
import 'package:lab_book/workspace/ws.dart' show WsScope, WsT;

import 'door.dart';

final _dir = Platform.environment['AGENT_DIR'], _bench = Platform.environment['AGENT_BENCH'], _tab = Platform.environment['AGENT_TAB'];

/// The whole window on a copy of the relation try document, with one way of the Relations tab beside the inspector
/// (AGENT_TAB=<its name in relationTabs>).
Future<Ws> _openTab(WidgetTester t, String name) async {
  final path = '${Directory.systemTemp.createTempSync('tab').path}/try.motolii';
  File('test/data/relation_try.motolii').copySync(path);
  Process.runSync('sqlite3', [path, 'delete from step;']);
  File('$_dir/document.txt').writeAsStringSync(path);
  final way = relationTabs.firstWhere((w) => w.$1 == name);
  return openWorkspace(t, document: path, extra: [WsPane('Relations ${way.$1}', DockG.desk, way.$2)]);
}

/// The relation bench on a copy of [doc] instead of the whole window (AGENT_BENCH=<a document>): undo begins there.
Future<Ws> _openBench(WidgetTester t, String doc) async {
  final path = '${Directory.systemTemp.createTempSync('bench').path}/bench.motolii';
  File(doc).copySync(path);
  File('$_dir/document.txt').writeAsStringSync(path);   // (where the tester's document is, to read what they made)
  Process.runSync('sqlite3', [path, 'delete from step;']);
  t.view.physicalSize = const Size(1600, 960);
  t.view.devicePixelRatio = 1;
  t.binding.defaultBinaryMessenger.setMockMethodCallHandler(const MethodChannel('motolii/stage'), (_) async => -1);
  await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => RelationBench(document: path)));
  for (var i = 0; i < 40 && find.byType(WsStage).evaluate().isEmpty; i++) {
    await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
    await t.pump();
  }
  await t.pumpAndSettle();
  return WsScope.read(t.element(find.byType(WsStage)));
}

Future<void> _fonts() async {
  Future<void> load(String family, String file) async {
    if (!File(file).existsSync()) return;
    await (FontLoader(family)..addFont(File(file).readAsBytes().then(ByteData.sublistView))).load();
  }

  for (final f in ['Inter', '.AppleSystemUIFont', 'Helvetica Neue', 'Roboto']) {
    await load(f, '/System/Library/Fonts/Supplemental/Arial.ttf');
  }
  await load('Menlo', '/System/Library/Fonts/Supplemental/Courier New.ttf');
}

/// Nine things on the stage: circles, squares and a star, in a few colours, here and there.
String _scene() {
  final path = '${Directory.systemTemp.createTempSync('agent').path}/agent.motolii';
  final m = Motolii.open(path)!
    ..say(0, 'behind', 'colour')
    ..say(0, 'background', 'f1ece2');   // (on paper: a black thing is seen)
  const things = [
    ('circle', -0.9, 0.45, 'e5432d'), ('rect', -0.4, 0.55, '2a62c9'), ('circle', 0.2, 0.5, 'f7d046'),
    ('star', 0.8, 0.4, '151515'), ('circle', -0.7, -0.1, '6bbf8c'), ('rect', 0.0, 0.0, 'e5432d'),
    ('circle', 0.6, -0.15, '2a62c9'), ('rect', -0.3, -0.55, 'f2a7c6'), ('circle', 0.5, -0.6, '151515'),
  ];
  for (final (shape, x, y, ink) in things) {
    final t = m.add('shape', shape);
    m
      ..set(t, 'size', .22)
      ..set(t, 'x', x)
      ..set(t, 'y', y)
      ..say(t, 'fill', ink);
  }
  m.close();
  Process.runSync('sqlite3', [path, 'delete from step;']);   // (the scene is where the tester begins: undo stops there)
  return path;
}

void main() {
  testWidgets("a tester's hand on the window", (t) async {
    final dir = _dir!;
    await t.runAsync(_fonts);
    final bench = _bench, tab = _tab, ws = tab != null ? await _openTab(t, tab) : bench != null ? await _openBench(t, bench) : await openWorkspace(t, document: _scene());
    final inbox = File('$dir/inbox.txt'), done = File('$dir/done.txt');
    if (!inbox.existsSync()) inbox.writeAsStringSync('');
    var n = 0, idle = 0;
    var pointer = 900;

    // (the window, with the inside's own picture of the stage where the window leaves it bare: in a test the stage's
    // texture is not there, so its picture is put in under the window's marks, every point still the surround's colour)
    Future<void> shot(String line) async {
      await settled(t, ws);
      final name = (n++).toString().padLeft(3, '0');
      final comp = t.getRect(find.byKey(const ValueKey('ws-stage-comp')));
      await t.runAsync(() async {
        final root = t.binding.renderViews.first.layer! as OffsetLayer;
        final image = await root.toImage(Offset.zero & t.view.physicalSize);
        final win = (await image.toByteData())!;
        final w = image.width;
        final still = '$dir/stage.png', waited = ws.inside!.broken;   // (what the window waited for, kept; then the shot's own still, which waits as the window never does, forgotten)
        if (waited.isNotEmpty) File('$dir/broken.txt').writeAsStringSync('$waited\n', mode: FileMode.append);
        final took = ws.inside!.still(still);
        ws.inside!.watch(true);
        if (took && File(still).existsSync()) {
          final codec = await ui.instantiateImageCodec(File(still).readAsBytesSync(), targetWidth: comp.width.round(), targetHeight: comp.height.round());
          final pic = (await codec.getNextFrame()).image, px = (await pic.toByteData())!;
          final l = comp.left.round(), top = comp.top.round(), bg = win.getUint32(((top + 1) * w + l + 1) * 4);
          // (a mark the window drew half-through in its accent: how much of it, from how far it is from the surround
          // towards the accent, laid again over the picture rather than over the surround)
          final b = [for (var k = 0; k < 3; k++) win.getUint8(((top + 1) * w + l + 1) * 4 + k)], acc = WsT.accent;
          final a = [(acc.r * 255).round(), (acc.g * 255).round(), (acc.b * 255).round()], d = [for (var k = 0; k < 3; k++) a[k] - b[k]];
          final dd = d.fold<int>(0, (s, v) => s + v * v);
          for (var y = 0; y < pic.height; y++) {
            for (var x = 0; x < pic.width; x++) {
              final at = ((top + y) * w + l + x) * 4, under = (y * pic.width + x) * 4;
              if (win.getUint32(at) == bg) {
                win.setUint32(at, px.getUint32(under));
                continue;
              }
              final p = [for (var k = 0; k < 3; k++) win.getUint8(at + k) - b[k]];
              final f = dd == 0 ? 1.0 : ((p[0] * d[0] + p[1] * d[1] + p[2] * d[2]) / dd).clamp(0.0, 1.0);
              var off = 0.0;
              for (var k = 0; k < 3; k++) {
                off += (p[k] - f * d[k]).abs();
              }
              if (off > 24 || f > .97) continue;   // (not a half-through accent: as the window drew it)
              for (var k = 0; k < 3; k++) {
                win.setUint8(at + k, (f * a[k] + (1 - f) * px.getUint8(under + k)).round().clamp(0, 255));
              }
            }
          }
        }
        final made = await ImmutableBuffer.fromUint8List(win.buffer.asUint8List());
        final desc = ui.ImageDescriptor.raw(made, width: w, height: image.height, pixelFormat: ui.PixelFormat.rgba8888);
        final back = (await (await desc.instantiateCodec()).getNextFrame()).image;
        final png = await back.toByteData(format: ui.ImageByteFormat.png);
        File('$dir/$name.png').writeAsBytesSync(png!.buffer.asUint8List());
      });
      done.writeAsStringSync('$name $line\n', mode: FileMode.append);
    }

    Future<void> through(List<Offset> ps) async {
      final g = await t.startGesture(ps.first, pointer: pointer++, kind: PointerDeviceKind.mouse);   // (a mouse: a drag starts within a few points, as a hand at a desk)
      for (var i = 1; i < ps.length; i++) {
        final a = ps[i - 1], b = ps[i];
        for (var k = 1; k <= 8; k++) {
          await g.moveTo(Offset.lerp(a, b, k / 8)!);
          await t.pump(const Duration(milliseconds: 16));
        }
      }
      await g.up();
    }

    await shot('look');
    var read = 0;
    while (idle < 4000) {
      final lines = inbox.readAsLinesSync();
      if (lines.length <= read) {
        idle++;
        await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 300)));
        await t.pump();
        continue;
      }
      idle = 0;
      final line = lines[read++].trim();
      if (line.isEmpty) continue;
      if (line == 'end') break;
      final w = line.split(RegExp(r'\s+')), nums = [for (final s in w.skip(1)) double.tryParse(s)].whereType<double>().toList();
      final ps = [for (var i = 0; i + 1 < nums.length; i += 2) Offset(nums[i], nums[i + 1])];
      try {
        switch (w.first) {
          case 'tap':
            await t.tapAt(ps.first, kind: PointerDeviceKind.mouse);
          case 'doubletap':
            await t.tapAt(ps.first, kind: PointerDeviceKind.mouse);
            await t.pump(const Duration(milliseconds: 60));
            await t.tapAt(ps.first, kind: PointerDeviceKind.mouse);
          case 'shifttap':
            await simulateKeyDownEvent(LogicalKeyboardKey.shiftLeft);
            await t.tapAt(ps.first, kind: PointerDeviceKind.mouse);
            await simulateKeyUpEvent(LogicalKeyboardKey.shiftLeft);
          case 'drag' || 'path':
            await through(ps);
          case 'key':
            final keys = w[1].split('+');
            final main = switch (keys.last) { 'z' => LogicalKeyboardKey.keyZ, 'escape' => LogicalKeyboardKey.escape, 'delete' => LogicalKeyboardKey.backspace, _ => LogicalKeyboardKey.space };
            final mods = [if (keys.contains('cmd')) LogicalKeyboardKey.metaLeft, if (keys.contains('shift')) LogicalKeyboardKey.shiftLeft];
            for (final k in mods) {
              await simulateKeyDownEvent(k);
            }
            await simulateKeyDownEvent(main);
            await simulateKeyUpEvent(main);
            for (final k in mods.reversed) {
              await simulateKeyUpEvent(k);
            }
          default:
            break;
        }
      } catch (e) {
        done.writeAsStringSync('--- $line: $e\n', mode: FileMode.append);
      }
      await shot(line);
    }
    await closeWorkspace(t);
  }, skip: _dir == null || !insideBuilt, timeout: const Timeout(Duration(minutes: 40)));
}
