import 'dart:math' as math;
import 'package:flutter/widgets.dart';
import 'icons.dart';
import 'tokens.dart';

// 400 wide. Header, meaning-level tabs, then the relations acting on the selection.
// Discovery = big colour (browser); editing = the open instrument; identity = a dot.
class InspectorPanel extends StatelessWidget {
  const InspectorPanel({super.key});
  @override
  Widget build(BuildContext context) => Container(
        width: 400,
        decoration: const BoxDecoration(border: Border(left: BorderSide(color: P.rule2))),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const _Tabs(),
            const _Head(),
            const _SubTabs(),
            const Padding(padding: EdgeInsets.fromLTRB(10, 10, 10, 0), child: _ScatterInstrument()),
            const Padding(padding: EdgeInsets.fromLTRB(10, 8, 10, 0), child: _Folded('Stagger', P.blue, _StaggerFace(), on: true, note: '0.12 s')),
            const Padding(padding: EdgeInsets.fromLTRB(10, 4, 10, 0), child: _Folded('Along Path', P.mint, _PathFace(), on: true, note: 'Path 1')),
            const Padding(padding: EdgeInsets.fromLTRB(10, 4, 10, 0), child: _Folded('Face Target', P.lemon, _FaceFace(), on: false, note: 'Camera')),
            const Padding(padding: EdgeInsets.fromLTRB(10, 4, 10, 0), child: _AddRow()),
            const Spacer(),
            const _History(),
          ],
        ),
      );
}

class _SubTabs extends StatelessWidget {
  const _SubTabs();
  @override
  Widget build(BuildContext context) => Container(
        height: 32,
        margin: const EdgeInsets.fromLTRB(10, 0, 10, 0),
        decoration: BoxDecoration(border: Border.all(color: P.rule2), borderRadius: BorderRadius.circular(2)),
        clipBehavior: Clip.antiAlias,
        child: Row(
          children: [
            for (final (i, (t, on)) in [('Transform', false), ('Relations', true), ('Effects', false), ('Material', false)].indexed)
              Expanded(
                child: Container(
                  alignment: Alignment.center,
                  decoration: BoxDecoration(color: on ? P.keyHi : null, border: i == 0 ? null : const Border(left: BorderSide(color: P.rule))),
                  child: Text(t, style: on ? P.bodyMed : P.body.copyWith(color: P.muted)),
                ),
              ),
          ],
        ),
      );
}

class _Tabs extends StatelessWidget {
  const _Tabs();
  @override
  Widget build(BuildContext context) => Container(
        height: 34,
        decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: P.rule2))),
        child: Row(
          children: [
            for (final (i, t) in ['INSPECTOR', 'PROJECT', 'LOOK'].indexed)
              Container(
                width: 92,
                alignment: Alignment.center,
                decoration: BoxDecoration(color: i == 0 ? P.keyHi : null, border: Border(top: BorderSide(color: i == 0 ? P.text : const Color(0x00000000), width: 2), right: const BorderSide(color: P.rule))),
                child: Text(t, style: i == 0 ? P.tabOn : P.tab),
              ),
            const Spacer(),
            const Padding(padding: EdgeInsets.only(right: 14), child: Kebab()),
          ],
        ),
      );
}

// Name, kind, count. The swatch is the same pink as everywhere Scatter appears.
class _Head extends StatelessWidget {
  const _Head();
  @override
  Widget build(BuildContext context) => Container(
        height: 64,
        padding: const EdgeInsets.fromLTRB(12, 0, 12, 0),
        child: Row(
          children: [
            Container(width: 26, height: 26, color: P.pink),
            const SizedBox(width: 12),
            const Column(
              mainAxisAlignment: MainAxisAlignment.center,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [Text('Jewel Field', style: P.title), SizedBox(height: 6), Text('Group  ·  300 objects  ·  6.0 s', style: P.small)],
            ),
            const Spacer(),
            const Icon1(Glyph.eye, size: 15, color: P.text2, stroke: 1.3),
            const SizedBox(width: 14),
            const Icon1(Glyph.lock, size: 13, color: P.muted),
          ],
        ),
      );
}

// The one that is open. The phenomenon first (left), the numbers second (right).
class _ScatterInstrument extends StatelessWidget {
  const _ScatterInstrument();
  @override
  Widget build(BuildContext context) => Container(
        height: 262,
        decoration: BoxDecoration(color: P.key, border: Border.all(color: P.rule2), borderRadius: BorderRadius.circular(2)),
        child: Column(
          children: [
            SizedBox(
              height: 46,
              child: Padding(
                padding: const EdgeInsets.only(left: 12, right: 12),
                child: Row(
                  children: [
                    const Ident(P.pink, size: 16),
                    const SizedBox(width: 12),
                    const Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [Text('Scatter', style: P.instrument), SizedBox(height: 5), Text('Scatter points to make spread.', style: P.small)],
                    ),
                    const Spacer(),
                    const Switch1(on: true, color: P.pink),
                    const SizedBox(width: 14),
                    const Kebab(),
                  ],
                ),
              ),
            ),
            const Rule(),
            Expanded(
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  const Expanded(child: Padding(padding: EdgeInsets.all(6), child: CustomPaint(painter: _ScatterField()))),
                  const Rule(vertical: true),
                  SizedBox(
                    width: 156,
                    child: Padding(
                      padding: const EdgeInsets.fromLTRB(12, 12, 12, 10),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          _slider('Density', '0.72', .72),
                          const SizedBox(height: 14),
                          _slider('Spread', '0.48', .48),
                          const SizedBox(height: 14),
                          _labelRow('Falloff', '0.36'),
                          const SizedBox(height: 4),
                          const SizedBox(height: 28, width: double.infinity, child: CustomPaint(painter: _CurvePreview())),
                          const Spacer(),
                          const Text('Shape', style: P.body),
                          const SizedBox(height: 8),
                          const _Shapes(),
                        ],
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      );
  Widget _labelRow(String l, String v) => Row(children: [Text(l, style: P.body), const Spacer(), Text(v, style: P.num)]);
  Widget _slider(String l, String v, double t) => Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _labelRow(l, v),
          const SizedBox(height: 8),
          LayoutBuilder(
            builder: (_, c) => Stack(
              alignment: Alignment.centerLeft,
              clipBehavior: Clip.none,
              children: [
                Container(height: 4, width: c.maxWidth, color: P.rule2),
                Container(height: 4, width: c.maxWidth * t, color: P.pink),
                Positioned(left: c.maxWidth * t - 5, child: Container(width: 10, height: 10, decoration: const BoxDecoration(color: P.pink, shape: BoxShape.circle))),
              ],
            ),
          ),
        ],
      );
}

class _CurvePreview extends CustomPainter {
  const _CurvePreview();
  @override
  void paint(Canvas cv, Size s) {
    final p = Path()..moveTo(0, s.height * .9)..cubicTo(s.width * .3, s.height * .9, s.width * .4, s.height * .1, s.width * .55, s.height * .1)..cubicTo(s.width * .7, s.height * .1, s.width * .8, s.height * .95, s.width, s.height * .95);
    cv.drawPath(p, Paint()..color = P.pink..style = PaintingStyle.stroke..strokeWidth = 1.4);
    cv.drawCircle(Offset(s.width * .55, s.height * .1), 3, Paint()..color = P.pink);
  }
  @override
  bool shouldRepaint(_CurvePreview o) => false;
}

class _Shapes extends StatelessWidget {
  const _Shapes();
  @override
  Widget build(BuildContext context) => Row(
        children: [
          Container(width: 22, height: 22, color: P.pink, child: const Center(child: Icon1(Glyph.circle, size: 14, color: P.ink, stroke: 1.8))),
          const SizedBox(width: 6),
          const Icon1(Glyph.rect, size: 15, color: P.text2, stroke: 1.4),
          const SizedBox(width: 6),
          const Icon1(Glyph.stylize, size: 15, color: P.text2, stroke: 1.4),
          const SizedBox(width: 6),
          Text('×', style: P.body.copyWith(fontSize: 16, color: P.text2)),
          const SizedBox(width: 6),
          const Icon1(Glyph.star, size: 14, color: P.text2),
        ],
      );
}

// Distribution field with handles: spread ring, density, falloff gradient.
class _ScatterField extends CustomPainter {
  const _ScatterField();
  @override
  void paint(Canvas cv, Size s) {
    final c = Offset(s.width / 2, s.height / 2);
    final r = (s.shortestSide / 2) - 10;
    // Falloff: pink fading outwards.
    cv.drawCircle(c, r, Paint()..shader = RadialGradient(colors: [P.pink.withValues(alpha: .10), P.pink.withValues(alpha: .0)], stops: const [.36, 1]).createShader(Rect.fromCircle(center: c, radius: r)));
    // Spread ring, dashed, with a handle.
    final dash = Paint()..color = P.pink.withValues(alpha: .7)..style = PaintingStyle.stroke..strokeWidth = 1;
    final ring = Path()..addOval(Rect.fromCircle(center: c, radius: r * .92));
    for (final m in ring.computeMetrics()) {
      var d = 0.0;
      while (d < m.length) {
        cv.drawPath(m.extractPath(d, d + 4), dash);
        d += 8;
      }
    }
    final rnd = math.Random(42);
    final dot = Paint()..color = P.pink;
    for (var i = 0; i < 90; i++) {
      final a = rnd.nextDouble() * math.pi * 2;
      final d = math.sqrt(rnd.nextDouble()) * r * .9;
      final o = c + Offset(math.cos(a) * d, math.sin(a) * d);
      final k = rnd.nextInt(3);
      final sz = 2.0 + rnd.nextDouble() * 5 * (1 - d / r * .5);
      if (k == 0) {
        cv.drawCircle(o, sz / 2, dot);
      } else {
        cv.save();
        cv.translate(o.dx, o.dy);
        cv.rotate(k == 1 ? math.pi / 4 : 0);
        cv.drawRect(Rect.fromCenter(center: Offset.zero, width: sz, height: sz), dot);
        cv.restore();
      }
    }
    final cross = Paint()..color = P.text..strokeWidth = 1;
    cv.drawLine(c - const Offset(8, 0), c + const Offset(8, 0), cross);
    cv.drawLine(c - const Offset(0, 8), c + const Offset(0, 8), cross);
    // Handles: spread (on the ring), falloff (on the radius).
    final h1 = c + Offset(math.cos(-.9) * r * .92, math.sin(-.9) * r * .92);
    cv.drawCircle(h1, 5, Paint()..color = const Color(0xFF1B1B1B));
    cv.drawCircle(h1, 5, Paint()..color = P.pink..style = PaintingStyle.stroke..strokeWidth = 1.5);
    final h2 = c + Offset(r * .36, 0);
    cv.drawRect(Rect.fromCenter(center: h2, width: 7, height: 7), Paint()..color = P.pink);
  }
  @override
  bool shouldRepaint(_ScatterField o) => false;
}

// A folded relation keeps its identity: the dot, then a live face of what it does.
class _Folded extends StatelessWidget {
  const _Folded(this.name, this.color, this.face, {required this.on, required this.note});
  final String name;
  final Color? color;
  final Widget face;
  final bool on;
  final String note;
  @override
  Widget build(BuildContext context) => Container(
        height: 40,
        padding: const EdgeInsets.only(left: 12, right: 12),
        decoration: BoxDecoration(color: P.key, border: Border.all(color: P.rule2), borderRadius: BorderRadius.circular(2)),
        child: Row(
          children: [
            Ident(on ? (color ?? P.text2) : P.rule2, size: 16),
            const SizedBox(width: 12),
            SizedBox(width: 92, child: Text(name, style: on ? P.row : P.row.copyWith(color: P.muted))),
            SizedBox(width: 90, height: 18, child: Opacity(opacity: on ? 1 : .3, child: face)),
            const SizedBox(width: 10),
            Text(note, style: P.numMuted),
            const Spacer(),
            Switch1(on: on, color: color ?? P.text2),
            const SizedBox(width: 12),
            const Kebab(),
          ],
        ),
      );
}

class _StaggerFace extends StatelessWidget {
  const _StaggerFace();
  @override
  Widget build(BuildContext context) => CustomPaint(painter: _Bars(P.blue));
}

class _Bars extends CustomPainter {
  const _Bars(this.c);
  final Color c;
  @override
  void paint(Canvas cv, Size s) {
    final p = Paint()..color = c;
    for (var i = 0; i < 12; i++) {
      final h = s.height * (0.25 + i / 11 * .75);
      cv.drawRect(Rect.fromLTWH(i * (s.width / 12), s.height - h, s.width / 12 - 2, h), p);
    }
  }
  @override
  bool shouldRepaint(_Bars o) => false;
}

class _PathFace extends StatelessWidget {
  const _PathFace();
  @override
  Widget build(BuildContext context) => CustomPaint(painter: _PathMarks());
}

class _PathMarks extends CustomPainter {
  @override
  void paint(Canvas cv, Size s) {
    final path = Path()..moveTo(0, s.height * .8)..cubicTo(s.width * .3, -s.height * .3, s.width * .6, s.height * 1.3, s.width, s.height * .2);
    cv.drawPath(path, Paint()..color = P.mint.withValues(alpha: .6)..style = PaintingStyle.stroke..strokeWidth = 1);
    for (final m in path.computeMetrics()) {
      for (var i = 0; i <= 8; i++) {
        final t = m.getTangentForOffset(m.length * i / 8)!;
        cv.drawCircle(t.position, 2, Paint()..color = P.mint);
      }
    }
  }
  @override
  bool shouldRepaint(_PathMarks o) => false;
}

class _FaceFace extends StatelessWidget {
  const _FaceFace();
  @override
  Widget build(BuildContext context) => CustomPaint(painter: _Arrows());
}

class _Arrows extends CustomPainter {
  @override
  void paint(Canvas cv, Size s) {
    final p = Paint()..color = P.lemon..strokeWidth = 1.2..style = PaintingStyle.stroke;
    final t = Offset(s.width - 4, s.height / 2);
    for (var i = 0; i < 5; i++) {
      final o = Offset(6 + i * 16.0, i.isEven ? 4 : s.height - 4);
      final d = (t - o) / (t - o).distance * 9;
      cv.drawLine(o, o + d, p);
    }
    cv.drawCircle(t, 3, Paint()..color = P.lemon);
  }
  @override
  bool shouldRepaint(_Arrows o) => false;
}

class _AddRow extends StatelessWidget {
  const _AddRow();
  @override
  Widget build(BuildContext context) => Container(
        height: 36,
        padding: const EdgeInsets.only(left: 14),
        decoration: BoxDecoration(border: Border.all(color: P.rule2), borderRadius: BorderRadius.circular(2)),
        child: Row(children: [const Icon1(Glyph.plus, size: 12, color: P.text2), const SizedBox(width: 12), const Text('Add Relation', style: P.bodyMed)]),
      );
}

// Undo history, folded to a strip at the bottom: a capability, quietly reachable.
class _History extends StatelessWidget {
  const _History();
  @override
  Widget build(BuildContext context) => Container(
        height: 30,
        padding: const EdgeInsets.symmetric(horizontal: 12),
        decoration: const BoxDecoration(border: Border(top: BorderSide(color: P.rule))),
        child: Row(children: [Text('HISTORY', style: P.section), const SizedBox(width: 12), Text('Scatter density .68 → .72', style: P.numMuted), const Spacer(), Text('24', style: P.numMuted)]),
      );
}
