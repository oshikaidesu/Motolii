// The Program shelf: what came as code (the door, filament/door.h), kept as it was written and listed by what it was
// written for: neither an effect, nor a thing to create, nor a preset. A list, or the picked one's code: what is typed
// is saved as it is typed, and the thing on the stage and its knobs in the inspector follow (doc_script_of reads a file
// that changed). As Godot and Unity keep scripts as files on a shelf with their knobs in the inspector; where the code is
// typed they differ, and it is here so the stage stays in sight (no source for that). The code view is re_editor's
// CodeEditor (MIT), coloured by re_highlight (highlight.js's rules and themes).
import 'dart:async';
import 'dart:io';

import 'package:flutter/widgets.dart';
import 'package:re_editor/re_editor.dart';
import 'package:re_highlight/languages/glsl.dart';
import 'package:re_highlight/languages/javascript.dart';
import 'package:re_highlight/styles/atom-one-dark.dart';
import 'package:re_highlight/styles/atom-one-light.dart';

import '../tokens.dart';
import 'dock_glyphs.dart';
import 'dock_parts.dart';
import 'effect_tiles.dart';
import 'ws.dart';

/// One program on the shelf: its file, what it was written for, and the hash that tells it from the others of those.
class Program {
  Program(this.file);
  final String file;
  String get stem => file.substring(file.lastIndexOf('/') + 1, file.lastIndexOf('.'));
  String get form => stem.contains('-') ? stem.substring(0, stem.lastIndexOf('-')) : stem;
  String get tail => stem.contains('-') ? stem.substring(stem.lastIndexOf('-') + 1) : '';

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
  void _put(Program p) {
    final m = c.ws.inside!;
    c.ws.after(m.later('door', thing: int.tryParse(c.ws.selected ?? '') ?? 0, word: p.file), (done, why) => _told(p, done, why), (made) => made > 0 ? c.ws.select('$made') : null);
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
    final shown = all.where((p) => (c.chip == 'All' || c.chip == p.form) && (c.hit(p.stem) || c.hit(p.first))).toList();
    final sel = (c.mem.programCode ? _stood(all) : null) ?? all.where((p) => p.file == c.sel).firstOrNull;
    if (all.isEmpty) return Padding(padding: EdgeInsets.all(Shape.s(24)), child: Text('Copy code from the web, then Paste', textAlign: TextAlign.center, style: T.label(Grey.g63)));
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (!c.mem.programCode) DockChips([('All', all.length), for (final e in forms.entries) (e.key, e.value)], active: c.chip, onPick: c.pickChip),
        Expanded(
          child: c.mem.programCode && sel != null
              ? ProgramCode(sel, () => _saved(sel), key: ValueKey(sel.file))
              : shown.isEmpty
              ? DockEmpty(c.query)
              : ListView(children: [for (final p in shown) _ProgramRow(p, c, onPut: () => _put(p))]),
        ),
        if (sel != null)
          DockFooter(
            preview: DockBadge(DockG.program, WsT.toneText, size: Shape.z(34)),
            title: sel.form,
            meta: c.mem.trouble[sel.file]?.replaceAll('the script ${sel.file}: ', '') ?? sel.tail,
            action: 'Create',
            fresh: true,
            onAction: () => _put(sel),
          ),
      ],
    );
  }
}

class _ProgramRow extends StatelessWidget {
  const _ProgramRow(this.p, this.c, {required this.onPut});
  final Program p;
  final DockCtx c;
  final VoidCallback onPut;
  @override
  Widget build(BuildContext context) {
    final sel = c.sel == p.file;
    return DockHover(
      onTap: () => c.pick(p.file),
      onDoubleTap: onPut,   // (two presses make it, as on Create)
      picks: true,
      builder: (context, h) => Container(
        padding: EdgeInsets.symmetric(horizontal: WsT.inset, vertical: Shape.s(6)),
        margin: EdgeInsets.only(bottom: WsT.gutter),
        color: sel ? WsT.accent : (h ? WsT.raised : WsT.card),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Expanded(child: Text(p.form, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(dockInk(sel, Grey.g95)))),
                Text(p.tail, style: T.micro(dockInk(sel, Grey.g56))),
              ],
            ),
            SizedBox(height: Shape.s(3)),
            Text(p.first, maxLines: 2, overflow: TextOverflow.ellipsis, style: T.value(dockInk(sel, Grey.g76))),
          ],
        ),
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
