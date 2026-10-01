import 'package:flutter/widgets.dart';
import 'icons.dart';
import 'tokens.dart';

// 56 px. Wordmark, transport keys, a readout display, the mode switch.
class TopBar extends StatelessWidget {
  const TopBar({super.key});
  @override
  Widget build(BuildContext context) => Container(
        height: 56,
        decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: P.rule2))),
        child: Stack(
          children: [
            const Positioned(left: 16, top: 14, child: Text('Motolii', style: P.wordmark)),
            const Positioned(left: 132, top: 15, child: Text('Motion\nfor More Relations.', style: P.tagline)),
            const Positioned(left: 312, top: 10, child: _Transport()),
            const Positioned(left: 470, top: 10, child: _Display()),
            const Positioned(left: 744, top: 10, child: _Segmented()),
            const Positioned(left: 1040, top: 10, child: _K(Glyph.fullscreen)),
            const Positioned(left: 1084, top: 10, child: _K(Glyph.pin)),
            const Positioned(left: 1128, top: 10, child: _K(Glyph.folder)),
            Positioned(left: 1188, top: 15, child: Text('jewel-field.rrd', style: P.tagline.copyWith(color: P.text2))),
            Positioned(left: 1188, top: 31, child: Text('saved 00:12 ago', style: P.tagline)),
            const Positioned(right: 16, top: 15, child: Text('Less numbers.\nMore motion.', textAlign: TextAlign.right, style: P.tagline)),
          ],
        ),
      );
}

class _Transport extends StatelessWidget {
  const _Transport();
  @override
  Widget build(BuildContext context) => Row(
        children: [
          _b(P.text, const Icon1(Glyph.play, size: 16, color: P.ink), lit: true),
          const SizedBox(width: 4),
          _b(P.key, const Icon1(Glyph.stop, size: 14, color: P.text2)),
          const SizedBox(width: 4),
          _b(P.key, const Icon1(Glyph.record, size: 16, color: P.text2)),
        ],
      );
  Widget _b(Color c, Widget i, {bool lit = false}) => Container(
        width: 42,
        height: 36,
        decoration: BoxDecoration(color: c, borderRadius: BorderRadius.circular(2), border: Border.all(color: lit ? c : P.rule2), boxShadow: const [BoxShadow(color: Color(0xFF0A0A0A), offset: Offset(0, 2))]),
        child: Center(child: i),
      );
}

// Plain readout. No well, no boxes: it is information, not a control.
class _Display extends StatelessWidget {
  const _Display();
  @override
  Widget build(BuildContext context) => SizedBox(
        height: 36,
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            const Text('120.00', style: P.lcd),
            _sep(),
            const Text('4 / 4', style: P.lcd),
            _sep(),
            const Text('00:02:13', style: P.lcd),
            const SizedBox(width: 12),
            const Icon1(Glyph.plus, size: 12, color: P.muted),
          ],
        ),
      );
  Widget _sep() => Container(width: 1, height: 14, margin: const EdgeInsets.symmetric(horizontal: 12), color: P.rule2);
}

class _Segmented extends StatelessWidget {
  const _Segmented();
  @override
  Widget build(BuildContext context) => Container(
        height: 36,
        decoration: BoxDecoration(border: Border.all(color: P.rule2), borderRadius: BorderRadius.circular(2), color: P.key),
        clipBehavior: Clip.antiAlias,
        child: Row(children: [_seg('EDIT', true), _seg('PLAY', false), _seg('EXPORT', false)]),
      );
  Widget _seg(String t, bool on) => Container(
        width: 84,
        alignment: Alignment.center,
        color: on ? P.blue : null,
        child: Text(t, style: on ? P.segment.copyWith(color: P.ink, fontWeight: FontWeight.w700) : P.segment),
      );
}

class _K extends StatelessWidget {
  const _K(this.g);
  final Glyph g;
  @override
  Widget build(BuildContext context) => Cap(width: 36, height: 36, child: Icon1(g, size: 15, color: P.text2, stroke: 1.4));
}
