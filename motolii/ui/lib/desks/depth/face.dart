import 'dart:math' as math;
import 'package:flutter/gestures.dart';
import 'package:flutter/widgets.dart';
import '../../browser/parts.dart';
import '../parts.dart';
import '../../hf/neutral.dart';
import '../../hf/metrics.dart' show Dn, Surface;

/// A layer in the scene: centre and size in world units. The target is the origin.
class DLayer {
  DLayer(this.x, this.y, this.z, this.w, this.h);
  double x, y, z, w, h;
}

class DCam {
  DCam(this.x, this.y, this.z);
  double x, y, z;
}

List<DLayer> depthDefaultLayers() => [
      DLayer(-170, 60, -140, 200, 112),
      DLayer(90, -20, 40, 240, 135),
      DLayer(230, 110, -210, 180, 100),
      DLayer(-60, -70, 210, 220, 124),
      DLayer(-250, 20, 120, 160, 90),
    ];
DCam depthDefaultCam() => DCam(0, 80, -620);

/// The four projections. Each is a flat 2D drawing; none is a perspective viewport.
/// top: x across, depth up. side: depth across, y up. front: x across, y up.
enum DView { top, front, side, topWide }

class DepthGeom {
  DepthGeom(this.size, this.view, this.cam, this.layers, {this.pad = 22});
  final Size size;
  final DView view;
  final DCam cam;
  final List<DLayer> layers;
  final double pad;

  (double, double) uv(double x, double y, double z) => switch (view) {
        DView.top => (x, z),
        DView.topWide => (z, x),
        DView.front => (x, y),
        DView.side => (z, y),
      };
  bool get fwdIsU => view == DView.side || view == DView.topWide;

  late final _b = () {
    var u0 = 0.0, u1 = 0.0, v0 = 0.0, v1 = 0.0;
    void take(double x, double y, double z, {double ex = 0, double ey = 0}) {
      final p = uv(x, y, z);
      u0 = math.min(u0, p.$1 - (view == DView.front || view == DView.top ? ex : (view == DView.topWide ? 0 : 0)));
      u1 = math.max(u1, p.$1 + (view == DView.front || view == DView.top ? ex : 0));
      v0 = math.min(v0, p.$2 - (view == DView.topWide ? ex : ey));
      v1 = math.max(v1, p.$2 + (view == DView.topWide ? ex : ey));
    }
    take(cam.x, cam.y, cam.z);
    for (final l in layers) { take(l.x, l.y, l.z, ex: l.w / 2, ey: l.h / 2); }
    if (view == DView.front) { u0 = math.min(u0, -360); u1 = math.max(u1, 360); v0 = math.min(v0, -220); v1 = math.max(v1, 220); }
    return Rect.fromLTRB(u0, v0, u1, v1).inflate(60);
  }();

  // Uniform everywhere except the strip, which is a depth ruler: depth and lateral offset get their own scale.
  double get ku => view == DView.topWide ? (size.width - pad * 2) / _b.width : math.min((size.width - pad * 2) / _b.width, (size.height - pad * 2) / _b.height);
  double get kv => view == DView.topWide ? (size.height - pad * 2) / _b.height : ku;
  double get k => ku;
  Offset px(double x, double y, double z) {
    final p = uv(x, y, z);
    return Offset(size.width / 2 + (p.$1 - _b.center.dx) * ku, size.height / 2 - (p.$2 - _b.center.dy) * kv);
  }
  double uPx(double u) => size.width / 2 + (u - _b.center.dx) * ku;
  double vPx(double v) => size.height / 2 - (v - _b.center.dy) * kv;
  Offset camPx() => px(cam.x, cam.y, cam.z);
  Offset layerPx(int i) => px(layers[i].x, layers[i].y, layers[i].z);

  /// The world point under a screen point, in the top view: x across, depth up.
  (double, double) world(Offset p) => ((p.dx - size.width / 2) / ku + _b.center.dx, -(p.dy - size.height / 2) / kv + _b.center.dy);

  /// Screen delta to world delta for dragging this view.
  (double, double, double) deltaWorld(Offset d) => switch (view) {
        DView.top => (d.dx / k, 0, -d.dy / k),
        DView.topWide => (-d.dy / kv, 0, d.dx / ku),
        DView.front => (d.dx / k, -d.dy / k, 0),
        DView.side => (0, -d.dy / k, d.dx / k),
      };
}

/// A layer as the host knows it in the top view: where it sits on the floor plan, whose it is, whether it can move.
class DepthItem {
  const DepthItem(this.name, this.x, this.z, {this.locked = false, this.selected = false});
  final String name;
  final double x, z;
  final bool locked, selected;
}

/// What a host gives the desk: the floor plan of the scene around its target, the camera on it, and how a press, a
/// drag and a release reach the document. The plan is the top view only; a host that has no height for the layers
/// cannot draw a front or a side view. With no host the desk plays with fixture layers in all three.
abstract class DepthHost implements Listenable {
  List<DepthItem> get items;
  ({double x, double z})? get camera;
  double get fovDegrees;
  bool get cameraSelectable;
  String? get targetName;

  /// Press on a layer ([hit] its index), on the camera (-1) or on nothing (-2).
  void press(int hit);

  /// A layer is dragged by [wx], [wz] from where it was pressed; the camera is dragged to the world point [wx], [wz].
  void drag(int hit, double wx, double wz);
  void release({bool cancel = false});
}

/// Depth is a view of the Stage, not an editor of its camera. Positions can be dragged here the way they are
/// dragged on the Stage; precise camera values (distance, FOV, focus) belong to the Inspector and only show as readouts.
class DepthDesk extends StatefulWidget {
  const DepthDesk({super.key, this.host});
  final DepthHost? host;
  @override
  State<DepthDesk> createState() => _DepthDeskState();
}

class _DepthDeskState extends State<DepthDesk> {
  final _cam = depthDefaultCam();
  final _layers = depthDefaultLayers();
  final double _fov = 46; // read from the camera; edited in the Inspector
  int _selected = 1;
  Offset? _from;

  DepthHost? get host => widget.host;
  DCam get cam => host == null ? _cam : (host!.camera == null ? DCam(0, 0, -600) : DCam(host!.camera!.x, 0, host!.camera!.z));
  List<DLayer> get layers => host == null ? _layers : [for (final i in host!.items) DLayer(i.x, 0, i.z, 0, 0)];
  double get fov => host == null ? _fov : host!.fovDegrees;
  int get selected => host == null ? _selected : math.max(0, host!.items.lastIndexWhere((i) => i.selected));
  set selected(int v) => _selected = v;
  DView view = DView.top;
  int? drag; // -1 camera, >=0 layer

  double get selDist {
    final l = layers;
    if (l.isEmpty) return 0;
    final a = l[selected], c = cam;
    return host == null ? (a.z - c.z).abs() : math.sqrt((a.x - c.x) * (a.x - c.x) + (a.z - c.z) * (a.z - c.z));
  }

  double get camDist => host == null ? -cam.z : math.sqrt(cam.x * cam.x + cam.z * cam.z);
  String get selName => host == null || host!.items.isEmpty ? 'Layer ${selected + 1}' : host!.items[selected].name;

  int _hit(DepthGeom g, Offset p) {
    var best = -2;
    var bd = 28.0;
    final cd = (g.camPx() - p).distance;
    if (cd < bd && (host == null || host!.cameraSelectable)) { bd = cd; best = -1; }
    for (var i = 0; i < layers.length; i++) {
      final dd = (g.layerPx(i) - p).distance;
      if (dd < bd) { bd = dd; best = i; }
    }
    return best;
  }

  @override
  Widget build(BuildContext context) => host == null ? _shell() : ListenableBuilder(listenable: host!, builder: (_, __) => _shell());

  Widget _shell() => DeskShell(
        kind: DeskKind.depth,
        title: 'Depth',
        subtitle: 'STAGE VIEW',
        trailing: host != null ? null : SizedBox(
          width: 84,
          child: Segmented(const ['Top', 'Front', 'Side'], view == DView.top ? 0 : (view == DView.front ? 1 : 2), height: 18, onChanged: (i) => setState(() => view = [DView.top, DView.front, DView.side][i])),
        ),
        full: (c, s) => Padding(
          padding: const EdgeInsets.fromLTRB(9, 9, 9, 9),
          child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            Expanded(child: _diagram(view, true)),
            const SizedBox(height: 9),
            Row(children: [
              Expanded(child: NumBox('Camera', '${camDist.round()}', compact: true)),
              const SizedBox(width: 4.5),
              Expanded(child: NumBox('FOV', '${fov.round()}°', compact: true)),
              const SizedBox(width: 4.5),
              Expanded(child: NumBox(selName, '${selDist.round()}', compact: true)),
            ]),
            const SizedBox(height: 9),
            Row(children: [_key(Surface.ink, 'Camera'), const SizedBox(width: 10.5), _key(kYellow, 'Selected'), const SizedBox(width: 10.5), _key(kBlue, 'Other layers')]),
            const SizedBox(height: 7.5),
            Text(host?.targetName == null ? 'Drag layers or the camera to move them. Camera settings live in Inspector.' : 'Looking at ${host!.targetName}. Drag layers or the camera to move them. Camera settings live in Inspector.', style: sans(Dn.labelSize, c: Surface.muted)),
          ]),
        ),
        strip: (c, s) => Padding(padding: const EdgeInsets.fromLTRB(6, 1.5, 6, 6), child: _diagram(DView.topWide, false)),
        tall: (c, s) => Padding(
          padding: const EdgeInsets.all(6),
          child: Column(children: [
            Expanded(child: _diagram(DView.top, false)),
            const SizedBox(height: 4.5),
            Row(children: [Expanded(child: NumBox('Dist', '${camDist.round()}', compact: true)), const SizedBox(width: 4), Expanded(child: NumBox('FOV', '${fov.round()}°', compact: true))]),
          ]),
        ),
      );

  Widget _key(Color c, String t) => Row(children: [Container(width: 7.5, height: 7.5, decoration: BoxDecoration(color: c, shape: BoxShape.circle)), const SizedBox(width: 4.5), Text(t, style: sans(Dn.labelSize, c: N.g69))]);

  Widget _diagram(DView v, bool detail) => LayoutBuilder(builder: (context, box) {
        final size = Size(box.maxWidth, box.maxHeight);
        final g = DepthGeom(size, v, cam, layers, pad: v == DView.topWide ? 8 : 26);
        final h0 = host;
        if (h0 != null) {
          // The host's document decides who is selected, so a press only tells it and the plan follows.
          return GestureDetector(
            key: const ValueKey('depth-diagram'),
            dragStartBehavior: DragStartBehavior.down,
            onTapDown: (d) => h0.press(_hit(g, d.localPosition)),
            onTapUp: (_) => h0.release(),
            onPanStart: (d) {
              final h = _hit(g, d.localPosition);
              drag = h == -2 ? null : h;
              _from = d.localPosition;
              h0.press(h);
            },
            onPanUpdate: (d) {
              final h = drag;
              if (h == null || _from == null) return;
              if (h == -1) {
                final w = g.world(d.localPosition);
                h0.drag(-1, w.$1, w.$2);
              } else {
                final w = g.deltaWorld(d.localPosition - _from!);
                h0.drag(h, w.$1, w.$3);
              }
            },
            onPanEnd: (_) {
              drag = null;
              h0.release();
            },
            onPanCancel: () {
              if (drag != null) h0.release(cancel: true);
              drag = null;
            },
            child: CustomPaint(size: size, painter: DepthPainter(g, fov, selected, detail)),
          );
        }
        return GestureDetector(
          key: const ValueKey('depth-diagram'),
          dragStartBehavior: DragStartBehavior.down,
          onTapDown: (d) { final h = _hit(g, d.localPosition); if (h >= 0) setState(() => selected = h); },
          onPanStart: (d) {
            final h = _hit(g, d.localPosition);
            drag = h == -2 ? null : h;
            if (h >= 0) setState(() => selected = h);
          },
          onPanUpdate: (d) {
            if (drag == null) return;
            final w = g.deltaWorld(d.delta);
            setState(() {
              if (drag == -1) {
                cam.x += w.$1; cam.y += w.$2; cam.z = math.min(cam.z + w.$3, -120);
              } else {
                final l = layers[drag!];
                l.x += w.$1; l.y += w.$2; l.z += w.$3;
              }
            });
          },
          onPanEnd: (_) => drag = null,
          child: CustomPaint(size: size, painter: DepthPainter(g, fov, selected, detail)),
        );
      });
}

Color depthLayerColor(int i, int selected) => i == selected ? kYellow : const [kBlue, kPink, kMint, kViolet][i % 4];

class DepthPainter extends CustomPainter {
  DepthPainter(this.g, this.fov, this.selected, this.detail);
  final DepthGeom g;
  final double fov;
  final int selected;
  final bool detail;

  void _text(Canvas c, String t, Offset o, TextStyle st, {bool centre = false}) {
    final tp = TextPainter(text: TextSpan(text: t, style: st), textDirection: TextDirection.ltr)..layout();
    tp.paint(c, Offset(centre ? o.dx - tp.width / 2 : o.dx, o.dy - tp.height / 2));
  }

  @override
  void paint(Canvas c, Size s) {
    final rr = RRect.fromRectAndRadius(Offset.zero & s, const Radius.circular(4.5));
    c.drawRRect(rr, Paint()..color = N.g07);
    c.save();
    c.clipRRect(rr);
    final v = g.view;
    final o = g.px(0, 0, 0);
    final cp = g.camPx();
    final half = fov * math.pi / 360;
    final vhalf = math.atan(math.tan(half) * .5625);
    final sel = g.layers.isEmpty ? DLayer(0, 0, 0, 0, 0) : g.layers[selected];
    final fd = sel.z - g.cam.z;
    final sp = g.px(sel.x, sel.y, sel.z);
    // a single ground line through the target keeps the drawing anchored; nothing else is a ruler
    final ground = Paint()..color = N.g15..strokeWidth = 1.2;
    c.drawLine(Offset(0, o.dy), Offset(s.width, o.dy), ground);
    c.drawLine(Offset(o.dx, 0), Offset(o.dx, s.height), ground);

    if (v == DView.front) {
      final w = 2 * fd * math.tan(half), h = w * .5625;
      final fr = Rect.fromCenter(center: g.px(0, 0, 0), width: w * g.k, height: h * g.k);
      c.drawRect(fr, Paint()..color = kBlue.withValues(alpha: .20));
      c.drawRect(fr, Paint()..color = kBlue..style = PaintingStyle.stroke..strokeWidth = 2);
      for (var i = 0; i < g.layers.length; i++) {
        final l = g.layers[i];
        final r = Rect.fromCenter(center: g.px(l.x, l.y, l.z), width: l.w * g.k, height: l.h * g.k);
        c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(2)), Paint()..color = depthLayerColor(i, selected).withValues(alpha: .92));
        if (i == selected) c.drawRRect(RRect.fromRectAndRadius(r.inflate(3), const Radius.circular(4)), Paint()..color = Surface.ink..style = PaintingStyle.stroke..strokeWidth = 2);
        if (detail) _text(c, '${i + 1}', r.center, sans(Dn.nameSize, c: N.g10, w: FontWeight.w700), centre: true);
      }
      c.drawCircle(cp, 9, Paint()..color = Surface.ink);
      c.drawCircle(cp, 4, Paint()..color = N.g07);
    } else {
      final fwdU = g.fwdIsU;
      final half2 = v == DView.side ? vhalf : half;
      Offset w2p(double du, double dv) => Offset(du * g.ku, -dv * g.kv);
      const far = 6000.0;
      final lat = far * math.tan(half2);
      final e1 = fwdU ? cp + w2p(far, -lat) : cp + w2p(-lat, far);
      final e2 = fwdU ? cp + w2p(far, lat) : cp + w2p(lat, far);
      // the field of view: one flat fan
      // what the camera sees: a light wash and its two edges, a view over the plan rather than a slab on it
      c.drawPath(Path()..moveTo(cp.dx, cp.dy)..lineTo(e1.dx, e1.dy)..lineTo(e2.dx, e2.dy)..close(), Paint()..color = kBlue.withValues(alpha: .08));
      final edge = Paint()..color = kBlue.withValues(alpha: .45)..strokeWidth = 1;
      c.drawLine(cp, e1, edge);
      c.drawLine(cp, e2, edge);
      // where the selected layer sits: a dashed line back to the camera
      final dash = Paint()..color = Surface.ink.withValues(alpha: .5)..strokeWidth = 1.4;
      final dv = sp - cp;
      final n = (dv.distance / 8).floor();
      for (var i = 0; i < n; i += 2) { c.drawLine(cp + dv * (i / n), cp + dv * ((i + 1) / n), dash); }
      // layers: fat bars across the depth axis
      for (var i = 0; i < g.layers.length; i++) {
        final l = g.layers[i];
        final p = g.px(l.x, l.y, l.z);
        final ext = math.max(detail ? 16.0 : 8.0, (v == DView.side ? l.h : l.w) * (fwdU ? g.kv : g.ku) / 2);
        final a = fwdU ? Offset(p.dx, p.dy - ext) : Offset(p.dx - ext, p.dy);
        final b = fwdU ? Offset(p.dx, p.dy + ext) : Offset(p.dx + ext, p.dy);
        final hot = i == selected;
        final thick = hot ? 16.0 : 12.0;
        if (hot) c.drawLine(a, b, Paint()..color = Surface.ink..strokeWidth = thick + 4..strokeCap = StrokeCap.round);
        c.drawLine(a, b, Paint()..color = depthLayerColor(i, selected)..strokeWidth = thick..strokeCap = StrokeCap.round);
        if (detail) _text(c, '${i + 1}', fwdU ? Offset(p.dx + 14, p.dy - ext - 8) : Offset(p.dx + ext + 14, p.dy), sans(Dn.nameSize, c: hot ? Surface.ink : N.g69, w: hot ? FontWeight.w700 : FontWeight.w400), centre: true);
      }
      // camera: a big white body with its lens toward the view
      c.save();
      c.translate(cp.dx, cp.dy);
      c.rotate(fwdU ? math.pi / 2 : 0);
      final body = RRect.fromRectAndRadius(const Rect.fromLTRB(-13, -3, 13, 17), const Radius.circular(2));
      c.drawPath(Path()..moveTo(-13, -3)..lineTo(-7, -15)..lineTo(7, -15)..lineTo(13, -3)..close(), Paint()..color = Surface.ink);
      c.drawRRect(body, Paint()..color = Surface.ink);
      c.restore();
    }
    // target: a plain cross
    final cross = Paint()..color = Surface.ink..strokeWidth = 2.4..strokeCap = StrokeCap.round;
    c.drawLine(o + const Offset(-9, 0), o + const Offset(9, 0), cross);
    c.drawLine(o + const Offset(0, -9), o + const Offset(0, 9), cross);
    if (detail) {
      _text(c, 'TARGET', o + const Offset(30, 18), sans(Dn.microSize, c: Surface.muted, ls: 1.2), centre: true);
      _text(c, switch (v) { DView.top => 'TOP', DView.front => 'FRONT', DView.side => 'SIDE', DView.topWide => 'TOP' }, const Offset(12, 14), sans(Dn.labelSize, c: Surface.muted, w: FontWeight.w600, ls: 1.4));
    }
    c.restore();
  }
  @override
  bool shouldRepaint(_) => true;
}
