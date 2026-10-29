// The Fonts shelf: each row set in its own face. Effects live in dock_effects.dart, files and media in dock_files.dart.
import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'desk_kit.dart' show DeskChip;
import 'dock_files.dart' show dockRead;
import 'dock_parts.dart';
import 'dock_tags.dart';
import 'ws.dart';

// ---------------------------------------------------------------- fonts

class _Font {
  const _Font(this.family, {this.path});
  final String family;
  final String? path; // (the font's file)
}


/// Joined: the font files of this machine that the inside sets words in (.ttf), and those beside the document: read
/// once, without waiting. A font is called by its file's name.
List<_Font>? _disk;
Future<void>? _reading;
List<_Font> _diskFonts(Ws ws) {
  fontsRead(ws);
  return _disk ?? const [];
}

/// How many fonts the shelf offers, once they are read (joined).
int? get fontsOffered => _disk?.length;

/// The reading itself: once, on the program's own clock (a test may wait for it). Only the fonts the inside sets the
/// shelf's own line in are offered (a font of other letters, or one ThorVG cannot read, is not).
Future<void> fontsRead(Ws ws) => _reading ??= Zone.root.run<Future<void>>(() async {
  final files = <String>[];
  for (final dir in ['/System/Library/Fonts/Supplemental', '/Library/Fonts', '${Platform.environment['HOME']}/Library/Fonts', ?ws.documentFolder]) {
    try {
      for (final e in await Directory(dir).list().toList()) {
        if (e is File && e.path.toLowerCase().endsWith('.ttf')) files.add(e.path);
      }
    } on FileSystemException catch (_) {} // (a folder that is not there, or may not be read: it has no fonts to offer)
  }
  final sets = await ws.inside!.setting(files, _sample);
  _disk = [
    for (final f in sets) _Font(f.split('/').last.substring(0, f.split('/').last.length - 4), path: f),
  ]..sort((a, b) => a.family.toLowerCase().compareTo(b.family.toLowerCase()));
  dockRead.value++;
});

/// Where the fonts set lately are kept, the newest first, eight (a test points it elsewhere: [fontRecentAt]), as the
/// Ease desk keeps its recent ways (ease_stars.json).
File _recentFile = File('${Platform.environment['HOME']}/.motolii/font_recent.json');
List<String>? _recent;
void fontRecentAt(File f) => (_recentFile = f, _recent = null);
List<String> fontRecent() => _recent ??= () {
  try {
    return [for (final s in jsonDecode(_recentFile.readAsStringSync()) as List) '$s'];
  } catch (_) {
    return <String>[];
  }
}();
void _used(String family) {
  final r = fontRecent()
    ..remove(family)
    ..insert(0, family);
  if (r.length > 8) r.removeRange(8, r.length);
  try {
    _recentFile.parent.createSync(recursive: true);
    _recentFile.writeAsStringSync(jsonEncode(r));
  } catch (_) {}   // (not kept: known while the window is open)
}

/// Joined: the font given to the picked text; with no text picked, a new text set in it.
void _setFont(DockCtx c, _Font f) {
  _used(f.family);
  final m = c.ws.inside!, t = int.tryParse(c.ws.selected ?? '');
  void told(bool done, String why) => c.set(() => done ? c.mem.trouble.remove(f.family) : c.mem.trouble[f.family] = why);
  if (t != null && m.word(t, 'kind') == 'text') return c.ws.after(m.later('say', thing: t, name: 'file', word: f.path!), told);   // (without waiting)
  m.later('together', number: 1);   // (one step: the text and its inks)
  c.ws.after(m.later('add', name: 'text', word: f.path!), null, (made) => c.ws.select('$made'));
  wsInked(c.ws, 'text', '');
  c.ws.after(m.later('together'), told);
}

/// With no text picked, a font pressed is the font of the role the shelf sets now (one press, as a colour pressed
/// colours on the Colors shelf); a picked text's own font is set as before, by two presses or Apply.
void _toRole(DockCtx c, _Font f) {
  final m = c.ws.inside, t = int.tryParse(c.ws.selected ?? '');
  if (m == null || f.path == null || (t != null && m.word(t, 'kind') == 'text')) return;
  _used(f.family);
  c.set(() => c.ws.fontRole == 'body' ? c.ws.bodyFont = f.path! : c.ws.titleFont = f.path!);
}

/// A new text in a role's font, at the playhead, in the inks, picked: Canva's Add a heading, Add a little bit of body text.
void _addIn(DockCtx c, String font) {
  final m = c.ws.inside!;
  m.later('together', number: 1);   // (one step: the text and its inks)
  c.ws.after(m.later('add', name: 'text', word: font), null, (made) => c.ws.select('$made'));
  wsInked(c.ws, 'text', '');
  c.ws.after(m.later('together'), (done, why) => c.set(() => done ? c.mem.trouble.remove('role') : c.mem.trouble['role'] = why));
}

/// The two fonts a title and a body text are set in, first on the shelf as PowerPoint's Headings and Body theme fonts are
/// first in its list of fonts (and as Canva's brand fonts, a heading's and a body's): with no text picked, a press on one
/// says which role a font pressed now is for; with a text picked, it puts that role's font on it (Canva's brand text
/// styles); its + adds a text in it (golf_noise.md ㉚, P04).
class _Roles extends StatelessWidget {
  const _Roles(this.c, this.fonts);
  final DockCtx c;
  final List<_Font> fonts;
  @override
  Widget build(BuildContext context) {
    final m = c.ws.inside!, t = int.tryParse(c.ws.selected ?? ''), onText = t != null && m.word(t, 'kind') == 'text';
    Widget role(String r, String label, String face) {
      final f = fonts.where((f) => f.path == face).firstOrNull ?? _Font(face.split('/').last.replaceFirst(RegExp(r'\.[^.]*$'), ''), path: face);
      final on = !onText && c.ws.fontRole == r;
      return Expanded(
        child: DockHover(
          key: ValueKey('font-role-$r'),
          onTap: () => onText ? c.ws.after(m.later('say', thing: t, name: 'file', word: face)) : c.set(() => c.ws.fontRole = r),
          builder: (context, h) => Container(
            height: Shape.z(46),
            padding: EdgeInsets.only(left: Shape.s(8), right: Shape.s(4)),
            color: dockFill(on, h, WsT.card),
            child: Row(
              children: [
                Expanded(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(label, style: T.label(dockInk(on, Grey.g63))),
                      SizedBox(height: Shape.s(2)),
                      Text(f.family, maxLines: 1, overflow: TextOverflow.ellipsis, style: _face(f, 14, dockInk(on, Grey.g95))),
                    ],
                  ),
                ),
                DeskChip(key: ValueKey('font-add-$r'), label: '+', onTap: () => _addIn(c, face)),
              ],
            ),
          ),
        ),
      );
    }

    return Padding(
      padding: EdgeInsets.only(bottom: WsT.gutter),
      child: Row(children: [role('title', 'Title', c.ws.titleFace), SizedBox(width: WsT.gutter), role('body', 'Body', c.ws.bodyFace)]),
    );
  }
}

const _sample = 'Motion for more relations';

class FontsShelf extends StatelessWidget {
  const FontsShelf(this.c, {super.key});
  final DockCtx c;
  @override
  Widget build(BuildContext context) => ValueListenableBuilder<int>(valueListenable: dockRead, builder: (context, _, _) => _shelf(context));

  Widget _shelf(BuildContext context) {
    final m = c.ws.inside;
    if (m == null) return const SizedBox.shrink();
    final fonts = _diskFonts(c.ws);
    // (which text of the document is set in which font file)
    final setIn = {
      for (final l in c.ws.layers)
        if (m.word(int.parse(l.id), 'kind') == 'text') m.whole(m.word(int.parse(l.id), 'file')): l,
    };
    bool used(_Font f) => setIn.containsKey(f.path);
    Set<String> tagsOf(_Font f) => DockTags.of('Fonts', f.family, f.path == null ? const <String>{} : (c.ws.inside?.fontTags(f.path!) ?? const <String>{}));
    final hits = fonts.where((f) => DockTags.hit(c.query, f.family, tagsOf(f))).toList();
    final vocab = <String, int>{};
    for (final f in hits) {
      for (final t in tagsOf(f)) {
        vocab[t] = (vocab[t] ?? 0) + 1;
      }
    }
    final top = vocab.entries.toList()..sort((a, b) => b.value.compareTo(a.value));
    bool keep(_Font f) => switch (c.chip) {
      'All' => true,
      'Used' => used(f),
      'Recent' => fontRecent().contains(f.family),
      'Starred' => c.starred(f.family),
      final tag => tagsOf(f).contains(tag),   // (a tag chip: the families with it)
    };
    final list = hits.where(keep).toList();
    if (c.chip == 'Recent') list.sort((a, b) => fontRecent().indexOf(a.family).compareTo(fontRecent().indexOf(b.family)));   // (the newest first)
    final sel = fonts.where((f) => f.family == c.sel).firstOrNull;
    final picked = sel == null ? null : setIn[sel.path], onText = picked != null && picked.id == c.ws.selected;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _Roles(c, fonts),
        DockChips(
          [
            ('All', hits.length),
            ('Used', hits.where(used).length),
            ('Recent', hits.where((f) => fontRecent().contains(f.family)).length),
            ('Starred', hits.where((f) => c.starred(f.family)).length),
            for (final t in top.take(8)) (t.key, t.value),
          ],
          active: c.chip,
          onPick: c.pickChip,
        ),
        Expanded(
          child: list.isEmpty
              ? DockEmpty(c.query)
              : LayoutBuilder(
                  builder: (context, box) {
                    String? usedBy(_Font f) => setIn[f.path]?.name;
                    if (box.maxWidth < 440) {
                      return ListView(
                        padding: EdgeInsets.zero,
                        children: [for (final f in list) _FontRow(f, c, usedBy: usedBy(f))],
                      );
                    }
                    final cols = wsCols(box.maxWidth, 200), w = wsCell(box.maxWidth, cols);
                    return ListView(
                      padding: EdgeInsets.zero,
                      children: [
                        Wrap(
                          spacing: WsT.gutter,
                          runSpacing: WsT.gutter,
                          children: [
                            for (final f in list)
                              SizedBox(
                                width: w,
                                child: _FontCard(f, c, usedBy: usedBy(f)),
                              ),
                          ],
                        ),
                      ],
                    );
                  },
                ),
        ),
        if (sel != null)
          DockFooter(
            preview: Container(
              color: WsT.toneText,
              alignment: Alignment.center,
              child: Text('Aa', style: _face(sel, 15, WsT.onAccent)),
            ),
            title: sel.family,
            meta: c.mem.trouble[sel.family] ?? 'TTF · ${File(sel.path!).parent.path.split('/').last}',
            // (the picked text is set in it; with no text picked, a new text is)
            action: m.word(int.tryParse(c.ws.selected ?? '') ?? 0, 'kind') != 'text' ? 'Place' : 'Apply',
            done: onText,
            doneLabel: 'In use',
            fresh: m.word(int.tryParse(c.ws.selected ?? '') ?? 0, 'kind') != 'text', // (no text picked: a new text)
            onAction: () => _setFont(c, sel),
          ),
      ],
    );
  }
}

TextStyle _face(_Font f, double size, Color c) => TextStyle(
  fontFamily: f.family,
  fontFamilyFallback: const ['.AppleSystemUIFont'],
  fontSize: size,
  color: T.ink(c),
  height: 1.1,
  decoration: TextDecoration.none,
);

class _FontRow extends StatelessWidget {
  const _FontRow(this.f, this.c, {this.usedBy});
  final _Font f;
  final DockCtx c;
  final String? usedBy;
  @override
  Widget build(BuildContext context) {
    final sel = c.sel == f.family, star = c.starred(f.family);
    return DockHover(
      onTap: () {
        c.pick(f.family);
        _toRole(c, f);
      },
      onDoubleTap: () => _setFont(c, f),   // (two presses set it; carried to the stage it does the same)
      builder: (context, h) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        height: Shape.z(46),
        margin: EdgeInsets.only(bottom: WsT.gutter),
        padding: EdgeInsets.only(left: WsT.inset, right: Shape.s(4)),
        color: dockFill(sel, h, WsT.card),
        child: Row(
          children: [
            Expanded(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  FittedBox(fit: BoxFit.scaleDown, alignment: Alignment.centerLeft, child: Text(_sample, maxLines: 1, style: _face(f, 17, dockInk(sel, Grey.g95)))),
                  SizedBox(height: Shape.s(5)),
                  Row(
                    children: [
                      Flexible(
                        child: Text('${f.family}  ·  ${DockTags.of('Fonts', f.family, f.path == null ? const <String>{} : (c.ws.inside?.fontTags(f.path!) ?? const <String>{})).take(2).join(' ')}', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(dockInk(sel, Grey.g76))),
                      ),
                      if (usedBy case final u?) ...[SizedBox(width: Shape.s(6)), DockTag(u, hue: sel ? null : WsT.toneText)],
                    ],
                  ),
                ],
              ),
            ),
            if (star || h || sel) DockStar(on: star, ink: sel ? WsT.onAccent : null, onTap: () => c.star(f.family)) else SizedBox(width: Shape.s(20)),
          ],
        ),
      ),
    );
  }
}

/// The wide form: a card per family with the sample large enough to judge the face, its facts under it.
class _FontCard extends StatelessWidget {
  const _FontCard(this.f, this.c, {this.usedBy});
  final _Font f;
  final DockCtx c;
  final String? usedBy;
  @override
  Widget build(BuildContext context) {
    final sel = c.sel == f.family, star = c.starred(f.family);
    return DockHover(
      onTap: () {
        c.pick(f.family);
        _toRole(c, f);
      },
      onDoubleTap: () => _setFont(c, f),
      builder: (context, h) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        height: Shape.z(96),
        padding: EdgeInsets.fromLTRB(WsT.inset, Shape.s(8), Shape.s(4), Shape.s(8)),
        color: dockFill(sel, h, WsT.card),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Spacer(),
                if (star || h || sel) DockStar(on: star, ink: sel ? WsT.onAccent : null, onTap: () => c.star(f.family)) else SizedBox(height: Shape.s(20)),
              ],
            ),
            const Spacer(),
            Text('Motion', maxLines: 1, softWrap: false, overflow: TextOverflow.fade, style: _face(f, 28, dockInk(sel, Grey.g95))),
            const Spacer(),
            Row(
              children: [
                Flexible(
                  child: Text('${f.family}  ·  ${DockTags.of('Fonts', f.family, f.path == null ? const <String>{} : (c.ws.inside?.fontTags(f.path!) ?? const <String>{})).take(2).join(' ')}', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(dockInk(sel, Grey.g76))),
                ),
                if (usedBy case final u?) ...[SizedBox(width: Shape.s(6)), DockTag(u, hue: sel ? null : WsT.toneText)],
              ],
            ),
          ],
        ),
      ),
    );
  }
}
