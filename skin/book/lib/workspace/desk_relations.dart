// The Relations tab: the world seen from outside the camera, as the Depth tab sees its depth: every thing where it is
// (off the camera too), the camera's frame, and each relation drawn as its shape (a ring, a row, a pusher's reach,
// a follow's threads). Front or top, or turned by a drag; a tap picks a thing or a relation, the inspector beside it.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'desk_kit.dart';
import 'relate.dart';
import 'stage_art.dart';
import 'ws.dart';

class DeskRelations extends StatefulWidget {
  const DeskRelations({super.key});
  @override
  State<DeskRelations> createState() => _DeskRelationsState();
}

class _DeskRelationsState extends State<DeskRelations> {
  double _yaw = 0, _pitch = 0;   // (turned: 0, 0 is from the front, the camera's own way)

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: WsScope.read(context),
    builder: (context, _) {
      final ws = WsScope.read(context), world = _World.of(ws);
      final front = _yaw == 0 && _pitch == 0, top = _yaw == 0 && _pitch == -math.pi / 2;
      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          DeskStrip(
            left: [DeskLayerChip('Relations · ${world.relations.length}', WsT.toneGroup)],
            right: [
              DeskChip(label: 'Front', on: front, onTap: () => setState(() => _yaw = _pitch = 0)),
              DeskChip(label: 'Top', on: top, onTap: () => setState(() {
                _yaw = 0;
                _pitch = -math.pi / 2;
              })),
            ],
          ),
          const SizedBox(height: WsT.gutter),
          Expanded(
            child: ColoredBox(
              color: WsT.card,
              child: LayoutBuilder(builder: (context, box) {
                final view = _View(world, box.biggest, _yaw, _pitch);
                return GestureDetector(
                  key: const ValueKey('ws-relations-map'),
                  behavior: HitTestBehavior.opaque,
                  onPanUpdate: (d) => setState(() {
                    _yaw += d.delta.dx * .01;
                    _pitch = (_pitch - d.delta.dy * .01).clamp(-math.pi / 2, math.pi / 2);
                  }),
                  onTapUp: (d) {
                    if (view.hit(d.localPosition) case final id?) ws.select(id);
                  },
                  child: CustomPaint(size: box.biggest, painter: _Map(view, ws.selected)),
                );
              }),
            ),
          ),
        ],
      );
    },
  );
}

typedef _P3 = ({double x, double y, double z});

/// What the map draws, read from the inside: the things, the camera, the relations.
class _World {
  _World(this.things, this.camera, this.relations, this.half);
  final Map<String, (_P3, Color, String)> things;   // (id: where, its colour, its name)
  final _P3? camera;
  final List<_Relation> relations;
  final Size half;   // (half the camera's frame, in the stage's units)

  static _World of(Ws ws) {
    final m = ws.inside;
    if (m == null) return _World(const {}, null, const [], Size.zero);
    final now = ws.frame / Ws.fps, things = <String, (_P3, Color, String)>{};
    _P3? camera;
    double v(int t, String n) => (n == 'x' || n == 'y' ? m.shows(t, n, now)?.$1 : null) ?? m.value(t, n, now) ?? 0;
    final rels = <_Relation>[];
    for (final l in ws.layers) {
      final t = int.tryParse(l.id);
      if (t == null) continue;
      final kind = m.word(t, 'kind');
      if (kind == 'behaviour') {
        if (wsRelations.contains(m.word(t, 'file'))) rels.add(_Relation(l.id, m.word(t, 'file'), [for (final s in m.word(t, 'members').split(' ')) if (s != '_' && s.isNotEmpty) s], v));
        continue;
      }
      final at = (x: v(t, 'x'), y: v(t, 'y'), z: v(t, 'z'));
      if (kind == 'camera') {
        camera = at;
        continue;
      }
      final fill = int.tryParse(m.word(t, 'fill'), radix: 16);
      things[l.id] = (at, fill == null ? wsTone(l.kind) : Color(0xFF000000 | fill), l.name);
    }
    return _World(things, camera, rels, Size(WsArt.comp.width / 2 / m.point, WsArt.comp.height / 2 / m.point));
  }
}

/// A relation's shape in the world: its kind, its things, and the knobs that place it.
class _Relation {
  _Relation(this.id, this.kind, this.members, double Function(int, String) v)
      : radius = v(int.parse(id), 'radius'),
        gap = v(int.parse(id), 'gap'),
        centre = (x: v(int.parse(id), 'centre_x'), y: v(int.parse(id), 'centre_y'), z: 0.0),
        leader = '${v(int.parse(id), 'leader').round()}';
  final String id, kind, leader;
  final List<String> members;
  final double radius, gap;
  final _P3 centre;
}

/// The world turned and fitted to the map.
class _View {
  _View(this.world, this.size, double yaw, double pitch) : _cy = math.cos(yaw), _sy = math.sin(yaw), _cp = math.cos(pitch), _sp = math.sin(pitch) {
    final all = [for (final (p, _, _) in world.things.values) _turn(p), for (final p in _frame) _turn(p), if (world.camera case final c?) _turn(c)];
    var r = Rect.fromLTRB(-1, -1, 1, 1);
    for (final p in all) {
      r = r.expandToInclude(Rect.fromCenter(center: p, width: 0, height: 0));
    }
    _fit = r.inflate(r.longestSide * .08 + .1);
    _scale = math.min(size.width / _fit.width, size.height / _fit.height);
  }
  final _World world;
  final Size size;
  final double _cy, _sy, _cp, _sp;
  late final Rect _fit;
  late final double _scale;

  List<_P3> get _frame => [for (final (sx, sy) in const [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]) (x: sx * world.half.width, y: sy * world.half.height, z: 0.0)];

  Offset _turn(_P3 p) {   // (turned about the up axis, then tipped: x right, y up on the map)
    final x = p.x * _cy + p.z * _sy, z = -p.x * _sy + p.z * _cy;
    return Offset(x, p.y * _cp - z * _sp);
  }

  Offset on(_P3 p) {
    final q = _turn(p);
    return Offset(size.width / 2 + (q.dx - _fit.center.dx) * _scale, size.height / 2 - (q.dy - _fit.center.dy) * _scale);
  }

  double get unit => _scale;

  /// The thing nearest the tap, else the relation whose shape passes nearest.
  String? hit(Offset at) {
    String? best;
    var d = 16.0;
    for (final e in world.things.entries) {
      final o = (on(e.value.$1) - at).distance;
      if (o < d) (best, d) = (e.key, o);
    }
    if (best != null) return best;
    for (final r in world.relations) {
      for (final p in shape(r)) {
        if ((on(p) - at).distance < 10) return r.id;
      }
    }
    return null;
  }

  /// Points along a relation's shape, in the world.
  List<_P3> shape(_Relation r) {
    final lead = world.things[r.leader]?.$1;
    final c = (x: r.centre.x + (lead?.x ?? 0), y: r.centre.y + (lead?.y ?? 0), z: lead?.z ?? 0.0);
    switch (r.kind) {
      case 'around' || 'push':
        final o = r.kind == 'push' ? lead : c;
        if (o == null) return const [];
        return [for (var k = 0; k <= 48; k++) (x: o.x + r.radius * math.sin(k / 48 * 2 * math.pi), y: o.y + r.radius * math.cos(k / 48 * 2 * math.pi), z: o.z)];
      case 'row':
        final half = r.gap * (r.members.length - 1) / 2;
        return [for (var k = 0; k <= 12; k++) (x: c.x - half + 2 * half * k / 12, y: c.y, z: c.z)];
      default:   // (a follow: from its leader to each of its things)
        return const [];
    }
  }
}

const _hues = [Color(0xFFD85A30), Color(0xFF7F77DD), Color(0xFF1D9E75), Color(0xFFD4537E), Color(0xFF378ADD)];

class _Map extends CustomPainter {
  _Map(this.v, this.selected);
  final _View v;
  final String? selected;

  @override
  void paint(Canvas c, Size s) {
    final line = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1;
    // (the camera's frame, and where it stands)
    line.color = WsT.toneCamera;
    c.drawPath(Path()..addPolygon([for (final p in v._frame) v.on(p)], true), line);
    if (v.world.camera case final cam?) c.drawCircle(v.on(cam), 5, Paint()..color = WsT.toneCamera);
    // (each relation's shape, its threads to its things, in its own hue)
    for (final (i, r) in v.world.relations.indexed) {
      final hue = _hues[i % _hues.length], picked = r.id == selected;
      line
        ..color = hue.withValues(alpha: picked ? 1 : .7)
        ..strokeWidth = picked ? 2 : 1;
      final pts = [for (final p in v.shape(r)) v.on(p)];
      for (var k = 0; k + 1 < pts.length; k += 2) {
        c.drawLine(pts[k], pts[k + 1], line);   // (broken: a relation, not a thing)
      }
      final lead = v.world.things[r.leader]?.$1;
      if (r.kind == 'follow' && lead != null) {
        for (final m in r.members) {
          if (v.world.things[m]?.$1 case final p?) c.drawLine(v.on(lead), v.on(p), line);
        }
      }
      if (lead != null) c.drawCircle(v.on(lead), 9, line);   // (its leader ringed)
      for (final m in r.members) {
        if (v.world.things[m]?.$1 case final p?) c.drawCircle(v.on(p), 7, Paint()..color = hue.withValues(alpha: .25));
      }
    }
    // (the things, the picked one ringed)
    for (final e in v.world.things.entries) {
      final at = v.on(e.value.$1);
      c.drawCircle(at, 4, Paint()..color = e.value.$2);
      c.drawCircle(at, 4, Paint()..color = Grey.g56..style = PaintingStyle.stroke..strokeWidth = .75);   // (a dark thing still seen on the dark map)
      if (e.key == selected) c.drawCircle(at, 8, Paint()..color = WsT.accent..style = PaintingStyle.stroke..strokeWidth = 1.5);
    }
  }

  @override
  bool shouldRepaint(_Map o) => true;
}
