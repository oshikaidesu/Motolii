part of 'inspector_parts.dart';

const _depthNames = {-50: 'Front', 0: 'Middle', 50: 'Back'};

String anchorName(double? x, double? y, double? z) {
  if (x == null || y == null || z == null) return 'Mixed';
  final grid = x % 50 == 0 && y % 50 == 0 && z % 50 == 0 && z.abs() <= 50;
  if (!grid) return 'Custom';
  return '${_anchorNames[(y ~/ 50) * 3 + x ~/ 50]} · ${_depthNames[z.toInt()]}';
}

/// The anchor in its own card: rarely moved, so it folds away and names itself; open, it is a box to point into.
class AnchorCard extends StatelessWidget {
  const AnchorCard({super.key});
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc;
    final ax = doc.get('anchor.x'), ay = doc.get('anchor.y'), az = doc.get('anchor.z');
    final open = doc.b['anchor.open'] ?? true;
    final centre = ax == 50 && ay == 50 && az == 0;
    return Sect(
      title: 'Anchor',
      tone: TransformZone.pivot.color,
      mark: CardMark.anchor,
      keyIds: const ['anchor'],
      open: open,
      hint: anchorName(ax, ay, az),
      onToggle: () => doc.flag('anchor.open', !open),
      trailing: centre || x.cfg.locked
          ? null
          : Hov(
              onTap: () {
                doc.beginGesture();
                doc.setMany({'anchor.x': 50, 'anchor.y': 50, 'anchor.z': 0});
                doc.endGesture();
              },
              builder: (_, h) => Container(
                height: Pop.titleH,
                padding: const EdgeInsets.symmetric(horizontal: 7),
                alignment: Alignment.center,
                decoration: BoxDecoration(color: h ? Grey.g26 : Pop.well, borderRadius: BorderRadius.circular(8)),
                child: Text('Centre', style: T.label(h ? Grey.g95 : Grey.g76)),
              ),
            ),
      children: const [AnchorCube()],
    );
  }
}

/// The layer as a box seen from the front and a little above: the front face is the layer, the depth runs back.
/// contract: a click snaps the anchor to the nearest of 27 points (3 across × 3 down × front, middle, back); one undo each.
class AnchorCube extends StatefulWidget {
  const AnchorCube({super.key});
  static const height = 116.0;
  @override
  State<AnchorCube> createState() => _AnchorCubeState();
}

/// The box seen orthographically from a turnable viewpoint; the default angle keeps all 27 points at least ~20 px apart.
class AnchorGeometry {
  AnchorGeometry(Size s, {this.yaw = defaultYaw, this.pitch = defaultPitch}) : centre = s.center(Offset.zero);
  final Offset centre;
  final double yaw, pitch;
  static const unit = 29.0, defaultYaw = 21 * math.pi / 180, defaultPitch = 34 * math.pi / 180;
  static const steps = [0.0, 50.0, 100.0], depths = [50.0, 0.0, -50.0];

  (double, double, double) _view(double x, double y, double z) {
    final u = (x - 50) / 50, v = (y - 50) / 50, w = z / 50;
    final rx = u * math.cos(yaw) + w * math.sin(yaw), rz = -u * math.sin(yaw) + w * math.cos(yaw);
    return (rx, v * math.cos(pitch) - rz * math.sin(pitch), v * math.sin(pitch) + rz * math.cos(pitch));
  }

  Offset at(double x, double y, double z) {
    final (sx, sy, _) = _view(x, y, z);
    return centre + Offset(sx, sy) * unit;
  }

  /// Larger is farther from the eye.
  double depth(double x, double y, double z) => _view(x, y, z).$3;

  /// Far to near, so nearer points paint over farther ones.
  List<(double, double, double)> get points => [
    for (final z in depths)
      for (final y in steps)
        for (final x in steps) (x, y, z),
  ]..sort((a, b) => depth(b.$1, b.$2, b.$3).compareTo(depth(a.$1, a.$2, a.$3)));

  (double, double, double)? pick(Offset p) {
    (double, double, double)? best;
    var bestD = 8.0;
    for (final q in points) {
      final d = (at(q.$1, q.$2, q.$3) - p).distance;
      if (d <= bestD) {
        best = q;
        bestD = d;
      }
    }
    return best;
  }
}

class _AnchorCubeState extends State<AnchorCube> {
  (double, double, double)? _near;
  double _yaw = AnchorGeometry.defaultYaw, _pitch = AnchorGeometry.defaultPitch;
  Offset? _down;
  bool _turning = false;

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc;
    final ax = doc.get('anchor.x'), ay = doc.get('anchor.y'), az = doc.get('anchor.z');
    final at = ax == null || ay == null || az == null ? null : (ax, ay, az);
    final on = !x.cfg.locked && at != null;
    final hot = doc.directHover == 'anchor';
    return Semantics(
      label: 'Anchor: ${anchorName(ax, ay, az)}',
      child: LayoutBuilder(
        builder: (context, box) {
          final size = Size(box.maxWidth, AnchorCube.height);
          final g = AnchorGeometry(size, yaw: _yaw, pitch: _pitch);
          return MouseRegion(
            cursor: _turning ? SystemMouseCursors.grabbing : (on && _near != null ? SystemMouseCursors.click : SystemMouseCursors.grab),
            onHover: (e) {
              final n = on ? g.pick(e.localPosition) : null;
              if (n != _near) setState(() => _near = n);
              if (on && !hot) doc.setDirectHover('anchor');
            },
            onExit: (_) {
              setState(() => _near = null);
              if (hot) doc.setDirectHover(null);
            },
            // contract: a click on a point places the anchor; a drag anywhere turns the box to look; a double-click squares the view again. Turning never edits.
            child: Listener(
              onPointerDown: (e) {
                if (e.buttons != kPrimaryButton) return;
                _down = e.localPosition;
                _turning = false;
              },
              onPointerMove: (e) {
                final d = _down;
                if (d == null) return;
                if (!_turning && (e.localPosition - d).distance < 4) return;
                setState(() {
                  _turning = true;
                  _near = null;
                  _yaw += e.delta.dx * .012;
                  _pitch = (_pitch + e.delta.dy * .012).clamp(-1.45, 1.45);
                });
              },
              onPointerUp: (e) {
                final d = _down;
                _down = null;
                if (_turning) {
                  setState(() => _turning = false);
                  return;
                }
                if (d == null || !on) return;
                final q = g.pick(e.localPosition);
                if (q == null) return;
                doc.beginGesture();
                doc.setMany({'anchor.x': q.$1, 'anchor.y': q.$2, 'anchor.z': q.$3});
                doc.endGesture();
              },
              onPointerCancel: (_) {
                _down = null;
                if (_turning) setState(() => _turning = false);
              },
              child: GestureDetector(
                onDoubleTap: () => setState(() {
                  _yaw = AnchorGeometry.defaultYaw;
                  _pitch = AnchorGeometry.defaultPitch;
                }),
                child: CustomPaint(size: size, painter: _CubePaint(g, at, _near, on, hot)),
              ),
            ),
          );
        },
      ),
    );
  }
}

class _CubePaint extends CustomPainter {
  const _CubePaint(this.g, this.at, this.near, this.on, this.hot);
  final AnchorGeometry g;
  final (double, double, double)? at, near;
  final bool on, hot;

  @override
  void paint(Canvas c, Size s) {
    Offset p(double x, double y, double z) => g.at(x, y, z);
    Path plate(double z) => Path()..addPolygon([p(0, 0, z), p(100, 0, z), p(100, 100, z), p(0, 100, z)], true);
    Paint line(Color col, [double w = 1]) => Paint()
      ..color = col
      ..style = PaintingStyle.stroke
      ..strokeWidth = w;
    final pivot = TransformZone.pivot.color;

    // edges shade by distance: the far ones recede
    const corners = [0.0, 100.0];
    final edges = <(Offset, Offset, double)>[];
    for (final a in corners) {
      for (final b in corners) {
        edges.add((p(0, a, b - 50), p(100, a, b - 50), g.depth(50, a, b - 50)));
        edges.add((p(a, 0, b - 50), p(a, 100, b - 50), g.depth(a, 50, b - 50)));
        edges.add((p(a, b, -50), p(a, b, 50), g.depth(a, b, 0)));
      }
    }
    for (final (a, b, d) in edges) {
      c.drawLine(a, b, line(Color.lerp(Grey.g63, Grey.g26, ((d + 1.2) / 2.4).clamp(0, 1))!));
    }

    final a = at;
    if (a != null && a.$3 > -50) {
      final cut = plate(a.$3);
      c.drawPath(cut, Paint()..color = pivot.withValues(alpha: on ? .16 : .08));
      c.drawPath(cut, line(pivot.withValues(alpha: on ? .6 : .3)));
      final m = p(a.$1, a.$2, a.$3), f = p(a.$1, a.$2, -50);
      c.drawLine(m, f, line(pivot.withValues(alpha: .55)));
      c.drawCircle(f, 1.6, Paint()..color = pivot.withValues(alpha: .55));
    }

    final front = plate(-50);
    c.drawPath(
      front,
      Paint()
        ..shader = LinearGradient(
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
          colors: [const Color(0xFF3D6BFF).withValues(alpha: .34), const Color(0xFF8FB0FF).withValues(alpha: .16)],
        ).createShader(front.getBounds()),
    );
    c.drawPath(front, line(const Color(0xFF8FB0FF).withValues(alpha: .7)));

    for (final q in g.points) {
      final isNear = q == near, d = g.depth(q.$1, q.$2, q.$3), k = ((d + 1.2) / 2.4).clamp(0.0, 1.0);
      c.drawCircle(p(q.$1, q.$2, q.$3), (isNear ? 1.4 : 0) + 2.6 - k * 1.2, Paint()..color = isNear ? Grey.g95 : Color.lerp(Grey.g76, Grey.g38, k)!);
    }

    if (a != null) {
      final ink = !on ? Grey.g56 : (hot ? pivot : Grey.g95);
      final m = p(a.$1, a.$2, a.$3);
      c.drawCircle(m, 7, Paint()..color = Grey.g07);
      c.drawCircle(m, 7, line(ink, 1.6));
      c.drawCircle(m, 2, Paint()..color = ink);
    }

    final tp = TextPainter(
      text: TextSpan(text: 'FRONT', style: T.micro(const Color(0xFF8FB0FF))),
      textDirection: TextDirection.ltr,
    )..layout();
    final base = p(50, 100, -50), up = p(50, 0, -50);
    final away = base + (base - up) / (base - up).distance * 10;
    if ((base - up).distance > 1) tp.paint(c, away - Offset(tp.width / 2, tp.height / 2));
  }

  @override
  bool shouldRepaint(_CubePaint o) =>
      o.at != at || o.near != near || o.on != on || o.hot != hot || o.g.yaw != g.yaw || o.g.pitch != g.pitch || o.g.centre != g.centre;
}
