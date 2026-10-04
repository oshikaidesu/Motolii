// The Fonts shelf: each row set in its own face. Effects live in dock_effects.dart, files and media in dock_files.dart.
import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'dock_parts.dart';
import 'ws.dart';

// ---------------------------------------------------------------- fonts

class _Font {
  const _Font(this.family, this.kind, this.styles, {this.weight = FontWeight.w500, this.italic = false});
  final String family, kind;
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

const _sample = 'Motion for more relations';
const _usedFont = 'Avenir Next';

class FontsShelf extends StatelessWidget {
  const FontsShelf(this.c, {super.key});
  final DockCtx c;
  @override
  Widget build(BuildContext context) {
    final title = c.ws.layers.where((l) => l.id == 'title').firstOrNull;
    final hits = _fonts.where((f) => c.hit(f.family)).toList();
    bool keep(_Font f) => switch (c.chip) {
      'All' => true,
      'Used' => f.family == _usedFont || c.applied(f.family),
      'Starred' => c.starred(f.family),
      final k => f.kind == k,
    };
    final list = hits.where(keep).toList();
    final sel = _fonts.where((f) => f.family == c.sel).firstOrNull;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        DockChips(
          [
            ('All', hits.length),
            ('Used', hits.where((f) => f.family == _usedFont || c.applied(f.family)).length),
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
                    String? usedBy(_Font f) => f.family == _usedFont ? title?.name : null;
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
            meta: '${sel.kind} · ${sel.styles} styles · Latin',
            action: 'Apply',
            done: c.applied(sel.family) || sel.family == _usedFont,
            doneLabel: 'In use',
            onAction: () => c.apply(sel.family),
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
