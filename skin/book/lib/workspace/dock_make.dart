// The making shelves: Colors (pop palettes, a swatch field, saved gradients) and Create (tiles for every new layer, shape and copier).
import 'package:flutter/widgets.dart';

import '../sets/inspector/inspector_parts.dart' show Pop;
import '../tokens.dart';
import 'dock_glyphs.dart';
import 'dock_parts.dart';
import 'ws.dart';

// ---------------------------------------------------------------- colors

class _Pal {
  const _Pal(this.id, this.name, this.colors, {this.saved = false});
  final String id, name;
  final List<Color> colors;
  final bool saved;
}

const _pals = <_Pal>[
  _Pal('pop', 'Pop Poster', [Color(0xFFB3C66B), Color(0xFF4781E5), Color(0xFFF08A5D), Color(0xFFE974AB), Color(0xFFEFCB4E), Color(0xFF15170D)]),
  _Pal('riso', 'Riso Print', [Color(0xFFFF48B0), Color(0xFF0078BF), Color(0xFFFFE800), Color(0xFF00A95C), Color(0xFFF15060), Color(0xFFF2EDE4)]),
  _Pal('coda', 'Coda Title', [Color(0xFFD9D2C3), Color(0xFFF08A5D), Color(0xFF7FB8B0), Color(0xFF3B3631), Color(0xFFF2C94C)], saved: true),
  _Pal('market', 'Night Market', [Color(0xFF1B1F3B), Color(0xFF53354A), Color(0xFFE84545), Color(0xFFFFB740), Color(0xFF2B9EB3)]),
  _Pal('candy', 'Candy Shop', [Color(0xFFFFB5C2), Color(0xFFB5EAD7), Color(0xFFC7CEEA), Color(0xFFFFDAC1), Color(0xFFE2F0CB), Color(0xFF9ADCFF)], saved: true),
  _Pal('signal', 'Signal', [Color(0xFFFF3B30), Color(0xFFFF9500), Color(0xFFFFCC00), Color(0xFF34C759), Color(0xFF007AFF), Color(0xFFAF52DE)]),
];

const _grads = <(String, List<Color>)>[
  ('Lime dusk', [Color(0xFFB3C66B), Color(0xFF4781E5)]),
  ('Hot pink', [Color(0xFFE974AB), Color(0xFFF08A5D), Color(0xFFEFCB4E)]),
  ('Deep sea', [Color(0xFF15170D), Color(0xFF2B9EB3), Color(0xFF7DD5B1)]),
];

String _hex(Color c) {
  String h(double v) => (v * 255).round().toRadixString(16).padLeft(2, '0').toUpperCase();
  return '#${h(c.r)}${h(c.g)}${h(c.b)}';
}

/// The swatch field: twelve hues down four tones, plus a grey ramp.
final _field = [
  for (final l in const [.78, .64, .5, .34]) [for (var i = 0; i < 12; i++) HSLColor.fromAHSL(1, i * 30.0, .62, l).toColor()],
  [for (var i = 0; i < 12; i++) HSLColor.fromAHSL(1, 0, 0, i / 11).toColor()],
];

class ColorsShelf extends StatelessWidget {
  const ColorsShelf(this.c, {super.key});
  final DockCtx c;

  @override
  Widget build(BuildContext context) {
    final used = <String, Color>{for (final l in c.ws.layers) l.name: wsTone(l.kind)};
    final chip = c.chip;
    final pals = _pals.where((p) => c.hit(p.name) && (chip == 'All' || (chip == 'Saved' ? p.saved : chip == 'Starter' && !p.saved))).toList();
    Widget block(List<Widget> kids) => Padding(
      padding: const EdgeInsets.only(bottom: WsT.gutter),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: kids),
    );
    Widget grid(double cell, List<Widget> items) => LayoutBuilder(
      builder: (context, box) {
        final cols = wsCols(box.maxWidth, cell), w = wsCell(box.maxWidth, cols);
        return Wrap(
          spacing: WsT.gutter,
          children: [for (final i in items) SizedBox(width: w, child: i)],
        );
      },
    );
    final blocks = <(Widget, double)>[
      if ((chip == 'All' || chip == 'Used') && c.hit('used'))
        (block([DockSection('Used here', count: used.length, hue: WsT.accent), _Strip('used', used.values.toList(), c, names: used.keys.toList())]), 2.0),
      if (chip != 'Used' && pals.isNotEmpty)
        (
          block([
            DockSection('Palettes', count: pals.length, hue: Fam.scatter.c),
            grid(220, [for (final p in pals) _PalCard(p, c)]),
          ]),
          1 + pals.length * 2.2,
        ),
      if ((chip == 'All' || chip == 'Saved') && c.hit('gradient'))
        (
          block([
            DockSection('Saved gradients', count: _grads.length, hue: Fam.stagger.c),
            grid(200, [for (final (i, (n, g)) in _grads.indexed) _Grad('grad-$i', n, g, c)]),
          ]),
          1.0 + _grads.length,
        ),
      if (chip == 'All' && c.hit('swatches'))
        (
          block([
            DockSection('Swatches', count: 60, hue: Fam.face.c),
            const SizedBox(height: WsT.gutter),
            for (final (j, row) in _field.indexed) _Strip('field-$j', row, c, height: 18, gap: 1),
          ]),
          4.0,
        ),
    ];
    final sel = _resolve(c.sel, used);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        DockChips(
          [
            ('All', null),
            ('Used', used.length),
            ('Saved', _pals.where((p) => p.saved).length + _grads.length),
            ('Starter', _pals.where((p) => !p.saved).length),
          ],
          active: chip,
          onPick: c.pickChip,
        ),
        Expanded(
          child: blocks.isEmpty
              ? DockEmpty(c.query)
              : DockMasonry(minColumn: 300, weights: [for (final b in blocks) b.$2], children: [for (final b in blocks) b.$1]),
        ),
        if (sel != null)
          DockFooter(
            preview: ColoredBox(color: sel.$2),
            title: _hex(sel.$2),
            meta: sel.$1,
            action: 'Fill',
            done: c.applied(c.sel!),
            doneLabel: 'Filled',
            onAction: () => c.apply(c.sel!),
          ),
      ],
    );
  }

  (String, Color)? _resolve(String? id, Map<String, Color> used) {
    if (id == null) return null;
    final parts = id.split('-');
    final i = int.tryParse(parts.last);
    if (i == null) return null;
    final key = parts.sublist(0, parts.length - 1).join('-');
    if (key == 'used' && i < used.length) return ('Used by ${used.keys.elementAt(i)}', used.values.elementAt(i));
    if (key.startsWith('field') && i < 12) {
      final hsl = HSLColor.fromColor(_field[int.parse(key.split('-').last)][i]);
      return ('Swatch · H ${hsl.hue.round()}° L ${(hsl.lightness * 100).round()}%', _field[int.parse(key.split('-').last)][i]);
    }
    if (key == 'grad' && i < _grads.length) return ('Gradient · ${_grads[i].$1} · ${_grads[i].$2.length} stops', _grads[i].$2.first);
    final p = _pals.where((p) => p.id == key).firstOrNull;
    if (p != null && i < p.colors.length) return ('${p.name} · ${i + 1} of ${p.colors.length}', p.colors[i]);
    return null;
  }
}

class _PalCard extends StatelessWidget {
  const _PalCard(this.p, this.c);
  final _Pal p;
  final DockCtx c;
  @override
  Widget build(BuildContext context) => Container(
    margin: const EdgeInsets.only(top: WsT.gutter),
    color: WsT.card,
    padding: const EdgeInsets.fromLTRB(WsT.inset, 6, WsT.inset, WsT.inset),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Row(
          children: [
            Expanded(
              child: Text(p.name, style: T.name(Grey.g95).copyWith(fontWeight: FontWeight.w700)),
            ),
            if (p.saved) ...[const DockTag('SAVED'), const SizedBox(width: 6)],
            Text('${p.colors.length}', style: T.value(Grey.g56).copyWith(fontSize: 10)),
          ],
        ),
        const SizedBox(height: 6),
        _Strip(p.id, p.colors, c, height: 26, pad: false),
      ],
    ),
  );
}

/// Flat swatches edge to edge; the picked one gets a light frame with a dark keyline inside, readable on any hue.
class _Strip extends StatelessWidget {
  const _Strip(this.id, this.colors, this.c, {this.names, this.height = 24, this.gap = WsT.gutter, this.pad = true});
  final String id;
  final List<Color> colors;
  final DockCtx c;
  final List<String>? names;
  final double height, gap;
  final bool pad;
  @override
  Widget build(BuildContext context) => Padding(
    padding: EdgeInsets.fromLTRB(pad ? WsT.inset : 0, pad ? gap : 0, pad ? WsT.inset : 0, 0),
    child: SizedBox(
      height: height,
      child: Row(
        children: [
          for (final (i, col) in colors.indexed) ...[
            if (i > 0) SizedBox(width: gap),
            Expanded(
              child: DockHover(
                onTap: () => c.pick('$id-$i'),
                builder: (context, h) {
                  final sel = c.sel == '$id-$i';
                  return Container(
                    decoration: BoxDecoration(
                      color: col,
                      borderRadius: BorderRadius.circular(sel || h ? 3 : 0),
                      border: sel ? Border.all(color: Grey.g95, width: 2) : (h ? Border.all(color: Grey.g76) : null),
                    ),
                    foregroundDecoration: sel
                        ? BoxDecoration(
                            border: Border.all(color: Grey.g07),
                            borderRadius: BorderRadius.circular(1),
                          )
                        : null,
                  );
                },
              ),
            ),
          ],
        ],
      ),
    ),
  );
}

class _Grad extends StatelessWidget {
  const _Grad(this.id, this.name, this.colors, this.c);
  final String id, name;
  final List<Color> colors;
  final DockCtx c;
  @override
  Widget build(BuildContext context) {
    final sel = c.sel == id;
    return DockHover(
      onTap: () => c.pick(id),
      builder: (context, h) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        height: 28,
        margin: const EdgeInsets.only(top: WsT.gutter),
        padding: const EdgeInsets.symmetric(horizontal: WsT.inset),
        color: dockFill(sel, h, WsT.card),
        child: Row(
          children: [
            Container(
              width: 64,
              height: 16,
              decoration: BoxDecoration(
                gradient: LinearGradient(colors: colors),
                borderRadius: BorderRadius.circular(WsT.chipRadius),
              ),
            ),
            const SizedBox(width: 8),
            Expanded(
              child: Text(name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(dockInk(sel))),
            ),
            Text('${colors.length} stops', style: T.label(dockInk(sel, Grey.g56))),
          ],
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------- create

class _Make {
  const _Make(this.name, this.g, this.hue, this.key, this.what);
  final String name, key, what;
  final DockG g;
  final Color hue;
}

final _makes = <(String, List<_Make>)>[
  (
    'Layers',
    [
      _Make('Shape', DockG.shape, WsT.toneShape, 'Q', 'Vector shape layer'),
      _Make('Text', DockG.text, WsT.toneText, 'T', 'Type layer'),
      _Make('Camera', DockG.camera, WsT.toneCamera, '⇧C', '3D camera, 50 mm'),
      _Make('Particles', DockG.particles, WsT.toneParticles, '⇧P', 'Emitter layer'),
      _Make('Solid', DockG.solid, WsT.toneImage, '⌘Y', 'Flat colour, comp size'),
      _Make('Adjustment', DockG.adjust, Pop.toneLayout, '⌥⌘Y', 'Effects for layers below'),
      _Make('Null', DockG.nullObj, Grey.g76, '⇧N', 'Invisible parent'),
      _Make('Group', DockG.group, WsT.toneGroup, '⌘G', 'Nest layers'),
      _Make('Audio', DockG.audio, WsT.toneAudio, '', 'Sound layer'),
    ],
  ),
  (
    'Shapes',
    [
      _Make('Rectangle', DockG.rect, WsT.toneShape, 'R', 'Rounded corners'),
      _Make('Ellipse', DockG.ellipse, WsT.toneShape, 'E', 'Circle with ⇧'),
      _Make('Polygon', DockG.polygon, WsT.toneShape, '', 'Sides · Roundness'),
      _Make('Star', DockG.star, WsT.toneShape, '', 'Points · Inner radius'),
      _Make('Line', DockG.line, WsT.toneShape, 'L', 'Open path'),
      _Make('Pen Path', DockG.pen, WsT.toneShape, 'P', 'Draw freely'),
    ],
  ),
  (
    'Copies',
    [
      _Make('Repeater', DockG.copies, Pop.tonePlace, '', 'Linear copies'),
      _Make('Grid', DockG.stylize, Pop.tonePlace, '', 'Rows × columns'),
      _Make('Radial', DockG.radial, Pop.tonePlace, '', 'Around a centre'),
    ],
  ),
  (
    'Templates',
    [
      _Make('Title Card', DockG.text, Pop.toneTextAnim, '', 'Kinetic title, 3 s'),
      _Make('Lower Third', DockG.rect, Pop.toneTextAnim, '', 'Name + role bar'),
      _Make('Logo Sting', DockG.star, Pop.toneTextAnim, '', 'Burst reveal, 2 s'),
      _Make('Countdown', DockG.time, Pop.toneTextAnim, '', '5 → 1 numbers'),
      _Make('Confetti', DockG.particles, Pop.toneTextAnim, '', 'Particle preset'),
      _Make('Grid Intro', DockG.stylize, Pop.toneTextAnim, '', 'Tiles fly in'),
    ],
  ),
];

class CreateShelf extends StatelessWidget {
  const CreateShelf(this.c, {super.key});
  final DockCtx c;
  @override
  Widget build(BuildContext context) {
    final groups = [
      for (final (g, items) in _makes)
        if (c.chip == 'All' || c.chip == g) (g, items.where((m) => c.hit(m.name)).toList()),
    ].where((e) => e.$2.isNotEmpty).toList();
    final sel = _makes.expand((e) => e.$2).where((m) => m.name == c.sel).firstOrNull;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        DockChips(
          [('All', _makes.fold<int>(0, (s, e) => s + e.$2.length)), for (final (g, items) in _makes) (g, items.length)],
          active: c.chip,
          onPick: c.pickChip,
        ),
        Expanded(
          child: groups.isEmpty
              ? DockEmpty(c.query)
              : DockMasonry(
                  minColumn: 300,
                  weights: [for (final (_, items) in groups) 1 + items.length / 3],
                  children: [
                    for (final (g, items) in groups)
                      Padding(
                        padding: const EdgeInsets.only(bottom: WsT.gutter),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.stretch,
                          children: [
                            DockSection(g, count: items.length, hue: items.first.hue),
                            const SizedBox(height: WsT.gutter),
                            LayoutBuilder(
                              builder: (context, box) {
                                final cols = wsCols(box.maxWidth, 74, min: 2), w = wsCell(box.maxWidth, cols);
                                return Wrap(
                                  spacing: WsT.gutter,
                                  runSpacing: WsT.gutter,
                                  children: [for (final m in items) SizedBox(width: w, height: 70, child: _MakeTile(m, c))],
                                );
                              },
                            ),
                          ],
                        ),
                      ),
                  ],
                ),
        ),
        if (sel != null)
          DockFooter(
            preview: DockBadge(sel.g, sel.hue, size: 34),
            title: '${sel.name} layer',
            meta: 'At ${c.ws.timecode()} · ${sel.what}',
            action: 'Create',
            done: c.applied(sel.name),
            doneLabel: 'Created',
            onAction: () => c.apply(sel.name),
          ),
      ],
    );
  }
}

class _MakeTile extends StatelessWidget {
  const _MakeTile(this.m, this.c);
  final _Make m;
  final DockCtx c;
  @override
  Widget build(BuildContext context) {
    final sel = c.sel == m.name;
    return DockHover(
      onTap: () => c.pick(m.name),
      builder: (context, h) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        padding: const EdgeInsets.all(7),
        color: sel ? WsT.accent : (h ? WsT.raised : WsT.card),
        child: Stack(
          children: [
            Align(
              alignment: Alignment.topLeft,
              child: DockGlyph(m.g, size: 26, color: sel ? WsT.onAccent : m.hue),
            ),
            if (m.key.isNotEmpty)
              Align(
                alignment: Alignment.topRight,
                child: Text(m.key, style: T.micro(dockInk(sel, Grey.g56))),
              ),
            Align(
              alignment: Alignment.bottomLeft,
              child: Text(
                m.name,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: T.name(dockInk(sel, Grey.g95)).copyWith(fontWeight: FontWeight.w700),
              ),
            ),
            if (c.applied(m.name))
              Align(
                alignment: const Alignment(1, .95),
                child: DockTag('+1', hue: sel ? null : WsT.accent),
              ),
          ],
        ),
      ),
    );
  }
}
