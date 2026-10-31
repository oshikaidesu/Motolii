// The whole window: top bar, then the panes (panes.dart) — browser shelves | Stage over Timeline, Inspector over Desk, rearrangeable.
// Each seat is its own file and reads the shared Ws.
import 'dock_parts.dart' show askDockSearch;
import 'dart:developer' show Timeline;
import 'dart:async';
import 'dart:io';
import 'dart:math' as math;

import 'package:docking/docking.dart';
import 'package:flutter/widgets.dart';
import '../parts/skin_parts.dart';
import 'stage_art.dart' show WsArt;
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
import 'home.dart';
import 'panes.dart';
import 'desk_kit.dart';
import 'key_drag.dart';
import 'keys.dart';
import 'stage.dart';
import 'stage_camera.dart';
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
  Future<void> _open(String doc) {
    _ws.stageTexture = null;
    _inside?.close();
    _inside = null;
    return _join(doc);
  }

  /// The open document's own picture kept in it as it is left for the list of them, and as the window closes (AbsentMotion
  /// keeps a project's as its editor is closed for its home: MainActivity.kt saveThumbnail), at its playhead or at the
  /// moment it was told (file.poster). Not as another is opened in its place: its last asks would go unanswered.
  void _keepPoster() => _inside?.poster('${Directory.systemTemp.path}/motolii_poster_$pid.png');

  /// The documents beside the open one listed over the window (AbsentMotion's home: home.dart).
  bool _home = false;

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
  Future<bool> _render() async {
    final m = _inside, doc = _doc;
    if (m == null || doc == null) return false;
    final mp4 = _ws.renderTo ?? '${doc.substring(0, doc.lastIndexOf('.'))}.mp4';
    _ws.renderTo = null;
    _ws.notice = 'Writing ${mp4.split('/').last} …';
    final done = await m.write(mp4);
    if (mounted) _ws.notice = done ? 'Written: ${mp4.split('/').last}' : 'Not written: ${m.trouble}';
    return done;
  }

  /// Documents of the list written out: one as AbsentMotion's list does it (opened, and its export settings asked:
  /// MainActivity.kt exportFromList); more, each beside itself at its own settings, one after another as DaVinci Resolve's
  /// Render Queue goes through other projects' jobs (opening each in turn), and the one that was open opened again.
  Future<void> _exportFrom(List<String> docs) async {
    setState(() => _home = false);
    if (docs.length == 1) {
      if (docs.first != _doc) await _open(docs.first);
      wsAskRender.value++;
      return;
    }
    final back = _doc;
    var written = 0;
    for (final d in docs) {
      if (d != _doc) await _open(d);
      if (mounted && _doc == d && await _render()) written++;
    }
    if (back != null && back != _doc) await _open(back);
    if (mounted) _ws.notice = 'Written: $written of ${docs.length} films, each beside its document';
  }

  /// The document and its media gathered into "<name> package" beside it, to be taken elsewhere whole (Alight Motion's
  /// project package, After Effects' Collect Files); the top bar says so.
  void _package() {
    final m = _inside, doc = _doc;
    if (m == null || doc == null) return;
    final name = doc.split('/').last.replaceFirst(RegExp(r'\.motolii$'), ''), folder = '${File(doc).parent.path}/$name package';
    final n = m.pack(folder, name);
    _ws.notice = n < 0 ? 'Not packaged: ${m.trouble}' : 'Packaged with $n ${n == 1 ? 'file' : 'files'}: $name package';
  }

  /// The picture at the playhead written beside the document (Premiere Pro's Export Frame); the top bar says so.
  void _exportFrame() {
    final m = _inside, doc = _doc;
    if (m == null || doc == null) return;
    final png = '${doc.substring(0, doc.lastIndexOf('.'))}_${_ws.frame}.png';
    _ws.notice = m.still(png) ? 'Written: ${png.split('/').last}' : 'Not written: ${m.trouble}';
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
    // The picked held together by a Null they all go with, made in their middle (Figma's group, not a box of its own; one
    // step). mask 'mask' or 'exclude': Alight Motion's Group and Mask and Group and Exclude, the top one the shape the
    // others are seen in or cut out by (the inside's group_mask, doc_moment.h; reports/alight_ui.md).
    void group(String mask) {
      final m = _inside, all = _picked();
      final frames = [for (final t in all) ?m?.frame(t)];
      if (m == null || frames.isEmpty) return;
      final c = Offset(frames.map((f) => f[0]).reduce((a, b) => a + b) / frames.length, frames.map((f) => f[1]).reduce((a, b) => a + b) / frames.length);
      m.later('together', number: 1);
      ws.after(m.later('add', name: 'shape', word: 'null'), null, (g) => ws.select('$g'));
      m.later('say', thing: -1, name: 'name', word: mask.isEmpty ? 'Group' : 'Group and Mask');
      if (mask.isNotEmpty) m.later('say', thing: -1, name: 'group_mask', word: mask);
      m.later('set', thing: -1, name: 'x', number: (c.dx - WsArt.centre.dx) / m.point);
      m.later('set', thing: -1, name: 'y', number: (WsArt.centre.dy - c.dy) / m.point);
      for (final t in all) {
        m.later('set', thing: t, name: 'parent', word: 'made');   // (each goes with it, where it is)
      }
      ws.after(m.later('together'), (_, _) => ws.reread());
    }

    void step(int by) {
      ws.playing = false;
      ws.frame = ws.frame + by;
    }

    return {
      'file.new': () {
        _open(_fresh());
        wsAskSize.value = true;   // (its shape and resolution asked first, as Alight Motion's new-project box)
      },
      'file.render': () => wsAskRender.value++,   // (the settings window first)
      'file.exportFrame': _exportFrame,
      'file.home': () => setState(() {   // (its own picture kept first: its card shows it as it is)
        _keepPoster();
        _home = true;
      }),
      'file.poster': () {   // (the moment its picture is drawn at from now on: Alight Motion's Set Project Thumbnail)
        if (m.set(0, 'poster_at', ws.frame / 60)) _ws.notice = 'Project picture: ${ws.frame}';   // (the playhead as it is seen: sixty frames a second)
      },
      'file.package': _package,
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
      'thing.saveLook': () {   // (kept on the Effects shelf by the layer's name: Alight Motion's Save as Preset)
        final m = _inside, l = _ws.layer;
        if (m == null || l == null) return;
        ws.after(m.later('keep_look', thing: int.parse(l.id), word: l.name), (done, why) {
          m.forgetFolders();
          _ws.notice = done ? 'Look kept: ${l.name}' : 'Not kept: $why';
          ws.reread();
        });
      },
      'thing.saveElement': () {   // (kept whole, with what it holds, among the elements on the Create shelf: Alight Motion's Save to My Elements)
        final m = _inside, l = _ws.layer;
        if (m == null || l == null) return;
        ws.after(m.later('keep_element', thing: int.parse(l.id), word: l.name), (done, why) {
          m.forgetFolders();
          _ws.notice = done ? 'Saved to My Elements: ${l.name}' : 'Not saved: $why';
          ws.reread();
        });
      },
      'edit.pasteStyle': () {   // (on every picked thing, one step)
        final all = {...ws.many, ?ws.selected};
        if (!File(_clipboard('style')).existsSync() || all.isEmpty) return;
        for (final id in all) {
          if (int.tryParse(id) case final t?) ws.after(m.later('paste_style', thing: t, word: _clipboard('style')));
        }
      },
      'edit.selectAll': () => _typed(const SelectAllTextIntent(SelectionChangedCause.keyboard)) ? null : ws.pickMany([for (final l in ws.layers) l.id]),   // (no words being written: every thing, as Ableton's Cmd+A in the arrangement)
      'edit.deselectAll': () => ws
        ..clearKeys()
        ..select(null),   // (nothing picked, as a click below the rows: After Effects' and Premiere Pro's Edit > Deselect All; with a thing over the whole stage, the one way back: golf_noise.md (63))
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
        if (ws.stretch case final s?) return ws.after(m.later('way', thing: int.parse(s.layer), name: s.name, number: s.at, word: 'Linear'), (_, _) => ws.reread());   // (a stretch picked: its way off, straight again, not the thing)
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
      'thing.outline': () {   // (Alight Motion's Convert to Outline: a shape a path of the outline it shows, its points then the Pen's: doc_to_path)
        final t = int.tryParse(ws.selected ?? '');
        if (t == null || m.word(t, 'kind') != 'shape' || const ['path', 'null', 'solid'].contains(m.word(t, 'shape'))) return;
        ws.after(m.later('outline', thing: t), (_, _) => ws.reread());
      },
      'thing.extractSound': () {   // (a film's sound to a thing of its own, the film no longer heard: Alight Motion's Extract Audio)
        final t = int.tryParse(ws.selected ?? '');
        if (t == null || m.word(t, 'kind') != 'film') return;
        ws.after(m.later('extract_sound', thing: t), null, (made) => ws.select('$made'));
      },
      'thing.group': () => group(''),
      'thing.groupMask': () => group('mask'),
      'thing.groupExclude': () => group('exclude'),
      'thing.ungroup': () {   // (the picked group's things on their own again, where they are; the Null gone; one step)
        final m = _inside, g = int.tryParse(ws.selected ?? '');
        if (m == null || g == null || m.word(g, 'shape') != 'null') return;
        final kids = [for (final t in m.things) if ((m.value(t, 'parent', 0) ?? 0).round() == g) t];
        m.later('together', number: 1);
        for (final t in kids) {
          m.later('set', thing: t, name: 'parent', number: 0);
        }
        m.later('remove', thing: g);
        ws.after(m.later('together'), (_, _) => ws.reread());
        ws.pickMany([for (final t in kids) '$t']);
      },
      'thing.unlinkElement': () {   // (an element put in made a group of its own, no longer following its file: AbsentMotion's Unlink from Asset, ElementUi.kt)
        final m = _inside, g = int.tryParse(ws.selected ?? '');
        if (m == null || g == null || m.word(g, 'element').isEmpty) return;
        ws.after(m.later('say', thing: g, name: 'element', word: ''), (_, _) => ws.reread());
      },
      // (the work area taken out and the gap closed: After Effects' Edit > Extract Work Area, under Split Layer; golf (72))
      'edit.extractWork': () => ws.after(m.later('extract'), (done, why) => done || why.isEmpty ? null : ws.notice = 'Not taken: ${why.split(':').first}'),
      // (After Effects' Cmd+Opt+B: "set work area to duration of selected layers or, if no layers are selected, set
      // work area to composition duration"; golf (82): back to the whole after Extract Work Area took two drags)
      'edit.workToPicked': () {
        final picked = {for (final t in _picked()) '$t'}, bars = [for (final l in ws.layers) if (picked.contains(l.id)) l];
        final (from, to) = bars.isEmpty ? (0.0, m.seconds) : (bars.map((l) => l.inF).reduce(math.min) / Ws.fps, bars.map((l) => l.outF).reduce(math.max) / Ws.fps);
        ws.after(m.later('work', number: from, word: '$to'));
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
      ...wsToolCommands(ws),   // (the stage's tools: stage.dart)
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
              item('Projects…', 'file.home'),
              PlatformMenu(label: 'Open', menus: [for (final f in _beside) PlatformMenuItem(label: f.split('/').last, onSelected: f == _doc ? null : () => _open(f))]),
            ],
          ),
          item('Export Video…', 'file.render'),   // (Live's File menu: Export Audio/Video)
          item('Export Frame', 'file.exportFrame'),
          item('Package Project', 'file.package'),
          item('Use Frame as Project Picture', 'file.poster'),
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
              item('Save Look', 'thing.saveLook'),
              item('Save to My Elements', 'thing.saveElement'),
              item('Select All', 'edit.selectAll'),
              item('Deselect All', 'edit.deselectAll'),
            ],
          ),
          PlatformMenuItemGroup(
            members: [
              item('Duplicate', 'thing.duplicate'),
              item('Convert to Outline', 'thing.outline'),
              item('Group', 'thing.group'),
              item('Group and Mask', 'thing.groupMask'),
              item('Group and Exclude', 'thing.groupExclude'),
              item('Ungroup', 'thing.ungroup'),
              item('Unlink from Element', 'thing.unlinkElement'),
              item('Split', 'thing.split'),
              item('Extract Work Area', 'edit.extractWork'),
              item('Set Work Area to Selected', 'edit.workToPicked'),
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
      // (a menu titled Help: the Mac puts its search of every menu item in it (Apple's HIG, the Help menu), so a feature
      //  forgotten is found by its name; and Motolii's ledger of what it has, as a Help menu's own help: golf (98))
      PlatformMenu(
        label: 'Help',
        menus: [PlatformMenuItem(label: 'Motolii Features (ASSETS.md)', onSelected: () => Process.run('open', ['${File(motoliiDir).parent.path}/ASSETS.md']))],
      ),
    ];
  }

  Future<void> _join(String doc) async {
    final inside = Motolii.open(doc);
    if (inside == null) return;
    _inside = inside;
    _own = EffectTiles.watch(inside, () {   // (a file put in the maker's own folder: on the shelves at once; an element written over: followed where it is put in)
      if (mounted) _ws.after(inside.later('follow_elements'), (_, _) => _ws.reread());
    });
    _ws.after(inside.later('follow_elements'));   // (as the document is opened: AbsentMotion's elements followed as a project is loaded)
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
    _keepPoster();
    FocusManager.instance.removeListener(_keepKeys);
    _windowFocus.dispose();
    _own?.cancel();
    _inside?.close();
    _ws.dispose();
    super.dispose();
  }

  /// The panes drag their tabs on an overlay; the window brings its own so it works under any host.
  _Window? _built;   // (one widget, kept: a card or a tab carried is a change of the overlay, and the whole window, every shelf's tiles, was built again for it: 18,358 widgets, 90 ms, each time one was taken up or let go)
  late final _entry = OverlayEntry(
    maintainState: true,
    builder: (context) => LayoutBuilder(
      builder: (context, box) {
        final window = _built != null && _built!.extra == widget.extra ? _built! : _built = _Window(widget.extra);
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
          child: Stack(
            children: [
              Overlay(initialEntries: [_entry]),
              if (_home && _doc != null && _ws.inside != null)
                Positioned.fill(
                  child: WsHome(
                    folder: File(_doc!).parent.path,
                    lib: _ws.inside!.libPath,
                    open: _doc!,
                    onOpen: (d) {
                      setState(() => _home = false);
                      if (d != _doc) _open(d);
                    },
                    onExport: _exportFrom,
                    onNew: () {
                      setState(() => _home = false);
                      _commands()['file.new']!();
                    },
                    onClose: () => setState(() => _home = false),
                  ),
                ),
            ],
          ),
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
  WsPane('Camera View', DockG.camera, () => const WsCameraView(), minSize: 240),
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
      DockingTabs([item('Stage'), item('Camera View')]),
    ]),
    DockingRow([item('Timeline'), item('Ease', size: WsT.inspector)], size: WsT.lower + _strip, minimalSize: 220),   // (the stretch between two keys, beside the keys; never lower than its seven shapes and its curve can be read: docking's own minimum, as VS Code's views say theirs)
  ]),
  DockingColumn([extra.isEmpty ? item('Inspector') : DockingTabs([item('Inspector'), for (final p in extra) item(p.name)]), DockingTabs([item('Camera'), item('Desk'), item('Relations')], size: WsT.lower + _strip)], size: WsT.inspector, minimalSize: 340),   // (the camera, whatever is picked)
]);

/// The paint each layer wears on the stage, so the Inspector's Fill names the colour you see.
