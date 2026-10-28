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
import 'effect_tiles.dart';
import 'ableton.dart';
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
part 'workspace_seat.dart';
part 'workspace_cards.dart';
part 'workspace_values.dart';
part 'workspace_keys.dart';

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
  StreamSubscription<FileSystemEvent>? _own;   // (the maker's own folder watched: effect_tiles.dart)

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
  /// Where a copied thing, or a copied look, is kept between windows (the inside's own document of it: doc_looks.h).
  static String _clipboard(String what) {
    final dir = Directory('${Platform.environment['HOME']}/Library/Caches/Motolii')..createSync(recursive: true);
    return '${dir.path}/clipboard_$what.motolii';
  }

  static const _times = [('Align Starts', 'start'), ('Align Ends', 'end'), ('Move to Playhead', 'playhead'), ('Stagger', 'stagger'), ('Extend Start to Playhead', 'extend'), ('Trim End to Playhead', 'trim_end')];
  static const _aligns = [('Align Left', 'left'), ('Align Centre', 'centre'), ('Align Right', 'right'), ('Align Top', 'top'), ('Align Middle', 'middle'), ('Align Bottom', 'bottom'), ('Spread Across', 'across'), ('Spread Down', 'down')];

  /// The things picked: together if more than one is, else the one.
  List<int> _picked() => [for (final id in _ws.many.isNotEmpty ? _ws.many : {?_ws.selected}) ?int.tryParse(id)];

  /// One step for each picked thing told so, all of it one step to undo.
  void _eachPicked(int Function(int) tell) {
    final m = _inside, all = _picked();
    if (m == null || all.isEmpty) return;
    m.later('together', number: 1);
    for (final t in all) {
      tell(t);
    }
    _ws.after(m.later('together'));
  }

  void _copy(String call, String what) {
    final t = int.tryParse(_ws.selected ?? '');
    if (_inside != null && t != null) _inside!.later(call, thing: t, word: _clipboard(what));
  }

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
      'edit.cut': () {   // (no words being written: the picked thing copied, then taken away, one step each as Live's)
        if (_typed(const CopySelectionTextIntent.cut(SelectionChangedCause.keyboard))) return;
        final t = int.tryParse(ws.selected ?? '');
        if (t == null) return;
        m.later('copy', thing: t, word: _clipboard('thing'));
        ws.after(m.later('remove', thing: t));
        ws.select(null);
      },
      'edit.copy': () => _typed(CopySelectionTextIntent.copy) ? null : _copy('copy', 'thing'),
      'edit.paste': () => _typed(const PasteTextIntent(SelectionChangedCause.keyboard))
          ? null
          : File(_clipboard('thing')).existsSync()
          ? ws.after(m.later('paste', word: _clipboard('thing'), number: ws.frame / Ws.fps), null, (made) => ws.select('$made'))   // (at the playhead, the pasted one picked: Live's paste at the insert marker)
          : null,
      'edit.copyStyle': () => _copy('copy_style', 'style'),
      'edit.pasteStyle': () {   // (on every picked thing, one step)
        final all = {...ws.many, ?ws.selected};
        if (!File(_clipboard('style')).existsSync() || all.isEmpty) return;
        for (final id in all) {
          if (int.tryParse(id) case final t?) ws.after(m.later('paste_style', thing: t, word: _clipboard('style')));
        }
      },
      'edit.selectAll': () => _typed(const SelectAllTextIntent(SelectionChangedCause.keyboard)) ? null : ws.pickMany([for (final l in ws.layers) l.id]),   // (no words being written: every thing, as Ableton's Cmd+A in the arrangement)
      'thing.deactivate': () {
        final t = int.tryParse(ws.selected ?? '');
        if (t == null) return;
        final hidden = ws.layers.any((l) => l.id == ws.selected && l.hidden);
        ws.after(m.later('set', thing: t, name: 'visible', number: hidden ? 1 : 0));
      },
      'thing.remove': () {
        if (wsMarkPicked.value case (final at, _)?) {   // (a picked mark first: Live's locator, deleted like a clip)
          wsMarkPicked.value = null;
          return ws.after(m.later('mark', number: at));
        }
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
      'thing.extractSound': () {   // (a film's sound to a thing of its own, the film no longer heard: Alight Motion's Extract Audio)
        final t = int.tryParse(ws.selected ?? '');
        if (t == null || m.word(t, 'kind') != 'film') return;
        ws.after(m.later('extract_sound', thing: t), null, (made) => ws.select('$made'));
      },
      'thing.split': () {
        final t = int.tryParse(ws.selected ?? '');
        if (t == null) return;
        ws.after(m.later('split', thing: t, number: ws.frame / Ws.fps), null, (made) => ws.select('$made'));   // (the part after the playhead picked, as Live does)
      },
      for (final (_, how) in _aligns) 'arrange.$how': () {   // (the picked: one against the picture, more against their own box)
        final all = _picked();
        if (all.isNotEmpty) ws.after(m.later('align', word: '$how ${all.join(' ')}'));
      },
      for (final how in const ['fit', 'fill', 'stretch']) 'arrange.$how': () => _eachPicked((t) => m.later('fit', thing: t, word: how)),
      for (final (_, how) in _times) 'time.$how': () {   // (the picked, in the order picked: Alight Motion's timing)
        final all = _picked();
        if (all.isNotEmpty) ws.after(m.later('time_align', word: '$how ${all.join(' ')}', number: ws.frame / Ws.fps));
      },
      'arrange.flipX': () => _eachPicked((t) => m.later('flip', thing: t, word: 'x')),
      'arrange.flipY': () => _eachPicked((t) => m.later('flip', thing: t, word: 'y')),
      'layers.sequence': () => ws.after(m.later('sequence')),   // (After Effects' Keyframe Assistant: one after another, from the top)
      'play.toggle': () => ws.playing = !ws.playing,
      ...wsAbletonCommands(ws),   // (Ableton Live's keys: ableton.dart)
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
          item('Export Video…', 'file.render'),   // (Live's File menu: Export Audio/Video)
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
              item('Copy Style', 'edit.copyStyle'),
              item('Paste Style', 'edit.pasteStyle'),
              item('Select All', 'edit.selectAll'),
            ],
          ),
          PlatformMenuItemGroup(
            members: [
              item('Duplicate', 'thing.duplicate'),
              item('Split', 'thing.split'),
              item('Extract Sound', 'thing.extractSound'),
              item('Rename', 'thing.rename'),
              item('Remove', 'thing.remove'),
            ],
          ),
          item('Solo', 'thing.solo'),
          item('Deactivate', 'thing.deactivate'),
        ],
      ),
      PlatformMenu(
        label: 'View',
        menus: [
          PlatformMenuItemGroup(   // (Live's View menu: the arrangement's zoom)
            members: [
              item('Zoom In', 'zoom.in'),
              item('Zoom Out', 'zoom.out'),
              item('Zoom to Selection', 'zoom.selection'),
              item('Zoom Back', 'zoom.back'),
              item('Optimize Width', 'zoom.width'),
            ],
          ),
          PlatformMenuItemGroup(
            members: [
              item('Play / Stop', 'play.fromStart'),
              item('Continue Play', 'play.continue'),
              item('Loop', 'loop.toggle'),
            ],
          ),
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
        label: 'Arrange',   // (Keynote's and Figma's Align, Alight Motion's Fit and Flip)
        menus: [
          PlatformMenuItemGroup(members: [for (final (label, how) in _aligns) item(label, 'arrange.$how')]),
          PlatformMenuItemGroup(members: [item('Fit to Picture', 'arrange.fit'), item('Fill Picture', 'arrange.fill'), item('Stretch to Picture', 'arrange.stretch')]),
          PlatformMenuItemGroup(members: [item('Flip Horizontal', 'arrange.flipX'), item('Flip Vertical', 'arrange.flipY')]),
        ],
      ),
      PlatformMenu(
        label: 'Animation',
        menus: [
          item('Sequence Layers', 'layers.sequence'),
          PlatformMenuItemGroup(members: [for (final (label, how) in _times) item(label, 'time.$how')]),
        ],
      ),
    ];
  }

  Future<void> _join(String doc) async {
    final inside = Motolii.open(doc);
    if (inside == null) return;
    _inside = inside;
    _own = EffectTiles.watch(inside, () {   // (a file put in the maker's own folder: on the shelves at once)
      if (mounted) _ws.reread();
    });
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
    _own?.cancel();
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

/// The paint each layer wears on the stage, so the Inspector's Fill names the colour you see.
