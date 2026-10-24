import 'dart:math' as math;
import 'inspector_studies.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_rows_sect.dart';
import '../../parts/controls.dart';
import '../../tokens.dart';
part 'inspector_idea_layer_2.dart';
part 'inspector_idea_layer_3.dart';
part 'inspector_idea_layer_4.dart';

// ---- Masks, time, links, matte: small working stages inside the card -----------------------------------------------
// Sources: motolii-doc store/mask.rs (MaskMode, feather, expansion, opacity, inverted), attrs.rs (Matte, MatteMode, clip_to_below),
// layout/clock.rs + time.rs (speed, loop fold, order delay, ghost), slot.rs (PropertyLink: source layer + property, time offset, scale + offset).

class LayerIdeas extends StatelessWidget {
  const LayerIdeas({super.key});
  @override
  Widget build(BuildContext context) => const Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    mainAxisSize: MainAxisSize.min,
    children: [
      IdeaLabel(
        'A',
        'Mask stage',
        'The masks drawn over the layer, edited in the card: drag points, the edge band for expansion, the outer ring for feather. The cut is the real combination of the modes.',
      ),
      _LiMaskCard(),
      IdeaLabel(
        'B',
        'Time strip',
        'The layer\'s source as a bar under comp time. Drag its end to stretch (speed), the bracket to loop, the faint copy to ghost; hover to see which source frame plays when.',
      ),
      _LiTimeCard(),
      IdeaLabel(
        'C',
        'Link wires',
        'What drives what, as wires: drag from another layer\'s property to one of this layer\'s. The wire carries its map (× and frames), scrubbed sideways in place.',
      ),
      _LiLinkCard(),
      IdeaLabel(
        'D',
        'Matte at a glance',
        'The layer, the one above and the one below as a tiny stack and the picture they make. Click a layer to cut with it, click the picture to try the next mode.',
      ),
      _LiMatteCard(),
    ],
  );
}

// ---- shared ---------------------------------------------------------------------------------------------------------
Paint _liLine(Color c, [double w = 1]) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..strokeJoin = StrokeJoin.round
  ..strokeCap = StrokeCap.round;

double _liRound(double v, int dec) => double.parse(v.toStringAsFixed(dec));

String _liSigned(double v, int dec) =>
    '${v > 0
        ? '+'
        : v < 0
        ? '−'
        : ''}${brief(v.abs(), dec)}';

void _liText(Canvas c, String s, Offset at, TextStyle st, {double ax = 0, Color? bg}) {
  final tp = TextPainter(
    text: TextSpan(text: s, style: st),
    textDirection: TextDirection.ltr,
    maxLines: 1,
  )..layout();
  final o = at - Offset(tp.width * ax, 0);
  final r = Rect.fromLTWH(o.dx - 3, o.dy - 2, tp.width + 6, tp.height + 4);
  if (bg != null) c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(3)), Paint()..color = bg);
  tp.paint(c, o);
}

void _liChecker(Canvas c, Rect r, [double k = 8]) {
  c.save();
  c.clipRRect(RRect.fromRectAndRadius(r, const Radius.circular(4)));
  c.drawRect(r, Paint()..color = Grey.g15);
  final p = Paint()..color = Grey.g20;
  for (var y = 0; r.top + y * k < r.bottom; y++) {
    for (var x = y.isOdd ? 1 : 0; r.left + x * k < r.right; x += 2) {
      c.drawRect(Rect.fromLTWH(r.left + x * k, r.top + y * k, k, k), p);
    }
  }
  c.restore();
}

void _liDash(Canvas c, Path p, Paint paint, [double on = 4, double off = 3]) {
  for (final m in p.computeMetrics()) {
    for (var d = 0.0; d < m.length; d += on + off) {
      c.drawPath(m.extractPath(d, math.min(d + on, m.length)), paint);
    }
  }
}

Widget _liRow(List<Widget> xs) => Padding(
  padding: const EdgeInsets.only(top: 4),
  child: Row(
    children: [
      for (var i = 0; i < xs.length; i++) ...[if (i > 0) SizedBox(width: Pop.tileGap), Expanded(child: xs[i])],
    ],
  ),
);

class _LiChip extends StatelessWidget {
  const _LiChip(this.text, {required this.on, this.onTap, this.tone});
  final String text;
  final bool on;
  final VoidCallback? onTap;
  final Color? tone; // (none: the skin's accent)
  @override
  Widget build(BuildContext context) => Hov(
    onTap: onTap,
    builder: (_, h) => Container(
      height: 18,
      padding: const EdgeInsets.symmetric(horizontal: 7),
      alignment: Alignment.center,
      decoration: BoxDecoration(color: on ? (tone ?? Pop.accent) : (h ? Grey.g26 : Grey.g20), borderRadius: BorderRadius.circular(9)),
      child: Text(text, maxLines: 1, style: T.label(on ? Pop.onAccent : (h ? Grey.g95 : Grey.g76)).copyWith(fontWeight: FontWeight.w600)),
    ),
  );
}

// ---- A. Mask stage --------------------------------------------------------------------------------------------------
const _liModes = ['Add', 'Subtract', 'Intersect', 'Difference'], _liModeWords = ['Add', 'Sub', 'Inter', 'Diff'];
final _liMaskNames = ['Ellipse', 'Box'], _liMaskTones = [Pop.toneMask, Color(0xFFE8C76A)];

/// contract: points are stored 0..1 of the layer's box; feather and expansion in layer px, the layer being [_liLayerW] wide.
const _liStageH = 168.0, _liLayerW = 800.0, _liRingGap = 6.0;

void _liSeedMasks(Doc doc) {
  final n = <String, double>{};
  for (var k = 0; k < 8; k++) {
    final a = k * math.pi / 4;
    n['li_m0.p$k.x'] = _liRound(.38 + .24 * math.cos(a), 3);
    n['li_m0.p$k.y'] = _liRound(.5 + .38 * math.sin(a), 3);
  }
  const box = [(.5, .2), (.84, .2), (.84, .7), (.5, .7)];
  for (final (k, (x, y)) in box.indexed) {
    n['li_m1.p$k.x'] = x;
    n['li_m1.p$k.y'] = y;
  }
  n.addAll({'li_m0.fe': 24, 'li_m0.ex': 0, 'li_m0.op': 100, 'li_m1.fe': 0, 'li_m1.ex': 10, 'li_m1.op': 100});
  doc.seed(n, strs: const {'li_m0.mode': 'Add', 'li_m1.mode': 'Subtract', 'li_m.sel': '0'}, flags: const {'li_m0.inv': false, 'li_m1.inv': false});
}

int _liSel(Doc doc) => int.tryParse(doc.s2['li_m.sel'] ?? '0') ?? 0;

Rect _liLayerRect(Size s) => Rect.fromLTRB(12, 8, s.width - 12, s.height - 20);

double _liArea(List<Offset> p) {
  var a = 0.0;
  for (var i = 0; i < p.length; i++) {
    final q = p[i], r = p[(i + 1) % p.length];
    a += q.dx * r.dy - r.dx * q.dy;
  }
  return a / 2;
}

/// Moves each point out along its corner's bisector so every edge moves [d] (negative = in); a stand-in for a true path offset.
List<Offset> _liGrow(List<Offset> p, double d) {
  if (d.abs() < 1e-6) return p;
  final n = p.length, s = _liArea(p) > 0 ? 1.0 : -1.0;
  Offset nrm(Offset u, Offset v) {
    final e = v - u, l = e.distance;
    return l < 1e-9 ? Offset.zero : Offset(e.dy, -e.dx) * (s / l);
  }

  return [
    for (var i = 0; i < n; i++)
      () {
        final n1 = nrm(p[(i - 1 + n) % n], p[i]), n2 = nrm(p[i], p[(i + 1) % n]), m = n1 + n2, ml = m.distance;
        if (ml < 1e-9) return p[i] + n1 * d;
        final b = m / ml, k = (b.dx * n1.dx + b.dy * n1.dy).clamp(.35, 1.0);
        return p[i] + b * (d / k);
      }(),
  ];
}

/// A closed path through the points: straight, or a Catmull-Rom curve that still passes through every point.
Path _liPath(List<Offset> p, bool smooth) {
  final path = Path();
  if (!smooth) return path..addPolygon(p, true);
  final n = p.length;
  path.moveTo(p[0].dx, p[0].dy);
  for (var i = 0; i < n; i++) {
    final p0 = p[(i - 1 + n) % n], p1 = p[i], p2 = p[(i + 1) % n], p3 = p[(i + 2) % n];
    final c1 = p1 + (p2 - p0) / 6, c2 = p2 - (p3 - p1) / 6;
    path.cubicTo(c1.dx, c1.dy, c2.dx, c2.dy, p2.dx, p2.dy);
  }
  return path..close();
}

/// Signed distance to the polygon: negative inside.
double _liSd(List<Offset> p, Offset q) {
  var d = double.infinity, inside = false;
  for (var i = 0, j = p.length - 1; i < p.length; j = i++) {
    final a = p[j], b = p[i], e = b - a, l2 = e.distanceSquared;
    final t = l2 == 0 ? 0.0 : (((q - a).dx * e.dx + (q - a).dy * e.dy) / l2).clamp(0.0, 1.0);
    d = math.min(d, (q - (a + e * t)).distance);
    if ((a.dy > q.dy) != (b.dy > q.dy) && q.dx < (b.dx - a.dx) * (q.dy - a.dy) / (b.dy - a.dy) + a.dx) inside = !inside;
  }
  return inside ? -d : d;
}

class _LiMask {
  const _LiMask(this.pts, this.ex, this.fe, this.op, this.mode, this.inv, this.smooth, this.tone);

  /// [pts] in stage px; [ex] and [fe] in stage px; [op] 0..1.
  final List<Offset> pts;
  final double ex, fe, op;
  final int mode;
  final bool inv, smooth;
  final Color tone;
  List<Offset> grown([double more = 0]) => _liGrow(pts, ex + more);
  Path path([double more = 0]) => _liPath(grown(more), smooth);
}

List<_LiMask> _liMasks(Doc doc, Rect r) {
  final k = r.width / _liLayerW;
  return [
    for (var m = 0; m < 2; m++)
      _LiMask(
        [for (var i = 0; i < (m == 0 ? 8 : 4); i++) r.topLeft + Offset((doc.get('li_m$m.p$i.x') ?? 0) * r.width, (doc.get('li_m$m.p$i.y') ?? 0) * r.height)],
        (doc.get('li_m$m.ex') ?? 0) * k,
        (doc.get('li_m$m.fe') ?? 0) * k,
        (doc.get('li_m$m.op') ?? 100) / 100,
        math.max(0, _liModes.indexOf(doc.s2['li_m$m.mode'] ?? 'Add')),
        doc.b['li_m$m.inv'] ?? false,
        m == 0,
        _liMaskTones[m],
      ),
  ];
}

void _liArt(Canvas c, Rect r) {
  c.drawRRect(
    RRect.fromRectAndRadius(r, const Radius.circular(4)),
    Paint()
      ..shader = const LinearGradient(
        begin: Alignment.topLeft,
        end: Alignment.bottomRight,
        colors: [Color(0xFF2E4FB8), Color(0xFF9B5CC8), Color(0xFFF08A5D)],
      ).createShader(r),
  );
  c.drawCircle(Offset(r.left + r.width * .66, r.top + r.height * .36), r.height * .2, Paint()..color = const Color(0xFFF2C94C));
  Offset at(double x, double y) => Offset(r.left + r.width * x, r.top + r.height * y);
  final hill = Path()
    ..moveTo(r.left, r.bottom)
    ..lineTo(at(0, .72).dx, at(0, .72).dy)
    ..quadraticBezierTo(at(.3, .48).dx, at(.3, .48).dy, at(.55, .74).dx, at(.55, .74).dy)
    ..quadraticBezierTo(at(.8, .94).dx, at(.8, .94).dy, at(1, .64).dx, at(1, .64).dy)
    ..lineTo(r.right, r.bottom)
    ..close();
  c.save();
  c.clipRRect(RRect.fromRectAndRadius(r, const Radius.circular(4)));
  c.drawPath(hill, Paint()..color = const Color(0xFF1E3A2F));
  c.restore();
}

class _LiMaskCard extends StatelessWidget {
  const _LiMaskCard();
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    _liSeedMasks(doc);
    final s = _liSel(doc);
    String word(int m) => '${_liMaskNames[m]} ${doc.s2['li_m$m.mode']}${(doc.b['li_m$m.inv'] ?? false) ? ' inv' : ''}';
    return Sect(
      title: 'Mask stage',
      tone: Pop.toneMask,
      mark: CardMark.mask,
      keyIds: const ['li_m0.fe', 'li_m0.ex', 'li_m0.op', 'li_m1.fe', 'li_m1.ex', 'li_m1.op'],
      brief: ['2 masks', word(0), word(1)],
      children: [
        const _LiMaskStage(),
        for (var m = 0; m < 2; m++) _LiMaskRow(m, s == m),
        KeyedSubtree(
          key: ValueKey('li_m$s'),
          child: _liRow([
            ldNum(doc, 'li_m$s.fe', 'Feather', unit: 'px', min: 0, max: 200),
            ldNum(doc, 'li_m$s.ex', 'Expansion', unit: 'px', min: -100, max: 100, bipolar: true),
            ldNum(doc, 'li_m$s.op', 'Opacity', unit: '%', min: 0, max: 100),
          ]),
        ),
      ],
    );
  }
}

class _LiMaskRow extends StatelessWidget {
  const _LiMaskRow(this.m, this.sel);
  final int m;
  final bool sel;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, inv = doc.b['li_m$m.inv'] ?? false;
    final mode = math.max(0, _liModes.indexOf(doc.s2['li_m$m.mode'] ?? 'Add'));
    return Padding(
      padding: const EdgeInsets.only(top: 4),
      child: SizedBox(
        height: Pop.cell,
        child: Row(
          children: [
            Expanded(
              child: Hov(
                onTap: () => doc.str('li_m.sel', '$m'),
                builder: (_, h) => Container(
                  height: Pop.cell,
                  padding: const EdgeInsets.symmetric(horizontal: 6),
                  decoration: BoxDecoration(color: sel ? Grey.g20 : (h ? Grey.g15 : null), borderRadius: BorderRadius.circular(6)),
                  child: Row(
                    children: [
                      Container(
                        width: 8,
                        height: 8,
                        decoration: BoxDecoration(color: _liMaskTones[m], borderRadius: BorderRadius.circular(2)),
                      ),
                      const SizedBox(width: 6),
                      Flexible(
                        child: Text(
                          _liMaskNames[m],
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: T.name(sel || h ? Grey.g100 : Grey.g76).copyWith(fontWeight: FontWeight.w600),
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            ),
            const SizedBox(width: 6),
            SizedBox(
              width: 172,
              child: Dis(
                child: Segmented(items: _liModeWords, index: mode, expand: true, onChanged: (i) => doc.str('li_m$m.mode', _liModes[i])),
              ),
            ),
            const SizedBox(width: 4),
            _LiChip('Inv', on: inv, tone: _liMaskTones[m], onTap: () => doc.flag('li_m$m.inv', !inv)),
          ],
        ),
      ),
    );
  }
}

enum _LiGrab { vertex, edge, ring, body }

/// contract: on the selected mask a press takes a point, else the edge band (expansion) or the outer ring (feather), whichever is nearer;
/// inside a mask it moves the mask (selecting it first). One undo per drag.
class _LiMaskStage extends StatefulWidget {
  const _LiMaskStage();
  @override
  State<_LiMaskStage> createState() => _LiMaskStageState();
}
