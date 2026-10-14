// The whole window: top bar, then the panes (panes.dart) — browser shelves | Stage over Timeline, Inspector over Desk, rearrangeable.
// Each seat is its own file and reads the shared Ws.
import 'dart:io';
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

  String? _doc; // (the document that is open: the one the window was given, or one made or opened from the menu)

  @override
  void initState() {
    super.initState();
    if (widget.document case final doc?) _join(doc);
  }

  /// Another document in place of the open one (made, if there is none at that place).
  void _open(String doc) {
    _ws.stageTexture = null;
    _inside?.close();
    _inside = null;
    _join(doc);
  }

  /// The documents beside the open one (read as one is opened), and a place for a new one there.
  List<String> _beside = const [];
  List<String> _documents() {
    final here = Directory(File(_doc!).parent.path);
    return [for (final f in here.listSync().whereType<File>()) if (f.path.endsWith('.motolii')) f.path]..sort();
  }

  /// A text box has the keys: the act is its own, by Flutter's own text actions (true: it was given it).
  bool _typed(Intent act) {
    final box = primaryFocus?.context;
    if (box == null || box.findAncestorStateOfType<EditableTextState>() == null) return false;
    Actions.maybeInvoke(box, act);
    return true;
  }

  String _fresh() {
    final here = File(_doc!).parent.path;
    for (var n = 1; ; n++) {
      if (!File('$here/untitled_$n.motolii').existsSync()) return '$here/untitled_$n.motolii';
    }
  }

  /// The document written as a film beside itself; the top bar says how it went.
  Future<void> _render() async {
    final m = _inside, doc = _doc;
    if (m == null || doc == null) return;
    final mp4 = '${doc.substring(0, doc.lastIndexOf('.'))}.mp4';
    _ws.notice = 'Writing ${mp4.split('/').last} …';
    final done = await m.write(mp4);
    if (mounted) _ws.notice = done ? 'Written: ${mp4.split('/').last}' : 'Not written: ${m.trouble}';
  }

  /// The window's menus, in the system's own menu bar: what the sample window has no seat for (a new document,
  /// another document, the film written out) and what the keyboard does elsewhere (undo, redo, take a thing out).
  List<PlatformMenuItem> _menus() {
    final m = _inside!;
    void then(bool done) {
      if (done) _ws.reread();
    }

    return [
      PlatformMenu(
        label: 'lab_book',
        menus: [if (PlatformProvidedMenuItem.hasMenu(PlatformProvidedMenuItemType.quit)) const PlatformProvidedMenuItem(type: PlatformProvidedMenuItemType.quit)],
      ),
      PlatformMenu(
        label: 'File',
        menus: [
          PlatformMenuItemGroup(
            members: [
              PlatformMenuItem(label: 'New Document', shortcut: const SingleActivator(LogicalKeyboardKey.keyN, meta: true), onSelected: () => _open(_fresh())),
              PlatformMenu(label: 'Open', menus: [for (final f in _beside) PlatformMenuItem(label: f.split('/').last, onSelected: f == _doc ? null : () => _open(f))]),
            ],
          ),
          PlatformMenuItem(label: 'Render', shortcut: const SingleActivator(LogicalKeyboardKey.keyR, meta: true), onSelected: _render),
        ],
      ),
      PlatformMenu(
        label: 'Edit',
        menus: [
          PlatformMenuItemGroup(
            members: [
              PlatformMenuItem(label: 'Undo', shortcut: const SingleActivator(LogicalKeyboardKey.keyZ, meta: true), onSelected: () => then(!_typed(const UndoTextIntent(SelectionChangedCause.keyboard)) && m.undo())),
              PlatformMenuItem(label: 'Redo', shortcut: const SingleActivator(LogicalKeyboardKey.keyZ, meta: true, shift: true), onSelected: () => then(!_typed(const RedoTextIntent(SelectionChangedCause.keyboard)) && m.redo())),
            ],
          ),
          // (the system's own Edit menu went when these menus came: what a text box does is given back to it)
          PlatformMenuItemGroup(
            members: [
              PlatformMenuItem(label: 'Cut', shortcut: const SingleActivator(LogicalKeyboardKey.keyX, meta: true), onSelected: () => _typed(const CopySelectionTextIntent.cut(SelectionChangedCause.keyboard))),
              PlatformMenuItem(label: 'Copy', shortcut: const SingleActivator(LogicalKeyboardKey.keyC, meta: true), onSelected: () => _typed(CopySelectionTextIntent.copy)),
              PlatformMenuItem(label: 'Paste', shortcut: const SingleActivator(LogicalKeyboardKey.keyV, meta: true), onSelected: () => _typed(const PasteTextIntent(SelectionChangedCause.keyboard))),
              PlatformMenuItem(label: 'Select All', shortcut: const SingleActivator(LogicalKeyboardKey.keyA, meta: true), onSelected: () => _typed(const SelectAllTextIntent(SelectionChangedCause.keyboard))),
            ],
          ),
          PlatformMenuItem(
            label: 'Remove',
            onSelected: () {
              final t = int.tryParse(_ws.selected ?? '');
              then(t != null && m.remove(t));
            },
          ),
        ],
      ),
    ];
  }

  Future<void> _join(String doc) async {
    final inside = Motolii.open(doc);
    if (inside == null) return;
    _inside = inside;
    _doc = doc;
    _beside = _documents();
    final id = await inside.texture();
    if (!mounted) return;
    if (id >= 0) _ws.stageTexture = id;
    _ws
      ..document = doc.split('/').last
      ..documentFolder = File(doc).parent.path
      ..notice = null
      ..onRender = _render
      ..onSize = (wide, high) {
        if (inside.resize(wide, high)) _ws.reread();
      }
      ..refit = () {
        // (a stage is opened at a size: the document closed and opened again, at the moment it was at)
        final at = inside.now;
        _open(doc);
        _inside?.at(at);
      }
      ..join(inside);
    setState(() {}); // (the menus are the open document's)
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
  Widget build(BuildContext context) {
    final window = WsScope(
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
    return _inside == null ? window : PlatformMenuBar(menus: _menus(), child: window);
  }
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
  final blend = m.word(t, 'blend'), stroke = m.word(t, 'stroke');
  return {
    if (m.word(t, 'kind') == 'text') ...{
      'text.content': m.word(t, 'text'),
      'font': m.word(t, 'file').split('/').last,
      'align': _aligned[((m.value(t, 'align', 0) ?? 0) * 2).round().clamp(0, 2)],
    },
    if (m.value(t, 'stroke_width', 0) != null) 'stroke': stroke.isEmpty ? 'None' : '#${stroke.toUpperCase()}',
    'blend': blend.isEmpty ? 'Normal' : blend.replaceRange(0, 1, blend[0].toUpperCase()),
    // (a flat thing's space is the document's word; a solid thing has none: it is 3D)
    'space': m.words(t, 'space').isEmpty ? 'd3' : _spaces.entries.firstWhere((e) => e.value == m.word(t, 'space'), orElse: () => _spaces.entries.first).key,
    'parent': _others(ws, t).entries.where((e) => e.value == (m.value(t, 'parent', 0) ?? 0)).firstOrNull?.key ?? 'None',
  };
}

const _aligned = ['Left', 'Centre', 'Right'];
const _spaces = {'d2': '2d', 'd25': '2.5d', 'd3': '3d'}; // (the inspector's three plates and the document's words; 3d a flat thing has not)

/// Joined: the on/off seats the document has something behind (Clip to below: seen only inside the frame of the thing
/// before it), and a change of them told to it. A seat with nothing behind it is not taken, and goes back.
Map<String, bool> _flags(Ws ws, String? id) {
  final m = ws.inside, t = int.tryParse(id ?? '');
  if (m == null || t == null) return const {};
  return {'clip': (m.value(t, 'clip', 0) ?? 0) > .5, 'ghost': false};
}

bool _flag(Ws ws, String? id, Map<String, bool> changed) {
  final m = ws.inside!, t = int.tryParse(id ?? '');
  if (t == null) return false;
  final all = changed.entries.every((e) => e.key == 'clip' && m.set(t, 'clip', e.value ? 1 : 0));
  ws.reread();
  return all;
}

/// Joined: what the choosers offer is what the document has words for (a thing with no blend lies as it comes: Normal).
Map<String, List<String>> _choices(Ws ws, String? id) {
  final m = ws.inside, t = int.tryParse(id ?? '');
  if (m == null || t == null) return const {};
  final blends = m.words(t, 'blend');
  return {
    'blend': blends.isEmpty ? const ['Normal'] : [for (final w in blends) w.replaceRange(0, 1, w[0].toUpperCase())],
    'parent': ['None', ..._others(ws, t).keys],
  };
}

/// Joined: the other things by the names the timeline shows (a name two things share is told apart by its number).
Map<String, int> _others(Ws ws, int t) {
  final rows = ws.layers.where((l) => l.id != '$t');
  return {
    for (final l in rows) (rows.where((o) => o.name == l.name).length > 1 ? '${l.name} ${l.id}' : l.name): int.parse(l.id),
  };
}

/// Joined: a change the document did not take is said in the top bar, by the inside's own reason (its first clause,
/// read as it was refused: every call of the inside forgets the last reason); the next one it takes clears it.
void _told(Ws ws, String? why) {
  if (why != null && why.isNotEmpty) {
    ws.notice = 'Not taken: ${why.split(':').first}';
  } else if (why == null && (ws.notice?.startsWith('Not taken') ?? false)) {
    ws.notice = null;
  }
}

bool _say(Ws ws, String? id, Map<String, String> changed) {
  final m = ws.inside!, t = int.tryParse(id ?? '');
  if (t == null) return false;
  var all = true;
  String? why;
  m.together(() {
    for (final e in changed.entries) {
      final ok = switch (e.key) {
        'text.content' => m.say(t, 'text', e.value), // (its lines too)
        'align' => switch (_aligned.indexOf(e.value)) {
          final i when i >= 0 => m.set(t, 'align', i / 2), // (Left, Centre, Right: ThorVG's 0, 0.5, 1; Justify it has not)
          _ => false,
        },
        'blend' => m.say(t, 'blend', e.value.toLowerCase()),
        'space' => m.words(t, 'space').isEmpty ? e.value == 'd3' : m.say(t, 'space', _spaces[e.value] ?? e.value), // (a solid is 3D and only that)
        'parent' => m.set(t, 'parent', (_others(ws, t)[e.value] ?? 0).toDouble()), // (it goes with that one, and stays where it is)
        'stroke' => m.say(t, 'stroke', e.value == 'None' ? '' : e.value.replaceFirst('#', '').toLowerCase()), // (None: no edge)
        _ => true, // (a seat the inside has nothing behind yet: the inspector keeps its own word)
      };
      if (!ok) why ??= m.trouble;
      all = all && ok;
    }
  });
  _told(ws, all ? null : why ?? '');
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

/// The inspector's numbers and the document's, in one place: an inspector id, the document's value it shows, and the
/// way across and back (c: the composition's size, p: its points to one unit of the stage; positions are in points
/// from the top left, scale in hundredths of a unit, the turn about z clockwise, opacity in hundredths).
typedef _Across = double Function(double value, Size c, double p);
const _seats = <(String, String, _Across, _Across)>[
  ('pos.x', 'x', _xOut, _xIn),
  ('pos.y', 'y', _yOut, _yIn),
  ('pos.z', 'z', _zOut, _zIn),
  ('scale.x', 'size', _hundredOut, _hundredIn),
  ('scale.y', 'size', _hundredOut, _hundredIn),
  ('scale.z', 'size', _hundredOut, _hundredIn), // (a thing has one size)
  ('rot.x', 'rotate_x', _same, _same),
  ('rot.y', 'rotate_y', _same, _same),
  ('rot.z', 'rotate', _minus, _minus),
  ('op', 'opacity', _hundredOut, _hundredIn),
  ('stroke.w', 'stroke_width', _same, _same),
  ('blur_radius', 'blur', _same, _same),
  ('text.size', 'size', _pointsOut, _pointsIn), // (a text's size in points of the picture: its own size, one unit high at size 1) // (the Blur card's radius: points of the picture at size 1) // (points of the picture at size 1: the edge grows with the thing, as in After Effects)
  // (the anchor: the sample's hundredths of the box from its top left, front -50 to back 50; the document's share of
  //  half the box from its middle, up and towards the eye)
  ('anchor.x', 'anchor_x', _acrossOut, _acrossIn),
  ('anchor.y', 'anchor_y', _downOut, _downIn),
  ('anchor.z', 'anchor_z', _backOut, _backIn),
];
double _acrossOut(double v, Size c, double p) => 50 + 50 * v;
double _acrossIn(double x, Size c, double p) => (x - 50) / 50;
double _downOut(double v, Size c, double p) => 50 - 50 * v;
double _downIn(double x, Size c, double p) => (50 - x) / 50;
double _backOut(double v, Size c, double p) => -50 * v;
double _backIn(double x, Size c, double p) => -x / 50;
double _xOut(double v, Size c, double p) => c.width / 2 + v * p;
double _xIn(double x, Size c, double p) => (x - c.width / 2) / p;
double _yOut(double v, Size c, double p) => c.height / 2 - v * p;
double _yIn(double x, Size c, double p) => (c.height / 2 - x) / p;
double _zOut(double v, Size c, double p) => v * p;
double _zIn(double x, Size c, double p) => x / p;
double _hundredOut(double v, Size c, double p) => v * 100;
double _hundredIn(double x, Size c, double p) => x / 100;
double _same(double v, Size c, double p) => v;
double _pointsOut(double v, Size c, double p) => v * p;
double _pointsIn(double x, Size c, double p) => x / p;
double _minus(double v, Size c, double p) => -v;

/// The document's values behind an inspector property (pos, scale, rot, op): its key mark is theirs.
List<String> _namesOf(String prop) => {
  for (final s in _seats)
    if (s.$1.split('.').first == prop) s.$2,
}.toList();

/// The selected layer's transform as the stage draws it at this frame, in the Inspector's ids.
Map<String, double> wsTransform(Ws ws, String? id) {
  if (ws.inside case final m?) {   // (joined: the document's own values, as the inspector counts them)
    final t = int.tryParse(id ?? '');
    if (t == null) return const {};
    final at = ws.frame / Ws.fps, p = m.point;
    double r1(double x) => (x * 10).roundToDouble() / 10 + 0; // (+ 0: no minus nought)
    return {
      for (final s in _seats) s.$1: r1(s.$3(m.value(t, s.$2, at) ?? (s.$2 == 'opacity' ? 1 : 0), WsArt.comp, p)), // (a thing that has no opacity is all there)
      if (m.value(t, 'wide', 0) case final wide?) 'scale.x': r1((m.value(t, 'size', at) ?? 1) * wide * 100), // (a flat shape may be wider than it is high: its scale x)
    };
  }
  final b = WsArt.boxOf(ArtScene.of(ws, camera: true), id);
  if (b == null) return const {};
  final s = (b.scale * 1000).roundToDouble() / 10, r = (b.rot * 180 / math.pi * 10).roundToDouble() / 10;
  return {'pos.x': b.anchor.dx.roundToDouble(), 'pos.y': b.anchor.dy.roundToDouble(), 'scale.x': s, 'scale.y': s, 'rot.z': r};
}

(String, double)? _asDocument(Ws ws, String id, double x) {
  final seat = _seats.where((s) => s.$1 == id).firstOrNull;
  return seat == null ? null : (seat.$2, seat.$4(x, WsArt.comp, ws.inside!.point));
}

/// Joined: what was changed in the inspector is a thing held and its numbers carried (the inside's one form of a
/// drag): while a hand is on it, it only shows; when the hand lets go it is the document's, by the document's rules.
void _write(Ws ws, String? id, Map<String, double> changed, {required bool live}) {
  final m = ws.inside!, t = int.tryParse(id ?? '');
  if (t == null || !m.hold(t)) return;
  // (a flat shape's scale x and y are the document's size and how many times as wide as high it is)
  final shaped = m.word(t, 'kind') == 'shape' && (changed.containsKey('scale.x') || changed.containsKey('scale.y'));
  if (shaped) {
    final size = m.value(t, 'size', ws.frame / Ws.fps) ?? 1, wide = m.value(t, 'wide', 0) ?? 1;
    final x = (changed['scale.x'] ?? size * wide * 100) / 100, y = (changed['scale.y'] ?? size * 100) / 100;
    m
      ..carry('size', y)
      ..carry('wide', x / y);
  }
  String? why;
  for (final e in changed.entries) {
    final to = shaped && e.key.startsWith('scale.') ? null : _asDocument(ws, e.key, e.value);
    if (to != null && !m.carry(to.$1, to.$2)) why ??= m.trouble;
  }
  if (live) {
    if (ws.playing) ws.playing = false; // (a hand on a value stops the play: what is changed is changed at one moment)
    if (why != null) _told(ws, why);
  } else {
    if (!m.letGo(true)) why ??= m.trouble;
    _told(ws, why);
    ws.reread();
  }
}

/// Joined: a property's key mark as the document has it at the playhead (the most keyed of its values).
Map<String, KeyS> _keyMarks(Ws ws, String? id) {
  final m = ws.inside, t = int.tryParse(id ?? '');
  if (m == null || t == null) return const {};
  final at = ws.frame / Ws.fps;
  return {
    for (final p in const ['pos', 'scale', 'rot', 'op', 'anchor']) p: KeyS.values[_namesOf(p).map((n) => m.keyed(t, n, at)).reduce(math.max)],
  };
}

/// Joined: a key mark changed in the inspector: a key at the playhead, that key taken away, or no keys at all.
void _mark(Ws ws, String? id, String prop, KeyS mark) {
  final m = ws.inside!, t = int.tryParse(id ?? '');
  if (t == null) return;
  final at = ws.frame / Ws.fps;
  for (final n in _namesOf(prop)) {
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
        effects: ws.inside == null ? 2 : (ws.fxOf(l?.id).contains('Gaussian Blur') ? 1 : 0), // (joined: the sample's first card, Blur, is the document's blur)
        title: l?.name,
        onRoute: ws.route,
        values: wsTransform(ws, l?.id),
        strs: {'fill': ?_fillWord(ws, l?.id, fill), ..._words(ws, l?.id)},
        onWords: ws.inside == null ? null : (changed) => _say(ws, l?.id, changed),
        onAnimateAll: ws.inside?.keying,
        choices: _choices(ws, l?.id),
        flags: _flags(ws, l?.id),
        onFlags: ws.inside == null ? null : (changed) => _flag(ws, l?.id, changed),
        keyMarks: _keyMarks(ws, l?.id),
        onValues: ws.inside == null ? null : (changed, {required live}) => _write(ws, l?.id, changed, live: live),
        onDrop: ws.inside == null ? null : () => ws.inside!.letGo(false),
        onKey: ws.inside == null ? null : (prop, mark) => _mark(ws, l?.id, prop, mark),
        together: ws.inside?.together,
        child: InspectorPanel(directTransform: true, width: box.maxWidth, height: box.maxHeight),
      ),
    );
  }
}
