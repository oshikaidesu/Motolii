import 'dart:io';
import 'package:flutter/widgets.dart';
import 'icons.dart';
import 'tokens.dart';

// Reference image: the concept art's own stage, cropped. Not a renderer.
const _refPath = String.fromEnvironment('PROTO_STAGE', defaultValue: '/Users/member_ottoto/rust_ae/Motolii/motolii/ui/lib/proto/stage_ref.png');

// The lit surface. Chrome around it is thin and dark.
class StagePanel extends StatelessWidget {
  const StagePanel({super.key});
  @override
  Widget build(BuildContext context) => Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const _CompTabs(),
          Expanded(
            child: Stack(
              fit: StackFit.expand,
              children: [
                Image.file(File(_refPath), fit: BoxFit.cover, alignment: Alignment.center),
                const Positioned(left: 10, top: 12, child: _Tools()),
                Positioned(right: 10, top: 10, child: _Readout()),
              ],
            ),
          ),
          const _BottomBar(),
        ],
      );
}

class _CompTabs extends StatelessWidget {
  const _CompTabs();
  @override
  Widget build(BuildContext context) => Container(
        height: 34,
        decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: P.rule2))),
        child: Row(
          children: [
            Container(
              width: 168,
              decoration: const BoxDecoration(color: P.keyHi, border: Border(top: BorderSide(color: P.text, width: 2), right: BorderSide(color: P.rule2))),
              padding: const EdgeInsets.only(left: 12, right: 10),
              child: Row(children: [const Text('Jewel Field', style: P.bodyMed), const Spacer(), Text('×', style: P.body.copyWith(color: P.muted, fontSize: 14))]),
            ),
            Container(
              width: 128,
              decoration: const BoxDecoration(border: Border(right: BorderSide(color: P.rule))),
              padding: const EdgeInsets.only(left: 12),
              alignment: Alignment.centerLeft,
              child: Text('Title Card', style: P.body.copyWith(color: P.muted)),
            ),
            Container(width: 34, alignment: Alignment.center, child: const Icon1(Glyph.plus, size: 13, color: P.muted)),
            const Spacer(),
            Padding(padding: const EdgeInsets.only(right: 12), child: Text('1920 × 1080  ·  30 fps  ·  6.0 s', style: P.numMuted)),
          ],
        ),
      );
}

class _Tools extends StatelessWidget {
  const _Tools();
  static const tools = [Glyph.arrow, Glyph.move, Glyph.rect, Glyph.ellipse, Glyph.pen, Glyph.type, Glyph.crop];
  @override
  Widget build(BuildContext context) => Container(
        width: 40,
        decoration: BoxDecoration(color: const Color(0xFF1A1A1A), border: Border.all(color: P.rule2), borderRadius: BorderRadius.circular(2)),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            for (final (i, t) in tools.indexed)
              Container(
                height: 43,
                decoration: BoxDecoration(color: i == 0 ? P.text : null, border: Border(bottom: BorderSide(color: i == tools.length - 1 ? const Color(0x00000000) : P.rule))),
                child: Center(child: Icon1(t, size: 16, color: i == 0 ? P.ink : P.text2, stroke: 1.4)),
              ),
          ],
        ),
      );
}

// Selection readout on the glass: what is selected and how many.
class _Readout extends StatelessWidget {
  @override
  Widget build(BuildContext context) => Container(
        padding: const EdgeInsets.fromLTRB(10, 7, 10, 7),
        decoration: BoxDecoration(color: const Color(0xCC121212), border: Border.all(color: P.rule2), borderRadius: BorderRadius.circular(2)),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(width: 8, height: 8, color: P.pink),
            const SizedBox(width: 8),
            Text('Jewel Field', style: P.num),
            const SizedBox(width: 10),
            Text('300 obj', style: P.numMuted),
            const SizedBox(width: 10),
            Text('Scatter + Stagger + Along Path', style: P.numMuted),
          ],
        ),
      );
}

class _BottomBar extends StatelessWidget {
  const _BottomBar();
  @override
  Widget build(BuildContext context) => Container(
        height: 40,
        decoration: const BoxDecoration(border: Border(top: BorderSide(color: P.rule2))),
        padding: const EdgeInsets.symmetric(horizontal: 8),
        child: Row(
          children: [
            _views(),
            const SizedBox(width: 12),
            _k('100%', 64, chevron: true),
            const SizedBox(width: 4),
            _ik(Glyph.fit),
            const SizedBox(width: 4),
            _ik(Glyph.fit2),
            const SizedBox(width: 12),
            _colors(),
            const Spacer(),
            Text('safe  ·  grid 8  ·  snap', style: P.numMuted),
            const SizedBox(width: 12),
            _ik(Glyph.fullscreen),
          ],
        ),
      );
  Widget _views() => Container(
        height: 26,
        decoration: BoxDecoration(border: Border.all(color: P.rule2), borderRadius: BorderRadius.circular(2)),
        clipBehavior: Clip.antiAlias,
        child: Row(
          children: [
            for (final (t, on) in [('CAMERA', true), ('FRONT', false), ('TOP', false), ('SIDE', false)])
              Container(padding: const EdgeInsets.symmetric(horizontal: 10), alignment: Alignment.center, color: on ? P.keyHi : null, child: Text(t, style: on ? P.tabOn : P.tab)),
          ],
        ),
      );
  Widget _k(String t, double w, {bool chevron = false}) => Cap(width: w, height: 26, padding: const EdgeInsets.symmetric(horizontal: 8), child: Row(children: [Text(t, style: P.num), if (chevron) ...[const Spacer(), const Icon1(Glyph.chevronDown, size: 10, color: P.muted)]]));
  Widget _ik(Glyph g) => Cap(width: 28, height: 26, child: Icon1(g, size: 12, color: P.text2));
  Widget _colors() => Row(
        children: [
          for (final c in [P.blue, P.mint, P.pink, null, null])
            Container(width: 10, height: 10, margin: const EdgeInsets.only(right: 6), decoration: BoxDecoration(color: c ?? const Color(0x00000000), border: Border.all(color: c ?? P.rule2), borderRadius: BorderRadius.circular(1))),
        ],
      );
}
