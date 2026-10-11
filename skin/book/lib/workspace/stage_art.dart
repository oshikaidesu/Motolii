// The stage's artwork: a pop poster frame drawn from Ws.layers at one frame, plus the geometry the stage needs to select it.
// contract: everything here is a pure function of an [ArtScene]; no clock, no randomness at paint time, so a frame always looks the same.
import 'dart:math' as math;

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'fx_catalog.dart';
import 'stage_fx.dart';
import 'ws.dart';

/// The poster's own inks: flat, saturated, opaque. They belong to the picture, never to the chrome.
abstract final class ArtInk {
  static const blue = Color(0xFF2A52E4), deep = Color(0xFF1D3BBE), sky = Color(0xFF4C79FF), navy = Color(0xFF0E1A4A);
  static const cream = Color(0xFFF6EFDD), orange = Color(0xFFFF7A3C), pink = Color(0xFFF06BAA), lime = Color(0xFFCBE057), ink = Color(0xFF15170D);
}

/// One layer as the picture needs it: a frozen copy, so a scene can be compared with the last one.
@immutable
class ArtLayer {
  ArtLayer(WsLayer l) : id = l.id, name = l.name, kind = l.kind, inF = l.inF, outF = l.outF, keys = List.unmodifiable([...l.keys]..sort());
  final String id, name;
  final WsKind kind;
  final int inF, outF;
  final List<int> keys;
  bool live(int f) => f >= inF && f <= outF;
  @override
  bool operator ==(Object other) =>
      other is ArtLayer && other.id == id && other.name == name && other.kind == kind && other.inF == inF && other.outF == outF && listEquals(other.keys, keys);
  @override
  int get hashCode => Object.hash(id, inF, outF, Object.hashAll(keys));
}

/// What the stage draws: the frame, the layers, and whether the camera layer frames the view.
@immutable
class ArtScene {
  ArtScene.of(Ws ws, {required this.camera})
    : frame = ws.frame,
      layers = List.unmodifiable([for (final l in ws.layers) ArtLayer(l)]),
      fx = Map.unmodifiable({
        for (final l in ws.layers)
          if ([...ws.fxOf(l.id), if (l.id == ws.selected) ?ws.trial] case final names when names.isNotEmpty)
            l.id: List<FxFamily>.unmodifiable([for (final n in names) ?fxNamed(n)?.family]),
      });
  final int frame;
  final bool camera;
  final List<ArtLayer> layers;

  /// The effect families on each layer, kept ones first and the one being tried last.
  final Map<String, List<FxFamily>> fx;
  ArtLayer? layer(String? id) => layers.where((l) => l.id == id).firstOrNull;
  String get _fxSig => fx.entries.map((e) => '${e.key}:${e.value.map((f) => f.index).join(',')}').join(';');
  @override
  bool operator ==(Object other) =>
      other is ArtScene && other.frame == frame && other.camera == camera && listEquals(other.layers, layers) && other._fxSig == _fxSig;
  @override
  int get hashCode => Object.hash(frame, camera, Object.hashAll(layers), _fxSig);
}

/// A layer's box in document px: centre, unscaled size, scale, rotation and its anchor.
class ArtBox {
  const ArtBox(this.c, this.size, {this.scale = 1, this.rot = 0, Offset? anchor}) : anchor = anchor ?? c;
  final Offset c, anchor;
  final Size size;
  final double scale, rot;
  Offset _local(double x, double y) => c + _rot(Offset(x * size.width / 2 * scale, y * size.height / 2 * scale), rot);
  List<Offset> get corners => [_local(-1, -1), _local(1, -1), _local(1, 1), _local(-1, 1)];
  Offset get top => _local(0, -1);
  bool contains(Offset p, [double pad = 0]) {
    final q = _rot(p - c, -rot);
    return q.dx.abs() <= size.width / 2 * scale + pad && q.dy.abs() <= size.height / 2 * scale + pad;
  }
}

Offset _rot(Offset p, double a) => Offset(p.dx * math.cos(a) - p.dy * math.sin(a), p.dx * math.sin(a) + p.dy * math.cos(a));

double _io(double t) => t < .5 ? 4 * t * t * t : 1 - math.pow(-2 * t + 2, 3) / 2;
double _back(double t) {
  const c1 = 1.70158, c3 = c1 + 1;
  return 1 + c3 * math.pow(t - 1, 3) + c1 * math.pow(t - 1, 2);
}

/// The value of a channel at frame [f]: [v] are the values at the layer's keys (cycled if shorter), eased key to key.
double _track(ArtLayer l, num f, List<double> v, [double Function(double) e = _io]) {
  final k = l.keys;
  if (k.isEmpty || f <= k.first) return v.first;
  for (var i = 0; i < k.length - 1; i++) {
    if (f < k[i + 1]) {
      final span = k[i + 1] - k[i], t = span <= 0 ? 1.0 : ((f - k[i]) / span).clamp(0.0, 1.0);
      final a = v[i % v.length], b = v[(i + 1) % v.length];
      return a + (b - a) * e(t);
    }
  }
  return v[(k.length - 1) % v.length];
}

class _Bit {
  const _Bit(this.x, this.y, this.kind, this.color, this.size, this.spin, this.fall, this.phase);
  final double x, y, size, spin, fall, phase;
  final int kind;
  final Color color;
}

/// The poster: document size, placement constants, painting and hit testing.
abstract final class WsArt {
  static const comp = Size(1920, 1080);
  static const centre = Offset(960, 540);
  static const _frame = 36.0, _baseline = 812.0, _left = 150.0, _titlePx = 410.0;
  static const _chips = [('MOTION', ArtInk.lime), ('TYPE', ArtInk.pink), ('LOOP', ArtInk.orange), ('30 FPS', ArtInk.cream)];
  static const _chipH = 64.0, _chipGap = 14.0, _chipTop = 872.0;
  static const _confettiArea = Rect.fromLTRB(90, 70, 1830, 1010);

  static final List<_Bit> _bits = () {
    final r = math.Random(7);
    const inks = [ArtInk.pink, ArtInk.lime, ArtInk.orange, ArtInk.cream, ArtInk.navy, ArtInk.sky];
    return [
      for (var i = 0; i < 64; i++)
        _Bit(
          _confettiArea.left + r.nextDouble() * _confettiArea.width,
          _confettiArea.top + r.nextDouble() * _confettiArea.height,
          r.nextInt(4),
          inks[i % inks.length],
          12 + r.nextDouble() * 16,
          (r.nextDouble() - .5) * .12,
          .6 + r.nextDouble() * 1.4,
          r.nextDouble() * math.pi * 2,
        ),
    ];
  }();

  // ---- the camera ----------------------------------------------------------------------------------------------------

  static (double, Offset) _cam(ArtScene s) {
    final l = s.layer('cam');
    if (l == null || !l.live(s.frame)) return (1, Offset.zero);
    return (_track(l, s.frame, const [1.0, 1.07, 1.0]), Offset(_track(l, s.frame, const [0.0, -36.0, 0.0]), _track(l, s.frame, const [0.0, 14.0, 0.0])));
  }

  /// Document px to the px the viewer sees (the camera applied when the scene looks through it).
  static Offset toView(ArtScene s, Offset p) {
    if (!s.camera) return p;
    final (z, pan) = _cam(s);
    return centre + pan + (p - centre) * z;
  }

  static Offset toDoc(ArtScene s, Offset p) {
    if (!s.camera) return p;
    final (z, pan) = _cam(s);
    return centre + (p - centre - pan) / z;
  }

  /// Where the camera frames, in document px.
  static ArtBox cameraBox(ArtScene s) {
    final (z, pan) = _cam(s);
    return ArtBox(centre - pan / z, comp, scale: 1 / z);
  }

  // ---- geometry per layer --------------------------------------------------------------------------------------------

  static (Offset, double, double) _blob(ArtLayer l, int f) => (
    Offset(_track(l, f, const [1350, 1310, 1380, 1330]), _track(l, f, const [500, 540, 490, 520])),
    _track(l, f, const [.86, 1.0, .94, 1.04]),
    _track(l, f, const [0, .3, .55, .9]),
  );

  static (Offset, double, double) _ring(ArtLayer l, int f) {
    final first = l.keys.length > 1 && f < l.keys[1];
    final s = first ? _track(l, f, const [0.0, 1.0, 1.0], _back) : _track(l, f, const [0.0, 1.0, 1.08]);
    return (Offset(1650, _track(l, f, const [300, 250, 230])), s.clamp(0.0, 2.0), _track(l, f, const [-1.2, 0, 1.1]));
  }

  static TextStyle _type(double px, Color c, {FontWeight w = FontWeight.w900, double track = 0}) => TextStyle(
    fontFamily: T.sans,
    fontFamilyFallback: const ['.AppleSystemUIFont'],
    fontSize: px,
    fontWeight: w,
    fontVariations: [FontVariation('wght', w.value.toDouble())],
    letterSpacing: track,
    height: 1,
    color: c,
  );

  static List<(TextPainter, double)> _letters(Color c) {
    var x = _left;
    return [
      for (final ch in const ['C', 'O', 'D', 'A'])
        () {
          final tp = TextPainter(
            text: TextSpan(text: ch, style: _type(_titlePx, c)),
            textDirection: TextDirection.ltr,
          )..layout();
          final at = x;
          x += tp.width - _titlePx * .035;
          return (tp, at);
        }(),
    ];
  }

  static Rect _titleRect() {
    final ls = _letters(ArtInk.cream), last = ls.last;
    return Rect.fromLTRB(_left, _baseline - _titlePx * .74, last.$2 + last.$1.width, _baseline + 6);
  }

  static List<Rect> _chipRects() {
    var x = _left + 6;
    return [
      for (final (label, _) in _chips)
        () {
          final tp = TextPainter(
            text: TextSpan(
              text: label,
              style: _type(30, ArtInk.ink, w: FontWeight.w800, track: 2),
            ),
            textDirection: TextDirection.ltr,
          )..layout();
          final r = Rect.fromLTWH(x, _chipTop, tp.width + 56, _chipH);
          x = r.right + _chipGap;
          return r;
        }(),
    ];
  }

  static Offset _bitAt(_Bit b, int age) {
    final h = _confettiArea.height;
    final y = _confettiArea.top + ((b.y - _confettiArea.top + age * b.fall * 2.2) % h);
    return Offset(b.x + math.sin(age * .05 + b.phase) * 22, y);
  }

  /// The box the stage frames when [id] is selected, or null when the layer has no picture at this frame.
  static ArtBox? boxOf(ArtScene s, String? id) {
    final l = s.layer(id), f = s.frame;
    if (l == null || l.kind == WsKind.audio) return null;
    if (l.kind == WsKind.camera) return s.camera ? const ArtBox(centre, comp) : cameraBox(s);
    if (!l.live(f)) return null;
    switch (l.id) {
      case 'blob':
        final (c, sc, r) = _blob(l, f);
        return ArtBox(c, const Size(600, 600), scale: sc, rot: r);
      case 'ring':
        final (c, sc, r) = _ring(l, f);
        return ArtBox(c, const Size(330, 330), scale: math.max(sc, .05), rot: r);
      case 'title':
        final t = _titleRect();
        return ArtBox(t.center, t.size, anchor: Offset(_left, _baseline));
      case 'chips':
        final rs = _chipRects(), u = rs.reduce((a, b) => a.expandToInclude(b));
        return ArtBox(u.center, u.size, anchor: u.centerLeft);
      case 'confetti':
        return ArtBox(_confettiArea.center, _confettiArea.size);
      default:
        return const ArtBox(centre, comp);
    }
  }

  /// The topmost layer under [p] (document px), or null. The background is locked: a click on it means "nothing".
  static String? hit(ArtScene s, Offset p) {
    final f = s.frame;
    bool on(String id) => s.layer(id)?.live(f) ?? false;
    if (on('chips') && _chipRects().any((r) => r.inflate(4).contains(p))) return 'chips';
    if (on('title') && _titleRect().contains(p)) return 'title';
    if (on('confetti')) {
      final age = f - s.layer('confetti')!.inF;
      for (final b in _bits) {
        if ((_bitAt(b, age) - p).distance <= b.size + 10) return 'confetti';
      }
    }
    if (on('ring')) {
      final (c, sc, _) = _ring(s.layer('ring')!, f);
      final d = (p - c).distance / math.max(sc, .01);
      if (d >= 100 && d <= 172) return 'ring';
    }
    if (on('blob')) {
      final (c, sc, _) = _blob(s.layer('blob')!, f);
      if ((p - c).distance <= 300 * sc) return 'blob';
    }
    return null;
  }

  // ---- painting ------------------------------------------------------------------------------------------------------

  /// Paints the frame in document px (the caller maps document px to the screen). Applies the camera when the scene looks through it.
  static void paint(Canvas c, ArtScene s) {
    final f = s.frame;
    c.save();
    if (s.camera) {
      final (z, pan) = _cam(s);
      c.translate(centre.dx + pan.dx, centre.dy + pan.dy);
      c.scale(z);
      c.translate(-centre.dx, -centre.dy);
    }
    for (final id in const ['bg', 'blob', 'ring', 'confetti', 'title', 'chips']) {
      final l = s.layer(id);
      if (l == null || !l.live(f)) continue;
      void at(Canvas c, int dt) {
        final g = math.max(l.inF, f + dt);
        switch (id) {
          case 'bg':
            _paintBg(c, g);
          case 'blob':
            _paintBlob(c, l, g);
          case 'ring':
            _paintRing(c, l, g);
          case 'confetti':
            _paintConfetti(c, l, g);
          case 'title':
            _paintTitle(c, l, g);
          case 'chips':
            _paintChips(c, l, g);
        }
      }

      final fams = s.fx[id] ?? const [], box = fams.isEmpty ? null : boxOf(s, id);
      box == null ? at(c, 0) : paintFx(c, fams, _bounds(box), at);
    }
    c.restore();
  }

  static Rect _bounds(ArtBox b) {
    final cs = b.corners;
    return Rect.fromLTRB(
      cs.map((p) => p.dx).reduce(math.min),
      cs.map((p) => p.dy).reduce(math.min),
      cs.map((p) => p.dx).reduce(math.max),
      cs.map((p) => p.dy).reduce(math.max),
    );
  }

  static void _paintBg(Canvas c, int f) {
    final p = Paint();
    c.drawRect(Offset.zero & comp, p..color = ArtInk.blue);
    c.drawCircle(const Offset(1430, 470), 450, p..color = ArtInk.sky);
    // halftone in the top-left corner: dots shrink away from the corner
    p.color = ArtInk.deep;
    for (var y = 0; y < 20; y++) {
      for (var x = 0; x < 30; x++) {
        final o = Offset(60.0 + x * 26 + (y.isOdd ? 13 : 0), 60.0 + y * 26), d = o.distance / 900;
        final r = 9 * (1 - d) - 1;
        if (r > .6) c.drawCircle(o, r, p);
      }
    }
    c.drawRect(const Rect.fromLTWH(0, 1000, 1920, 80), p..color = ArtInk.deep);
    c.drawRect(
      Rect.fromLTRB(_frame, _frame, comp.width - _frame, comp.height - _frame),
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 4
        ..color = ArtInk.cream,
    );
    void words(String t, Offset at, {bool right = false, double px = 24}) {
      final tp = TextPainter(
        text: TextSpan(
          text: t,
          style: _type(px, ArtInk.cream, w: FontWeight.w700, track: 4),
        ),
        textDirection: TextDirection.ltr,
      )..layout();
      tp.paint(c, right ? at - Offset(tp.width, 0) : at);
    }

    words('NO. 03', const Offset(_left, 74));
    words('A MOTION STUDY  ·  SPRING 2026', const Offset(1770, 74), right: true);
    words('MOTOLII  /  POP SERIES', const Offset(1770, 1012), right: true, px: 22);
    words('${(f ~/ Ws.fps).toString().padLeft(2, '0')}:${(f % Ws.fps).toString().padLeft(2, '0')}', const Offset(_left, 1012), px: 22);
  }

  static Path _blobPath(double ph, double r) {
    const n = 14;
    final pts = [
      for (var i = 0; i < n; i++)
        () {
          final a = i / n * math.pi * 2, k = r * (1 + .11 * math.sin(3 * a + ph) + .06 * math.sin(5 * a - ph * 1.4));
          return Offset(math.cos(a) * k, math.sin(a) * k);
        }(),
    ];
    final path = Path()..moveTo(pts[0].dx, pts[0].dy);
    for (var i = 0; i < n; i++) {
      final p0 = pts[(i - 1 + n) % n], p1 = pts[i], p2 = pts[(i + 1) % n], p3 = pts[(i + 2) % n];
      final c1 = p1 + (p2 - p0) / 6, c2 = p2 - (p3 - p1) / 6;
      path.cubicTo(c1.dx, c1.dy, c2.dx, c2.dy, p2.dx, p2.dy);
    }
    return path..close();
  }

  static void _paintBlob(Canvas c, ArtLayer l, int f) {
    final (at, sc, r) = _blob(l, f);
    final ph = f * .035;
    c.save();
    c.translate(at.dx, at.dy);
    c.rotate(r);
    c.scale(sc);
    final path = _blobPath(ph, 262);
    c.drawPath(path.shift(const Offset(30, 30)), Paint()..color = ArtInk.pink);
    c.drawPath(path, Paint()..color = ArtInk.orange);
    final wave = Path()..moveTo(-170, 30);
    for (var i = 0; i < 6; i++) {
      final x = -170 + i * 60.0;
      wave.quadraticBezierTo(x + 15, i.isEven ? -10 : 70, x + 30, 30);
      wave.quadraticBezierTo(x + 45, i.isEven ? 70 : -10, x + 60, 30);
    }
    c.drawPath(
      wave,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 16
        ..strokeCap = StrokeCap.round
        ..strokeJoin = StrokeJoin.round
        ..color = ArtInk.navy,
    );
    c.drawCircle(const Offset(96, -118), 34, Paint()..color = ArtInk.cream);
    c.restore();
  }

  static void _paintRing(Canvas c, ArtLayer l, int f) {
    final (at, sc, r) = _ring(l, f);
    if (sc <= .01) return;
    c.save();
    c.translate(at.dx, at.dy);
    c.rotate(r);
    c.scale(sc);
    final band = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 58
      ..strokeCap = StrokeCap.butt
      ..color = ArtInk.lime;
    c.drawArc(Rect.fromCircle(center: Offset.zero, radius: 136), -math.pi / 2 + .32, math.pi * 2 - .64, false, band);
    c.drawCircle(Offset.zero, 70, Paint()..color = ArtInk.navy);
    c.drawCircle(const Offset(0, -136), 18, Paint()..color = ArtInk.cream);
    c.restore();
  }

  static void _paintConfetti(Canvas c, ArtLayer l, int f) {
    final age = f - l.inF;
    for (final (i, b) in _bits.indexed) {
      final burst = _track(l, f - (i % 9) * 2, const [0.0, 1.0], _back);
      if (burst <= .01) continue;
      final at = _bitAt(b, age), p = Paint()..color = b.color, s = b.size * burst;
      c.save();
      c.translate(at.dx, at.dy);
      c.rotate(b.phase + b.spin * age);
      switch (b.kind) {
        case 0:
          c.drawCircle(Offset.zero, s * .55, p);
        case 1:
          c.drawRect(Rect.fromCenter(center: Offset.zero, width: s, height: s), p);
        case 2:
          c.drawPath(Path()..addPolygon([Offset(0, -s * .7), Offset(s * .65, s * .5), Offset(-s * .65, s * .5)], true), p);
        default:
          c.drawPath(
            Path()
              ..moveTo(-s, 0)
              ..quadraticBezierTo(-s / 2, -s * .7, 0, 0)
              ..quadraticBezierTo(s / 2, s * .7, s, 0),
            Paint()
              ..style = PaintingStyle.stroke
              ..strokeWidth = s * .32
              ..strokeCap = StrokeCap.round
              ..color = b.color,
          );
      }
      c.restore();
    }
  }

  static void _paintTitle(Canvas c, ArtLayer l, int f) {
    final mask = _titleRect();
    c.save();
    c.clipRect(Rect.fromLTRB(mask.left - 4, mask.top - 40, mask.right + 30, _baseline + 26));
    final shadow = _letters(ArtInk.navy), face = _letters(ArtInk.cream);
    for (var i = 0; i < face.length; i++) {
      final rise = _track(l, f - i * 3, const [1.0, 0.0, 0.0, -1.2]) * _titlePx * .9;
      final (tp, x) = face[i];
      final top = _baseline - tp.computeDistanceToActualBaseline(TextBaseline.alphabetic) + rise;
      shadow[i].$1.paint(c, Offset(x + 18, top + 18));
      tp.paint(c, Offset(x, top));
    }
    c.restore();
  }

  static void _paintChips(Canvas c, ArtLayer l, int f) {
    final rs = _chipRects();
    for (final (i, (label, ink)) in _chips.indexed) {
      final p = _track(l, f - i * 5, const [0.0, 1.0, 1.0], _back), bob = _track(l, f - i * 4, const [0.0, 0.0, -12.0]);
      if (p <= .01) continue;
      final r = rs[i].shift(Offset(0, bob));
      c.save();
      c.translate(r.center.dx, r.center.dy);
      c.scale(p);
      final local = Rect.fromCenter(center: Offset.zero, width: r.width, height: r.height);
      c.drawRRect(RRect.fromRectAndRadius(local.shift(const Offset(6, 6)), const Radius.circular(_chipH / 2)), Paint()..color = ArtInk.navy);
      c.drawRRect(RRect.fromRectAndRadius(local, const Radius.circular(_chipH / 2)), Paint()..color = ink);
      final tp = TextPainter(
        text: TextSpan(
          text: label,
          style: _type(30, ArtInk.ink, w: FontWeight.w800, track: 2),
        ),
        textDirection: TextDirection.ltr,
      )..layout();
      tp.paint(c, Offset(-tp.width / 2 + 1, -tp.height / 2));
      c.restore();
    }
  }
}
