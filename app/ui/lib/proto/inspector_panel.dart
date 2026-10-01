// Inspector as one instrument. Responsive by rule, not by pixels:
//   wide   : gadget | precision side by side
//   normal : gadget over precision rows
//   narrow : mini gadget over compact rows, header collapses to name + more
import 'dart:math' as math;
import 'package:flutter/widgets.dart';
import 'icons.dart';
import 'tokens.dart';

enum Fit { wide, normal, narrow }

Fit fitFor(double w) => w >= 440 ? Fit.wide : (w >= 300 ? Fit.normal : Fit.narrow);

class InspectorPanel2 extends StatelessWidget {
  const InspectorPanel2({super.key});
  @override
  Widget build(BuildContext context) => LayoutBuilder(
        builder: (_, c) {
          final f = fitFor(c.maxWidth);
          return DecoratedBox(
            decoration: const BoxDecoration(border: Border(left: BorderSide(color: P.rule2), right: BorderSide(color: P.rule2))),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                _Header(f),
                _Nameplate(f),
                _Views(f),
                _ScatterSurface(f),
                _Folded(f, 'STAGGER', P.blue, const _StaggerMini(), '.12s', true),
                _Folded(f, 'ALONG PATH', P.mint, const _PathMini(), 'Path 1', true),
                _Folded(f, 'FACE', P.lemon, const _FaceMini(), 'Camera', false),
                _AddRelation(f),
                const Spacer(),
                _History(f),
              ],
            ),
          );
        },
      );
}

// --- Header: panel name strongest, alternates weaker, collapses when narrow.
class _Header extends StatelessWidget {
  const _Header(this.f);
  final Fit f;
  @override
  Widget build(BuildContext context) => Container(
        height: 28,
        padding: const EdgeInsets.symmetric(horizontal: 12),
        decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: P.rule2))),
        child: Row(
          children: [
            Text('INSPECTOR', style: P.tabOn.copyWith(fontWeight: FontWeight.w700, letterSpacing: 1.0)),
            if (f != Fit.narrow) ...[
              const SizedBox(width: 18),
              Text('PROJECT', style: P.tab),
              const SizedBox(width: 14),
              Text('LOOK', style: P.tab),
            ],
            const Spacer(),
            const Kebab(),
          ],
        ),
      );
}

// --- Nameplate: what this instrument currently holds.
class _Nameplate extends StatelessWidget {
  const _Nameplate(this.f);
  final Fit f;
  @override
  Widget build(BuildContext context) => Container(
        height: f == Fit.narrow ? 52 : 60,
        padding: const EdgeInsets.symmetric(horizontal: 12),
        child: Row(
          children: [
            Container(width: f == Fit.narrow ? 20 : 26, height: f == Fit.narrow ? 20 : 26, color: P.pink),
            const SizedBox(width: 12),
            Expanded(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('JEWEL FIELD', overflow: TextOverflow.clip, softWrap: false, style: P.title.copyWith(fontSize: f == Fit.narrow ? 14 : 17, letterSpacing: 0.6)),
                  const SizedBox(height: 6),
                  Text(f == Fit.narrow ? 'Group · 300 obj' : 'Group  ·  300 objects  ·  6.0 s', overflow: TextOverflow.clip, softWrap: false, style: P.numMuted),
                ],
              ),
            ),
            const Icon1(Glyph.eye, size: 15, color: P.text2, stroke: 1.3),
          ],
        ),
      );
}

// --- Meaning level. Underline, not a boxed segmented control.
class _Views extends StatelessWidget {
  const _Views(this.f);
  final Fit f;
  static const names = ['TRANSFORM', 'RELATIONS', 'EFFECTS', 'MATERIAL'];
  @override
  Widget build(BuildContext context) => Container(
        height: 30,
        decoration: const BoxDecoration(border: Border(top: BorderSide(color: P.rule), bottom: BorderSide(color: P.rule2))),
        child: Row(
          children: [
            for (final (i, t) in names.indexed)
              Expanded(
                child: Container(
                  alignment: Alignment.center,
                  decoration: BoxDecoration(border: Border(bottom: BorderSide(color: i == 1 ? P.text : const Color(0x00000000), width: 2))),
                  child: FittedBox(fit: BoxFit.scaleDown, child: Text(t, style: (i == 1 ? P.tabOn : P.tab).copyWith(fontSize: f == Fit.narrow ? 9 : 10.5, letterSpacing: f == Fit.narrow ? 0.2 : 0.6))),
                ),
              ),
          ],
        ),
      );
}

// --- The open relation. No card: the panel itself is the Scatter's chassis.
class _ScatterSurface extends StatelessWidget {
  const _ScatterSurface(this.f);
  final Fit f;
  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: const BoxDecoration(color: Color(0xFF181818), border: Border(bottom: BorderSide(color: P.rule2))),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Container(
            height: 40,
            padding: const EdgeInsets.symmetric(horizontal: 12),
            child: Row(
              children: [
                const Ident(P.pink, size: 12),
                const SizedBox(width: 10),
                Text('SCATTER', style: P.row.copyWith(fontSize: 13, letterSpacing: 1.0, fontWeight: FontWeight.w700)),
                if (f != Fit.narrow) ...[const SizedBox(width: 12), Text('300 → field', style: P.numMuted)],
                const Spacer(),
                const Switch1(on: true, color: P.pink),
              ],
            ),
          ),
          if (f == Fit.wide) SizedBox(height: 236, child: Row(children: [const Expanded(child: Padding(padding: EdgeInsets.fromLTRB(12, 0, 4, 12), child: ScatterField())), SizedBox(width: 176, child: Padding(padding: const EdgeInsets.fromLTRB(8, 4, 16, 14), child: _Precision(f, stacked: true)))]))
          else ...[
            SizedBox(height: f == Fit.normal ? 176 : 104, child: const Padding(padding: EdgeInsets.fromLTRB(12, 0, 12, 8), child: ScatterField())),
            Padding(padding: EdgeInsets.fromLTRB(12, 4, 12, 12), child: _Precision(f, stacked: false)),
          ],
        ],
      ),
    );
  }
}

class _Precision extends StatelessWidget {
  const _Precision(this.f, {required this.stacked});
  final Fit f;
  final bool stacked;
  @override
  Widget build(BuildContext context) => Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          _p('DENSITY', '.72', .72),
          _gap(),
          _p('SPREAD', '.48', .48),
          _gap(),
          _falloff(),
          _gap(),
          _p('SEED', '42', null),
          const SizedBox(height: 12),
          const _Shapes(),
        ],
      );
  Widget _gap() => SizedBox(height: stacked ? 14 : 8);
  Widget _p(String l, String v, double? t) => stacked
      ? Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [Row(children: [Text(l, style: P.numMuted), const Spacer(), Text(v, style: P.num)]), const SizedBox(height: 7), _Bar(t)])
      : SizedBox(height: 16, child: Row(children: [SizedBox(width: f == Fit.narrow ? 54 : 64, child: Text(l, style: P.numMuted)), Expanded(child: Padding(padding: const EdgeInsets.symmetric(horizontal: 8), child: _Bar(t))), SizedBox(width: 24, child: Text(v, textAlign: TextAlign.right, style: P.num))]));
  Widget _falloff() => stacked
      ? Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [Row(children: [Text('FALLOFF', style: P.numMuted), const Spacer(), Text('.36', style: P.num)]), const SizedBox(height: 4), const SizedBox(height: 30, child: CustomPaint(painter: _Curve()))])
      : SizedBox(height: 22, child: Row(children: [SizedBox(width: f == Fit.narrow ? 54 : 64, child: Text('FALLOFF', style: P.numMuted)), const Expanded(child: Padding(padding: EdgeInsets.symmetric(horizontal: 8), child: CustomPaint(painter: _Curve(), size: Size.infinite))), SizedBox(width: 24, child: Text('.36', textAlign: TextAlign.right, style: P.num))]));
}

class _Bar extends StatelessWidget {
  const _Bar(this.t);
  final double? t;
  @override
  Widget build(BuildContext context) => LayoutBuilder(
        builder: (_, c) => SizedBox(
          height: 10,
          child: t == null
              ? Align(alignment: Alignment.centerLeft, child: Container(height: 1, width: c.maxWidth, color: P.rule2))
              : Stack(alignment: Alignment.centerLeft, clipBehavior: Clip.none, children: [
                  Container(height: 2, width: c.maxWidth, color: P.rule2),
                  Container(height: 2, width: c.maxWidth * t!, color: P.pink),
                  Positioned(left: c.maxWidth * t! - 4, child: Container(width: 8, height: 8, decoration: const BoxDecoration(color: P.pink, shape: BoxShape.circle))),
                ]),
        ),
      );
}

class _Curve extends CustomPainter {
  const _Curve();
  @override
  void paint(Canvas cv, Size s) {
    final p = Path()..moveTo(0, s.height * .9)..cubicTo(s.width * .3, s.height * .9, s.width * .4, s.height * .12, s.width * .55, s.height * .12)..cubicTo(s.width * .7, s.height * .12, s.width * .8, s.height * .95, s.width, s.height * .95);
    cv.drawPath(p, Paint()..color = P.pink..style = PaintingStyle.stroke..strokeWidth = 1.3);
    cv.drawCircle(Offset(s.width * .55, s.height * .12), 2.5, Paint()..color = P.pink);
  }
  @override
  bool shouldRepaint(_Curve o) => false;
}

class _Shapes extends StatelessWidget {
  const _Shapes();
  @override
  Widget build(BuildContext context) => Row(
        children: [
          Container(width: 22, height: 22, color: P.pink, child: const Center(child: Icon1(Glyph.circle, size: 14, color: P.ink, stroke: 1.8))),
          const SizedBox(width: 8),
          const Icon1(Glyph.rect, size: 15, color: P.text2, stroke: 1.4),
          const SizedBox(width: 8),
          const Icon1(Glyph.stylize, size: 15, color: P.text2, stroke: 1.4),
          const SizedBox(width: 8),
          Text('×', style: P.body.copyWith(fontSize: 16, color: P.text2, height: 1)),
          const SizedBox(width: 8),
          const Icon1(Glyph.star, size: 14, color: P.text2),
        ],
      );
}

// Phenomenon, with the same handles the numbers move.
class ScatterField extends StatelessWidget {
  const ScatterField({super.key});
  @override
  Widget build(BuildContext context) => const CustomPaint(painter: _FieldPainter(), size: Size.infinite);
}

class _FieldPainter extends CustomPainter {
  const _FieldPainter();
  @override
  void paint(Canvas cv, Size s) {
    final c = Offset(s.width / 2, s.height / 2);
    final r = s.shortestSide / 2 - 6;
    cv.drawCircle(c, r, Paint()..shader = RadialGradient(colors: [P.pink.withValues(alpha: .12), P.pink.withValues(alpha: 0)], stops: const [.36, 1]).createShader(Rect.fromCircle(center: c, radius: r)));
    final dash = Paint()..color = P.pink.withValues(alpha: .7)..style = PaintingStyle.stroke..strokeWidth = 1;
    for (final m in (Path()..addOval(Rect.fromCircle(center: c, radius: r * .92))).computeMetrics()) {
      var d = 0.0;
      while (d < m.length) {
        cv.drawPath(m.extractPath(d, d + 4), dash);
        d += 8;
      }
    }
    final rnd = math.Random(42);
    final dot = Paint()..color = P.pink;
    final n = (r * 1.1).round().clamp(40, 120);
    for (var i = 0; i < n; i++) {
      final a = rnd.nextDouble() * math.pi * 2;
      final d = math.sqrt(rnd.nextDouble()) * r * .88;
      final o = c + Offset(math.cos(a) * d, math.sin(a) * d);
      final k = rnd.nextInt(3);
      final sz = (1.6 + rnd.nextDouble() * 4.5 * (1 - d / r * .5)) * (r / 80).clamp(.7, 1.3);
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
    cv.drawLine(c - const Offset(7, 0), c + const Offset(7, 0), cross);
    cv.drawLine(c - const Offset(0, 7), c + const Offset(0, 7), cross);
    final h1 = c + Offset(math.cos(-.9) * r * .92, math.sin(-.9) * r * .92);
    cv.drawCircle(h1, 5, Paint()..color = const Color(0xFF181818));
    cv.drawCircle(h1, 5, Paint()..color = P.pink..style = PaintingStyle.stroke..strokeWidth = 1.5);
  }
  @override
  bool shouldRepaint(_FieldPainter o) => false;
}

// --- A folded relation is a miniature instrument: the phenomenon stays visible.
class _Folded extends StatelessWidget {
  const _Folded(this.f, this.name, this.color, this.face, this.note, this.on);
  final Fit f;
  final String name, note;
  final Color color;
  final Widget face;
  final bool on;
  @override
  Widget build(BuildContext context) => Container(
        height: 36,
        padding: const EdgeInsets.symmetric(horizontal: 12),
        decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: P.rule))),
        child: Row(
          children: [
            Ident(on ? color : P.rule2, size: 12),
            const SizedBox(width: 10),
            SizedBox(width: f == Fit.narrow ? 86 : 108, child: Text(name, softWrap: false, overflow: TextOverflow.clip, style: P.row.copyWith(fontSize: f == Fit.narrow ? 10 : 12, letterSpacing: f == Fit.narrow ? 0.4 : 0.8, fontWeight: FontWeight.w600, color: on ? P.text : P.muted))),
            Expanded(child: Padding(padding: const EdgeInsets.only(right: 10), child: SizedBox(height: 16, child: Opacity(opacity: on ? 1 : .3, child: face)))),
            if (f != Fit.narrow) ...[SizedBox(width: 44, child: Text(note, textAlign: TextAlign.right, style: P.numMuted)), const SizedBox(width: 12)],
            Switch1(on: on, color: color, w: 26, h: 14),
          ],
        ),
      );
}

class _StaggerMini extends StatelessWidget {
  const _StaggerMini();
  @override
  Widget build(BuildContext context) => const CustomPaint(painter: _Slope());
}

class _Slope extends CustomPainter {
  const _Slope();
  @override
  void paint(Canvas cv, Size s) {
    final n = math.max(6, (s.width / 6).floor());
    final w = s.width / n;
    for (var i = 0; i < n; i++) {
      final h = s.height * (0.2 + i / (n - 1) * .8);
      cv.drawRect(Rect.fromLTWH(i * w, s.height - h, w - 2, h), Paint()..color = P.blue);
    }
  }
  @override
  bool shouldRepaint(_Slope o) => false;
}

class _PathMini extends StatelessWidget {
  const _PathMini();
  @override
  Widget build(BuildContext context) => const CustomPaint(painter: _Marks());
}

class _Marks extends CustomPainter {
  const _Marks();
  @override
  void paint(Canvas cv, Size s) {
    final path = Path()..moveTo(0, s.height * .85)..cubicTo(s.width * .3, -s.height * .4, s.width * .6, s.height * 1.4, s.width, s.height * .15);
    cv.drawPath(path, Paint()..color = P.mint.withValues(alpha: .5)..style = PaintingStyle.stroke..strokeWidth = 1);
    for (final m in path.computeMetrics()) {
      for (var i = 0; i <= 9; i++) {
        cv.drawCircle(m.getTangentForOffset(m.length * i / 9)!.position, 2, Paint()..color = P.mint);
      }
    }
  }
  @override
  bool shouldRepaint(_Marks o) => false;
}

class _FaceMini extends StatelessWidget {
  const _FaceMini();
  @override
  Widget build(BuildContext context) => const CustomPaint(painter: _Aim());
}

class _Aim extends CustomPainter {
  const _Aim();
  @override
  void paint(Canvas cv, Size s) {
    final p = Paint()..color = P.lemon..strokeWidth = 1.2..style = PaintingStyle.stroke;
    final t = Offset(s.width - 6, s.height / 2);
    final n = math.max(3, (s.width / 22).floor());
    for (var i = 0; i < n; i++) {
      final o = Offset(6 + i * ((s.width - 30) / n), i.isEven ? 3 : s.height - 3);
      final d = (t - o) / (t - o).distance * 9;
      cv.drawLine(o, o + d, p);
    }
    cv.drawCircle(t, 3.5, p);
    cv.drawCircle(t, 1.2, Paint()..color = P.lemon);
  }
  @override
  bool shouldRepaint(_Aim o) => false;
}

class _AddRelation extends StatelessWidget {
  const _AddRelation(this.f);
  final Fit f;
  @override
  Widget build(BuildContext context) => Container(
        height: 34,
        padding: const EdgeInsets.symmetric(horizontal: 12),
        decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: P.rule))),
        child: Row(children: [const Icon1(Glyph.plus, size: 12, color: P.text2), const SizedBox(width: 10), Text(f == Fit.narrow ? 'ADD' : 'ADD RELATION', style: P.tab.copyWith(color: P.text2))]),
      );
}

class _History extends StatelessWidget {
  const _History(this.f);
  final Fit f;
  @override
  Widget build(BuildContext context) => Container(
        height: 28,
        padding: const EdgeInsets.symmetric(horizontal: 12),
        decoration: const BoxDecoration(border: Border(top: BorderSide(color: P.rule2))),
        child: Row(children: [Text('HISTORY', style: P.section), const SizedBox(width: 10), if (f != Fit.narrow) Expanded(child: Text('Scatter density .68 → .72', overflow: TextOverflow.clip, softWrap: false, style: P.numMuted)) else const Spacer(), Text('24', style: P.numMuted)]),
      );
}
