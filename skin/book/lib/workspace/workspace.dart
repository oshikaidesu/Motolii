// The whole window: top bar, then the panes (panes.dart) — browser shelves | Stage over Timeline, Inspector over Desk, rearrangeable.
// Each seat is its own file and reads the shared Ws.
import 'dart:math' as math;

import 'package:docking/docking.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../sets/inspector/inspector_parts.dart';
import 'desk.dart';
import 'dock.dart';
import 'dock_glyphs.dart';
import 'panes.dart';
import 'stage.dart';
import 'stage_art.dart';
import 'timeline.dart';
import 'top_bar.dart';
import '../motolii.dart';
import 'ws.dart';

class Workspace extends StatefulWidget {
  const Workspace({super.key, this.document});

  /// A document of the inside (motolii.dart) to join the window to: its stage is then the inside's own picture.
  /// null: the sample, with its stand-in art.
  final String? document;
  @override
  State<Workspace> createState() => _WorkspaceState();
}

class _WorkspaceState extends State<Workspace> {
  final _ws = Ws();
  Motolii? _inside;

  @override
  void initState() {
    super.initState();
    if (widget.document case final doc?) _join(doc);
  }

  Future<void> _join(String doc) async {
    final inside = Motolii.open(doc);
    if (inside == null) return;
    _inside = inside;
    final id = await inside.texture();
    if (!mounted) return;
    if (id >= 0) _ws.stageTexture = id;
    _ws.join(inside);
  }

  @override
  void dispose() {
    _inside?.close();
    _ws.dispose();
    super.dispose();
  }

  /// Enter keeps a try-on and Esc drops it, wherever focus is in the window.
  KeyEventResult _key(FocusNode _, KeyEvent e) {
    if (e is! KeyDownEvent || _ws.trial == null) return KeyEventResult.ignored;
    switch (e.logicalKey) {
      case LogicalKeyboardKey.escape:
        _ws.dropFx();
      case LogicalKeyboardKey.enter || LogicalKeyboardKey.numpadEnter:
        _ws.keepFx();
      default:
        return KeyEventResult.ignored;
    }
    return KeyEventResult.handled;
  }

  /// The panes drag their tabs on an overlay; the window brings its own so it works under any host.
  late final _entry = OverlayEntry(
    maintainState: true,
    builder: (context) => LayoutBuilder(
      builder: (context, box) {
        const window = _Window();
        if (box.maxWidth >= WsT.minWindow.width && box.maxHeight >= WsT.minWindow.height) return window;
        return FittedBox(
          child: SizedBox.fromSize(size: WsT.refWindow, child: window),
        );
      },
    ),
  );

  @override
  Widget build(BuildContext context) => WsScope(
    ws: _ws,
    child: Focus(
      autofocus: true,
      onKeyEvent: _key,
      child: ColoredBox(
        color: WsT.ground,
        child: Overlay(initialEntries: [_entry]),
      ),
    ),
  );
}

class _Window extends StatelessWidget {
  const _Window();
  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      const SizedBox(height: WsT.topBar, child: WsTopBar()),
      const SizedBox(height: WsT.gutter),
      Expanded(
        child: WsPanes(panes: _panes, preset: _preset, shelves: {for (final s in dockShelves) s.$1}),
      ),
    ],
  );
}

final _panes = [
  for (final (name, g, _) in dockShelves) WsPane(name, g, () => WsShelf(name), minSize: 240, iconOnly: true),
  WsPane('Stage', DockG.stage, () => const WsStage(), minSize: 320),
  WsPane('Inspector', DockG.inspector, () => const _InspectorSeat(), minSize: 200),
  WsPane('Timeline', DockG.timeline, () => const WsTimeline(), minSize: 160),
  WsPane('Desk', DockG.desk, () => const WsDesk(), minSize: WsT.lower),
];

/// A pane's tab strip, added on top of the reference heights so the seats keep theirs.
const _strip = 22.0;

/// The reference arrangement: the browser shelves (one strip of tabs) | Stage over Timeline on the left, Inspector over Desk on the right.
/// Side and lower panes start at their reference sizes in px and the panes without one share what is left.
/// contract: give the filling panes no weight; `multi_split_view` turns sizes into weights and would scale them against any weight given.
DockingArea _preset(DockingItem Function(String name, {double? size, double? weight}) item) => DockingRow([
  DockingColumn([
    DockingRow([
      DockingTabs([for (final (name, _, _) in dockShelves) item(name)], size: WsT.dock),
      item('Stage'),
    ]),
    item('Timeline', size: WsT.lower + _strip),
  ]),
  DockingColumn([item('Inspector'), item('Desk', size: WsT.lower + _strip)], size: WsT.inspector, minimalSize: 340),
]);

Kind _kindOf(WsLayer? l) => switch (l?.kind) {
  null || WsKind.audio => Kind.none,
  WsKind.text => Kind.text,
  WsKind.camera => Kind.camera,
  WsKind.group => Kind.group,
  _ => Kind.shape,
};

String _hex(Color c) => '#${(c.toARGB32() & 0xFFFFFF).toRadixString(16).padLeft(6, '0').toUpperCase()}';

/// The paint each layer wears on the stage, so the Inspector's Fill names the colour you see.
const _fills = {'blob': ArtInk.orange, 'ring': ArtInk.lime, 'title': ArtInk.cream, 'chips': ArtInk.pink, 'confetti': ArtInk.pink, 'bg': ArtInk.blue};

/// Joined: the picked thing's words in the inspector's ids (a text's content; how it lies on what is behind), and a change of them told to the document.
Map<String, String> _words(Ws ws, String? id) {
  final m = ws.inside, t = int.tryParse(id ?? '');
  if (m == null || t == null) return const {};
  final blend = m.word(t, 'blend');
  return {
    if (m.word(t, 'kind') == 'text') ...{'text.content': m.word(t, 'text'), 'font': m.word(t, 'file').split('/').last},
    'blend': blend.isEmpty ? 'Normal' : blend.replaceRange(0, 1, blend[0].toUpperCase()),
  };
}

bool _say(Ws ws, String? id, Map<String, String> changed) {
  final m = ws.inside!, t = int.tryParse(id ?? '');
  if (t == null) return false;
  var all = true;
  m.together(() {
    for (final e in changed.entries) {
      final ok = switch (e.key) {
        'text.content' => m.say(t, 'text', e.value.replaceAll('\n', ' ')),
        'blend' => m.say(t, 'blend', e.value.toLowerCase()),
        _ => true, // (a seat the inside has nothing behind yet: the inspector keeps its own word)
      };
      all = all && ok;
    }
  });
  ws.reread();
  return all;
}

/// What the Inspector's Fill names: joined, the document's word for the thing (a colour, or the material Glass); else the sample's paint.
String? _fillWord(Ws ws, String? id, Color? sample) {
  final m = ws.inside;
  if (m == null) return sample == null ? null : _hex(sample);
  final t = int.tryParse(id ?? ''), f = t == null ? '' : m.word(t, 'fill');
  return f.isEmpty ? 'None' : (f == 'glass' ? 'Glass' : '#${f.toUpperCase()}'); // (nothing said: nothing is named, not the last thing's fill)
}

/// The selected layer's transform as the stage draws it at this frame, in the Inspector's ids.
Map<String, double> _transform(Ws ws, String? id) {
  if (ws.inside case final m?) {   // (joined: the document's own values, in the composition's points)
    final t = int.tryParse(id ?? '');
    if (t == null) return const {};
    final at = ws.frame / Ws.fps, p = m.point;
    double v(String name) => m.value(t, name, at) ?? 0;
    double r1(double x) => (x * 10).roundToDouble() / 10;
    final s = r1(v('size') * 100);
    return {
      'pos.x': r1(WsArt.comp.width / 2 + v('x') * p),
      'pos.y': r1(WsArt.comp.height / 2 - v('y') * p),
      'pos.z': r1(v('z') * p),
      'scale.x': s,
      'scale.y': s,
      'scale.z': s, // (a thing has one size)
      'rot.x': r1(v('rotate_x')) + 0,
      'rot.y': r1(v('rotate_y')) + 0,
      'rot.z': r1(-v('rotate')) + 0,   // (+ 0: no minus nought)
      'op': r1((m.value(t, 'opacity', at) ?? 1) * 100), // (a thing that has no opacity is all there)
    };
  }
  final b = WsArt.boxOf(ArtScene.of(ws, camera: true), id);
  if (b == null) return const {};
  final s = (b.scale * 1000).roundToDouble() / 10, r = (b.rot * 180 / math.pi * 10).roundToDouble() / 10;
  return {'pos.x': b.anchor.dx.roundToDouble(), 'pos.y': b.anchor.dy.roundToDouble(), 'scale.x': s, 'scale.y': s, 'rot.z': r};
}

/// The document's names for the inspector's ids, and the inspector's number as the document's (and back, in _transform):
/// positions in the composition's points from its top left, scale in hundredths of a unit, turn clockwise.
const _propNames = {
  'pos': ['x', 'y', 'z'],
  'scale': ['size'],
  'rot': ['rotate', 'rotate_x', 'rotate_y'],
  'op': ['opacity'],
};
(String, double)? _asDocument(Ws ws, String id, double x) {
  final p = ws.inside!.point;
  return switch (id) {
    'pos.x' => ('x', (x - WsArt.comp.width / 2) / p),
    'pos.y' => ('y', (WsArt.comp.height / 2 - x) / p),
    'pos.z' => ('z', x / p),
    'scale.x' || 'scale.y' || 'scale.z' => ('size', x / 100),
    'rot.x' => ('rotate_x', x),
    'rot.y' => ('rotate_y', x),
    'rot.z' => ('rotate', -x),
    'op' => ('opacity', x / 100),
    _ => null,
  };
}

/// Joined: what was changed in the inspector goes to the document. While a hand is on it, it is only shown; when the
/// hand lets go it is put there (the document's own rule: a key at the playhead if the value has keys, else the value).
void _write(Ws ws, String? id, Map<String, double> changed, {required bool live}) {
  final m = ws.inside!, t = int.tryParse(id ?? '');
  if (t == null) return;
  final at = ws.frame / Ws.fps;
  for (final e in changed.entries) {
    final to = _asDocument(ws, e.key, e.value);
    if (to == null) continue;
    final (name, number) = to;
    live ? m.tryValue(t, name, number) : m.put(t, name, at, number);
  }
  if (live && ws.playing) ws.playing = false; // (a hand on a value stops the play: what is changed is changed at one moment)
  if (!live) ws.reread();
}

/// Joined: a property's key mark as the document has it at the playhead (the most keyed of its values).
Map<String, KeyS> _keyMarks(Ws ws, String? id) {
  final m = ws.inside, t = int.tryParse(id ?? '');
  if (m == null || t == null) return const {};
  final at = ws.frame / Ws.fps;
  return {
    for (final p in _propNames.entries) p.key: KeyS.values[p.value.map((n) => m.keyed(t, n, at)).reduce(math.max)],
  };
}

/// Joined: a key mark changed in the inspector: a key at the playhead, that key taken away, or no keys at all.
void _mark(Ws ws, String? id, String prop, KeyS mark) {
  final m = ws.inside!, t = int.tryParse(id ?? '');
  if (t == null) return;
  final at = ws.frame / Ws.fps;
  for (final n in _propNames[prop] ?? const <String>[]) {
    switch (mark) {
      case KeyS.at:
        m.keyHere(t, n, at);
      case KeyS.anim:
        m.unkey(t, n, at);
      case KeyS.off:
        m.stay(t, n, at);
    }
  }
  ws.reread();
}

/// The Inspector of the lab, fed by the workspace's selection; its panel links open the dock's shelves or the desk.
class _InspectorSeat extends StatelessWidget {
  const _InspectorSeat();
  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context), l = ws.layer, k = _kindOf(l), fill = _fills[l?.id];
    return LayoutBuilder(
      builder: (context, box) => InspHost(
        key: ValueKey(k),
        cfg: const Cfg(),
        kind: k,
        effects: ws.inside == null ? 2 : 0, // (joined: the inside has no effects yet, so none are listed)
        title: l?.name,
        onRoute: ws.route,
        values: _transform(ws, l?.id),
        strs: {'fill': ?_fillWord(ws, l?.id, fill), ..._words(ws, l?.id)},
        onWords: ws.inside == null ? null : (changed) => _say(ws, l?.id, changed),
        keyMarks: _keyMarks(ws, l?.id),
        onValues: ws.inside == null ? null : (changed, {required live}) => _write(ws, l?.id, changed, live: live),
        onDrop: ws.inside?.tryDrop,
        onKey: ws.inside == null ? null : (prop, mark) => _mark(ws, l?.id, prop, mark),
        together: ws.inside?.together,
        child: InspectorPanel(directTransform: true, width: box.maxWidth, height: box.maxHeight),
      ),
    );
  }
}
