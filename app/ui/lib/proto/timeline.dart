import 'package:flutter/widgets.dart';
import 'icons.dart';
import 'tokens.dart';

// Full width under stage and inspector. Rows 20 px; colour is the track's tape.
const _headH = 30.0, _rulerH = 24.0, _rowH = 20.0, _namesW = 236.0;

class TimelinePanel extends StatelessWidget {
  const TimelinePanel({super.key});
  @override
  Widget build(BuildContext context) => Container(
        decoration: const BoxDecoration(border: Border(top: BorderSide(color: P.rule2))),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const _Head(),
            Expanded(
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  const SizedBox(width: _namesW, child: _Names()),
                  const Rule(vertical: true, color: P.rule2),
                  const Expanded(child: CustomPaint(painter: _TrackPainter())),
                ],
              ),
            ),
          ],
        ),
      );
}

class _Head extends StatelessWidget {
  const _Head();
  @override
  Widget build(BuildContext context) => Container(
        height: _headH,
        decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: P.rule2))),
        child: Row(
          children: [
            for (final (i, t) in ['TIMELINE', 'GRAPH', 'AUDIO'].indexed)
              Container(
                width: 88,
                alignment: Alignment.center,
                decoration: BoxDecoration(color: i == 0 ? P.keyHi : null, border: Border(top: BorderSide(color: i == 0 ? P.text : const Color(0x00000000), width: 2), right: const BorderSide(color: P.rule))),
                child: Text(t, style: i == 0 ? P.tabOn : P.tab),
              ),
            const SizedBox(width: 16),
            Text('0 / 180', style: P.num),
            const SizedBox(width: 16),
            Text('work 0:00 – 6:00', style: P.numMuted),
            const Spacer(),
            for (final t in ['FIT', 'MARKER', 'SNAP'])
              Padding(padding: const EdgeInsets.only(right: 6), child: Cap(height: 20, padding: const EdgeInsets.symmetric(horizontal: 8), child: Text(t, style: P.tab))),
            const SizedBox(width: 8),
            const Padding(padding: EdgeInsets.only(right: 14), child: Kebab()),
          ],
        ),
      );
}

// (name, colour, depth, on). Row 2 (Scatter) is the selected relation.
const _rows = <(String, Color?, int, bool)>[
  ('Jewel Field', P.pink, 0, true),
  ('Transform', null, 1, true),
  ('Scatter', P.pink, 1, true),
  ('Stagger', P.blue, 1, true),
  ('Along Path', P.mint, 1, true),
  ('Face Target', P.lemon, 1, false),
  ('Glow', null, 1, true),
  ('Title Card', null, 0, true),
  ('Camera', null, 0, true),
  ('Audio', null, 0, true),
];

class _Names extends StatelessWidget {
  const _Names();
  @override
  Widget build(BuildContext context) => Column(
        children: [
          Container(
            height: _rulerH,
            padding: const EdgeInsets.only(left: 12, right: 10),
            decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: P.rule2))),
            child: Row(children: [Text('LAYERS', style: P.section), const Spacer(), Text('M  S  L', style: P.numMuted)]),
          ),
          for (final (i, r) in _rows.indexed)
            Container(
              height: _rowH,
              padding: EdgeInsets.only(left: r.$3 == 0 ? 10 : 28, right: 10),
              decoration: BoxDecoration(
                color: i == 2 ? P.keyHi : null,
                border: const Border(bottom: BorderSide(color: Color(0xFF1C1C1C))),
              ),
              child: Row(
                children: [
                  if (r.$3 == 0) ...[
                    Icon1(i == 0 ? Glyph.chevronDown : Glyph.chevronRight, size: 9, color: P.text2),
                    const SizedBox(width: 8),
                    if (r.$2 != null) Container(width: 8, height: 8, margin: const EdgeInsets.only(right: 8), color: r.$2),
                  ],
                  if (r.$3 == 1) Container(width: 12, height: 12, margin: const EdgeInsets.only(right: 8), decoration: BoxDecoration(color: r.$2 == null ? null : (r.$4 ? r.$2 : const Color(0x00000000)), border: Border.all(color: r.$2 ?? P.text2, width: 1.2), borderRadius: BorderRadius.circular(2))),
                  Text(r.$1, style: r.$3 == 0 ? P.trackGroup : (r.$4 ? P.track : P.track.copyWith(color: P.muted))),
                  const Spacer(),
                  if (r.$1 == 'Audio') const Icon1(Glyph.lockSmall, size: 10, color: P.muted) else Text(r.$4 ? '·  ·  ·' : 'm  ·  ·', style: P.numMuted),
                ],
              ),
            ),
        ],
      );
}

class _TrackPainter extends CustomPainter {
  const _TrackPainter();
  @override
  void paint(Canvas cv, Size s) {
    final secW = (s.width - 12) / 6.4;
    double x(double t) => 12 + t * secW;
    double rowY(int i) => _rulerH + i * _rowH + _rowH / 2;
    const phT = 2.13;
    cv.drawRect(Rect.fromLTWH(0, _rulerH + 2 * _rowH, s.width, _rowH), Paint()..color = P.keyHi);
    for (var i = 0; i < _rows.length; i++) {
      cv.drawLine(Offset(0, _rulerH + (i + 1) * _rowH), Offset(s.width, _rulerH + (i + 1) * _rowH), Paint()..color = const Color(0xFF1C1C1C));
    }
    cv.drawLine(Offset(0, _rulerH), Offset(s.width, _rulerH), Paint()..color = P.rule2);
    final tick = Paint()..color = P.rule2..strokeWidth = 1;
    final minor = Paint()..color = const Color(0xFF2A2A2A)..strokeWidth = 1;
    for (var i = 0; i <= 6; i++) {
      cv.drawLine(Offset(x(i.toDouble()), _rulerH - 8), Offset(x(i.toDouble()), _rulerH), tick);
      for (var k = 1; k < 30; k++) {
        cv.drawLine(Offset(x(i + k / 30), _rulerH - (k % 5 == 0 ? 4 : 2)), Offset(x(i + k / 30), _rulerH), minor);
      }
      cv.drawLine(Offset(x(i.toDouble()), _rulerH), Offset(x(i.toDouble()), s.height), Paint()..color = const Color(0xFF2A2A2A));
      final tp = TextPainter(text: TextSpan(text: '0:0$i', style: P.ruler), textDirection: TextDirection.ltr)..layout();
      tp.paint(cv, Offset(x(i.toDouble()) + 4, 5));
      final fp = TextPainter(text: TextSpan(text: '${i * 30}f', style: P.ruler.copyWith(color: P.dim)), textDirection: TextDirection.ltr)..layout();
      fp.paint(cv, Offset(x(i.toDouble()) + 4, 14));
    }
    // Work area.
    cv.drawRect(Rect.fromLTWH(x(0), 0, x(6) - x(0), 3), Paint()..color = P.rule2);
    void bar(int row, double a, double b, Color c, {double alpha = 1}) {
      // Identity strip (dark, saturated) + semi-bright body.
      final r = Rect.fromLTWH(x(a), rowY(row) - 6, x(b) - x(a), 12);
      cv.drawRect(r, Paint()..color = c.withValues(alpha: alpha * .82));
      cv.drawRect(Rect.fromLTWH(r.left, r.top, 4, r.height), Paint()..color = Color.lerp(c, P.ink, .35)!.withValues(alpha: alpha));
    }
    void key(int row, double t, Color c, {bool hollow = false}) {
      cv.save();
      cv.translate(x(t), rowY(row));
      cv.rotate(0.7853981);
      final r = Rect.fromCenter(center: Offset.zero, width: 6.5, height: 6.5);
      cv.drawRect(r, Paint()..color = hollow ? P.bg : c);
      cv.drawRect(r, Paint()..color = hollow ? c : P.ink..style = PaintingStyle.stroke..strokeWidth = 1);
      cv.restore();
    }
    // Group extent, quiet.
    bar(0, 0, 6, P.pink, alpha: .18);
    bar(1, .3, 1.9, P.text2, alpha: .35);
    key(1, .3, P.text2, hollow: true); key(1, 1.1, P.text2); key(1, 1.9, P.text2, hollow: true);
    bar(2, .25, 5.6, P.pink);
    key(2, 1.4, P.pink, hollow: true); key(2, 2.9, P.pink, hollow: true);
    bar(3, .2, 4.4, P.blue);
    key(3, .2, P.blue, hollow: true); key(3, 1.2, P.blue, hollow: true); key(3, 4.4, P.blue, hollow: true);
    bar(4, .35, 5.0, P.mint);
    key(4, .35, P.mint, hollow: true); key(4, 1.5, P.mint, hollow: true);
    bar(5, .35, 3.2, P.lemon, alpha: .35);
    bar(6, 1.0, 4.0, P.text2, alpha: .35);
    key(6, 1.0, P.text2, hollow: true); key(6, 2.0, P.text2, hollow: true);
    bar(7, 4.2, 6.0, P.text2, alpha: .35);
    key(8, 0.9, P.text2, hollow: true); key(8, 3.6, P.text2, hollow: true);
    final ay = rowY(9);
    final wave = Paint()..color = const Color(0xFF5FA88A).withValues(alpha: .7)..strokeWidth = 1;
    var seed = 11;
    for (var px = x(.2); px < x(6.2); px += 2) {
      seed = (seed * 1103515245 + 12345) & 0x7fffffff;
      final a = (seed % 100) / 100 * 6 + 1;
      cv.drawLine(Offset(px, ay - a), Offset(px, ay + a), wave);
    }
    // Marker.
    final mx = x(4.0);
    cv.drawLine(Offset(mx, _rulerH - 8), Offset(mx, s.height), Paint()..color = P.lemon.withValues(alpha: .5)..strokeWidth = 1);
    final mp = TextPainter(text: TextSpan(text: 'DROP', style: P.ruler.copyWith(color: P.lemon)), textDirection: TextDirection.ltr)..layout();
    mp.paint(cv, Offset(mx + 4, _rulerH - 19));
    // Playhead.
    final ph = x(phT);
    cv.drawLine(Offset(ph, 0), Offset(ph, s.height), Paint()..color = P.blue..strokeWidth = 1.5);
    cv.drawPath(Path()..moveTo(ph - 6, 0)..lineTo(ph + 6, 0)..lineTo(ph, 9)..close(), Paint()..color = P.blue);
  }
  @override
  bool shouldRepaint(_TrackPainter o) => false;
}
