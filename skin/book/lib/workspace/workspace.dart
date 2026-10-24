// The whole window: top bar, then the panes (panes.dart) — browser shelves | Stage over Timeline, Inspector over Desk, rearrangeable.
// Each seat is its own file and reads the shared Ws.
import 'dart:developer' show Timeline;
import 'dart:async';
import 'dart:io';
import 'dart:math' as math;

import 'package:docking/docking.dart';
import 'package:flutter/widgets.dart';
import '../parts/skin_parts.dart';
import 'zoom.dart';
import 'package:lab_book/sets/inspector/inspector_parts_insp_host.dart';

import '../sets/inspector/inspector_parts.dart';
import '../theme.dart';
import 'desk.dart';
import 'desk_depth.dart';
import 'desk_relations.dart';
import 'desk_ease.dart';
import 'dock.dart';
import 'dock_glyphs.dart';
import 'panes.dart';
import 'desk_kit.dart';
import 'key_drag.dart';
import 'keys.dart';
import 'stage.dart';
import 'stage_view.dart' show WsRelationStack;
import 'timeline.dart';
import 'top_bar.dart';
import '../motolii.dart';
import 'relate.dart';
import 'ws.dart';
part 'workspace_2.dart';

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
    FocusManager.instance.addListener(_keepKeys);
    ColorTheme.watch(() => WidgetsBinding.instance.reassembleApplication());
    SkinParts.follow();   // (the theme's drawn parts: an SVG folder, read when the theme names it)   // (the theme file: read, and read again when written; the window built again in place)
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
    final mp4 = _ws.renderTo ?? '${doc.substring(0, doc.lastIndexOf('.'))}.mp4';
    _ws.renderTo = null;
    _ws.notice = 'Writing ${mp4.split('/').last} …';
    final done = await m.write(mp4);
    if (mounted) _ws.notice = done ? 'Written: ${mp4.split('/').last}' : 'Not written: ${m.trouble}';
  }

  /// The window's menus, in the system's own menu bar: what the sample window has no seat for (a new document,
  /// another document, the film written out) and what the keyboard does elsewhere (undo, redo, take a thing out).
  // The window holds the focus when nothing in it does (a field left, a pane closed): its keys are heard there.
  final _windowFocus = FocusNode(debugLabel: 'window');
  void _keepKeys() {
    final now = FocusManager.instance.primaryFocus;
    final above = now == null || (now is FocusScopeNode && _windowFocus.ancestors.contains(now));   // (fallen out of the window)
    if (!mounted || !above || _windowFocus.context == null) return;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted && !_windowFocus.hasFocus) _windowFocus.requestFocus();
    });
  }

  /// The window's commands by name (keys.dart: the keys that say them, the window's own and the maker's file).
  late final _keymap = Keymap.load();
  Map<String, VoidCallback> _commands() {
    final m = _inside!, ws = _ws;
    void step(int by) {
      ws.playing = false;
      ws.frame = ws.frame + by;
    }

    return {
      'file.new': () => _open(_fresh()),
      'file.render': () => wsAskRender.value++,   // (the settings window first)
      'edit.undo': () => _typed(const UndoTextIntent(SelectionChangedCause.keyboard)) ? null : ws.after(m.later('undo')),
      'edit.redo': () => _typed(const RedoTextIntent(SelectionChangedCause.keyboard)) ? null : ws.after(m.later('redo')),
      'edit.cut': () => _typed(const CopySelectionTextIntent.cut(SelectionChangedCause.keyboard)),
      'edit.copy': () => _typed(CopySelectionTextIntent.copy),
      'edit.paste': () => _typed(const PasteTextIntent(SelectionChangedCause.keyboard)),
      'edit.selectAll': () => _typed(const SelectAllTextIntent(SelectionChangedCause.keyboard)) ? null : ws.pickMany([for (final l in ws.layers) l.id]),   // (no words being written: every thing, as Ableton's Cmd+A in the arrangement)
      'thing.deactivate': () {
        final t = int.tryParse(ws.selected ?? '');
        if (t == null) return;
        final hidden = ws.layers.any((l) => l.id == ws.selected && l.hidden);
        ws.after(m.later('set', thing: t, name: 'visible', number: hidden ? 1 : 0));
      },
      'thing.remove': () {
        if (ws.keys.isNotEmpty) return wsRemoveKeys(ws, ws.keys.toList());   // (keys picked: those, not the thing)
        final all = ws.many.isNotEmpty ? ws.many.toList() : [if (ws.selected != null) ws.selected!];   // (things picked together: all of them, one step)
        final ts = [for (final id in all) if (int.tryParse(id) case final t?) t];
        if (ts.isEmpty) return;
        m.later('together', number: 1);
        for (final t in ts) {
          m.later('remove', thing: t);   // (without waiting; undo brings them back)
        }
        ws.after(m.later('together'));
        ws.select(null);   // (nothing picked after it: a second Delete takes nothing else)
      },
      'thing.duplicate': () {
        final t = int.tryParse(ws.selected ?? '');
        if (t == null) return;
        ws.after(m.later('duplicate', thing: t), null, (made) => ws.select('$made'));   // (the new one picked, as After Effects does)
      },
      'layers.sequence': () => ws.after(m.later('sequence')),   // (After Effects' Keyframe Assistant: one after another, from the top)
      'play.toggle': () => ws.playing = !ws.playing,
      'frame.previous': () => step(-1),
      'frame.next': () => step(1),
      'frame.back10': () => step(-10),
      'frame.forward10': () => step(10),
      'frame.first': () => step(-ws.frame),
      'frame.last': () => step(Ws.duration - ws.frame),
    };
  }

  void _wear(String? skin) => setState(() => ColorTheme.wear(skin));   // (the menus again too: the one worn is the one not offered)

  List<PlatformMenuItem> _menus() {
    final does = _commands();
    PlatformMenuItem item(String label, String command) => PlatformMenuItem(label: label, shortcut: _keymap.keyOf(command), onSelected: does[command]);
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
              item('New Document', 'file.new'),
              PlatformMenu(label: 'Open', menus: [for (final f in _beside) PlatformMenuItem(label: f.split('/').last, onSelected: f == _doc ? null : () => _open(f))]),
            ],
          ),
          item('Render', 'file.render'),
        ],
      ),
      PlatformMenu(
        label: 'Edit',
        menus: [
          PlatformMenuItemGroup(
            members: [
              item('Undo', 'edit.undo'),
              item('Redo', 'edit.redo'),
            ],
          ),
          // (the system's own Edit menu went when these menus came: what a text box does is given back to it)
          PlatformMenuItemGroup(
            members: [
              item('Cut', 'edit.cut'),
              item('Copy', 'edit.copy'),
              item('Paste', 'edit.paste'),
              item('Select All', 'edit.selectAll'),
            ],
          ),
          item('Duplicate', 'thing.duplicate'),
          item('Remove', 'thing.remove'),
        ],
      ),
      PlatformMenu(
        label: 'View',
        menus: [
          PlatformMenu(
            label: 'Skin',
            menus: [
              PlatformMenuItemGroup(
                members: [
                  PlatformMenuItem(label: 'Default', onSelected: ColorTheme.skin == null ? null : () => _wear(null)),
                  for (final n in ColorTheme.skinNames()) PlatformMenuItem(label: n, onSelected: n == ColorTheme.skin ? null : () => _wear(n)),
                ],
              ),
              PlatformMenuItem(label: 'Show Skins Folder', onSelected: () => Process.run('open', [(Directory(ColorTheme.skins)..createSync(recursive: true)).path])),
            ],
          ),
        ],
      ),
      PlatformMenu(
        label: 'Animation',
        menus: [
          item('Sequence Layers', 'layers.sequence'),
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
        _ws.after(inside.later('size', word: '$wide $high'));   // (without waiting: the document's picture then says it does not fit, and it is opened again)
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
    FocusManager.instance.removeListener(_keepKeys);
    _windowFocus.dispose();
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
        return FittedBox( // (laid out no smaller than the least window, shrunk only as far as that needs: a zoomed window stays zoomed)
          child: SizedBox(width: math.max(box.maxWidth, WsT.minWindow.width), height: math.max(box.maxHeight, WsT.minWindow.height), child: window),
        );
      },
    ),
  );

  @override
  Widget build(BuildContext context) {
    final window = WsScope(
      ws: _ws,
      child: Focus(
        focusNode: _windowFocus,
        autofocus: true,
        child: ColoredBox(
          color: WsT.ground,
          child: Overlay(initialEntries: [_entry]),
        ),
      ),
    );
    if (_ws.inside == null) return ColoredBox(color: WsT.ground); // (the document still opening: no seat is built on nothing)
    return PlatformMenuBar(menus: _menus(), child: WsKeys(keymap: _keymap, commands: _commands(), child: WsZoomed(child: window)));
  }
}

class _Window extends StatelessWidget {
  const _Window(this.extra);
  final List<WsPane> extra;
  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      SizedBox(height: WsT.topBar, child: WsTopBar()),
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
  WsPane('Relations', DockG.desk, () => const DeskRelations(), minSize: WsT.lower),
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
  DockingColumn([extra.isEmpty ? item('Inspector') : DockingTabs([item('Inspector'), for (final p in extra) item(p.name)]), DockingTabs([item('Camera'), item('Desk'), item('Relations')], size: WsT.lower + _strip)], size: WsT.inspector, minimalSize: 340),   // (the camera, whatever is picked)
]);

/// Joined: the document's word for what a thing is, where the inspector's kinds have none (film, picture, solid).
String? _kindWord(Ws ws, String? id) {
  final t = int.tryParse(id ?? ''), w = t == null ? '' : ws.inside?.word(t, 'kind') ?? '';
  if (w == 'behaviour') return wsRelations.contains(ws.inside?.word(t!, 'file')) ? 'Relation' : 'Behaviour';
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

/// Joined: the other things by their row's number on the timeline and the name it shows there.
Map<String, int> _others(Ws ws, int t) {
  final rows = ws.layers.where((l) => l.id != '$t' && ws.inside?.word(int.parse(l.id), 'kind') != 'behaviour');   // (things, not relations or behaviours)
  return {for (final l in rows) '${ws.layers.indexOf(l) + 1}. ${l.name}': int.parse(l.id)};   // (as After Effects' parent list: the row's number on the timeline, then the name)
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
