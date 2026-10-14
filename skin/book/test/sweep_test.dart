// Every control the window shows, worked one at a time, and the document asked what it did (the judge is the inside,
// never the screen: what the window claims and what the document holds must agree). A knob that moves nothing, a knob
// whose own number, read back from the document, stays or drags others with it, a handle that moves something other
// than what it is drawn for: each is a failure.
import 'dart:io';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart' show InspectorPanel, LinkMark, NumField;
import 'package:lab_book/workspace/stage_view.dart';
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/workspace.dart' show Workspace, wsTransform;
import 'package:lab_book/workspace/ws.dart';

final _built = File('$motoliiDir/out/libmotolii.dylib').existsSync();
const _font = '$motoliiDir/out/arial_black.ttf';

/// What each thing is made as, and the things the sweep works on.
const _things = [('solid', 'cube', ''), ('solid', 'ring', 'glass'), ('shape', 'rect', ''), ('text', _font, ''), ('light', 'sun', '')];

/// The values a handle on the stage is drawn for (a corner sizes, the knob turns, the cross moves the point it turns about, the body moves).
const _handleFor = {
  'corner': {'scale_x', 'scale_y'},
  'turn': {'rotate'},
  'anchor': {'anchor_x', 'anchor_y', 'x', 'y'}, // (the point it turns about, the thing staying put; where it lies on the body, the body is taken and moved, as After Effects' selection tool)
  'body': {'x', 'y', 'z'},
};

Future<Ws> _pump(WidgetTester t) async {
  t.view.physicalSize = const Size(1600, 2400);
  t.view.devicePixelRatio = 1;
  t.binding.defaultBinaryMessenger.setMockMethodCallHandler(const MethodChannel('motolii/stage'), (_) async => -1);
  t.binding.defaultBinaryMessenger.setMockMethodCallHandler(SystemChannels.menu, (_) async => null);
  final doc = '${Directory.systemTemp.createTempSync('sweep').path}/sweep.motolii';
  await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => Workspace(document: doc)));
  await t.pumpAndSettle();
  return WsScope.read(t.element(find.byType(WsTimeline)));
}

/// Whether a knob is greyed by its row (a row that is off takes no hand: not a knob to work).
bool _off(Element e) {
  var off = false;
  e.visitAncestorElements((a) => !(off = a.widget is IgnorePointer && (a.widget as IgnorePointer).ignoring));
  return off;
}

/// The whole document as values, thing by thing (the stage's own is thing 0).
Map<String, Object?> _all(Motolii m, double at) => {
  for (final t in [0, ...m.things])
    for (final n in m.values(t)) '$t:$n': m.value(t, n, at) ?? m.word(t, n),
};

/// The values that differ between two looks at the document, without the thing's number.
Set<String> _moved(Map<String, Object?> a, Map<String, Object?> b) => {
  for (final k in {...a.keys, ...b.keys})
    if (a[k] != b[k]) k.substring(k.indexOf(':') + 1),
};

Future<Ws> _picked(WidgetTester t, (String, String, String) what) async {
  final ws = await _pump(t), m = ws.inside!;
  final (kind, file, fill) = what;
  final thing = m.add(kind, file);
  expect(thing, isNot(0), reason: m.trouble);
  if (fill.isNotEmpty) m.say(thing, 'fill', fill);
  ws
    ..reread()
    ..select('$thing');
  await t.pumpAndSettle();
  return ws;
}

void main() {
  WidgetController.hitTestWarningShouldBeFatal = true; // (a hand that lands on something other than what it aimed at fails, as Flutter's own guard has it)

  for (final what in _things) {
    testWidgets('every knob on the cards of a ${what.$1} ${what.$2.split('/').last} moves a value of its own', (t) async {
      final ws = await _picked(t, what), m = ws.inside!, at = ws.frame / Ws.fps;
      // (the scale's link let go: each axis on its own, as a hand that wants another aspect lets it go first)
      final link = find.descendant(of: find.byType(InspectorPanel), matching: find.byType(LinkMark));
      if (link.evaluate().isNotEmpty) {
        await t.tap(link.first);
        await t.pumpAndSettle();
      }
      final ids = [
        for (final e in find.descendant(of: find.byType(InspectorPanel), matching: find.byType(NumField)).evaluate())
          if ((e.widget as NumField).enabled && !_off(e)) (e.widget as NumField).id,
      ];
      final dead = <String>[], deaf = <String>[], spill = <String>[], undone = <String>[];
      for (final id in ids.toSet()) {
        final knob = find.byWidgetPredicate((w) => w is NumField && w.id == id).first;
        await t.ensureVisible(knob);
        await t.pumpAndSettle();
        final before = _all(m, at), shown = wsTransform(ws, ws.selected);
        var moved = <String>{};
        for (final by in const [Offset(40, -40), Offset(-80, 80)]) {
          await t.drag(knob, by);
          await t.pumpAndSettle();
          moved = _moved(before, _all(m, at));
          if (moved.isNotEmpty) break;
        }
        // (what the cards show now, read from the document: the knob's own number moved, and no other's)
        final now = wsTransform(ws, ws.selected), others = [for (final k in ids) if (k != id && now[k] != shown[k]) k];
        if (moved.isEmpty) {
          dead.add(id);
        } else if (now[id] == shown[id]) {
          deaf.add('$id (moved ${moved.join(', ')})');
        } else if (others.isNotEmpty) {
          spill.add('$id moved ${others.join(', ')} too');
        }
        if (moved.isNotEmpty) m.undo(); // (one drag is one step)
        ws.reread();
        await t.pumpAndSettle();
        if (_moved(before, _all(m, at)).isNotEmpty) undone.add(id);
      }
      // ignore: avoid_print
      print('${what.$1} ${what.$2.split('/').last}: ${ids.length} knobs; moves nothing: $dead; its own number stays: $deaf; others move with it: $spill; not undone in one step: $undone');
      expect([...dead.map((d) => 'moves nothing: $d'), ...deaf.map((d) => 'its own number stays: $d'), ...spill, ...undone.map((u) => 'not undone in one step: $u')], isEmpty);
      expect(t.takeException(), isNull);
      await t.pumpWidget(const SizedBox());
      t.view.reset();
    }, skip: !_built);

    if (what.$1 == 'light') continue; // (a light has no frame on the stage, so no handles: its knobs are swept above)
    testWidgets('every handle drawn on a ${what.$1} ${what.$2.split('/').last} moves what it is drawn for', (t) async {
      final ws = await _picked(t, what), m = ws.inside!, at = ws.frame / Ws.fps, stage = find.byType(StageView);
      final handles = (t.state(stage) as StageHandles).handles, origin = t.getTopLeft(stage), wrong = <String>[];
      expect(handles, isNotEmpty, reason: 'the picked thing shows its handles');
      for (final MapEntry(key: name, value: at0) in handles.entries) {
        final want = _handleFor[name.split(' ').first]!, before = _all(m, at);
        await t.dragFrom(origin + at0, const Offset(30, 20));
        await t.pumpAndSettle();
        final moved = _moved(before, _all(m, at));
        if (moved.isEmpty || !want.containsAll(moved)) wrong.add('$name moved ${moved.isEmpty ? 'nothing' : moved.join(', ')} (drawn for ${want.join(', ')})');
        if (moved.isNotEmpty) m.undo();
        ws.reread();
        await t.pumpAndSettle();
      }
      // ignore: avoid_print
      print('${what.$1} ${what.$2.split('/').last}: ${handles.keys.join(', ')}; wrong: $wrong');
      expect(wrong, isEmpty);
      expect(t.takeException(), isNull);
      await t.pumpWidget(const SizedBox());
      t.view.reset();
    }, skip: !_built);
  }
}
