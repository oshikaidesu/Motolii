// A relation shown three ways: a tile (browser), a card with its gadget (inspector), and the gadget alone (a small picture you can hold).
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'controls.dart';

/// The family's mark: drawn, not a font glyph, so it carries the family's colour and stays crisp at any size.
class FamIcon extends StatelessWidget {
  const FamIcon(this.fam, {super.key, this.size = 22, this.color});
  final Fam fam;
  final double size;
  final Color? color;
  @override
  Widget build(BuildContext context) => SizedBox.square(dimension: size, child: CustomPaint(painter: _Icon(fam, color ?? fam.c)));
}

class _Icon extends CustomPainter {
  const _Icon(this.fam, this.c);
  final Fam fam;
  final Color c;
  @override
  void paint(Canvas cv, Size s) {
    final w = s.width, st = Paint()..color = c..style = PaintingStyle.stroke..strokeWidth = w * .09..strokeCap = StrokeCap.round, fl = Paint()..color = c;
    switch (fam.name) {
      case 'Scatter':
        for (final p in const [Offset(.2, .3), Offset(.5, .18), Offset(.78, .34), Offset(.34, .62), Offset(.66, .7), Offset(.22, .84), Offset(.84, .84)]) {
          cv.drawCircle(p * w, w * .075, fl);
        }
      case 'Along Path':
        cv.drawPath(Path()..moveTo(w * .12, w * .78)..cubicTo(w * .3, w * .1, w * .55, w * .95, w * .88, w * .22), st);
        cv.drawCircle(Offset(w * .88, w * .22), w * .08, fl);
      case 'Stagger':
        for (var i = 0; i < 4; i++) {
          cv.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(w * (.12 + i * .12), w * (.14 + i * .2), w * .46, w * .12), Radius.circular(w * .03)), fl..color = c.withValues(alpha: .4 + i * .2));
        }
      case 'Face':
        cv.drawCircle(Offset(w / 2, w / 2), w * .34, st);
        cv.drawCircle(Offset(w / 2, w / 2), w * .09, fl..color = c);
      case 'Follow':
        cv.drawLine(Offset(w * .12, w * .5), Offset(w * .8, w * .5), st);
        cv.drawPath(Path()..moveTo(w * .6, w * .28)..lineTo(w * .86, w * .5)..lineTo(w * .6, w * .72), st);
      default: // Attach
        cv.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(w * .1, w * .36, w * .46, w * .28), Radius.circular(w * .14)), st);
        cv.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(w * .44, w * .36, w * .46, w * .28), Radius.circular(w * .14)), st);
    }
  }

  @override
  bool shouldRepaint(_Icon o) => o.fam != fam || o.c != c;
}

/// A tile of the browser. concept: filled with the family colour (as in the art); quiet: grey, only the mark is coloured; glow: dark, a tinted edge and a soft light behind the mark.
class RelationTile extends StatelessWidget {
  const RelationTile(this.fam, {super.key, this.look = Look.concept, this.width = 88, this.height = 76});
  final Fam fam;
  final Look look;
  final double width, height;
  @override
  Widget build(BuildContext context) {
    final filled = look == Look.concept;
    final ink = filled ? N.g07 : N.g91;
    return Container(
      width: width,
      height: height,
      decoration: BoxDecoration(
        color: filled ? fam.c : N.g13,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: look == Look.glow ? fam.c.withValues(alpha: .45) : (filled ? const Color(0x00000000) : N.g20), width: 1),
        boxShadow: look == Look.glow ? [BoxShadow(color: fam.c.withValues(alpha: .22), blurRadius: 16)] : null,
      ),
      child: Flex(direction: Axis.vertical, mainAxisAlignment: MainAxisAlignment.center, children: [
        FamIcon(fam, color: filled ? N.g07 : fam.c),
        const SizedBox(height: 7),
        Text(fam.name, style: T.name(ink).copyWith(fontSize: 10.5)),
      ]),
    );
  }
}

/// The gadgets: a relation's parameters as one picture you can hold. Pictures only; the handles are drawn, nothing moves.
class Gadget extends StatelessWidget {
  const Gadget(this.fam, {super.key, this.size = 150});
  final Fam fam;
  final double size;
  @override
  Widget build(BuildContext context) => SizedBox.square(dimension: size, child: CustomPaint(painter: _Gadget(fam)));
}

class _Gadget extends CustomPainter {
  const _Gadget(this.fam);
  final Fam fam;
  @override
  void paint(Canvas cv, Size s) {
    final w = s.width, c = fam.c, mid = Offset(w / 2, w / 2);
    Paint line([double a = 1, double sw = 1.2]) => Paint()..color = c.withValues(alpha: a)..style = PaintingStyle.stroke..strokeWidth = sw;
    void handle(Offset o) {
      cv.drawCircle(o, 6, Paint()..color = c);
      cv.drawCircle(o, 6, Paint()..color = N.g100..style = PaintingStyle.stroke..strokeWidth = 1.5);
    }

    switch (fam.name) {
      case 'Scatter':
        cv.drawCircle(mid, w * .46, line(.5, 1));
        final r = math.Random(3);
        for (var i = 0; i < 46; i++) {
          final d = w * .43 * math.sqrt(r.nextDouble()), a = r.nextDouble() * 6.283;
          cv.drawCircle(mid + Offset(math.cos(a), math.sin(a)) * d, 1.2 + r.nextDouble() * 1.6, Paint()..color = c);
        }
        handle(mid + Offset(w * .3, -w * .3));
      case 'Along Path':
        final p = Path()..moveTo(w * .08, w * .82)..cubicTo(w * .25, w * .1, w * .45, w * .95, w * .62, w * .5)..cubicTo(w * .75, w * .2, w * .85, w * .3, w * .92, w * .14);
        cv.drawPath(p, line(1, 1.6));
        final m = p.computeMetrics().first;
        for (var i = 0; i < 5; i++) {
          cv.drawCircle(m.getTangentForOffset(m.length * i / 4)!.position, i == 4 ? 6 : 3.5, Paint()..color = c);
        }
      case 'Stagger':
        cv.drawLine(Offset(w * .1, w * .1), Offset(w * .1, w * .9), line(.35, 1));
        cv.drawLine(Offset(w * .1, w * .9), Offset(w * .9, w * .9), line(.35, 1));
        for (var i = 0; i < 7; i++) {
          final f = i / 6;
          cv.drawCircle(Offset(w * (.2 + f * .66), w * (.82 - f * .62)), 2.5 + f * 4, Paint()..color = c.withValues(alpha: .35 + f * .65));
        }
      case 'Face':
        cv.drawCircle(mid, w * .36, line(.8));
        cv.drawCircle(mid, w * .18, line(.45));
        cv.drawLine(mid, mid + Offset(w * .36, -w * .14), line());
        handle(mid);
        handle(mid + Offset(w * .36, -w * .14));
      case 'Follow':
        final p = Path()..moveTo(w * .14, w * .72)..cubicTo(w * .35, w * .85, w * .5, w * .42, w * .84, w * .26);
        for (final m in p.computeMetrics()) {
          for (var d = 0.0; d < m.length; d += 11) {
            cv.drawPath(m.extractPath(d, d + 6), line(.8, 1.4));
          }
        }
        handle(Offset(w * .14, w * .72));
        cv.drawPath(Path()..moveTo(w * .76, w * .34)..lineTo(w * .9, w * .24)..lineTo(w * .8, w * .18)..close(), Paint()..color = N.g95);
      default: // Attach
        final a = Offset(w * .3, w * .62), b = Offset(w * .7, w * .38);
        cv.drawLine(a, b, line(.9, 1.4));
        cv.drawCircle(a, 9, line());
        cv.drawCircle(b, 9, line());
        handle(a);
        handle(b);
    }
  }

  @override
  bool shouldRepaint(_Gadget o) => o.fam != fam;
}

/// The inspector's relation card: name, switch, the gadget beside its values. See Look for the three finishes.
class RelationCard extends StatelessWidget {
  const RelationCard(this.fam, {super.key, this.look = Look.concept, this.on = true, this.density = .72, this.spread = .48, this.onToggle});
  final Fam fam;
  final Look look;
  final bool on;
  final double density, spread;
  final ValueChanged<bool>? onToggle;
  @override
  Widget build(BuildContext context) => Container(
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          color: look == Look.quiet ? N.g13 : const Color(0xFF1B1B1D),
          borderRadius: BorderRadius.circular(10),
          border: Border.all(color: look == Look.glow ? fam.c.withValues(alpha: .4) : N.g20),
          boxShadow: look == Look.glow ? [BoxShadow(color: fam.c.withValues(alpha: .16), blurRadius: 24)] : null,
        ),
        child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, mainAxisSize: MainAxisSize.min, children: [
          Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
            Container(width: 10, height: 10, decoration: BoxDecoration(color: fam.c, shape: BoxShape.circle)),
            const SizedBox(width: 9),
            Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
              Text(fam.name, style: T.title()),
              const SizedBox(height: 4),
              Text(fam.jp, style: T.label(N.g56)),
            ]),
            const Spacer(),
            PillSwitch(on: on, fam: fam, onChanged: onToggle),
          ]),
          const SizedBox(height: 12),
          Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.start, children: [
            Gadget(fam, size: 132),
            const SizedBox(width: 12),
            Expanded(child: Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, children: [
              ValueWell(label: 'Density', value: density, fam: fam),
              const SizedBox(height: 6),
              ValueWell(label: 'Spread', value: spread, fam: fam),
              const SizedBox(height: 6),
              ValueWell(label: 'Falloff', value: .36, fam: fam),
            ])),
          ]),
        ]),
      );
}
