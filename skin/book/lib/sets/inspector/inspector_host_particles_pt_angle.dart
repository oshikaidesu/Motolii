// Direction or Spread as a number block whose picture is the emission cone in the layer's own frame.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';
import 'package:lab_book/tokens.dart';

/// Direction or Spread as a number block whose picture is the emission cone in the layer's own frame.
/// contract: 0° points right and -90° up (y down), unlike the generic dial; Direction's cone turns under the pointer, Spread's number scrubs.
class PtAngle extends StatelessWidget {
  const PtAngle(this.p, {super.key});
  static const dir = 0, spread = 1;
  final int p;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, prm = ptMotion[p], id = 'pt_${prm.id}';
    return RowInfo(
      unit: null,
      keys: doc.keys[id] ?? KeyS.off,
      child: NumField(
        id: id,
        caption: prm.label,
        unit: '°',
        min: prm.min,
        max: prm.max,
        glyph: p == dir
            ? const _PtTurn()
            : CustomPaint(size: const Size(22, 22), painter: _ConePaint(doc.get('pt_direction') ?? 0, doc.get(id) ?? 0, false, true)),
        glyphOwnsPointer: p == dir,
      ),
    );
  }
}

class _PtTurn extends StatefulWidget {
  const _PtTurn();
  @override
  State<_PtTurn> createState() => _PtTurnState();
}

class _PtTurnState extends State<_PtTurn> {
  double? _last;
  bool _hot = false;
  double _at(Offset p) => math.atan2(p.dy - 11, p.dx - 11) * 180 / math.pi;
  void _end() {
    if (_last != null) Ctx.read(context).doc.endGesture();
    setState(() => _last = null);
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, on = !x.cfg.locked && !RowOff.offOf(context);
    return MouseRegion(
      cursor: on ? SystemMouseCursors.grab : SystemMouseCursors.basic,
      onEnter: (_) => setState(() => _hot = true),
      onExit: (_) => setState(() => _hot = false),
      child: Listener(
        onPointerDown: on
            ? (e) {
                doc.beginGesture();
                setState(() => _last = _at(e.localPosition));
              }
            : null,
        onPointerMove: (e) {
          final last = _last;
          if (last == null) return;
          final a = _at(e.localPosition);
          final d = (a - last + 540) % 360 - 180;
          _last = a;
          doc.set('pt_direction', ((doc.get('pt_direction') ?? 0) + d).roundToDouble());
        },
        onPointerUp: (_) => _end(),
        onPointerCancel: (_) => _end(),
        child: CustomPaint(
          size: const Size(22, 22),
          painter: _ConePaint(doc.get('pt_direction') ?? 0, doc.get('pt_spread') ?? 0, _hot || _last != null, false),
        ),
      ),
    );
  }
}

class _ConePaint extends CustomPainter {
  const _ConePaint(this.dir, this.spread, this.hot, this.wide);
  final double dir, spread;
  final bool hot, wide;
  @override
  void paint(Canvas c, Size s) {
    final o = s.center(Offset.zero), a = dir * math.pi / 180, sw = spread.clamp(0.0, 360.0) * math.pi / 180;
    c.drawCircle(o, 9, Paint()..color = hot ? Grey.g38 : Grey.g26);
    c.drawArc(Rect.fromCircle(center: o, radius: 8), a - sw / 2, math.max(sw, .03), true, Paint()..color = wide ? Grey.g76 : Grey.g44);
    c.drawLine(
      o,
      o + Offset(math.cos(a), math.sin(a)) * 8,
      Paint()
        ..color = wide ? Grey.g56 : Grey.g95
        ..strokeWidth = wide ? 1.2 : 2
        ..strokeCap = StrokeCap.round,
    );
    c.drawCircle(o, 1.8, Paint()..color = wide ? Grey.g56 : Grey.g95);
  }

  @override
  bool shouldRepaint(_ConePaint o) => o.dir != dir || o.spread != spread || o.hot != hot || o.wide != wide;
}
