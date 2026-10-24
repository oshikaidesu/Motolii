// Relation gadgets: the picture is the control. Every gadget draws in a unit square (0..1), owns its handles, and reports a few numbers underneath.
// Shared machinery: Skin (the three Looks), HandleField (drag, hover, clock), GadgetShell (the card around a gadget).
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../parts/controls.dart';
import '../../tokens.dart';
part 'relation_parts_2.dart';
part 'relation_parts_3.dart';

// [open] a handle's grab radius; the art does not size it.
const _hitR = 16.0;
// [open] corner radius of a gadget's well.
const _wellR = 8.0;
// [open] how far the hover ring grows beyond the handle.
const _haloR = 5.0;
// [open] corner radius of a gadget card (follows RelationCard's 10).
const _cardR = 10.0;
// handle radius: brief says 6px crisp, 1.5px white ring.
const _hr = 6.0;

double _clamp(double v, double a, double b) => v < a ? a : (v > b ? b : v);
Offset _cl(Offset o, [double lo = .04, double hi = .96]) => Offset(_clamp(o.dx, lo, hi), _clamp(o.dy, lo, hi));
double _mix(double a, double b, double t) => a + (b - a) * t;
String _f(double v, [int d = 2]) => v.toStringAsFixed(d);
Offset _rot(Offset o, double a) => Offset(o.dx * math.cos(a) - o.dy * math.sin(a), o.dx * math.sin(a) + o.dy * math.cos(a));
Offset _dir(double a) => Offset(math.cos(a), math.sin(a));

void _dash(Canvas cv, Path p, Paint pt, {double on = 4, double off = 4}) {
  for (final m in p.computeMetrics()) {
    for (var d = 0.0; d < m.length; d += on + off) {
      cv.drawPath(m.extractPath(d, math.min(d + on, m.length)), pt);
    }
  }
}

double _bez(double x1, double y1, double x2, double y2, double x) {
  var lo = 0.0, hi = 1.0;
  for (var i = 0; i < 24; i++) {
    final t = (lo + hi) / 2, bx = 3 * (1 - t) * (1 - t) * t * x1 + 3 * (1 - t) * t * t * x2 + t * t * t;
    if (bx < x) {
      lo = t;
    } else {
      hi = t;
    }
  }
  final t = (lo + hi) / 2;
  return 3 * (1 - t) * (1 - t) * t * y1 + 3 * (1 - t) * t * t * y2 + t * t * t;
}

/// The three finishes. concept: soft fills and family-tinted guides. quiet: outlines only, guides grey. glow: concept plus a soft light behind the marks (the only place a glow is allowed).
class Skin {
  const Skin(this.look, this.c);
  final Look look;
  final Color c;
  bool get glow => look == Look.glow;
  bool get quiet => look == Look.quiet;
  double get fillA => quiet ? 0 : 1;
  double get lw => quiet ? 1.0 : 1.4;
  Color get guide => quiet ? N.g38 : c.withValues(alpha: .38);

  Paint stroke(Color col, [double? w]) => Paint()
    ..color = col
    ..style = PaintingStyle.stroke
    ..strokeWidth = w ?? lw
    ..strokeCap = StrokeCap.round
    ..strokeJoin = StrokeJoin.round;
  Paint fill(Color col) => Paint()..color = col;

  void line(Canvas cv, Path p, {Color? col, double? w, double a = 1}) {
    final k = (col ?? c).withValues(alpha: a);
    if (glow) cv.drawPath(p, stroke(k.withValues(alpha: a * .35), (w ?? lw) * 3.2)..maskFilter = const MaskFilter.blur(BlurStyle.normal, 5));
    cv.drawPath(p, stroke(k, w));
  }

  void ring(Canvas cv, Offset p, double r, {Color? col, double a = 1, double? w}) => line(cv, Path()..addOval(Rect.fromCircle(center: p, radius: r)), col: col, a: a, w: w);

  void dot(Canvas cv, Offset p, double r, {Color? col, double a = 1}) {
    final k = (col ?? c).withValues(alpha: a);
    if (glow) cv.drawCircle(p, r * 2.4, Paint()..color = k.withValues(alpha: a * .45)..maskFilter = MaskFilter.blur(BlurStyle.normal, r * 1.4));
    cv.drawCircle(p, r, fill(k));
  }

  void handle(Canvas cv, Offset p, double hot, Color col) {
    if (hot > 0) cv.drawCircle(p, _hr + 3 + _haloR * hot, stroke(col.withValues(alpha: .45 * hot), 1.5));
    if (glow) cv.drawCircle(p, _hr * 2, Paint()..color = col.withValues(alpha: .5)..maskFilter = const MaskFilter.blur(BlurStyle.normal, 6));
    cv.drawCircle(p, _hr, fill(col));
    cv.drawCircle(p, _hr, stroke(N.g100, 1.5));
  }

  void arrow(Canvas cv, Offset a, Offset b, double alpha) {
    final d = b - a, l = d.distance;
    if (l < 1) return;
    final u = d / l, hd = math.min(4.5, l * .5);
    final p = Path()
      ..moveTo(a.dx, a.dy)
      ..lineTo(b.dx, b.dy)
      ..moveTo(b.dx, b.dy)
      ..relativeLineTo(_rot(-u, .5).dx * hd, _rot(-u, .5).dy * hd)
      ..moveTo(b.dx, b.dy)
      ..relativeLineTo(_rot(-u, -.5).dx * hd, _rot(-u, -.5).dy * hd);
    line(cv, p, a: alpha);
  }
}

typedef FieldPaint = void Function(Canvas cv, Size s, List<Offset> h, Skin k, int? hot, double hotT, double t);

/// A square well with draggable handles. Handles live in unit space; the owner interprets drags, so a gadget may project a handle onto any constraint.
class HandleField extends StatefulWidget {
  const HandleField({super.key, required this.size, required this.skin, required this.handles, required this.onChanged, required this.paint, this.onTap, this.ticking = false, this.grabAnywhere = false, this.colors});
  final double size;
  final Skin skin;
  final List<Offset> handles;
  final void Function(int index, Offset unit) onChanged;
  final void Function(int index)? onTap;
  final FieldPaint paint;
  final bool ticking, grabAnywhere;
  final List<Color>? colors;
  @override
  State<HandleField> createState() => _HandleFieldState();
}

class _HandleFieldState extends State<HandleField> with TickerProviderStateMixin {
  late final AnimationController _hov = AnimationController(vsync: this, duration: const Duration(milliseconds: 120));
  AnimationController? _clock;
  int? _hot;
  int? _ghost;
  bool _drag = false;

  @override
  void initState() {
    super.initState();
    _syncClock();
  }

  void _syncClock() {
    if (widget.ticking && _clock == null) {
      _clock = AnimationController(vsync: this, duration: const Duration(hours: 1))..repeat();
    } else if (!widget.ticking && _clock != null) {
      _clock!.dispose();
      _clock = null;
    }
  }

  @override
  void didUpdateWidget(HandleField old) {
    super.didUpdateWidget(old);
    _syncClock();
  }

  @override
  void dispose() {
    _hov.dispose();
    _clock?.dispose();
    super.dispose();
  }

  int? _pick(Offset lp) {
    int? best;
    var bd = _hitR;
    for (var i = 0; i < widget.handles.length; i++) {
      final d = (widget.handles[i] * widget.size - lp).distance;
      if (d <= bd) {
        bd = d;
        best = i;
      }
    }
    return best;
  }

  void _focus(int? i) {
    if (i == _hot) return;
    setState(() {
      _hot = i;
      if (i != null) _ghost = i;
    });
    if (i == null) {
      _hov.reverse();
    } else {
      _hov.forward(from: 0);
    }
  }

  @override
  Widget build(BuildContext context) {
    final sk = widget.skin, size = widget.size;
    return MouseRegion(
      cursor: _hot == null ? SystemMouseCursors.basic : (_drag ? SystemMouseCursors.grabbing : SystemMouseCursors.grab),
      onHover: (e) {
        if (!_drag) _focus(_pick(e.localPosition));
      },
      onExit: (_) {
        if (!_drag) _focus(null);
      },
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onPanDown: (d) {
          var i = _pick(d.localPosition);
          if (i == null && widget.grabAnywhere) {
            i = 0;
            widget.onChanged(0, _cl(d.localPosition / size));
          }
          _drag = i != null;
          _focus(i);
        },
        onPanUpdate: (d) {
          if (_drag && _hot != null) widget.onChanged(_hot!, _cl(d.localPosition / size));
        },
        onPanEnd: (_) {
          _drag = false;
          _focus(null);
        },
        onPanCancel: () {
          _drag = false;
          _focus(null);
        },
        onTapUp: (d) {
          final i = _pick(d.localPosition);
          if (i != null) widget.onTap?.call(i);
        },
        child: Container(
          width: size,
          height: size,
          decoration: BoxDecoration(color: sk.quiet ? N.g10 : N.g07, borderRadius: BorderRadius.circular(_wellR)),
          child: ClipRRect(
            borderRadius: BorderRadius.circular(_wellR),
            child: CustomPaint(painter: _FieldPainter(widget, _hov, _clock, () => _drag ? _hot : _ghost)),
          ),
        ),
      ),
    );
  }
}

class _FieldPainter extends CustomPainter {
  _FieldPainter(this.w, this.hov, this.clock, this.ghost) : super(repaint: Listenable.merge([hov, clock]));
  final HandleField w;
  final AnimationController hov;
  final AnimationController? clock;
  final int? Function() ghost;
  @override
  void paint(Canvas cv, Size s) {
    final hs = [for (final h in w.handles) h * s.width];
    final g = ghost(), ht = Curves.easeOut.transform(hov.value);
    w.paint(cv, s, hs, w.skin, g, ht, (clock?.value ?? 0) * 3600);
    for (var i = 0; i < hs.length; i++) {
      w.skin.handle(cv, hs[i], i == g ? ht : 0, w.colors?[i] ?? w.skin.c);
    }
  }

  @override
  bool shouldRepaint(_FieldPainter o) => true;
}

/// The card around a gadget: number, name, Japanese gloss, the picture, three named readouts.
class GadgetShell extends StatelessWidget {
  const GadgetShell({super.key, required this.no, required this.title, required this.jp, required this.tone, required this.look, required this.reads, required this.field, required this.size});
  final int no;
  final String title, jp;
  final Color tone;
  final Look look;
  final List<(String, String)> reads;
  final Widget field;
  final double size;
  @override
  Widget build(BuildContext context) => Container(
        width: size + 28,
        padding: const EdgeInsets.all(14),
        decoration: BoxDecoration(
          color: look == Look.quiet ? N.g10 : N.g13,
          borderRadius: BorderRadius.circular(_cardR),
          border: Border.all(color: look == Look.glow ? tone.withValues(alpha: .4) : N.g20),
          boxShadow: look == Look.glow ? [BoxShadow(color: tone.withValues(alpha: .14), blurRadius: 24)] : null,
        ),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
          Row(children: [
            Text('$no', style: T.micro(N.g44)),
            const SizedBox(width: 6),
            Expanded(child: Text(title, style: T.name(), maxLines: 1, overflow: TextOverflow.ellipsis)),
            Container(width: 7, height: 7, decoration: BoxDecoration(color: tone, shape: BoxShape.circle)),
          ]),
          const SizedBox(height: 5),
          Text(jp, style: T.label(N.g56)),
          const SizedBox(height: 10),
          field,
          const SizedBox(height: 10),
          for (final r in reads)
            Padding(
              padding: const EdgeInsets.only(top: 5),
              child: Row(children: [
                Text(r.$1, style: T.label(N.g56), maxLines: 1),
                const SizedBox(width: 10),
                Expanded(child: Text(r.$2, style: T.value(N.g76), textAlign: TextAlign.right, maxLines: 1, overflow: TextOverflow.ellipsis)),
              ]),
            ),
        ]),
      );
}

abstract class GadgetWidget extends StatefulWidget {
  const GadgetWidget({super.key, this.look = Look.concept, this.size = 176, this.framed = true});
  final Look look;
  final double size;
  final bool framed;
}

Widget _gadget(GadgetWidget w, {
  required int no,
  required String title,
  required String jp,
  required Color tone,
  required List<(String, String)> reads,
  required List<Offset> handles,
  required void Function(int, Offset) onChanged,
  required FieldPaint paint,
  void Function(int)? onTap,
  bool ticking = false,
  bool grabAnywhere = false,
  List<Color>? colors,
}) {
  final field = HandleField(size: w.size, skin: Skin(w.look, tone), handles: handles, onChanged: onChanged, paint: paint, onTap: onTap, ticking: ticking, grabAnywhere: grabAnywhere, colors: colors);
  if (!w.framed) return field;
  return GadgetShell(no: no, title: title, jp: jp, tone: tone, look: w.look, reads: reads, field: field, size: w.size);
}

// ---------------------------------------------------------------- 1 Falloff
class FalloffGadget extends GadgetWidget {
  const FalloffGadget({super.key, super.look, super.size, super.framed, this.curve = 1.6, this.rings = 3, this.onChanged});
  final double curve;
  final int rings;
  final void Function(Offset c, double r)? onChanged;
  @override
  State<FalloffGadget> createState() => _FalloffState();
}

class _FalloffState extends State<FalloffGadget> {
  Offset c = const Offset(.5, .5);
  double r = .3, a = -.2;
  @override
  Widget build(BuildContext context) {
    final w = widget;
    return _gadget(w, no: 1, title: 'Falloff', jp: '影響の広がり', tone: C.mode,
        reads: [('Center', '${_f(c.dx)}, ${_f(c.dy)}'), ('Radius', _f(r)), ('Falloff', _f(w.curve, 1))],
        handles: [c, c + _dir(a) * r],
        onChanged: (i, p) {
          setState(() {
            if (i == 0) {
              c = _cl(p, .1, .9);
            } else {
              final v = p - c;
              r = _clamp(v.distance, .08, .46);
              a = v.direction;
            }
          });
          w.onChanged?.call(c, r);
        },
        paint: (cv, s, h, k, hot, ht, t) {
          final R = r * s.width;
          if (k.fillA > 0) {
            final cols = [for (var i = 0; i <= 8; i++) k.c.withValues(alpha: math.pow(1 - i / 8, w.curve).toDouble() * .5)];
            cv.drawCircle(h[0], R, Paint()..shader = RadialGradient(colors: cols, stops: [for (var i = 0; i <= 8; i++) i / 8]).createShader(Rect.fromCircle(center: h[0], radius: R)));
          }
          for (var i = 1; i <= w.rings; i++) {
            final last = i == w.rings;
            k.ring(cv, h[0], R * i / w.rings, col: last ? k.c : null, a: last ? .8 : (k.quiet ? 1 : .38));
          }
          cv.drawLine(h[0], h[1], k.stroke(k.guide));
        });
  }
}

// ---------------------------------------------------------------- 2 Direction / Attract
class DirectionGadget extends GadgetWidget {
  const DirectionGadget({super.key, super.look, super.size, super.framed, this.pull = .55, this.cells = 7});
  final double pull;
  final int cells;
  @override
  State<DirectionGadget> createState() => _DirectionState();
}

class _DirectionState extends State<DirectionGadget> {
  Offset c = const Offset(.5, .5);
  double a = -.9, len = .22;
  @override
  Widget build(BuildContext context) {
    final w = widget;
    return _gadget(w, no: 2, title: 'Direction / Attract', jp: '向き・引き寄せ', tone: Fam.face.c,
        reads: [('Direction', '${(a * 180 / math.pi).round()}°'), ('Strength', _f(len)), ('Range', _f(w.pull))],
        handles: [c, c + _dir(a) * len],
        onChanged: (i, p) => setState(() {
              if (i == 0) {
                c = _cl(p, .15, .85);
              } else {
                final v = p - c;
                len = _clamp(v.distance, .1, .4);
                a = v.direction;
              }
            }),
        paint: (cv, s, h, k, hot, ht, t) {
          final n = w.cells, uni = _dir(a);
          for (var i = 0; i < n; i++) {
            for (var j = 0; j < n; j++) {
              final g = Offset((i + .5) / n, (j + .5) / n), to = c - g, dist = to.distance;
              if (dist < .6 / n) continue;
              final v = Offset.lerp(uni, to / dist, w.pull)!, vl = v.distance, d = vl > 1e-4 ? v / vl : uni;
              final fall = 1 - math.min(dist, .8) / 1.6, L = (1 / n) * .78 * (len / .22) * fall;
              k.arrow(cv, (g - d * (L / 2)) * s.width, (g + d * (L / 2)) * s.width, .45 + .45 * fall);
            }
          }
          cv.drawLine(h[0], h[1], k.stroke(k.guide));
        });
  }
}

// ---------------------------------------------------------------- 3 Scatter
class ScatterGadget extends GadgetWidget {
  const ScatterGadget({super.key, super.look, super.size, super.framed, this.density = 60, this.seed = 3});
  final int density, seed;
  @override
  State<ScatterGadget> createState() => _ScatterState();
}

class _ScatterState extends State<ScatterGadget> {
  Offset c = const Offset(.5, .5);
  double r = .36, a = -.8;
  @override
  Widget build(BuildContext context) {
    final w = widget;
    return _gadget(w, no: 3, title: 'Scatter', jp: '散らす', tone: Fam.scatter.c,
        reads: [('Density', '${w.density}'), ('Spread', _f(r)), ('Boundary', 'seed ${w.seed}')],
        handles: [c, c + _dir(a) * r],
        onChanged: (i, p) => setState(() {
              if (i == 0) {
                c = _cl(p, .2, .8);
              } else {
                final v = p - c;
                r = _clamp(v.distance, .1, .46);
                a = v.direction;
              }
            }),
        paint: (cv, s, h, k, hot, ht, t) {
          final R = r * s.width, rnd = math.Random(w.seed);
          if (k.fillA > 0) cv.drawCircle(h[0], R, k.fill(k.c.withValues(alpha: .06)));
          _dash(cv, Path()..addOval(Rect.fromCircle(center: h[0], radius: R)), k.stroke(k.c.withValues(alpha: .55), 1));
          for (var i = 0; i < w.density; i++) {
            final d = R * math.sqrt(rnd.nextDouble()), an = rnd.nextDouble() * math.pi * 2, sz = 1.1 + rnd.nextDouble() * 1.7;
            k.dot(cv, h[0] + _dir(an) * d, sz);
          }
        });
  }
}

// ---------------------------------------------------------------- 4 Along Path
class AlongPathGadget extends GadgetWidget {
  const AlongPathGadget({super.key, super.look, super.size, super.framed, this.count = 7, this.bias = 1.0});
  final int count;
  final double bias;
  @override
  State<AlongPathGadget> createState() => _AlongState();
}
