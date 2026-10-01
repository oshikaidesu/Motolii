import 'package:flutter/widgets.dart';
import 'icons.dart';
import 'tokens.dart';

// 296 wide. Keys you press to create; the palette of the instrument.
class BrowserPanel extends StatelessWidget {
  const BrowserPanel({super.key});
  @override
  Widget build(BuildContext context) => Container(
        width: 284,
        decoration: const BoxDecoration(border: Border(right: BorderSide(color: P.rule2))),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const _Tabs(),
            const _Search(),
            Expanded(
              child: ClipRect(
                child: Padding(
                  padding: const EdgeInsets.fromLTRB(12, 12, 12, 0),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      _section('CREATE'),
                      const _Row([('Text', Glyph.text, null), ('Shape', Glyph.shape, null), ('Image', Glyph.image, null), ('Camera', Glyph.camera, null)]),
                      const SizedBox(height: 4),
                      const _Row([('Repeater', Glyph.repeater, null), ('Grid', Glyph.grid, null), ('Circle', Glyph.circle, null), ('Spiral', Glyph.spiral, null)]),
                      const SizedBox(height: 18),
                      _section('RELATIONS'),
                      const _Row([('Scatter', Glyph.scatter, P.pink), ('Along Path', Glyph.alongPath, P.mint), ('Stagger', Glyph.stagger, P.blue)]),
                      const SizedBox(height: 4),
                      const _Row([('Face', Glyph.face, P.lemon), ('Follow', Glyph.follow, P.peach), ('Attach', Glyph.attach, P.lavender)]),
                      const SizedBox(height: 18),
                      _section('EFFECTS'),
                      const _Row([('Blur', Glyph.blur, null), ('Glow', Glyph.glow, null), ('Color', Glyph.color, null)], left: true),
                      const SizedBox(height: 4),
                      const _Row([('Composite', Glyph.composite, null), ('Distort', Glyph.distort, null), ('Stylize', Glyph.stylize, null)], left: true),
                      const SizedBox(height: 18),
                      _section('PRESETS'),
                      const _Presets(),
                    ],
                  ),
                ),
              ),
            ),
            const Rule(),
            SizedBox(
              height: 36,
              child: Row(
                children: [
                  const SizedBox(width: 14),
                  const Icon1(Glyph.plus, size: 12, color: P.text2),
                  const SizedBox(width: 12),
                  const Text('New Preset', style: P.list),
                  const Spacer(),
                  Text('v0.5.0', style: P.tagline),
                  const SizedBox(width: 12),
                ],
              ),
            ),
          ],
        ),
      );
  Widget _section(String t) => Padding(padding: const EdgeInsets.only(left: 2, bottom: 8), child: Text(t, style: P.section));
}

// Tabs are cut into the chassis: the open one is a lit slot with a top rule.
class _Tabs extends StatelessWidget {
  const _Tabs();
  static const tabs = ['OBJECTS', 'EFFECTS', 'MEDIA', 'COLORS', 'FONTS'];
  @override
  Widget build(BuildContext context) => Container(
        height: 34,
        decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: P.rule2))),
        child: Row(
          children: [
            for (final (i, t) in tabs.indexed)
              Expanded(
                child: Container(
                  alignment: Alignment.center,
                  decoration: BoxDecoration(
                    color: i == 0 ? P.keyHi : null,
                    border: Border(top: BorderSide(color: i == 0 ? P.text : const Color(0x00000000), width: 2), right: const BorderSide(color: P.rule)),
                  ),
                  child: Text(t, style: i == 0 ? P.tabOn : P.tab),
                ),
              ),
          ],
        ),
      );
}

class _Search extends StatelessWidget {
  const _Search();
  @override
  Widget build(BuildContext context) => Container(
        height: 30,
        decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: P.rule))),
        padding: const EdgeInsets.symmetric(horizontal: 12),
        child: Row(children: [const Icon1(Glyph.search, size: 12, color: P.muted), const SizedBox(width: 10), Text('Search  ⌘K', style: P.tab)]),
      );
}

class _Row extends StatelessWidget {
  const _Row(this.items, {this.left = false});
  final List<(String, Glyph, Color?)> items;
  final bool left;
  @override
  Widget build(BuildContext context) => Row(
        children: [for (final (i, it) in items.indexed) ...[if (i > 0) const SizedBox(width: 4), Expanded(child: _Tile(it.$1, it.$2, it.$3, left: left || it.$3 != null))]],
      );
}

// A key: 62 tall, sharp, one rule. Coloured ones are Relations — the parts.
class _Tile extends StatelessWidget {
  const _Tile(this.label, this.g, this.color, {this.left = false});
  final String label;
  final Glyph g;
  final Color? color;
  final bool left;
  @override
  Widget build(BuildContext context) {
    final ink = color == null ? P.text : P.ink;
    return Container(
      height: 60,
      decoration: BoxDecoration(color: color ?? P.key, border: Border.all(color: color == null ? P.rule2 : color!), borderRadius: BorderRadius.circular(2)),
      child: Stack(
        children: [
          Positioned(top: 9, left: left ? 9 : 0, right: left ? null : 0, child: Align(alignment: left ? Alignment.centerLeft : Alignment.center, child: Icon1(g, size: 22, color: ink, stroke: 1.5))),
          Positioned(bottom: 8, left: left ? 9 : 0, right: left ? null : 0, child: Text(label, textAlign: left ? TextAlign.left : TextAlign.center, softWrap: false, style: color == null ? P.tileLabel : P.tileLabelInk)),
        ],
      ),
    );
  }
}

class _Presets extends StatelessWidget {
  const _Presets();
  static const names = [('Jewel Field', '300'), ('Neon Tunnel', '48'), ('Photo Scatter', '120'), ('VJ Loop', '16'), ('Title Minimal', '3')];
  @override
  Widget build(BuildContext context) => Column(
        children: [
          for (final (i, n) in names.indexed)
            Container(
              height: 26,
              padding: const EdgeInsets.only(left: 8, right: 8),
              decoration: BoxDecoration(color: i == 0 ? P.keyHi : null, border: Border(left: BorderSide(color: i == 0 ? P.text : const Color(0x00000000), width: 2))),
              child: Row(children: [const Icon1(Glyph.preset, size: 12, color: P.muted), const SizedBox(width: 10), Text(n.$1, style: i == 0 ? P.listOn : P.list), const Spacer(), Text(n.$2, style: P.numMuted)]),
            ),
        ],
      );
}
