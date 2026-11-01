// The Program shelf: what came as code (the door, filament/door.h), kept as it was written and listed by what it was
// written for: neither an effect, nor a thing to create, nor a preset. A list, or the picked one's code: what is typed
// is saved as it is typed, and the thing on the stage and its knobs in the inspector follow (doc_script_of reads a file
// that changed). As Godot and Unity keep scripts as files on a shelf with their knobs in the inspector; where the code is
// typed they differ, and it is here so the stage stays in sight (no source for that). The code view is re_editor's
// CodeEditor (MIT), coloured by re_highlight (highlight.js's rules and themes). Its tiles: program_tiles.dart. Its name is
// kept beside the file and not in it (the file's name is what documents hold it by, its lines what its knobs are named
// by; no source for the file beside it). A right click on a tile: Live's browser renames by its context menu (Live 12
// manual, 4.2.1 and 4.5) and shows a file in Finder or Explorer (4.7.7).
import 'dart:async';
import 'dart:io';

import 'package:flutter/services.dart' show LogicalKeyboardKey;
import 'package:flutter/widgets.dart';
import 'package:re_editor/re_editor.dart';
import 'package:re_highlight/languages/glsl.dart';
import 'package:re_highlight/languages/javascript.dart';
import 'package:re_highlight/styles/atom-one-dark.dart';
import 'package:re_highlight/styles/atom-one-light.dart';

import '../sets/foundations/text_hand.dart';
import '../tokens.dart';
import 'dock_effects.dart';
import 'dock_glyphs.dart';
import 'dock_parts.dart';
import 'effect_tiles.dart';
import 'program_tiles.dart';
import 'ws.dart';
import 'ws_menu.dart';

/// One program on the shelf: its file, what it was written for, and the hash that tells it from the others of those.
class Program {
  Program(this.file);
  final String file;
  String get stem => file.substring(file.lastIndexOf('/') + 1, file.lastIndexOf('.'));
  String get form => stem.contains('-') ? stem.substring(0, stem.lastIndexOf('-')) : stem;
  String get tail => stem.contains('-') ? stem.substring(stem.lastIndexOf('-') + 1) : '';
  File get _named => File('${file.substring(0, file.lastIndexOf('.'))}.name');

  /// What it is called with no name given: what it was written for and its hash.
  String get plain => '$form $tail'.trim();

  /// What it is called: its maker's word, or what it says of itself (the site's title door_net.h wrote at its head, a
  /// file of many tabs' description).
  String get name {
    try {
      if (_named.existsSync()) return _named.readAsStringSync().trim();
      final lines = File(file).readAsLinesSync();
      final kept = RegExp(r'^/\*\{.*"DESCRIPTION":"([^"]*)"').firstMatch(lines.first)?[1];
      if (kept != null && kept != 'Shadertoy') return kept;
      return [for (final l in lines.take(8)) RegExp(r'^// (.+) by [^,]+, [^,]+, openprocessing\.org/sketch/\d+').firstMatch(l)?[1]].nonNulls.firstOrNull ?? plain;
    } catch (_) {
      return plain;
    }
  }

  /// Nothing written: what it is called with none.
  void rename(String to) {
    if (to.trim().isNotEmpty) return _named.writeAsStringSync(to.trim());
    if (_named.existsSync()) _named.deleteSync();
  }

  /// The first line of it that says something.
  String get first {
    try {
      final kept = RegExp(r'^/\*\{.*"DESCRIPTION":"([^"]*)"(?:,"CREDIT":"([^"]*)")?').firstMatch(File(file).readAsLinesSync().first);   // (what a file of many tabs says it is)
      if (kept != null && kept[1] != 'Shadertoy') return kept[2] == null ? kept[1]! : '${kept[1]} · ${kept[2]}';
      return File(file).readAsLinesSync().map((l) => l.trim()).firstWhere((l) => l.isNotEmpty && !l.startsWith('//') && !l.startsWith('#') && !l.startsWith('/*'), orElse: () => '');
    } catch (_) {
      return '';
    }
  }
}

/// The maker's programs, the last written first. Kept code stands for what it became: Hydra's, a p5 sketch's shader.
List<Program> programs() {
  final dir = Directory('${EffectTiles.own}/programs');
  if (!dir.existsSync()) return const [];
  final all = dir.listSync().whereType<File>().where((f) => RegExp(r'\.(js|fs|hydra)$').hasMatch(f.path)).toList();
  final made = {for (final f in all) if (RegExp(r'\.(hydra|js)$').hasMatch(f.path)) f.path.replaceFirst(RegExp(r'\.(hydra|js)$'), '.fs')};
  final kept = all.where((f) => !made.contains(f.path)).toList()..sort((a, b) => b.lastModifiedSync().compareTo(a.lastModifiedSync()));
  return [for (final f in kept) Program(f.path)];
}

class ProgramShelf extends StatefulWidget {
  const ProgramShelf(this.c, {super.key});
  final DockCtx c;
  @override
  State<ProgramShelf> createState() => _ProgramShelfState();
}

class _ProgramShelfState extends State<ProgramShelf> {
  DockCtx get c => widget.c;
  void _told(Program p, bool done, String why) => c.set(() => done ? c.mem.trouble.remove(p.file) : c.mem.trouble[p.file] = why);
  final _menu = GlobalKey<WsMenuState>();
  String? _renaming;   // (the file whose name is being written)
  final _words = TextEditingController();
  late final FocusNode _focus = FocusNode()..addListener(() => _focus.hasFocus ? null : _renamed());

  @override
  void dispose() {
    _words.dispose();
    _focus.dispose();
    super.dispose();
  }

  void _put(Program p) {
    final m = c.ws.inside!;
    c.ws.after(m.later('door', thing: int.tryParse(c.ws.selected ?? '') ?? 0, word: p.file), (done, why) => _told(p, done, why), (made) {
      if (made <= 0) return;
      if (p.name != p.plain) c.ws.after(m.later('say', thing: made, name: 'name', word: p.name));
      c.ws.select('$made');
    });
  }

  void _rename(Program p) {
    setState(() => _renaming = p.file);
    _words
      ..text = p.name
      ..selection = TextSelection(baseOffset: 0, extentOffset: p.name.length);
    WidgetsBinding.instance.addPostFrameCallback((_) => _focus.requestFocus());   // (its field is there from the next frame)
  }

  void _renamed() {
    final file = _renaming;
    if (file == null) return;
    Program(file).rename(_words.text);
    setState(() => _renaming = null);
  }

  Widget _naming(Program p) => CallbackShortcuts(
    bindings: {
      const SingleActivator(LogicalKeyboardKey.escape): () {
        _words.text = p.name;
        _focus.unfocus();
      },
    },
    child: Container(
      color: Grey.g91,
      padding: EdgeInsets.symmetric(horizontal: Shape.s(4)),
      alignment: Alignment.centerLeft,
      child: TextHand(key: const ValueKey('ws-program-name'), field: (k) => EditableText(key: k, rendererIgnoresPointer: true,
        controller: _words,
        focusNode: _focus,
        style: T.micro(Grey.g13),
        cursorColor: Grey.g13,
        backgroundCursorColor: Grey.g38,
        maxLines: 1,
        onSubmitted: (_) => _focus.unfocus(),
      )),
    ),
  );

  void _menuOn(Program p, Offset at) {
    c.pick(p.file);
    final box = _menu.currentContext?.findRenderObject() as RenderBox?;
    _menu.currentState?.show(box?.globalToLocal(at) ?? Offset.zero, [
      WsMenuItem('Create', () => _put(p)),
      WsMenuItem('Rename…', () => _rename(p)),
      const WsMenuItem.rule(),
      WsMenuItem(Platform.isWindows ? 'Show in Explorer' : 'Show in Finder', () => Platform.isWindows ? Process.run('explorer', ['/select,${p.file.replaceAll('/', r'\')}']) : Process.run('open', ['-R', p.file])),
    ]);
  }

  /// What was made from it follows, its knobs in the inspector too; why its code does not run, under the code.
  void _saved(Program p) {
    final m = c.ws.inside;
    if (m == null) return;
    c.ws.after(m.later('code', word: p.file), (done, why) {
      _told(p, done, why);
      c.ws.reread();
    });
  }

  /// The program the picked thing is made from: its code is the one shown (Godot opens a node's script from the node).
  Program? _stood(List<Program> all) {
    final m = c.ws.inside, t = int.tryParse(c.ws.selected ?? '');
    if (m == null || t == null) return null;
    final said = '${m.word(t, 'file')} ${m.word(t, 'script')} ${m.word(t, 'fx')}';
    return all.where((p) => said.contains(p.file) || said.contains(p.file.replaceFirst(RegExp(r'\.(hydra|js)$'), '.fs'))).firstOrNull;
  }

  @override
  Widget build(BuildContext context) {
    final all = programs();
    final forms = <String, int>{};
    for (final p in all) {
      forms[p.form] = (forms[p.form] ?? 0) + 1;
    }
    final shown = all.where((p) => (c.chip == 'All' || c.chip == p.form) && (c.hit(p.name) || c.hit(p.stem) || c.hit(p.first))).toList();
    final sel = (c.mem.programCode ? _stood(all) : null) ?? all.where((p) => p.file == c.sel).firstOrNull;
    if (all.isEmpty) return Padding(padding: EdgeInsets.all(Shape.s(24)), child: Text('Copy code from the web, then Paste', textAlign: TextAlign.center, style: T.label(Grey.g63)));
    if (!c.mem.programCode) {
      ProgramTiles.ask(all.map((p) => p.file), () {
        if (mounted) setState(() {});
      });
    }
    String tile(Program p) => File(ProgramTiles.home(p.file)).existsSync() ? ProgramTiles.home(p.file) : '';
    return WsMenu(
      key: _menu,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (!c.mem.programCode) DockChips([('All', all.length), for (final e in forms.entries) (e.key, e.value)], active: c.chip, onPick: c.pickChip),
          Expanded(
            child: c.mem.programCode && sel != null
                ? ProgramCode(sel, () => _saved(sel), key: ValueKey(sel.file))
                : shown.isEmpty
                ? DockEmpty(c.query)
                : SingleChildScrollView(
                    child: LayoutBuilder(
                      builder: (context, box) {
                        final cols = wsCols(box.maxWidth, Shape.z(84)), w = wsCell(box.maxWidth, cols);
                        return Wrap(
                          spacing: WsT.gutter,
                          runSpacing: WsT.gutter,
                          children: [
                            for (final p in shown)
                              SizedBox(
                                width: w,
                                child: GestureDetector(
                                  onSecondaryTapDown: (d) => _menuOn(p, d.globalPosition),
                                  child: EffectTile(p.name, key: ValueKey('ws-program-${p.stem}-${tile(p).isEmpty ? 0 : File(tile(p)).lastModifiedSync().millisecondsSinceEpoch}'), file: tile(p), on: null, onTap: () => c.pick(p.file), onApply: () => _put(p), picked: c.sel == p.file, naming: _renaming == p.file ? _naming(p) : null),   // (two presses make it, as on Create; drawn again under the same name, an Image with the same file goes on showing the first: the date in its key)
                                ),
                              ),
                          ],
                        );
                      },
                    ),
                  ),
          ),
          if (sel != null && !c.mem.programCode && tile(sel).isNotEmpty)   // (large, as the Effects shelf's)
            SizedBox(height: Shape.z(140), child: TileMoving(tile(sel), playing: true)),
          if (sel != null)
            DockFooter(
              preview: DockBadge(DockG.program, WsT.toneText, size: Shape.z(34)),
              title: sel.name,
              meta: c.mem.trouble[sel.file]?.replaceAll('the script ${sel.file}: ', '') ?? sel.first,
              action: 'Create',
              fresh: true,
              onAction: () => _put(sel),
            ),
        ],
      ),
    );
  }
}

/// A program's code: what is typed is written to its file a moment after the last key, and told.
class ProgramCode extends StatefulWidget {
  const ProgramCode(this.p, this.saved, {super.key});
  final Program p;
  final VoidCallback saved;
  @override
  State<ProgramCode> createState() => _ProgramCodeState();
}

class _ProgramCodeState extends State<ProgramCode> {
  late String _written = File(widget.p.file).readAsStringSync();
  late final _text = CodeLineEditingController.fromText(_written);
  Timer? _soon;

  void _save() {
    if (_written == _text.text) return;
    File(widget.p.file).writeAsStringSync(_written = _text.text);
    widget.saved();
  }

  @override
  void didUpdateWidget(ProgramCode old) {
    super.didUpdateWidget(old);
    final now = File(widget.p.file).readAsStringSync();
    if (now != _written && !(_soon?.isActive ?? false)) _text.text = _written = now;   // (written by another editor)
  }

  @override
  void dispose() {
    if (_soon?.isActive ?? false) _save();
    _soon?.cancel();
    _text.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => CodeEditor(
    controller: _text,
    wordWrap: true,   // (a shelf's width: a line pushed sideways is out of sight)
    onChanged: (_) {
      _soon?.cancel();
      _soon = Timer(const Duration(milliseconds: 400), _save);
    },
    indicatorBuilder: (context, editing, chunks, notifier) => DefaultCodeLineNumber(controller: editing, notifier: notifier),
    style: CodeEditorStyle(
      fontSize: T.value().fontSize,
      fontFamily: T.mono,
      fontHeight: 1.4,
      textColor: Grey.g91,
      backgroundColor: WsT.well,
      selectionColor: WsT.accent.withAlpha(90),
      cursorColor: Grey.g91,
      codeTheme: CodeHighlightTheme(
        languages: {widget.p.file.endsWith('.fs') ? 'glsl' : 'javascript': CodeHighlightThemeMode(mode: widget.p.file.endsWith('.fs') ? langGlsl : langJavascript)},
        theme: Grey.g07.computeLuminance() > .5 ? atomOneLightTheme : atomOneDarkTheme,
      ),
    ),
  );
}
