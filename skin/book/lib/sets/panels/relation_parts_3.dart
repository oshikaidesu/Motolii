// Part of relation_parts.dart (cut there by size, as it was).
part of 'relation_parts.dart';

class _FollowState extends State<FollowGadget> {
  Offset leader = const Offset(.8, .28), me = const Offset(.2, .72), vel = const Offset(1, 0);
  double last = 0;
  final trail = <Offset>[];
  @override
  Widget build(BuildContext context) {
    final w = widget;
    return _gadget(w, no: 14, title: 'Follow', jp: '追従させる', tone: Fam.follow.c, ticking: true, grabAnywhere: true,
        reads: [('Target', 'leader'), ('Lag', '${_f(w.lag)} s'), ('Smooth', '${w.trail}')],
        handles: [leader],
        onChanged: (i, p) => setState(() => leader = p),
        paint: (cv, s, h, k, hot, ht, t) {
          var dt = t - last;
          last = t;
          if (dt < 0 || dt > .2) dt = .016;
          final prev = me;
          final tgt = leader + Offset(math.cos(t * 1.3), math.sin(t * 1.3)) * .07;
          me = Offset.lerp(me, tgt, 1 - math.exp(-dt / math.max(.03, w.lag)))!;
          if ((me - prev).distance > 1e-5) vel = (me - prev) / (me - prev).distance;
          trail.add(me);
          while (trail.length > w.trail) {
            trail.removeAt(0);
          }
          final tp = Path();
          for (var i = 0; i < trail.length; i++) {
            final q = trail[i] * s.width;
            i == 0 ? tp.moveTo(q.dx, q.dy) : tp.lineTo(q.dx, q.dy);
          }
          if (trail.length > 1) _dash(cv, tp, k.stroke(k.c.withValues(alpha: .55), 1.2));
          final m = me * s.width, home = const Offset(.16, .74) * s.width;
          _dash(cv, Path()..moveTo(home.dx, home.dy)..quadraticBezierTo(s.width * .3, s.width * .9, m.dx, m.dy), k.stroke(k.c.withValues(alpha: .5), 1.2), on: 5, off: 5);
          _dash(cv, Path()..moveTo(m.dx, m.dy)..lineTo(h[0].dx, h[0].dy), k.stroke(k.guide, 1), on: 2, off: 4);
          final tri = Path()
            ..moveTo(m.dx + vel.dx * 9, m.dy + vel.dy * 9)
            ..lineTo(m.dx + _rot(vel, 2.5).dx * 8, m.dy + _rot(vel, 2.5).dy * 8)
            ..lineTo(m.dx + _rot(vel, -2.5).dx * 8, m.dy + _rot(vel, -2.5).dy * 8)
            ..close();
          if (k.glow) cv.drawPath(tri, Paint()..color = k.c.withValues(alpha: .5)..maskFilter = const MaskFilter.blur(BlurStyle.normal, 6));
          cv.drawPath(tri, k.fill(N.g95));
        });
  }
}

// ---------------------------------------------------------------- 15 Audio Reaction
class AudioGadget extends GadgetWidget {
  const AudioGadget({super.key, super.look, super.size, super.framed, this.smooth = .5});
  final double smooth;
  @override
  State<AudioGadget> createState() => _AudioState();
}

class _AudioState extends State<AudioGadget> {
  static const bars = 18;
  double th = .46, r0 = .3, r1 = .7;
  final sm = List<double>.filled(bars, 0);
  @override
  Widget build(BuildContext context) {
    final w = widget;
    return _gadget(w, no: 15, title: 'Audio Reaction', jp: '音で動かす', tone: Fam.follow.c, ticking: true,
        reads: [('Sensitivity', _f(1 - th)), ('Range', '${(r0 * bars).round()}–${(r1 * bars).round()}'), ('Smooth', _f(w.smooth))],
        handles: [Offset(.93, th), Offset(r0, .93), Offset(r1, .93)],
        onChanged: (i, p) => setState(() {
              if (i == 0) th = _clamp(p.dy, .2, .8);
              if (i == 1) r0 = _clamp(p.dx, .08, r1 - .08);
              if (i == 2) r1 = _clamp(p.dx, r0 + .08, .92);
            }),
        paint: (cv, s, h, k, hot, ht, t) {
          final W = s.width, base = .84 * W, cell = (.84 * W) / bars, top = .62 * W;
          final beat = math.pow(.5 + .5 * math.sin(t * 3.2), 3).toDouble();
          for (var i = 0; i < bars; i++) {
            final f = i / (bars - 1), raw = (.5 + .5 * math.sin(t * (1.7 + i * .31) + i * 1.3)) * (.35 + .65 * math.sin(f * math.pi)) * .7 + beat * (1 - f) * .45;
            sm[i] += (raw.clamp(0.0, 1.0) - sm[i]) * (1 - w.smooth * .92);
            final x = .08 * W + cell * (i + .5), bound = x >= r0 * W && x <= r1 * W, hh = sm[i] * top, hit = bound && base - hh < th * W;
            final col = bound ? k.c : N.g38;
            final rr = RRect.fromLTRBR(x - cell * .31, base - hh, x + cell * .31, base, const Radius.circular(2));
            if (hit && k.glow) cv.drawRRect(rr, Paint()..color = k.c.withValues(alpha: .5)..maskFilter = const MaskFilter.blur(BlurStyle.normal, 6));
            cv.drawRRect(rr, k.fill(col.withValues(alpha: bound ? (hit ? 1 : .55) : .6)));
          }
          _dash(cv, Path()..moveTo(.06 * W, th * W)..lineTo(h[0].dx, th * W), k.stroke(k.guide, 1));
          cv.drawLine(h[1], h[2], k.stroke(k.guide, 1));
        });
  }
}

/// The six families' gadgets for a card's body (no frame).
Widget gadgetFor(Fam f, {Look look = Look.concept, double size = 132}) {
  switch (f.name) {
    case 'Scatter':
      return ScatterGadget(look: look, size: size, framed: false, density: 40);
    case 'Along Path':
      return AlongPathGadget(look: look, size: size, framed: false);
    case 'Stagger':
      return StaggerGadget(look: look, size: size, framed: false);
    case 'Face':
      return DirectionGadget(look: look, size: size, framed: false, cells: 5);
    case 'Follow':
      return FollowGadget(look: look, size: size, framed: false);
    default:
      return GraphGadget(look: look, size: size, framed: false);
  }
}

/// Any of the 15 by number (1..15), with default parameters.
Widget gadgetByNo(int no, {Look look = Look.concept, double size = 176, bool framed = true}) {
  switch (no) {
    case 1:
      return FalloffGadget(look: look, size: size, framed: framed);
    case 2:
      return DirectionGadget(look: look, size: size, framed: framed);
    case 3:
      return ScatterGadget(look: look, size: size, framed: framed);
    case 4:
      return AlongPathGadget(look: look, size: size, framed: framed);
    case 5:
      return StaggerGadget(look: look, size: size, framed: framed);
    case 6:
      return ScaleGadget(look: look, size: size, framed: framed);
    case 7:
      return RotationGadget(look: look, size: size, framed: framed);
    case 8:
      return ColorGadget(look: look, size: size, framed: framed);
    case 9:
      return CurveGadget(look: look, size: size, framed: framed);
    case 10:
      return NoiseGadget(look: look, size: size, framed: framed);
    case 11:
      return GraphGadget(look: look, size: size, framed: framed);
    case 12:
      return RegionGadget(look: look, size: size, framed: framed);
    case 13:
      return RepeatGadget(look: look, size: size, framed: framed);
    case 14:
      return FollowGadget(look: look, size: size, framed: framed);
    default:
      return AudioGadget(look: look, size: size, framed: framed);
  }
}

const gadgetNames = ['Falloff', 'Direction / Attract', 'Scatter', 'Along Path', 'Stagger', 'Scale Distribution', 'Rotation', 'Color / Gradient', 'Curve / Easing', 'Noise / Variation', 'Graph / Link', 'Region / Mask', 'Repeat / Grid', 'Follow', 'Audio Reaction'];
