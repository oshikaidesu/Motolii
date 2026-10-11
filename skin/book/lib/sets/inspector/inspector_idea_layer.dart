part of 'inspector_parts.dart';

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
      for (var i = 0; i < xs.length; i++) ...[if (i > 0) const SizedBox(width: Pop.tileGap), Expanded(child: xs[i])],
    ],
  ),
);

class _LiChip extends StatelessWidget {
  const _LiChip(this.text, {required this.on, this.onTap, this.tone = Pop.accent});
  final String text;
  final bool on;
  final VoidCallback? onTap;
  final Color tone;
  @override
  Widget build(BuildContext context) => Hov(
    onTap: onTap,
    builder: (_, h) => Container(
      height: 18,
      padding: const EdgeInsets.symmetric(horizontal: 7),
      alignment: Alignment.center,
      decoration: BoxDecoration(color: on ? tone : (h ? Grey.g26 : Grey.g20), borderRadius: BorderRadius.circular(9)),
      child: Text(text, maxLines: 1, style: T.label(on ? Pop.onAccent : (h ? Grey.g95 : Grey.g76)).copyWith(fontWeight: FontWeight.w600)),
    ),
  );
}

// ---- A. Mask stage --------------------------------------------------------------------------------------------------
const _liModes = ['Add', 'Subtract', 'Intersect', 'Difference'], _liModeWords = ['Add', 'Sub', 'Inter', 'Diff'];
const _liMaskNames = ['Ellipse', 'Box'], _liMaskTones = [Pop.toneMask, Color(0xFFE8C76A)];

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
            _ldNum(doc, 'li_m$s.fe', 'Feather', unit: 'px', min: 0, max: 200),
            _ldNum(doc, 'li_m$s.ex', 'Expansion', unit: 'px', min: -100, max: 100, bipolar: true),
            _ldNum(doc, 'li_m$s.op', 'Opacity', unit: '%', min: 0, max: 100),
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

class _LiMaskStageState extends State<_LiMaskStage> {
  (_LiGrab, int, int)? _hot, _grab;
  Offset _last = Offset.zero;
  Size _size = Size.zero;

  (_LiGrab, int, int)? _hit(Doc doc, Offset q) {
    final ms = _liMasks(doc, _liLayerRect(_size)), sel = _liSel(doc), s = ms[sel];
    for (final (i, p) in s.pts.indexed) {
      if ((p - q).distance <= 6) return (_LiGrab.vertex, sel, i);
    }
    final d = _liSd(s.pts, q), de = (d - s.ex).abs(), dr = (d - s.ex - _liRingGap - s.fe).abs();
    if (math.min(de, dr) <= 4) return (dr < de ? _LiGrab.ring : _LiGrab.edge, sel, 0);
    if (_liSd(s.grown(), q) < 0) return (_LiGrab.body, sel, 0);
    for (var m = 0; m < ms.length; m++) {
      if (m != sel && _liSd(ms[m].grown(), q) < 0) return (_LiGrab.body, m, 0);
    }
    return null;
  }

  void _move(Doc doc, Offset q) {
    final g = _grab;
    if (g == null) return;
    final r = _liLayerRect(_size), k = r.width / _liLayerW, m = g.$2, mk = _liMasks(doc, r)[m];
    switch (g.$1) {
      case _LiGrab.vertex:
        doc.setMany({
          'li_m$m.p${g.$3}.x': _liRound(((q.dx - r.left) / r.width).clamp(-.1, 1.1), 4),
          'li_m$m.p${g.$3}.y': _liRound(((q.dy - r.top) / r.height).clamp(-.1, 1.1), 4),
        });
      case _LiGrab.edge:
        doc.set('li_m$m.ex', (_liSd(mk.pts, q) / k).clamp(-100.0, 100.0).roundToDouble());
      case _LiGrab.ring:
        doc.set('li_m$m.fe', ((_liSd(mk.pts, q) - mk.ex - _liRingGap) / k).clamp(0.0, 200.0).roundToDouble());
      case _LiGrab.body:
        final d = q - _last;
        doc.setMany({
          for (var i = 0; i < mk.pts.length; i++) ...{
            'li_m$m.p$i.x': _liRound((doc.get('li_m$m.p$i.x') ?? 0) + d.dx / r.width, 4),
            'li_m$m.p$i.y': _liRound((doc.get('li_m$m.p$i.y') ?? 0) + d.dy / r.height, 4),
          },
        });
    }
    _last = q;
  }

  void _end(Doc doc) {
    if (_grab == null) return;
    doc.endGesture();
    setState(() => _grab = null);
  }

  String _note(Doc doc) {
    final g = _grab, h = _hot, sel = _liSel(doc);
    if (g != null) {
      return switch (g.$1) {
        _LiGrab.edge => 'Expansion ${_liSigned(doc.get('li_m${g.$2}.ex') ?? 0, 0)} px',
        _LiGrab.ring => 'Feather ${brief(doc.get('li_m${g.$2}.fe'), 0)} px',
        _LiGrab.vertex => 'Point ${g.$3 + 1}',
        _LiGrab.body => 'Move ${_liMaskNames[g.$2]}',
      };
    }
    return switch (h?.$1) {
      _LiGrab.vertex => 'Drag the point',
      _LiGrab.edge => 'Edge: drag in or out to expand',
      _LiGrab.ring => 'Ring: drag out to feather',
      _LiGrab.body => h!.$2 == sel ? 'Drag to move' : 'Click to edit ${_liMaskNames[h.$2]}',
      null => '${_liMaskNames[sel]} · points, edge, ring',
    };
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, on = !x.cfg.locked;
    return LayoutBuilder(
      builder: (context, box) {
        _size = Size(box.maxWidth, _liStageH);
        final r = _liLayerRect(_size), g = _grab ?? _hot;
        return MouseRegion(
          cursor: !on || g == null
              ? SystemMouseCursors.basic
              : switch (g.$1) {
                  _LiGrab.vertex => SystemMouseCursors.precise,
                  _LiGrab.body => g.$2 == _liSel(doc) ? SystemMouseCursors.move : SystemMouseCursors.click,
                  _ => SystemMouseCursors.grab,
                },
          onHover: (e) {
            final h = on ? _hit(doc, e.localPosition) : null;
            if (h != _hot) setState(() => _hot = h);
          },
          onExit: (_) {
            if (_hot != null) setState(() => _hot = null);
          },
          child: Listener(
            onPointerDown: (e) {
              if (!on || e.buttons != kPrimaryButton) return;
              final h = _hit(doc, e.localPosition);
              if (h == null) return;
              _last = e.localPosition;
              doc.beginGesture();
              setState(() => _grab = h);
              if ('${h.$2}' != doc.s2['li_m.sel']) doc.str('li_m.sel', '${h.$2}');
            },
            onPointerMove: (e) => _move(doc, e.localPosition),
            onPointerUp: (_) => _end(doc),
            onPointerCancel: (_) {
              if (_grab == null) return;
              doc.cancelGesture();
              setState(() => _grab = null);
            },
            child: CustomPaint(size: _size, painter: _LiMaskPaint(_liMasks(doc, r), _liSel(doc), r, g, _note(doc))),
          ),
        );
      },
    );
  }
}

class _LiMaskPaint extends CustomPainter {
  const _LiMaskPaint(this.ms, this.sel, this.r, this.hot, this.note);
  final List<_LiMask> ms;
  final int sel;
  final Rect r;
  final (_LiGrab, int, int)? hot;
  final String note;

  static const _blend = [BlendMode.srcOver, BlendMode.dstOut, BlendMode.dstIn, BlendMode.xor];
  static const _ops = [PathOperation.union, PathOperation.difference, PathOperation.intersect, PathOperation.xor];

  @override
  void paint(Canvas c, Size s) {
    final box = Offset.zero & s, full = Path()..addRect(box), startFull = ms.first.mode == 1 || ms.first.mode == 2;
    c.drawRRect(RRect.fromRectAndRadius(box, const Radius.circular(6)), Paint()..color = Pop.well);
    _liChecker(c, r);
    c.saveLayer(r, Paint()..color = const Color(0x2E000000));
    _liArt(c, r);
    c.restore();
    Path shape(_LiMask m) => m.inv ? Path.combine(PathOperation.difference, full, m.path()) : m.path();
    // the feathered picture: each mask's alpha laid down by its mode, then the art kept where the alpha is
    c.saveLayer(box, Paint());
    if (startFull) c.drawRect(box, Paint()..color = Grey.g100);
    for (final m in ms) {
      final p = Paint()
        ..color = Grey.g100.withValues(alpha: m.op)
        ..blendMode = _blend[m.mode];
      if (m.fe > .3) p.maskFilter = MaskFilter.blur(BlurStyle.normal, m.fe / 2);
      c.drawPath(shape(m), p);
    }
    c.saveLayer(box, Paint()..blendMode = BlendMode.srcIn);
    _liArt(c, r);
    c.restore();
    c.restore();
    // the hard cut, by path operations, as a thin line
    var cut = startFull ? (Path()..addRect(r)) : Path();
    for (final m in ms) {
      cut = Path.combine(_ops[m.mode], cut, shape(m));
    }
    c.drawPath(Path.combine(PathOperation.intersect, cut, Path()..addRect(r)), _liLine(Grey.g100.withValues(alpha: .5)));
    for (var m = 0; m < ms.length; m++) {
      final k = ms[m];
      if (m != sel) {
        _liDash(c, k.path(), _liLine(k.tone.withValues(alpha: .8)));
        continue;
      }
      final mine = hot != null && hot!.$2 == m;
      if (k.ex.abs() > .5) c.drawPath(_liPath(k.pts, k.smooth), _liLine(Grey.g56));
      c.drawPath(k.path(), _liLine(k.tone, mine && hot!.$1 == _LiGrab.edge ? 3 : 1.5));
      final ring = mine && hot!.$1 == _LiGrab.ring;
      _liDash(c, k.path(_liRingGap + k.fe), _liLine(k.tone.withValues(alpha: ring ? 1 : .55), ring ? 1.8 : 1));
      for (final (i, p) in k.pts.indexed) {
        final h = mine && hot!.$1 == _LiGrab.vertex && hot!.$3 == i, q = Rect.fromCenter(center: p, width: h ? 8 : 6, height: h ? 8 : 6);
        c.drawRect(q, Paint()..color = h ? k.tone : Grey.g07);
        c.drawRect(q, _liLine(k.tone));
      }
    }
    _liText(c, note, Offset(12, s.height - 15), T.label(Grey.g76));
  }

  @override
  bool shouldRepaint(_LiMaskPaint o) => true;
}

// ---- B. Time strip --------------------------------------------------------------------------------------------------
const _liSrcLen = 2.0, _liWin = 6.0, _liFps = 30.0, _liLoopModes = ['Off', 'Loop', 'Ping-pong'];
const _liBrTop = 14.0, _liBrBot = 28.0, _liBarTop = 31.0, _liBarBot = 51.0, _liGhTop = 55.0, _liGhBot = 67.0, _liStripH = 90.0;

/// contract: comp time t reads source time (t - delay) × speed; Loop folds it into [0, loop), Ping-pong runs every other fold backwards;
/// without a loop the layer ends with its source ([_liSrcLen]). Ghost is the same clock read [ghost] seconds later.
class _LiClock {
  _LiClock(Doc doc)
    : sp = math.max(.05, (doc.get('li_t.speed') ?? 100) / 100),
      loop = doc.get('li_t.loop') ?? 1,
      delay = doc.get('li_t.delay') ?? 0,
      mode = math.max(0, _liLoopModes.indexOf(doc.s2['li_t.mode'] ?? 'Off')),
      ghost = (doc.b['li_t.ghostOn'] ?? false) ? (doc.get('li_t.ghost') ?? 0) / _liFps : null;
  final double sp, loop, delay;
  final int mode;
  final double? ghost;
  double get cycle => mode == 0 || loop <= 1e-6 ? _liSrcLen : loop;
  double get cycleEnd => delay + cycle / sp;
  double? src(double t) {
    final s = (t - delay) * sp;
    if (s < 0) return null;
    if (mode == 0 || loop <= 1e-6) return s <= _liSrcLen ? s : null;
    final r = (s / loop).floor(), a = s - r * loop;
    return mode == 2 && r.isOdd ? loop - a : a;
  }
}

double _liX(Size s, double t) => 8 + (s.width - 16) * t / _liWin;
double _liT(Size s, double x) => (x - 8) / (s.width - 16) * _liWin;

class _LiTimeCard extends StatelessWidget {
  const _LiTimeCard();
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    doc.seed(
      const {'li_t.speed': 100, 'li_t.loop': 1.2, 'li_t.delay': .3, 'li_t.ghost': 6},
      strs: const {'li_t.mode': 'Ping-pong'},
      flags: const {'li_t.ghostOn': true},
    );
    final k = _LiClock(doc);
    return Sect(
      title: 'Time strip',
      tone: Pop.toneTime,
      mark: CardMark.clock,
      keyIds: const ['li_t.speed', 'li_t.loop', 'li_t.delay', 'li_t.ghost'],
      brief: [
        'Speed ${brief(k.sp * 100, 0)}%',
        if (k.mode > 0) '${_liLoopModes[k.mode]} ${brief(k.loop, 2)} s',
        if (k.delay != 0) '${_liSigned(k.delay, 2)} s',
        if (k.ghost != null) 'Ghost ${brief(doc.get('li_t.ghost'), 0)} f',
      ],
      children: [
        const _LiTimeStrip(),
        Padding(
          padding: const EdgeInsets.only(top: 4),
          child: SizedBox(
            height: Pop.cell,
            child: Dis(
              child: Segmented(items: _liLoopModes, index: k.mode, expand: true, onChanged: (i) => doc.str('li_t.mode', _liLoopModes[i])),
            ),
          ),
        ),
        _liRow([
          _ldNum(doc, 'li_t.speed', 'Speed', unit: '%', min: 5, max: 800),
          _ldNum(doc, 'li_t.loop', 'Loop', unit: 's', dec: 2, min: .1, max: _liSrcLen, perPx: .01, step: .1, enabled: k.mode > 0),
          _ldNum(doc, 'li_t.delay', 'Delay', unit: 's', dec: 2, min: -2, max: 4, perPx: .01, step: .1, bipolar: true),
          _ldNum(doc, 'li_t.ghost', 'Ghost', unit: 'f', min: -60, max: 60, perPx: .25, bipolar: true, enabled: k.ghost != null),
        ]),
      ],
    );
  }
}

enum _LiTGrab { speed, loop, bar, ghost }

/// contract: the bracket row sets the loop length (a click flips Loop / Ping-pong), the bar's end grip sets speed, the bar body the delay,
/// the faint row the ghost (a click turns it on or off). Hover only reads; one undo per drag.
class _LiTimeStrip extends StatefulWidget {
  const _LiTimeStrip();
  @override
  State<_LiTimeStrip> createState() => _LiTimeStripState();
}

class _LiTimeStripState extends State<_LiTimeStrip> {
  double? _hx;
  _LiTGrab? _grab, _hot;
  Offset _down = Offset.zero;
  double _start = 0, _acc = 0;
  bool _moved = false;
  Size _size = Size.zero;

  _LiTGrab? _hit(_LiClock k, Offset q) {
    final y = q.dy;
    if (y >= _liBrTop - 3 && y < _liBarTop - 1) return _LiTGrab.loop;
    if (y >= _liBarTop - 1 && y <= _liBarBot + 1) {
      if ((q.dx - _liX(_size, k.cycleEnd)).abs() <= 6) return _LiTGrab.speed;
      if (q.dx >= _liX(_size, k.delay) - 2 && (k.mode > 0 || q.dx <= _liX(_size, k.cycleEnd))) return _LiTGrab.bar;
      return null;
    }
    if (y > _liBarBot + 1 && y <= _liGhBot + 3) return _LiTGrab.ghost;
    return null;
  }

  void _move(Doc doc, Offset q, double dx) {
    final g = _grab;
    if (g == null) return;
    if (!_moved && (q - _down).distance < 3) return;
    _moved = true;
    _acc += dx;
    final k = _LiClock(doc), t = _liT(_size, q.dx), perPx = _liWin / (_size.width - 16);
    switch (g) {
      case _LiTGrab.loop:
        if (k.mode == 0) doc.s2['li_t.mode'] = 'Loop';
        doc.set('li_t.loop', _liRound(((t - k.delay) * k.sp).clamp(.1, _liSrcLen), 2));
      case _LiTGrab.speed:
        doc.set('li_t.speed', (k.cycle / math.max(.05, t - k.delay) * 100).clamp(5.0, 800.0).roundToDouble());
      case _LiTGrab.bar:
        doc.set('li_t.delay', _liRound((_start + _acc * perPx).clamp(-2.0, 4.0), 2));
      case _LiTGrab.ghost:
        doc.b['li_t.ghostOn'] = true;
        doc.set('li_t.ghost', (_start + _acc * perPx * _liFps).clamp(-60.0, 60.0).roundToDouble());
    }
  }

  void _up(Doc doc) {
    final g = _grab;
    if (g == null) return;
    if (!_moved) {
      final k = _LiClock(doc);
      if (g == _LiTGrab.loop) doc.str('li_t.mode', k.mode == 2 ? 'Loop' : 'Ping-pong');
      if (g == _LiTGrab.ghost) doc.flag('li_t.ghostOn', k.ghost == null);
    }
    doc.endGesture();
    setState(() => _grab = null);
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, on = !x.cfg.locked, k = _LiClock(doc);
    return LayoutBuilder(
      builder: (context, box) {
        _size = Size(box.maxWidth, _liStripH);
        final g = _grab ?? _hot;
        return MouseRegion(
          cursor: !on || g == null
              ? SystemMouseCursors.basic
              : g == _LiTGrab.bar
              ? SystemMouseCursors.grab
              : SystemMouseCursors.resizeLeftRight,
          onHover: (e) => setState(() {
            _hx = e.localPosition.dx;
            _hot = on ? _hit(k, e.localPosition) : null;
          }),
          onExit: (_) => setState(() {
            _hx = null;
            _hot = null;
          }),
          child: Listener(
            onPointerDown: (e) {
              if (!on || e.buttons != kPrimaryButton) return;
              final h = _hit(k, e.localPosition);
              if (h == null) return;
              _down = e.localPosition;
              _moved = false;
              _acc = 0;
              _start = h == _LiTGrab.ghost ? (doc.get('li_t.ghost') ?? 0) : k.delay;
              doc.beginGesture();
              setState(() => _grab = h);
            },
            onPointerMove: (e) {
              _hx = e.localPosition.dx;
              _move(doc, e.localPosition, e.delta.dx);
            },
            onPointerUp: (_) => _up(doc),
            onPointerCancel: (_) {
              if (_grab == null) return;
              doc.cancelGesture();
              setState(() => _grab = null);
            },
            child: CustomPaint(size: _size, painter: _LiTimePaint(k, _hx, g)),
          ),
        );
      },
    );
  }
}

class _LiTimePaint extends CustomPainter {
  const _LiTimePaint(this.k, this.hx, this.hot);
  final _LiClock k;
  final double? hx;
  final _LiTGrab? hot;

  static Color _shade(double s) => Color.lerp(const Color(0xFF2A1A22), Pop.toneTime, (s / _liSrcLen).clamp(0.0, 1.0))!;

  /// The bar as columns coloured by the source time they play, so speed reads as compression and Ping-pong as a mirrored ramp.
  void _bar(Canvas c, Size s, double top, double bot, double shift, double alpha) {
    double? prev;
    for (var px = math.max(8.0, _liX(s, k.delay + shift)); px < s.width - 8; px += 2) {
      final sv = k.src(_liT(s, px) - shift);
      if (sv == null) {
        if (prev != null) break;
        continue;
      }
      final first = _liT(s, px) - shift <= k.cycleEnd;
      c.drawRect(Rect.fromLTWH(px, top, 2, bot - top), Paint()..color = _shade(sv).withValues(alpha: alpha * (first ? 1 : .6)));
      if (prev != null && (sv * 4).floor() != (prev * 4).floor()) {
        c.drawLine(Offset(px, top), Offset(px, top + (bot - top) * .35), _liLine(Grey.g07.withValues(alpha: .6 * alpha)));
      }
      prev = sv;
    }
  }

  @override
  void paint(Canvas c, Size s) {
    c.drawRRect(RRect.fromRectAndRadius(Offset.zero & s, const Radius.circular(6)), Paint()..color = Pop.well);
    for (var i = 0; i <= _liWin; i++) {
      final x = _liX(s, i.toDouble());
      c.drawLine(Offset(x, 2), Offset(x, _liGhBot), _liLine(i == 0 ? Grey.g38 : Grey.g20));
      if (i < _liWin) _liText(c, '${i}s', Offset(x + 3, 2), T.label(Grey.g44));
    }
    final x0 = _liX(s, k.delay), xe = _liX(s, k.cycleEnd), mid = (_liBrTop + _liBrBot) / 2;
    // loop bracket
    if (k.mode > 0) {
      for (var t = k.cycleEnd; t < _liWin; t += k.cycle / k.sp) {
        final x = _liX(s, t);
        c.drawLine(Offset(x, mid), Offset(x, _liBrBot), _liLine(Grey.g38));
      }
      final hot = this.hot == _LiTGrab.loop, ink = hot ? Grey.g100 : Pop.toneTime;
      c.drawPath(
        Path()
          ..moveTo(x0, _liBrBot)
          ..lineTo(x0, _liBrTop + 2)
          ..lineTo(xe, _liBrTop + 2)
          ..lineTo(xe, _liBrBot),
        _liLine(ink, hot ? 1.8 : 1.3),
      );
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(xe, mid + 1), width: 5, height: 12), const Radius.circular(2)), Paint()..color = ink);
      _liText(c, '${k.mode == 2 ? '↔' : '→'} ${brief(k.loop, 2)} s', Offset((x0 + xe) / 2, _liBrTop + 4), T.label(Grey.g95), ax: .5, bg: Pop.well);
    } else {
      _liText(c, '+ Loop: drag here', Offset(x0 + 2, _liBrTop + 3), T.label(hot == _LiTGrab.loop ? Grey.g91 : Grey.g44));
    }
    // the bar, its outline, the speed grip
    _bar(c, s, _liBarTop, _liBarBot, 0, 1);
    final xEnd = k.mode == 0 ? xe : s.width - 8;
    c.drawRRect(
      RRect.fromRectAndRadius(Rect.fromLTRB(math.max(8, x0), _liBarTop, math.min(xEnd, s.width - 8), _liBarBot), const Radius.circular(3)),
      _liLine(hot == _LiTGrab.bar ? Grey.g91 : Grey.g44),
    );
    final sHot = hot == _LiTGrab.speed;
    c.drawRRect(
      RRect.fromRectAndRadius(
        Rect.fromCenter(center: Offset(xe, (_liBarTop + _liBarBot) / 2), width: sHot ? 6 : 4, height: _liBarBot - _liBarTop + 6),
        const Radius.circular(2),
      ),
      Paint()..color = sHot ? Pop.accent : Grey.g95,
    );
    final spw = '${brief(k.sp * 100, 0)}%', right = xe + 40 < s.width - 8;
    _liText(
      c,
      spw,
      Offset(right ? xe + 6 : xe - 6, _liBarTop + 5),
      T.label(Grey.g95).copyWith(fontWeight: FontWeight.w700),
      ax: right ? 0 : 1,
      bg: Pop.well,
    );
    // ghost
    final g = k.ghost;
    if (g != null) {
      _bar(c, s, _liGhTop, _liGhBot, g, hot == _LiTGrab.ghost ? .7 : .45);
      _liText(c, 'Ghost ${_liSigned(g * _liFps, 0)} f', Offset(math.max(10, _liX(s, k.delay + g)) + 2, _liGhTop + 1), T.label(Grey.g76));
    } else {
      _liDash(
        c,
        Path()..addRRect(RRect.fromRectAndRadius(Rect.fromLTRB(math.max(8, x0), _liGhTop, math.min(xe, s.width - 8), _liGhBot), const Radius.circular(3))),
        _liLine(hot == _LiTGrab.ghost ? Grey.g76 : Grey.g38),
      );
      _liText(c, 'Ghost: drag or click', Offset(math.max(8, x0) + 4, _liGhTop + 1), T.label(Grey.g44));
    }
    // hover: which source frame plays at this comp time
    final h = hx;
    final ry = _liGhBot + 7;
    if (h != null && h >= 8 && h <= s.width - 8) {
      final t = _liT(s, h), sv = k.src(t), gv = g == null ? null : k.src(t - g);
      c.drawLine(Offset(h, 2), Offset(h, _liGhBot), _liLine(Grey.g95.withValues(alpha: .7)));
      if (sv != null) c.drawCircle(Offset(h, (_liBarTop + _liBarBot) / 2), 3, Paint()..color = Grey.g100);
      _liText(
        c,
        '${t.toStringAsFixed(2)} s  →  ${sv == null ? 'not playing' : 'source f ${(sv * _liFps).round()}'}${gv == null ? '' : '  ·  ghost f ${(gv * _liFps).round()}'}',
        Offset(8, ry),
        T.label(Grey.g91),
      );
      if (sv != null) {
        final o = Offset(s.width - 16, ry + 5);
        c.save();
        c.translate(o.dx, o.dy);
        c.rotate(sv / _liSrcLen * math.pi);
        c.drawRRect(
          RRect.fromRectAndRadius(Rect.fromCenter(center: Offset.zero, width: 10, height: 10), const Radius.circular(2)),
          Paint()..color = _shade(sv),
        );
        c.restore();
      }
    } else {
      final end = k.mode == 0 ? 'ends ${brief(k.cycleEnd, 2)} s' : 'loops to the end';
      _liText(c, 'Source ${brief(_liSrcLen, 1)} s · ${brief(k.sp * 100, 0)}% · $end · hover to read', Offset(8, ry), T.label(Grey.g56));
    }
  }

  @override
  bool shouldRepaint(_LiTimePaint o) =>
      o.hx != hx || o.hot != hot || o.k.sp != k.sp || o.k.loop != k.loop || o.k.delay != k.delay || o.k.mode != k.mode || o.k.ghost != k.ghost;
}

// ---- C. Link wires --------------------------------------------------------------------------------------------------
const _liSrcs = [
  ('Logo', 'Opacity', 80.0),
  ('Logo', 'Rotation', 30.0),
  ('Null 1', 'Position X', 240.0),
  ('Null 1', 'Rotation', 90.0),
  ('Audio', 'Level', 60.0),
];
const _liTgts = [('Opacity', 62.0), ('Position X', 420.0), ('Position Y', 540.0), ('Rotation', 0.0), ('Scale', 100.0), ('Blur', 24.0)];
const _liPitch = 24.0, _liPillH = 20.0, _liSrcW = 118.0, _liTgtW = 112.0, _liNodeH = 6 * _liPitch + 4, _liTagW = 96.0;

Rect _liSrcRect(int i) => Rect.fromLTWH(0, 4 + (_liTgts.length - _liSrcs.length) * _liPitch / 2 + i * _liPitch, _liSrcW, _liPillH);
Rect _liTgtRect(double w, int i) => Rect.fromLTWH(w - _liTgtW, 4 + i * _liPitch, _liTgtW, _liPillH);

/// contract: one source per target (PropertyLink lives on the driven property); the shown value is source × Multiply + Offset, Delay in frames.
List<(int, int)> _liLinks(Doc doc) => [
  for (var t = 0; t < _liTgts.length; t++)
    if (int.tryParse(doc.s2['li_l.$t.src'] ?? '') case final s?) (t, s),
];

(Offset, Offset, Offset, Offset) _liWire(Offset a, Offset b) {
  final d = math.max(24.0, (b.dx - a.dx) * .5);
  return (a, a + Offset(d, 0), b - Offset(d, 0), b);
}

Offset _liBez((Offset, Offset, Offset, Offset) w, double t) {
  final u = 1 - t;
  return w.$1 * (u * u * u) + w.$2 * (3 * u * u * t) + w.$3 * (3 * u * t * t) + w.$4 * (t * t * t);
}

class _LiLinkCard extends StatelessWidget {
  const _LiLinkCard();
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    doc.seed(
      {
        for (var t = 0; t < _liTgts.length; t++) ...{'li_l.$t.mul': t == 0 ? .5 : 1, 'li_l.$t.add': 0, 'li_l.$t.f': t == 0 ? 5 : 0},
      },
      strs: const {'li_l.0.src': '0', 'li_l.3.src': '3', 'li_l.sel': '0'},
    );
    final links = _liLinks(doc), sel = int.tryParse(doc.s2['li_l.sel'] ?? '');
    final cur = links.where((l) => l.$1 == sel).firstOrNull;
    return Sect(
      title: 'Link wires',
      tone: Pop.toneLink,
      mark: CardMark.link,
      keyIds: [
        for (final (t, _) in links) ...['li_l.$t.mul', 'li_l.$t.add'],
      ],
      brief: [
        links.isEmpty ? 'No links' : '${links.length} link${links.length == 1 ? '' : 's'}',
        for (final (t, s) in links.take(2)) '${_liTgts[t].$1} ← ${_liSrcs[s].$1}',
      ],
      children: [
        const _LiWires(),
        if (cur == null)
          const Cap('Drag from a property on the left to one of this layer\'s on the right. Click a wire to edit it.')
        else ...[
          Padding(
            padding: const EdgeInsets.only(top: 6),
            child: SizedBox(
              height: 18,
              child: Row(
                children: [
                  Expanded(
                    child: Text(
                      '${_liTgts[cur.$1].$1}  ←  ${_liSrcs[cur.$2].$1} · ${_liSrcs[cur.$2].$2}',
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: T.name(Grey.g91).copyWith(fontWeight: FontWeight.w600),
                    ),
                  ),
                  _LiChip(
                    'Unlink',
                    on: false,
                    onTap: () {
                      doc.s2['li_l.sel'] = '';
                      doc.str('li_l.${cur.$1}.src', '');
                    },
                  ),
                ],
              ),
            ),
          ),
          KeyedSubtree(
            key: ValueKey('li_l${cur.$1}'),
            child: _liRow([
              _ldNum(doc, 'li_l.${cur.$1}.mul', 'Multiply', dec: 2, min: -4, max: 4, perPx: .01, step: .1, bipolar: true),
              _ldNum(doc, 'li_l.${cur.$1}.add', 'Offset', dec: 1, perPx: .5),
              _ldNum(doc, 'li_l.${cur.$1}.f', 'Delay', unit: 'f', min: -120, max: 120, perPx: .25, zero: 'None', bipolar: true),
            ]),
          ),
        ],
      ],
    );
  }
}

/// contract: a press on a left pill draws a wire; let go on a right pill to link (replacing that property's link). A click on a wire,
/// its tag or a driven property selects that link; the tag's two numbers scrub sideways.
class _LiWires extends StatefulWidget {
  const _LiWires();
  @override
  State<_LiWires> createState() => _LiWiresState();
}

class _LiWiresState extends State<_LiWires> {
  int? _from, _over;
  Offset? _wire;
  Offset _down = Offset.zero;
  bool _moved = false;

  int? _tgtAt(double w, Offset p) {
    for (var t = 0; t < _liTgts.length; t++) {
      if (_liTgtRect(w, t).inflate(3).contains(p)) return t;
    }
    return null;
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, on = !x.cfg.locked;
    final links = _liLinks(doc), sel = int.tryParse(doc.s2['li_l.sel'] ?? '');
    return LayoutBuilder(
      builder: (context, box) {
        final w = box.maxWidth, midX = (_liSrcW + w - _liTgtW) / 2;
        (Offset, Offset, Offset, Offset) wire((int, int) l) =>
            _liWire(Offset(_liSrcRect(l.$2).right, _liSrcRect(l.$2).center.dy), Offset(_liTgtRect(w, l.$1).left, _liTgtRect(w, l.$1).center.dy));
        final tags = <(int, double)>[];
        for (final l in [...links]..sort((a, b) => _liBez(wire(a), .5).dy.compareTo(_liBez(wire(b), .5).dy))) {
          final y = _liBez(wire(l), .5).dy.clamp(9.0, _liNodeH - 9);
          tags.add((l.$1, tags.isEmpty ? y : math.max(y, tags.last.$2 + 19)));
        }
        double value(int t) {
          final s = int.tryParse(doc.s2['li_l.$t.src'] ?? '');
          return s == null ? _liTgts[t].$2 : _liSrcs[s].$3 * (doc.get('li_l.$t.mul') ?? 1) + (doc.get('li_l.$t.add') ?? 0);
        }

        return Padding(
          padding: const EdgeInsets.only(top: 2),
          child: Listener(
            onPointerDown: (e) {
              final p = e.localPosition;
              _down = p;
              _moved = false;
              if (!on || e.buttons != kPrimaryButton) return;
              for (var i = 0; i < _liSrcs.length; i++) {
                if (_liSrcRect(i).inflate(2).contains(p)) {
                  setState(() {
                    _from = i;
                    _wire = p;
                  });
                  return;
                }
              }
            },
            onPointerMove: (e) {
              if ((e.localPosition - _down).distance > 3) _moved = true;
              if (_from == null) return;
              setState(() {
                _wire = e.localPosition;
                _over = _tgtAt(w, e.localPosition);
              });
            },
            onPointerUp: (e) {
              final p = e.localPosition, f = _from;
              if (f != null) {
                final t = _tgtAt(w, p);
                setState(() {
                  _from = null;
                  _wire = null;
                  _over = null;
                });
                if (t != null && _moved) {
                  doc.s2['li_l.sel'] = '$t';
                  doc.str('li_l.$t.src', '$f');
                }
                return;
              }
              if (_moved || !on) return;
              final t = _tgtAt(w, p);
              if (t != null) {
                if (links.any((l) => l.$1 == t)) doc.str('li_l.sel', '$t');
                return;
              }
              for (final l in links) {
                final c = wire(l);
                for (var i = 0; i <= 24; i++) {
                  if ((_liBez(c, i / 24) - p).distance < 6) {
                    doc.str('li_l.sel', '${l.$1}');
                    return;
                  }
                }
              }
            },
            onPointerCancel: (_) => setState(() {
              _from = null;
              _wire = null;
              _over = null;
            }),
            child: SizedBox(
              height: _liNodeH,
              child: Stack(
                children: [
                  Positioned.fill(
                    child: CustomPaint(
                      painter: _LiWirePaint([for (final l in links) (wire(l), l.$1 == sel)], _from == null ? null : (_liSrcRect(_from!).centerRight, _wire!)),
                    ),
                  ),
                  for (var i = 0; i < _liSrcs.length; i++)
                    Positioned.fromRect(
                      rect: _liSrcRect(i),
                      child: _LiSrcPill(i, used: links.any((l) => l.$2 == i), live: _from == i, key: ValueKey('li_src$i')),
                    ),
                  for (var t = 0; t < _liTgts.length; t++)
                    Positioned.fromRect(
                      rect: _liTgtRect(w, t),
                      child: _LiTgtPill(
                        t,
                        value(t),
                        driven: links.any((l) => l.$1 == t),
                        hot: _over == t || sel == t && links.any((l) => l.$1 == t),
                        key: ValueKey('li_tgt$t'),
                      ),
                    ),
                  for (final (t, y) in tags)
                    Positioned(
                      left: midX - _liTagW / 2,
                      top: y - 9,
                      width: _liTagW,
                      height: 18,
                      child: _LiTag(t, sel: sel == t),
                    ),
                ],
              ),
            ),
          ),
        );
      },
    );
  }
}

class _LiSrcPill extends StatelessWidget {
  const _LiSrcPill(this.i, {required this.used, required this.live, super.key});
  final int i;
  final bool used, live;
  @override
  Widget build(BuildContext context) => MouseRegion(
    cursor: SystemMouseCursors.grab,
    child: Container(
      padding: const EdgeInsets.only(left: 7, right: 4),
      decoration: BoxDecoration(color: live ? Grey.g26 : Grey.g20, borderRadius: BorderRadius.circular(10)),
      child: Row(
        children: [
          Text(_liSrcs[i].$1, maxLines: 1, style: T.label(Grey.g56)),
          const SizedBox(width: 4),
          Expanded(
            child: Text(
              _liSrcs[i].$2,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: T.label(Grey.g91).copyWith(fontWeight: FontWeight.w600),
            ),
          ),
          Container(
            width: 8,
            height: 8,
            decoration: BoxDecoration(
              color: used || live ? Pop.toneLink : Grey.g07,
              shape: BoxShape.circle,
              border: Border.all(color: Pop.toneLink),
            ),
          ),
        ],
      ),
    ),
  );
}

class _LiTgtPill extends StatelessWidget {
  const _LiTgtPill(this.t, this.v, {required this.driven, required this.hot, super.key});
  final int t;
  final double v;
  final bool driven, hot;
  @override
  Widget build(BuildContext context) => Container(
    padding: const EdgeInsets.only(left: 4, right: 7),
    decoration: BoxDecoration(
      color: hot ? Grey.g26 : Grey.g20,
      borderRadius: BorderRadius.circular(10),
      border: Border.all(color: hot ? Pop.toneLink : const Color(0x00000000)),
    ),
    child: Row(
      children: [
        Container(
          width: 8,
          height: 8,
          decoration: BoxDecoration(
            color: driven ? Pop.toneLink : Grey.g07,
            shape: BoxShape.circle,
            border: Border.all(color: driven ? Pop.toneLink : Grey.g44),
          ),
        ),
        const SizedBox(width: 5),
        Expanded(
          child: Text(
            _liTgts[t].$1,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: T.label(Grey.g91).copyWith(fontWeight: FontWeight.w600),
          ),
        ),
        Text(brief(v, 1), maxLines: 1, style: T.value(driven ? Pop.toneLink : Grey.g56).copyWith(fontSize: 10)),
      ],
    ),
  );
}

/// The wire's map on the wire: Multiply and Delay, each a sideways scrub; a click selects the link.
class _LiTag extends StatelessWidget {
  const _LiTag(this.t, {required this.sel});
  final int t;
  final bool sel;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, f = doc.get('li_l.$t.f') ?? 0;
    void pick() {
      if (doc.s2['li_l.sel'] != '$t') doc.str('li_l.sel', '$t');
    }

    return Container(
      decoration: BoxDecoration(
        color: Grey.g07,
        borderRadius: BorderRadius.circular(9),
        border: Border.all(color: sel ? Pop.toneLink : Grey.g38, width: sel ? 1.4 : 1),
      ),
      child: Row(
        children: [
          Expanded(child: _LiScrub('li_l.$t.mul', '× ${brief(doc.get('li_l.$t.mul'), 2)}', perPx: .01, min: -4, max: 4, dec: 2, onTap: pick)),
          Container(width: 1, height: 10, color: Grey.g38),
          Expanded(child: _LiScrub('li_l.$t.f', f == 0 ? '0 f' : '${_liSigned(f, 0)} f', perPx: .25, min: -120, max: 120, dec: 0, onTap: pick)),
        ],
      ),
    );
  }
}

class _LiScrub extends StatefulWidget {
  const _LiScrub(this.id, this.text, {required this.perPx, required this.min, required this.max, required this.dec, required this.onTap});
  final String id, text;
  final double perPx, min, max;
  final int dec;
  final VoidCallback onTap;
  @override
  State<_LiScrub> createState() => _LiScrubState();
}

class _LiScrubState extends State<_LiScrub> {
  double _start = 0, _acc = 0;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, on = !x.cfg.locked;
    return MouseRegion(
      cursor: on ? SystemMouseCursors.resizeLeftRight : SystemMouseCursors.basic,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: widget.onTap,
        onHorizontalDragStart: on
            ? (_) {
                _start = doc.get(widget.id) ?? 0;
                _acc = 0;
                doc.beginGesture();
                widget.onTap();
              }
            : null,
        onHorizontalDragUpdate: on
            ? (e) {
                _acc += e.delta.dx * x.cfg.mult();
                doc.set(widget.id, _liRound((_start + _acc * widget.perPx).clamp(widget.min, widget.max), widget.dec));
              }
            : null,
        onHorizontalDragEnd: on ? (_) => doc.endGesture() : null,
        onHorizontalDragCancel: on ? doc.cancelGesture : null,
        child: Center(child: Text(widget.text, maxLines: 1, style: T.value(Grey.g95).copyWith(fontSize: 10))),
      ),
    );
  }
}

class _LiWirePaint extends CustomPainter {
  const _LiWirePaint(this.wires, this.live);
  final List<((Offset, Offset, Offset, Offset), bool)> wires;
  final (Offset, Offset)? live;
  @override
  void paint(Canvas c, Size s) {
    Path bez((Offset, Offset, Offset, Offset) w) => Path()
      ..moveTo(w.$1.dx, w.$1.dy)
      ..cubicTo(w.$2.dx, w.$2.dy, w.$3.dx, w.$3.dy, w.$4.dx, w.$4.dy);
    for (final (w, sel) in [...wires]..sort((a, b) => a.$2 ? 1 : (b.$2 ? -1 : 0))) {
      c.drawPath(bez(w), _liLine(sel ? Pop.toneLink : Pop.toneLink.withValues(alpha: .5), sel ? 2 : 1.3));
    }
    final l = live;
    if (l != null) _liDash(c, bez(_liWire(l.$1, l.$2)), _liLine(Grey.g95, 1.5));
  }

  @override
  bool shouldRepaint(_LiWirePaint o) => true;
}

// ---- D. Matte at a glance -------------------------------------------------------------------------------------------
const _liMatteModes = ['Alpha', 'Inverted Alpha', 'Luma', 'Inverted Luma'], _liMatteWords = ['Alpha', 'Inv α', 'Luma', 'Inv L'];

/// Top to bottom; index 1 is this layer, 2 is the layer below (the clip base).
const _liStack = ['Title', 'Jewel', 'Card'];
const _liLuma = ColorFilter.matrix([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, .2126, .7152, .0722, 0, 0]);

void _liLayer(Canvas c, Rect r, int i) {
  switch (i) {
    case 0:
      final tp = TextPainter(
        text: TextSpan(
          text: 'JEWEL',
          style: TextStyle(
            fontFamily: T.sans,
            fontSize: r.height * .4,
            fontWeight: FontWeight.w900,
            height: 1,
            foreground: Paint()..shader = LinearGradient(colors: [Grey.g100, Grey.g26]).createShader(r.deflate(r.width * .12)),
          ),
        ),
        textDirection: TextDirection.ltr,
      )..layout();
      tp.paint(c, r.center - Offset(tp.width / 2, tp.height / 2));
    case 1:
      const cols = [Pop.toneTime, Pop.tonePlace, Pop.toneTextAnim, Pop.tonePath, Pop.toneSolid];
      final w = r.width / 6;
      c.save();
      c.clipRect(r);
      for (var k = -4; k < 8; k++) {
        final x = r.left + k * w;
        c.drawPath(
          Path()
            ..moveTo(x, r.bottom)
            ..lineTo(x + w, r.bottom)
            ..lineTo(x + w + r.height, r.top)
            ..lineTo(x + r.height, r.top)
            ..close(),
          Paint()..color = cols[(k + 4) % cols.length],
        );
      }
      c.restore();
    default:
      c.drawRRect(
        RRect.fromRectAndRadius(Rect.fromCenter(center: r.center, width: r.width * .66, height: r.height * .7), Radius.circular(r.height * .14)),
        Paint()..color = const Color(0xFF55627A),
      );
  }
}

class _LiMatteCard extends StatelessWidget {
  const _LiMatteCard();
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    doc.seed(const {}, strs: const {'li_d.src': 'Title', 'li_d.mode': 'Luma'}, flags: const {'li_d.clip': false});
    final src = doc.s2['li_d.src'] ?? 'None', none = src == 'None', clip = doc.b['li_d.clip'] ?? false;
    final mode = math.max(0, _liMatteModes.indexOf(doc.s2['li_d.mode'] ?? 'Alpha'));
    final where = const ['is opaque', 'is clear', 'is bright', 'is dark'][mode];
    final say = none
        ? (clip ? 'Jewel shows only inside Card, the layer below.' : 'Jewel draws whole. Click Title or Card to cut it with that layer.')
        : 'Jewel shows where $src $where${clip ? ', and only inside Card' : ''}. $src itself is not drawn.';
    return Sect(
      title: 'Matte & clip',
      tone: Pop.toneBlend,
      mark: CardMark.overlay,
      brief: [none ? 'No matte' : 'Matte $src', if (!none) _liMatteModes[mode], if (clip) 'Clip to below'],
      children: [
        Padding(
          padding: const EdgeInsets.only(top: 2),
          child: SizedBox(
            height: 124,
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                SizedBox(
                  width: 128,
                  child: Column(
                    children: [
                      for (var i = 0; i < _liStack.length; i++) ...[if (i > 0) const SizedBox(height: 4), Expanded(child: _LiStackTile(i))],
                    ],
                  ),
                ),
                const SizedBox(width: 6),
                const Expanded(child: _LiMattePreview(key: ValueKey('li_matte_preview'))),
              ],
            ),
          ),
        ),
        Padding(
          padding: const EdgeInsets.only(top: 4),
          child: Row(
            children: [
              Expanded(
                child: SizedBox(
                  height: Pop.cell,
                  child: Dis(
                    child: Opacity(
                      opacity: none ? .5 : 1,
                      child: Segmented(
                        items: _liMatteWords,
                        index: none ? -1 : mode,
                        expand: true,
                        onChanged: (i) {
                          if (none) doc.s2['li_d.src'] = 'Title';
                          doc.str('li_d.mode', _liMatteModes[i]);
                        },
                      ),
                    ),
                  ),
                ),
              ),
              const SizedBox(width: 6),
              _LiChip('Clip to below', on: clip, tone: Pop.toneBlend, onTap: () => doc.flag('li_d.clip', !clip)),
            ],
          ),
        ),
        Cap(say),
      ],
    );
  }
}

class _LiStackTile extends StatelessWidget {
  const _LiStackTile(this.i);
  final int i;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, name = _liStack[i], me = i == 1, used = doc.s2['li_d.src'] == name;
    final clip = doc.b['li_d.clip'] ?? false, on = !me && !x.cfg.locked;
    final role = me ? 'This layer' : (used ? 'Matte · hidden' : (i == 2 && clip ? 'Clip base' : (i == 0 ? 'Above' : 'Below')));
    return Hov(
      onTap: on ? () => doc.str('li_d.src', used ? 'None' : name) : null,
      cursor: on ? SystemMouseCursors.click : SystemMouseCursors.basic,
      builder: (_, h) => Container(
        padding: const EdgeInsets.all(4),
        decoration: BoxDecoration(
          color: used ? Grey.g20 : (h && on ? Grey.g15 : Pop.well),
          borderRadius: BorderRadius.circular(6),
          border: Border.all(color: used ? Pop.toneBlend : (me ? Grey.g38 : const Color(0x00000000))),
        ),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            SizedBox(width: 44, child: CustomPaint(painter: _LiThumb(i, used))),
            const SizedBox(width: 6),
            Expanded(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(name, maxLines: 1, style: T.name(me || used || h ? Grey.g100 : Grey.g76).copyWith(fontWeight: FontWeight.w600)),
                  const SizedBox(height: 3),
                  Text(role, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(used ? Pop.toneBlend : Grey.g56)),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _LiThumb extends CustomPainter {
  const _LiThumb(this.i, this.dim);
  final int i;
  final bool dim;
  @override
  void paint(Canvas c, Size s) {
    final r = Offset.zero & s;
    _liChecker(c, r, 4);
    c.save();
    c.clipRRect(RRect.fromRectAndRadius(r, const Radius.circular(4)));
    c.saveLayer(r, Paint()..color = Color.fromRGBO(0, 0, 0, dim ? .55 : 1));
    _liLayer(c, r, i);
    c.restore();
    c.restore();
  }

  @override
  bool shouldRepaint(_LiThumb o) => o.i != i || o.dim != dim;
}

/// contract: a click on the picture steps the matte mode (with no matte it takes the layer above, as a matte usually is).
class _LiMattePreview extends StatelessWidget {
  const _LiMattePreview({super.key});
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, src = doc.s2['li_d.src'] ?? 'None', clip = doc.b['li_d.clip'] ?? false;
    final mode = math.max(0, _liMatteModes.indexOf(doc.s2['li_d.mode'] ?? 'Alpha'));
    return Hov(
      onTap: x.cfg.locked ? null : () => src == 'None' ? doc.str('li_d.src', 'Title') : doc.str('li_d.mode', _liMatteModes[(mode + 1) % _liMatteModes.length]),
      builder: (_, h) => CustomPaint(painter: _LiMattePaint(_liStack.indexOf(src), mode, clip, h)),
    );
  }
}

class _LiMattePaint extends CustomPainter {
  const _LiMattePaint(this.src, this.mode, this.clip, this.hot);
  final int src, mode;
  final bool clip, hot;
  @override
  void paint(Canvas c, Size s) {
    final box = Offset.zero & s, r = box.deflate(5);
    c.drawRRect(RRect.fromRectAndRadius(box, const Radius.circular(6)), Paint()..color = hot ? Grey.g20 : Pop.well);
    _liChecker(c, r, 6);
    c.save();
    c.clipRRect(RRect.fromRectAndRadius(r, const Radius.circular(4)));
    if (src != 2) _liLayer(c, r, 2);
    c.saveLayer(r, Paint());
    _liLayer(c, r, 1);
    if (clip) {
      c.saveLayer(r, Paint()..blendMode = BlendMode.dstIn);
      _liLayer(c, r, 2);
      c.restore();
    }
    if (src >= 0) {
      final p = Paint()..blendMode = mode.isOdd ? BlendMode.dstOut : BlendMode.dstIn;
      if (mode >= 2) p.colorFilter = _liLuma;
      c.saveLayer(r, p);
      _liLayer(c, r, src);
      c.restore();
    }
    c.restore();
    if (src != 0) _liLayer(c, r, 0);
    c.restore();
    final tag = [if (src >= 0) '${_liMatteModes[mode]} · ${_liStack[src]}', if (clip) 'Clip'].join('  ·  ');
    if (tag.isNotEmpty)
      _liText(c, tag, Offset(r.left + 6, r.top + 5), T.label(Grey.g95).copyWith(fontWeight: FontWeight.w600), bg: Grey.g07.withValues(alpha: .8));
    final hint = src < 0 ? 'Click: matte with Title' : 'Click: next mode';
    if (hot) _liText(c, hint, Offset(r.right - 6, r.bottom - 15), T.label(Grey.g95), ax: 1, bg: Grey.g07.withValues(alpha: .8));
  }

  @override
  bool shouldRepaint(_LiMattePaint o) => o.src != src || o.mode != mode || o.clip != clip || o.hot != hot;
}
