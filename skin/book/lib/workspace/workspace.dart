// The whole window: top bar, then the panes (panes.dart) — browser shelves | Stage over Timeline, Inspector over Desk, rearrangeable.
// Each seat is its own file and reads the shared Ws.
import 'dart:developer' show Timeline;
import 'dart:async';
import 'dart:io';
import 'dart:math' as math;

import 'package:docking/docking.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_parts_insp_host.dart';

import '../sets/inspector/inspector_parts.dart';
import 'desk.dart';
import 'desk_depth.dart';
import 'desk_ease.dart';
import 'dock.dart';
import 'dock_glyphs.dart';
import 'panes.dart';
import 'desk_kit.dart';
import 'stage.dart';
import 'timeline.dart';
import 'top_bar.dart';
import '../motolii.dart';
import 'ws.dart';

class Workspace extends StatefulWidget {
  const Workspace({super.key, this.document, this.extra = const [], this.tools = const [], this.view = const {}});

  /// A document of the inside (motolii.dart) to join the window to: its stage is then the inside's own picture.
  /// null: the sample, with its stand-in art.
  final String? document;

  /// Panes of your own, beside the window's: each joins the Inspector's tabs. A pane reads and changes the document
  /// through [WsScope.of] (its `inside` is the document, `selected` the picked thing, `frame` the moment).
  final List<WsPane> extra;

  /// Desk tools of your own, after the window's (a DeskTool with its `build`).
  final List<DeskTool> tools;

  /// How the window looks when it opens (Ws.view: as it was left, say).
  final Map<String, Object?> view;
  @override
  State<Workspace> createState() => _WorkspaceState();
}

class _WorkspaceState extends State<Workspace> {
  late final _ws = Ws()
    ..tools = [...deskTools, ...widget.tools]
    ..view.addAll(widget.view);
  Motolii? _inside;

  String? _doc; // (the document that is open: the one the window was given, or one made or opened from the menu)

  @override
  void initState() {
    super.initState();
    _join(widget.document ?? '${Directory.systemTemp.createTempSync('motolii').path}/Untitled.motolii'); // (no document given: a new one, as After Effects opens an untitled project)
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
              PlatformMenuItem(label: 'Undo', shortcut: const SingleActivator(LogicalKeyboardKey.keyZ, meta: true), onSelected: () => _typed(const UndoTextIntent(SelectionChangedCause.keyboard)) ? null : _ws.after(m.later('undo'))),
              PlatformMenuItem(label: 'Redo', shortcut: const SingleActivator(LogicalKeyboardKey.keyZ, meta: true, shift: true), onSelected: () => _typed(const RedoTextIntent(SelectionChangedCause.keyboard)) ? null : _ws.after(m.later('redo'))),
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
              if (t != null) _ws.after(m.later('remove', thing: t));   // (without waiting)
            },
          ),
        ],
      ),
      PlatformMenu(
        label: 'Animation',
        menus: [
          PlatformMenuItem(label: 'Sequence Layers', onSelected: () => _ws.after(m.later('sequence'))),   // (After Effects' Keyframe Assistant: one after another, from the top)
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

  /// The panes drag their tabs on an overlay; the window brings its own so it works under any host.
  late final _entry = OverlayEntry(
    maintainState: true,
    builder: (context) => LayoutBuilder(
      builder: (context, box) {
        final window = _Window(widget.extra);
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
        child: ColoredBox(
          color: WsT.ground,
          child: Overlay(initialEntries: [_entry]),
        ),
      ),
    );
    if (_ws.inside == null) return ColoredBox(color: WsT.ground); // (the document still opening: no seat is built on nothing)
    return PlatformMenuBar(menus: _menus(), child: window);
  }
}

class _Window extends StatelessWidget {
  const _Window(this.extra);
  final List<WsPane> extra;
  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      const SizedBox(height: WsT.topBar, child: WsTopBar()),
      const SizedBox(height: WsT.gutter),
      Expanded(
        child: WsPanes(panes: [..._panes, ...extra], preset: (item) => _preset(item, extra), shelves: {for (final s in dockShelves) s.$1}),
      ),
    ],
  );
}

final _panes = [
  for (final (name, g, _) in dockShelves) WsPane(name, g, () => WsShelf(name), minSize: 240, iconOnly: true),
  WsPane('Stage', DockG.stage, () => const WsStage(), minSize: 320),
  WsPane('Inspector', DockG.inspector, () => const _InspectorSeat(), minSize: 200),
  WsPane('Timeline', DockG.timeline, () => const WsTimeline(), minSize: 160),
  WsPane('Ease', DockG.time, () => const DeskEase(), minSize: 260),
  WsPane('Camera', DockG.camera, () => const DeskDepth(), minSize: WsT.lower),
  WsPane('Desk', DockG.desk, () => const WsDesk(), minSize: WsT.lower),
];

/// A pane's tab strip, added on top of the reference heights so the seats keep theirs.
const _strip = 22.0;

/// The reference arrangement: the browser shelves (one strip of tabs) | Stage over Timeline and Ease on the left, Inspector over Camera on the right.
/// Side and lower panes start at their reference sizes in px and the panes without one share what is left.
/// contract: give the filling panes no weight; `multi_split_view` turns sizes into weights and would scale them against any weight given.
DockingArea _preset(DockingItem Function(String name, {double? size, double? weight}) item, List<WsPane> extra) => DockingRow([
  DockingColumn([
    DockingRow([
      DockingTabs([for (final (name, _, _) in dockShelves) item(name)], size: WsT.dock),
      item('Stage'),
    ]),
    DockingRow([item('Timeline'), item('Ease', size: WsT.inspector)], size: WsT.lower + _strip),   // (the stretch between two keys, beside the keys)
  ]),
  DockingColumn([extra.isEmpty ? item('Inspector') : DockingTabs([item('Inspector'), for (final p in extra) item(p.name)]), DockingTabs([item('Camera'), item('Desk')], size: WsT.lower + _strip)], size: WsT.inspector, minimalSize: 340),   // (the camera, whatever is picked)
]);

/// Joined: the document's word for what a thing is, where the inspector's kinds have none (film, picture, solid).
String? _kindWord(Ws ws, String? id) {
  final t = int.tryParse(id ?? ''), w = t == null ? '' : ws.inside?.word(t, 'kind') ?? '';
  return const {'film', 'picture', 'solid', 'light'}.contains(w) ? w.replaceRange(0, 1, w[0].toUpperCase()) : null;
}

Kind _kindOf(WsLayer? l) => switch (l?.kind) {
  null || WsKind.audio => Kind.none,
  WsKind.text => Kind.text,
  WsKind.camera => Kind.camera,
  WsKind.group => Kind.group,
  _ => Kind.shape,
};


/// The paint each layer wears on the stage, so the Inspector's Fill names the colour you see.

/// Joined: the picked thing's words in the inspector's ids (a text's content; how it lies on what is behind), and a change of them told to the document.
Map<String, String> _words(Ws ws, String? id) {
  final m = ws.inside, t = id == null ? 0 : int.tryParse(id); // (nothing picked: the stage's own cards)
  if (m == null || t == null) return const {};
  return {
    ..._cardColours(m, t),
    if (m.word(t, 'kind') == 'light') 'light': m.word(t, 'file'), // (which light it is: a sun, a point, a spot; the Light card draws it so)
    for (final (_, names) in m.cards(t))
      for (final n in names)
        if (m.shown(n)?.look == 'thing')
          'v_$n': _others(ws, t).entries.where((e) => e.value == (m.value(t, n, 0) ?? 0)).firstOrNull?.key ?? 'None' // (another thing, by its name)
        else if (m.words(t, n).isNotEmpty)
          'v_$n': m.word(t, n).isEmpty ? m.words(t, n).first : m.word(t, n), // (one of its words; none said: the first, as it comes)
  };
}

/// What a card's value may be, for a hand-drawn card's chooser: its words, or another thing by name (None first).
Map<String, List<String>> _choices(Ws ws, String? id) {
  final m = ws.inside, t = id == null ? 0 : int.tryParse(id);
  if (m == null || t == null) return const {};
  return {
    for (final (_, names) in m.cards(t))
      for (final n in names)
        if (m.shown(n)?.look == 'thing') 'v_$n': ['None', ..._others(ws, t).keys] else if (m.words(t, n).isNotEmpty) 'v_$n': m.words(t, n),
  };
}

/// How the inspector is looked at (its folds, an effect's Advanced shown): the window's view, not the document's.
Map<String, bool> _flags(Ws ws, String? id) => {
  for (final e in ws.view.entries)
    if (e.key.startsWith('inspector.') && e.value is bool) e.key.substring(10): e.value as bool,
};

bool _flag(Ws ws, String? id, Map<String, bool> changed) {
  bool looked(String k) => k.endsWith('.open') || k.endsWith('.adv') || k.startsWith('fold.');
  for (final e in changed.entries) {
    if (looked(e.key)) ws.view['inspector.${e.key}'] = e.value;
  }
  final other = changed.keys.where((k) => !looked(k)).firstOrNull;
  if (other == null) return true;
  _told(ws, 'the document has no ${other.endsWith('.on') ? 'bypass' : other}: '); // (an effect switched off and kept)
  return false;
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
  // (asked without waiting, one step: the window goes on while the inside works it, then reads it again; refused, the
  //  words go back to the document's own when it is read)
  m.later('together', number: 1);
  for (final e in changed.entries) {
    if (!e.key.startsWith('v_')) continue;
    final n = e.key.substring(2);
    m.shown(n)?.look == 'thing'
        ? m.later('set', thing: t, name: n, number: (_others(ws, t)[e.value] ?? 0).toDouble()) // (another thing, told by its name: it goes with that one)
        : m.later('say', thing: t, name: n, word: e.value); // (a card's value of that name)
  }
  ws.after(m.later('together'), (done, no) => _told(ws, done ? null : no));
  return true;
}


/// The seats a thing has: the document's own names, and the inspector's numbers whose value it holds (a light has no
/// scale and no opacity, a flat thing no scale z: those are not shown).
Set<String> _seatsOf(Set<String> names) => {...names, for (final n in names) 'v_$n', 'depth'}; // (a hand-drawn card asks by the value's id, 'v_x')


/// Joined: the colours on the inside's own cards, as the inspector writes a colour (none said: a dash).
Map<String, String> _cardColours(Motolii m, int t) => {
  for (final (_, names) in m.cards(t))
    for (final n in names)
      if (m.words(t, n).isEmpty && m.range(t, n) == null)
        'v_$n': m.shown(n)?.look == 'words'
            ? m.word(t, n) // (words as they are, to be typed over)
            : m.word(t, n).isEmpty || m.word(t, n) == 'none'
            ? '—'
            : m.shown(n)?.look == 'file'
            ? m.word(t, n).split('/').last // (a file by its name)
            : m.shown(n)?.look == 'numbers'
            ? m.word(t, n) // (numbers, one for each copy, as they are)
            : m.word(t, n).split(' ').where((c) => c.isNotEmpty).map((c) => int.tryParse(c, radix: 16) != null && c.length == 6 ? '#${c.toUpperCase()}' : c.replaceRange(0, 1, c[0].toUpperCase())).join(' '), // (a colour, or colours in their order; a word that is not one, a material, as it is: Glass)
};

/// Joined: the cards the inside shows a thing's values on beyond the sample's (motolii_cards), each as Params in the
/// effects' manner: a number its range and what it is now, a word the words it may be. Nothing of them is typed here.
List<Fx>? _valueCards(Ws ws, String? id) {
  final m = ws.inside, t = id == null ? 0 : int.tryParse(id); // (nothing picked: the stage's own, thing 0)
  if (m == null || t == null) return null;
  // (made again only when the cards it has, their values or the things it may name change: a number let go of changes
  //  none of them, and the numbers the cards show are read from wsTransform; what a card says a number is now, its
  //  @default, the inspector takes only when it makes its cards anew)
  final shape = '${m.cards(t)}|${ws.layers.map((l) => l.name).join(',')}';
  final made = _cardsMade[t];
  if (made != null && made.$1 == shape && identical(made.$3, m)) return made.$2;
  final cards = _makeCards(ws, m, t);
  _cardsMade[t] = (shape, cards, m);
  return cards;
}
final _cardsMade = <int, (String, List<Fx>, Motolii)>{};
int? _holding; // (the thing a hand on the inspector holds now: held once, carried at every step, let go when it lets go)

List<Fx> _makeCards(Ws ws, Motolii m, int t) {
  final at = ws.frame / Ws.fps;
  String field(String n) {
    final sh = m.shown(n), label = '@label(${sh?.label ?? n.replaceAll('_', ' ').replaceRange(0, 1, n[0].toUpperCase())})';
    if (sh?.look == 'thing') {
      // (another thing, by the names the timeline shows: None, then the others; its number is the document's)
      final others = _others(ws, t), at = others.values.toList().indexOf((m.value(t, n, 0) ?? 0).round());
      return '  /// $label @enum(${['None', ...others.keys.map((o) => o.replaceAll(',', ' '))].join(', ')}) @default(${at + 1})\n  $n: u32,';
    }
    if (m.words(t, n) case final words when words.isNotEmpty) {
      return '  /// $label @enum(${words.join(', ')}) @default(${math.max(0, words.indexOf(m.word(t, n)))})\n  $n: u32,';
    }
    if (m.range(t, n) case (final lo0, final hi0)) {
      final a = _shown(m, t, n, sh != null && sh.high > sh.low ? sh.low : lo0), b = _shown(m, t, n, sh != null && sh.high > sh.low ? sh.high : hi0), lo = math.min(a, b), hi = math.max(a, b);
      final ui = switch (sh?.look) {
        'onoff' => '@toggle',
        'angle' => '@ui(angle)',
        'seed' => '@ui(seed)',
        'count' => hi - lo <= 12 ? '@ui(stepper)' : '@ui(bar)',
        'share' when lo >= 0 && hi <= 100 => '@ui(percent)',
        _ => lo < 0 && hi > 0 ? '@ui(bipolar)' : '@ui(bar)',
      };
      final whole = sh?.look == 'count' || sh?.look == 'onoff' || sh?.look == 'seed';
      return '  /// $label $ui @range($lo, $hi) @unit(${sh?.unit ?? ''}) @default(${_shown(m, t, n, m.value(t, n, at) ?? lo0)})\n  $n: ${whole ? 'u32' : 'f32'},';
    }
    return '  /// $label${const ['numbers', 'colours', 'words', 'file'].contains(sh?.look) ? ' @unit(${sh!.look})' : ''}\n  $n: vec3<f32>,'; // (any word: colours, six digits each, given from the Colors shelf; or numbers, one for each copy)
  }

  return [
    for (final (title, names) in m.cards(t))
      Fx('v', title, 'struct Params {\n${[
        for (final n in names) if (m.shown(n)?.fold == null) field(n),
        for (final fold in {for (final n in names) ?m.shown(n)?.fold}) ...[
          '  // @advanced($fold)',
          for (final n in names) if (m.shown(n)?.fold == fold) field(n),
        ],
      ].join('\n')}\n}', tone: Pop.toneLayer, mark: CardMark.layer),
  ];
}

/// Joined: how many of a knob's units make one of the document's (the inside's shown look: a length of the stage in
/// points of the picture, a share in hundredths; else as it is).
/// A value of the inside's own cards as the inspector counts it: another thing by its place (None first), a number
/// in the card's units, a word by its place among those it may be (the stage's and a thing's alike).
double _onCard(Ws ws, Motolii m, int t, String n, double at) =>
    m.shown(n)?.look == 'thing' ? (_others(ws, t).values.toList().indexOf((m.value(t, n, 0) ?? 0).round()) + 1).toDouble() : m.words(t, n).isEmpty ? _shown(m, t, n, m.value(t, n, at) ?? 0) : math.max(0, m.words(t, n).indexOf(m.word(t, n))).toDouble(); // (the inside's own cards: a number as it is, a word by its place among those it may be)

/// A value as a hand reads it, and back (Motolii.reading).
double _shown(Motolii m, int t, String n, double v) { final (k, f) = m.reading(t, n); return v * k + f; }
double _unshown(Motolii m, int t, String n, double x) { final (k, f) = m.reading(t, n); return (x - f) / k; }

// (the moment of the picture on the stage, at a frame: what is read there was set down by the inside as it drew it,
//  not waited for while it tells the stage a moment the playhead has just been put at)
double _drawn(Ws ws) => ws.inside?.drawnAt ?? ws.frame / Ws.fps;

/// Joined: only where a thing is, how large and how turned (the inspector's seats), as the stage's readout shows them:
/// not the cards' hundreds of numbers [wsTransform] reads.
Map<String, double> wsPlace(Ws ws, String id) {
  final m = ws.inside, t = int.tryParse(id);
  if (m == null || t == null) return const {};
  final at = _drawn(ws);
  double r1(double x) => (x * 10).roundToDouble() / 10 + 0;
  return {for (final n in const ['x', 'y', 'scale_y', 'rotate']) n: r1(_shown(m, t, n, m.value(t, n, at) ?? (n == 'scale_y' ? 1 : 0)))};
}

/// The selected layer's transform as the stage draws it at this frame, in the Inspector's ids.
Map<String, double> wsTransform(Ws ws, String? id) {
  if (ws.inside case final m?) {   // (the document's own values, as the inspector counts them)
    final t = int.tryParse(id ?? '');
    if (id == null) { // (nothing picked: the stage's own cards, thing 0)
      final at = _drawn(ws);
      return {for (final (_, names) in m.cards(0)) for (final n in names) 'v_$n': _onCard(ws, m, 0, n, at)};
    }
    if (t == null) return const {};
    final at = _drawn(ws);
    double r1(double x) => (x * 10).roundToDouble() / 10 + 0; // (+ 0: no minus nought)
    return {
      'depth': r1(m.frame(t)?[7] ?? 0), // (how deep it is in points of the picture: a flat thing is Flat)
      for (final (_, names) in m.cards(t))
        for (final n in names)
          'v_$n': _onCard(ws, m, t, n, at),
    };
  }
  return const {};
}

(String, double)? _asDocument(Ws ws, String id, double x) {
  if (id.startsWith('v_') && ws.inside!.shown(id.substring(2))?.look == 'thing') { // (another thing: None, then the others by their place)
    final others = _others(ws, int.tryParse(ws.selected ?? '') ?? 0).values.toList(), i = x.round();
    return (id.substring(2), i >= 1 && i <= others.length ? others[i - 1].toDouble() : 0);
  }
  if (id.startsWith('v_')) return (id.substring(2), _unshown(ws.inside!, int.tryParse(ws.selected ?? '') ?? 0, id.substring(2), x)); // (a card of the inside's own: the value of that name, in the document's own units)
  return null;
}

/// Joined: what was changed in the inspector is a thing held and its numbers carried (the inside's one form of a
/// drag): while a hand is on it, it only shows; when the hand lets go it is the document's, by the document's rules.
void _write(Ws ws, String? id, Map<String, double> changed, {required bool live}) {
  final m = ws.inside!, t = id == null ? 0 : int.tryParse(id); // (nothing picked: the stage's own, thing 0)
  // (a word on a card of the inside's own, picked: the word in that place, said; one step)
  final said = {for (final e in changed.entries) if (t != null && e.key.startsWith('v_') && m.words(t, e.key.substring(2)).isNotEmpty) e.key: e.value};
  if (said.isNotEmpty) {
    if (live) return;
    m.later('together', number: 1);   // (one step, without waiting)
    for (final e in said.entries) {
      final ways = m.words(t!, e.key.substring(2));
      m.later('say', thing: t, name: e.key.substring(2), word: ways[e.value.round().clamp(0, ways.length - 1)]);
    }
    ws.after(m.later('together'), (done, why) => _told(ws, done ? null : why));
    changed = {for (final e in changed.entries) if (!said.containsKey(e.key)) e.key: e.value};
    if (changed.isEmpty) return;
  }
  if (t == null || (_holding != t && !m.hold(t))) return; // (held once a hand: each step after only carries)
  _holding = live ? t : null;
  String? why;
  for (final e in changed.entries) {
    final to = _asDocument(ws, e.key, e.value);
    if (to != null && !m.carry(to.$1, to.$2)) why ??= m.trouble;
    if (to == null) why ??= 'the document has no ${e.key}: '; // (a seat the document has no value for says so, rather than keep the number)
  }
  if (live) {
    if (ws.playing) ws.playing = false; // (a hand on a value stops the play: what is changed is changed at one moment)
    if (why != null) _told(ws, why);
    ws.carried.value++; // (the stage's readout follows the hand)
  } else {
    ws.after(m.later('let_go', number: 1), (done, no) => _told(ws, why ?? (done ? null : no))); // (the hand let go: the window does not wait for the step)
  }
}

/// Joined: a property's key mark as the document has it at the playhead (the most keyed of its values).
/// Joined: the key marks the inspector shows (for the checks: the same the inspector is given).
Map<String, KeyS> wsKeyMarks(Ws ws, String? id) => _keyMarks(ws, id);

Map<String, KeyS> _keyMarks(Ws ws, String? id) {
  final m = ws.inside, t = id == null ? 0 : int.tryParse(id);
  if (m == null || t == null) return const {};
  final at = _drawn(ws);
  final marks = {for (final MapEntry(:key, :value) in _keyable(ws, t).entries) key: KeyS.values[m.keyed(t, value, at)]};
  return {
    ...marks,
    for (final key in marks.keys) // (a property's mark is the most of its values': a key on any, keyed)
      if (propOf(key) != key) propOf(key): marks.entries.where((e) => propOf(e.key) == propOf(key)).map((e) => e.value).reduce((a, b) => b.index > a.index ? b : a),
  };
}

/// Joined: the fields of the cards whose values take keys, by the inspector's id and the document's name.
Map<String, String> _keyable(Ws ws, int t) {
  final m = ws.inside!, moving = m.moving(t).toSet();
  return {
    for (final (_, names) in m.cards(t))
      for (final n in names)
        if (moving.contains(n)) 'v_$n': n,
  };
}

/// Joined: a key mark changed in the inspector: a key at the playhead, that key taken away, or no keys at all.
void _mark(Ws ws, String? id, String prop, KeyS mark) {
  final m = ws.inside!, t = id == null ? 0 : int.tryParse(id);
  if (t == null) return;
  final at = ws.frame / Ws.fps, has = m.values(t).toSet();
  // (only the values it has: a flat thing has no scale_z, and a key asked of it was refused and took the whole step,
  //  the number let go of with it, back: with Animate on, a flat thing's first scale was lost)
  m.later('together', number: 1);   // (one step, without waiting)
  for (final n in [for (final MapEntry(:key, :value) in _keyable(ws, t).entries) if (key == prop || propOf(key) == prop) value].where(has.contains)) {
    m.later(switch (mark) { KeyS.at => 'key_here', KeyS.anim => 'unkey', KeyS.off => 'stay' }, thing: t, name: n, number: at);
  }
  ws.after(m.later('together'));
}

/// The Inspector of the lab, fed by the workspace's selection; its panel links open the dock's shelves or the desk.
class _InspectorSeat extends StatefulWidget {
  const _InspectorSeat();
  @override
  State<_InspectorSeat> createState() => _InspectorSeatState();
}

class _InspectorSeatState extends State<_InspectorSeat> {
  // (the inspector follows only what it shows: the document, what is picked, Animate, and the moment only while what is
  //  picked has keys; played, a thing with no keys shows the same numbers, so it is not built again each picture)
  Ws? _ws;
  Object? _seen;
  (int, bool)? _stageKeyed;
  bool _keyed(Ws ws) {
    final l = ws.layer;
    if (l != null) return l.keys.isNotEmpty;
    final m = ws.inside;
    if (m == null) return true;
    if (_stageKeyed?.$1 != ws.rev) _stageKeyed = (ws.rev, m.moving(0).any((n) => m.keyMoments(0, n).isNotEmpty)); // (the stage's: once a document)
    return _stageKeyed!.$2;
  }

  // (the moment alone moved: the values and key marks told to the inspector as it stands; anything else: built again)
  final _live = ValueNotifier<(Map<String, double>, Map<String, KeyS>)>((const {}, const {}));
  int? _frame;
  Object _now(Ws ws) => (ws.rev, ws.selected, ws.animating);

  // (the reading of the document changed only numbers of what is picked, none that changes its cards: they are told
  //  to the fields that show them, as a moment's are; anything else, or not known: the inspector built again)
  bool _onlyNumbers(Ws ws) {
    final m = ws.inside, c = ws.lastChange, t = int.tryParse(ws.selected ?? '') ?? (ws.selected == null ? 0 : null);
    if (m == null || c == null || c.things || t == null || const bool.fromEnvironment('NOPOUR')) return false;
    for (final (thing, name) in c.values) {
      if (name == 'name' || (thing == t && (m.shaping.contains(name) || m.range(t, name) == null || m.shown(name)?.look == 'thing'))) return false; // (another thing: shown by its name, a word) // (a name changed anywhere: things are named on cards; a word: it is not watched)
    }
    return true;
  }
  void _changed() => Timeline.timeSync('inspector at a moment', _changedNow); // (named in Flutter's timeline: frames_test TRACE)
  void _changedNow() {
    final ws = _ws!, now = _now(ws), was = _seen;
    if (now != _seen) {
      _seen = now;
      _frame = ws.frame;
      if (was is (int, String?, bool) && was.$2 == ws.selected && was.$3 == ws.animating && _onlyNumbers(ws)) {
        _live.value = (Ws.timed('transform', () => wsTransform(ws, ws.layer?.id)), Ws.timed('key marks', () => _keyMarks(ws, ws.layer?.id)));
        return;
      }
      if (mounted) setState(() {});
    } else if (ws.frame != _frame) {
      _frame = ws.frame;
      if (!_keyed(ws)) return;
      if (ws.inside?.busy ?? false) { // (the inside is telling the stage this moment: read at the next, or when it rests, rather than wait)
        _settle?.cancel();
        _settle = Timer(const Duration(milliseconds: 150), () => mounted ? setState(() {}) : null);
        return;
      }
      _live.value = (Ws.timed('transform', () => wsTransform(ws, ws.layer?.id)), Ws.timed('key marks', () => _keyMarks(ws, ws.layer?.id)));
      _settle?.cancel();
      _settle = Timer(const Duration(milliseconds: 150), () => mounted ? setState(() {}) : null); // (the moment at rest: built once, whatever does not watch its own number shows it too)
    }
  }

  Timer? _settle;

  // (the panel itself, made once a size: a new one each build would build its whole tree again, whatever changed)
  (Size, InspectorPanel)? _panel;
  InspectorPanel _panelOf(BoxConstraints box) {
    final size = Size(box.maxWidth, box.maxHeight);
    if (_panel?.$1 != size) _panel = (size, InspectorPanel(width: box.maxWidth, height: box.maxHeight));
    return _panel!.$2;
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final ws = WsScope.read(context);
    if (ws != _ws) {
      _ws?.removeListener(_changed);
      _ws = ws..addListener(_changed);
      _seen = _now(ws);
      _frame = ws.frame;
    }
  }

  @override
  void dispose() {
    _ws?.removeListener(_changed);
    _settle?.cancel();
    _live.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final ws = WsScope.read(context), l = ws.layer, k = _kindOf(l), m = ws.inside!;
    if (int.tryParse(l?.id ?? '') case final t?) m.soloFollow(t); // (the thing picked is the one shown alone)
    return LayoutBuilder(
      builder: (context, box) => InspHost(
        key: ValueKey(k),
        cfg: const Cfg(),
        kind: k,
        fx: const [], // (an effect is one of the inside's cards, shown while it is on)
        kindWord: _kindWord(ws, l?.id),
        cards: Ws.timed('value cards', () => _valueCards(ws, l?.id)),
        seats: Ws.timed('seats', () => int.tryParse(l?.id ?? '') == null ? null : _seatsOf(m.values(int.parse(l!.id)).toSet())), // (a card of values the thing has not is not shown)
        title: l?.name,
        onRoute: ws.route,
        onPath: int.tryParse(l?.id ?? '') == null
            ? null
            : (to, n, copy) => m.path(int.parse(l!.id), copy: copy, to: to, n: n)?.indexed.map((e) => e.$1 % 4 < 2 ? e.$2 * m.point : e.$2).toList(), // (where it is in points of the picture, as the card's numbers are) // (the wave card's preview asks once, then plays by itself)
        onSolo: int.tryParse(l?.id ?? '') == null
            ? null
            : (side, on) {
                m.soloShow(int.parse(l!.id), side, on: on);
                return on ? m.soloTexture(int.parse(l.id), side) : Future.value(-1);
              }, // (a look card's picture of the thing alone)
        onAim: (at) => ws.aim = at.startsWith('v_') ? at.substring(2) : null, // (a colour link of a card is aimed at that value)
        values: Ws.timed('transform', () => wsTransform(ws, l?.id)),
        strs: Ws.timed('words', () => _words(ws, l?.id)),
        choices: Ws.timed('choices', () => _choices(ws, l?.id)),
        onWords: (changed) => _say(ws, l?.id, changed),
        onAnimateAll: (on) => ws.animating = on,
        animating: ws.animating,
        flags: Ws.timed('flags', () => _flags(ws, l?.id)),
        onFlags: (changed) => _flag(ws, l?.id, changed),
        keyMarks: Ws.timed('key marks', () => _keyMarks(ws, l?.id)),
        onValues: (changed, {required live}) => _write(ws, l?.id, changed, live: live),
        onDrop: () => ws.after(m.later('let_go')),   // (put down unchanged, without waiting)
        onKey: (prop, mark) => _mark(ws, l?.id, prop, mark),
        together: (tells) { // (one step, asked without waiting: begun and ended in the inside's queue around what is told)
          m.later('together', number: 1);
          tells();
          ws.after(m.later('together'));
        },
        live: _live,
        child: _panelOf(box), // (the same panel while the pane keeps its size: built again only by what it reads, Ctx)
      ),
    );
  }
}
