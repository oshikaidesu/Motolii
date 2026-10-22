// The Relations tab: the world seen from outside the camera, as the Depth tab sees its depth: every thing where it is
// (off the camera too), the camera's frame, and each relation drawn as its shape (a ring, a row, a pusher's reach,
// a follow's threads). Front or top, or turned by a drag; a tap picks a thing or a relation, the inspector beside it.
import 'dart:math' as math;

import 'package:flutter/gestures.dart' show PointerScrollEvent;
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
  double _yaw = 0, _pitch = 0, _zoom = 1;   // (turned: 0, 0 is from the front, the camera's own way; a scroll comes closer)
  (int, Map<String, List<_P3>>)? _paths;   // (each thing's way through the whole stretch, as the document stood: read again when it changes)

  /// A picked relation's things at each of its keys' moments, in time's order: the shapes it goes through, not the way.
  (Object, Map<String, List<_P3>>)? _stations;
  Map<String, List<_P3>> _stationsOf(Ws ws) {
    final m = ws.inside, r = int.tryParse(ws.selected ?? '');
    final at = (ws.rev, ws.selected);
    if (_stations?.$1 == at) return _stations!.$2;
    final out = <String, List<_P3>>{};
    if (m != null && r != null && m.word(r, 'kind') == 'behaviour') {
      final moments = {for (final n in m.moving(r)) ...m.keyMoments(r, n)}.toList()..sort();
      for (final s in m.word(r, 'members').split(' ')) {
        final t = int.tryParse(s);
        if (t == null || moments.length < 2) continue;
        double v(String n, double a) => m.shows(t, n, a)?.$1 ?? m.value(t, n, a) ?? 0;
        out[s] = [for (final a in moments) (x: v('x', a), y: v('y', a), z: v('z', a))];
      }
    }
    return (_stations = (at, out)).$2;
  }

  Map<String, List<_P3>> _pathsOf(Ws ws) {
    if (_paths?.$1 == ws.rev) return _paths!.$2;
    final m = ws.inside, out = <String, List<_P3>>{};
    if (m != null) {
      const steps = 60;
      for (final l in ws.layers) {
        final t = int.tryParse(l.id);
        if (t == null || m.word(t, 'kind') == 'behaviour' || m.word(t, 'kind') == 'camera') continue;
        double v(String n, double at) => m.shows(t, n, at)?.$1 ?? m.value(t, n, at) ?? 0;
        final way = [for (var k = 0; k <= steps; k++) (x: v('x', k / steps * Ws.duration / Ws.fps), y: v('y', k / steps * Ws.duration / Ws.fps), z: v('z', k / steps * Ws.duration / Ws.fps))];
        if (way.any((p) => (p.x - way.first.x).abs() + (p.y - way.first.y).abs() + (p.z - way.first.z).abs() > 1e-4)) out[l.id] = way;   // (only what moves)
      }
    }
    return (_paths = (ws.rev, out)).$2;
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: WsScope.read(context),
    builder: (context, _) {
      final ws = WsScope.read(context), world = _World.of(ws);
      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          DeskStrip(
            left: [DeskLayerChip('Relations · ${world.relations.length}', WsT.toneGroup)],
          ),
          const SizedBox(height: WsT.gutter),
          Expanded(
            child: ColoredBox(
              color: WsT.card,
              child: LayoutBuilder(builder: (context, box) {
                final view = _View(world, box.biggest, _yaw, _pitch, paths: _pathsOf(ws), zoom: _zoom);
                return Listener(
                  onPointerSignal: (e) {
                    if (e is PointerScrollEvent) setState(() => _zoom = (_zoom * math.exp(-e.scrollDelta.dy / 300)).clamp(.3, 8.0));
                  },
                  child: GestureDetector(
                  key: const ValueKey('ws-relations-map'),
                  behavior: HitTestBehavior.opaque,
                  onPanUpdate: (d) => setState(() {
                    _yaw += d.delta.dx * .01;
                    _pitch = (_pitch - d.delta.dy * .01).clamp(-math.pi / 2, math.pi / 2);
                  }),
                  onTapUp: (d) {
                    if (view.axisAt(d.localPosition) case final a?) {   // (an axis of the gizmo: seen from that side, as Blender's)
                      final (yaw, pitch) = switch (a) { 'x' => (-math.pi / 2, 0.0), 'y' => (0.0, -math.pi / 2), _ => (0.0, 0.0) };
                      return setState(() {
                        _yaw = yaw;
                        _pitch = pitch;
                      });
                    }
                    if (view.hit(d.localPosition) case final id?) ws.select(id);
                  },
                  child: CustomPaint(size: box.biggest, painter: _Map(view, ws.selected, {if (_pathsOf(ws)[ws.selected] case final w?) ws.selected!: w}, _stationsOf(ws))),
                ));
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
    double v(int t, String n) => (n == 'x' || n == 'y' || n == 'z' ? m.shows(t, n, now)?.$1 : null) ?? m.value(t, n, now) ?? 0;   // (a place where it shows: a relation moves depth too)
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
      things[l.id] = (at, l.tone, l.name);   // (its label colour: the same as its row's)
    }
    return _World(things, camera, rels, Size(WsArt.comp.width / 2 / m.point, WsArt.comp.height / 2 / m.point));
  }
}

/// A relation's shape in the world: its kind, its things, and the knobs that place it.
class _Relation {
  _Relation(this.id, this.kind, this.members, double Function(int, String) v)
      : radius = v(int.parse(id), 'radius'),
        gap = v(int.parse(id), 'gap'),
        tilt = v(int.parse(id), 'tilt') * math.pi / 180,
        centre = (x: v(int.parse(id), 'centre_x'), y: v(int.parse(id), 'centre_y'), z: 0.0),
        leader = '${v(int.parse(id), 'leader').round()}';
  final String id, kind, leader;
  final List<String> members;
  final double radius, gap, tilt;
  final _P3 centre;
}

/// The world turned and fitted to the map.
class _View {
  _View(this.world, this.size, double yaw, double pitch, {Map<String, List<_P3>> paths = const {}, double zoom = 1}) : _cy = math.cos(yaw), _sy = math.sin(yaw), _cp = math.cos(pitch), _sp = math.sin(pitch) {
    // (fitted to the things and their ways, not to the camera's frame: what moves fills the map; a scroll comes closer)
    final all = [for (final (p, _, _) in world.things.values) _turn(p), for (final w in paths.values) for (final p in w) _turn(p)];
    if (all.isEmpty) all.addAll([for (final p in _frame) _turn(p)]);
    var r = Rect.fromCenter(center: all.first, width: .2, height: .2);
    for (final p in all) {
      r = r.expandToInclude(Rect.fromCenter(center: p, width: 0, height: 0));
    }
    _fit = r.inflate(r.longestSide * .08 + .05);
    _scale = math.min(size.width / _fit.width, size.height / _fit.height) * zoom;
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

  /// The axes gizmo in the corner, as Blender's: each axis's end, turned as the world is (x red, y green, z blue).
  Offset get axisMiddle => Offset(size.width - 38, size.height - 38);
  Map<String, Offset> get axisEnds => {
    for (final (n, p) in const [('x', (x: 1.0, y: 0.0, z: 0.0)), ('y', (x: 0.0, y: 1.0, z: 0.0)), ('z', (x: 0.0, y: 0.0, z: 1.0))]) n: axisMiddle + Offset(_turn(p).dx, -_turn(p).dy) * 20,
  };
  String? axisAt(Offset at) {
    for (final e in axisEnds.entries) {
      if ((e.value - at).distance < 9) return e.key;
    }
    return null;
  }

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
        final t = r.kind == 'push' ? 0.0 : r.tilt;   // (an around tipped back by its tilt)
        return [
          for (var k = 0; k <= 48; k++)
            (x: o.x + r.radius * math.sin(k / 48 * 2 * math.pi), y: o.y + r.radius * math.cos(k / 48 * 2 * math.pi) * math.cos(t), z: o.z + r.radius * math.cos(k / 48 * 2 * math.pi) * math.sin(t)),
        ];
      case 'row':
        final half = r.gap * (r.members.length - 1) / 2;
        return [for (var k = 0; k <= 12; k++) (x: c.x - half + 2 * half * k / 12, y: c.y, z: c.z)];
      default:   // (a follow: from its leader to each of its things)
        return const [];
    }
  }
}

class _Map extends CustomPainter {
  _Map(this.v, this.selected, this.paths, this.stations);
  final _View v;
  final String? selected;
  final Map<String, List<_P3>> paths, stations;   // (the picked thing's way; a picked relation's shapes at its keys)

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
    for (final r in v.world.relations) {
      final hue = wsLabel(r.id), picked = r.id == selected;   // (in its own label colour, as its row)
      line
        ..color = hue.withValues(alpha: picked ? 1 : .7)
        ..strokeWidth = picked ? 2 : 1;
      final pts = [for (final p in v.shape(r)) v.on(p)];
      for (var k = 0; k + 1 < pts.length; k += 2) {   // (broken: a relation, not a thing; each stretch in the colour of the thing it carries, as its row's bar)
        if (r.members.isNotEmpty) line.color = wsLabel(r.members[(k * r.members.length / pts.length).floor().clamp(0, r.members.length - 1)]).withValues(alpha: picked ? 1 : .8);
        c.drawLine(pts[k], pts[k + 1], line);
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
    // (each moving thing's way through the stretch, in its colour, a dot each step: a morph from one relation to another
    //  shows as the ways bending from one shape into the other, an ease as the dots closing up and spreading)
    for (final e in paths.entries) {   // (as After Effects: the picked thing's way only, a dot a step)
      final hue = v.world.things[e.key]?.$2 ?? WsT.accent, pts = [for (final p in e.value) v.on(p)];
      c.drawPath(Path()..addPolygon(pts, false), Paint()..color = hue.withValues(alpha: .6)..style = PaintingStyle.stroke..strokeWidth = 1);
      for (final p in pts) {
        c.drawCircle(p, 1.4, Paint()..color = hue);
      }
    }
    for (final e in stations.entries) {   // (a picked relation: where each thing stands at each of its keys, joined straight, ringed hollow)
      final hue = v.world.things[e.key]?.$2 ?? WsT.accent, pts = [for (final p in e.value) v.on(p)];
      c.drawPath(Path()..addPolygon(pts, false), Paint()..color = hue.withValues(alpha: .35)..style = PaintingStyle.stroke..strokeWidth = 1);
      for (final p in pts) {
        c.drawCircle(p, 5, Paint()..color = hue..style = PaintingStyle.stroke..strokeWidth = 1.2);
      }
    }
    // (the things, the picked one ringed)
    for (final e in v.world.things.entries) {
      final at = v.on(e.value.$1);
      c.drawCircle(at, 4, Paint()..color = e.value.$2);
      c.drawCircle(at, 4, Paint()..color = Grey.g56..style = PaintingStyle.stroke..strokeWidth = .75);   // (a dark thing still seen on the dark map)
      if (e.key == selected) c.drawCircle(at, 8, Paint()..color = WsT.accent..style = PaintingStyle.stroke..strokeWidth = 1.5);
    }
    // (the axes gizmo: a line to each axis's end, a coloured ball with its letter; a touch looks from that side)
    const hue = {'x': Color(0xFFFF3352), 'y': Color(0xFF8BDC00), 'z': Color(0xFF2890FF)};
    final mid = v.axisMiddle;
    for (final e in v.axisEnds.entries) {
      c.drawLine(mid, e.value, Paint()..color = hue[e.key]!..strokeWidth = 1.5);
      c.drawCircle(mid - (e.value - mid), 3, Paint()..color = hue[e.key]!.withValues(alpha: .4));   // (its other way, faint)
    }
    for (final e in v.axisEnds.entries) {
      c.drawCircle(e.value, 7, Paint()..color = hue[e.key]!);
      final t = TextPainter(text: TextSpan(text: e.key.toUpperCase(), style: T.micro(const Color(0xFF111111)).copyWith(fontWeight: FontWeight.w700)), textDirection: TextDirection.ltr)..layout();
      t.paint(c, e.value - Offset(t.width / 2, t.height / 2));
      t.dispose();
    }
  }

  @override
  bool shouldRepaint(_Map o) => true;
}
