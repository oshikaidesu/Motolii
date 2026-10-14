// The Fonts shelf: each row set in its own face. Effects live in dock_effects.dart, files and media in dock_files.dart.
import 'dart:async';
import 'dart:io';

import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'dock_files.dart' show dockRead;
import 'dock_parts.dart';
import 'ws.dart';

// ---------------------------------------------------------------- fonts

class _Font {
  const _Font(this.family, this.kind, this.styles, {this.weight = FontWeight.w500, this.italic = false, this.path});
  final String family, kind;
  final String? path; // (joined: the font's file)
  final int styles;
  final FontWeight weight;
  final bool italic;
}

const _fonts = <_Font>[
  _Font('Avenir Next', 'Sans', 12, weight: FontWeight.w700),
  _Font('Helvetica Neue', 'Sans', 14),
  _Font('Futura', 'Display', 6, weight: FontWeight.w700),
  _Font('Gill Sans', 'Sans', 6),
  _Font('Optima', 'Sans', 5),
  _Font('Didot', 'Serif', 3, italic: true),
  _Font('Georgia', 'Serif', 4),
  _Font('Baskerville', 'Serif', 6),
  _Font('Hoefler Text', 'Serif', 5),
  _Font('Bodoni 72', 'Display', 4, weight: FontWeight.w700),
  _Font('Rockwell', 'Display', 4, weight: FontWeight.w700),
  _Font('Copperplate', 'Display', 3),
  _Font('American Typewriter', 'Mono', 5),
  _Font('Menlo', 'Mono', 4),
  _Font('Courier New', 'Mono', 4),
];

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
    for (final f in sets) _Font(f.split('/').last.substring(0, f.split('/').last.length - 4), '', 1, path: f),
  ]..sort((a, b) => a.family.toLowerCase().compareTo(b.family.toLowerCase()));
  dockRead.value++;
});

/// Joined: the font given to the picked text; with no text picked, a new text set in it.
void _setFont(DockCtx c, _Font f) {
  final m = c.ws.inside!, t = int.tryParse(c.ws.selected ?? '');
  final made = t != null && m.word(t, 'kind') == 'text' ? (m.say(t, 'file', f.path!) ? t : 0) : m.add('text', f.path!);
  c.set(() => made == 0 ? c.mem.trouble[f.family] = m.trouble : c.mem.trouble.remove(f.family));
  if (made == 0) return;
  c.ws.reread();
  c.ws.select('$made');
}

const _sample = 'Motion for more relations';
const _usedFont = 'Avenir Next';

class FontsShelf extends StatelessWidget {
  const FontsShelf(this.c, {super.key});
  final DockCtx c;
  @override
  Widget build(BuildContext context) => ValueListenableBuilder<int>(valueListenable: dockRead, builder: (context, _, _) => _shelf(context));

  Widget _shelf(BuildContext context) {
    final m = c.ws.inside, fonts = m == null ? _fonts : _diskFonts(c.ws);
    final title = c.ws.layers.where((l) => l.id == 'title').firstOrNull;
    // (joined: which text of the document is set in which font file)
    final setIn = {
      if (m != null)
        for (final l in c.ws.layers)
          if (m.word(int.parse(l.id), 'kind') == 'text') m.whole(m.word(int.parse(l.id), 'file')): l,
    };
    bool used(_Font f) => m == null ? f.family == _usedFont || c.applied(f.family) : setIn.containsKey(f.path);
    final hits = fonts.where((f) => c.hit(f.family)).toList();
    bool keep(_Font f) => switch (c.chip) {
      'All' => true,
      'Used' => used(f),
      'Starred' => c.starred(f.family),
      final k => f.kind == k,
    };
    final list = hits.where(keep).toList();
    final sel = fonts.where((f) => f.family == c.sel).firstOrNull;
    final picked = sel == null ? null : setIn[sel.path], onText = picked != null && picked.id == c.ws.selected;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        DockChips(
          [
            ('All', hits.length),
            ('Used', hits.where(used).length),
            for (final k in const ['Sans', 'Serif', 'Display', 'Mono']) (k, hits.where((f) => f.kind == k).length),
            ('Starred', hits.where((f) => c.starred(f.family)).length),
          ],
          active: c.chip,
          onPick: c.pickChip,
        ),
        Expanded(
          child: list.isEmpty
              ? DockEmpty(c.query)
              : LayoutBuilder(
                  builder: (context, box) {
                    String? usedBy(_Font f) => m == null ? (f.family == _usedFont ? title?.name : null) : setIn[f.path]?.name;
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
            meta: m == null ? '${sel.kind} · ${sel.styles} styles · Latin' : c.mem.trouble[sel.family] ?? 'TTF · ${File(sel.path!).parent.path.split('/').last}',
            // (joined: the picked text is set in it; with no text picked, a new text is)
            action: m != null && m.word(int.tryParse(c.ws.selected ?? '') ?? 0, 'kind') != 'text' ? 'Place' : 'Apply',
            done: m != null ? onText : c.applied(sel.family) || sel.family == _usedFont,
            doneLabel: 'In use',
            fresh: m != null && m.word(int.tryParse(c.ws.selected ?? '') ?? 0, 'kind') != 'text', // (no text picked: a new text)
            onAction: m != null ? () => _setFont(c, sel) : () => c.apply(sel.family),
          ),
      ],
    );
  }
}

TextStyle _face(_Font f, double size, Color c) => TextStyle(
  fontFamily: f.family,
  fontFamilyFallback: const ['.AppleSystemUIFont'],
  fontSize: size,
  fontWeight: f.weight,
  fontStyle: f.italic ? FontStyle.italic : FontStyle.normal,
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
      onTap: () => c.pick(f.family),
      builder: (context, h) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        height: 46,
        margin: const EdgeInsets.only(bottom: WsT.gutter),
        padding: const EdgeInsets.only(left: WsT.inset, right: 4),
        color: dockFill(sel, h, WsT.card),
        child: Row(
          children: [
            Expanded(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(_sample, maxLines: 1, softWrap: false, overflow: TextOverflow.fade, style: _face(f, 17, dockInk(sel, Grey.g95))),
                  const SizedBox(height: 5),
                  Row(
                    children: [
                      Flexible(
                        child: Text(f.family, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(dockInk(sel, Grey.g76))),
                      ),
                      Flexible(
                        child: Text('  ·  ${f.styles} styles', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(dockInk(sel, Grey.g56))),
                      ),
                      if (usedBy != null || c.applied(f.family)) ...[const SizedBox(width: 6), DockTag(usedBy ?? 'Used', hue: sel ? null : WsT.toneText)],
                    ],
                  ),
                ],
              ),
            ),
            if (star || h || sel) DockStar(on: star, ink: sel ? WsT.onAccent : null, onTap: () => c.star(f.family)) else const SizedBox(width: 20),
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
      onTap: () => c.pick(f.family),
      builder: (context, h) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        height: 96,
        padding: const EdgeInsets.fromLTRB(WsT.inset, 8, 4, 8),
        color: dockFill(sel, h, WsT.card),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Expanded(
                  child: Text(f.kind.toUpperCase(), style: T.micro(dockInk(sel, Grey.g56)).copyWith(fontWeight: FontWeight.w700, letterSpacing: .8)),
                ),
                if (star || h || sel) DockStar(on: star, ink: sel ? WsT.onAccent : null, onTap: () => c.star(f.family)) else const SizedBox(height: 20),
              ],
            ),
            const Spacer(),
            Text('Motion', maxLines: 1, softWrap: false, overflow: TextOverflow.fade, style: _face(f, 28, dockInk(sel, Grey.g95))),
            const Spacer(),
            Row(
              children: [
                Flexible(
                  child: Text(f.family, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(dockInk(sel, Grey.g76))),
                ),
                Text('  ·  ${f.styles}', style: T.label(dockInk(sel, Grey.g56))),
                if (usedBy != null || c.applied(f.family)) ...[const SizedBox(width: 6), DockTag(usedBy ?? 'Used', hue: sel ? null : WsT.toneText)],
              ],
            ),
          ],
        ),
      ),
    );
  }
}
