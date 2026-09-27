import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/gestures.dart' show kPrimaryButton;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../hf/shell/place.dart';
import '../../session/editor_session.dart';

/// The hand on the Stage picture: what a press lands on and what a drag means, sent as the host's `stageGesture`
/// (begin, update, commit, cancel) in composition pixels. A press on a layer selects it (Shift adds) and a drag moves
/// the selection; the selected 2D / 2.5D layer's frame has corner, side and rotation handles; a 3D layer's gizmo is
/// the host's own mesh, drawn and grabbed as sent. [scale] is screen points per composition pixel.
class StageTouch extends StatefulWidget {
  const StageTouch({super.key, required this.c, required this.view, required this.scale});
  final EditorSession c;
  final String view;
  final double scale;
  @override
  State<StageTouch> createState() => _StageTouchState();
}

class _StageTouchState extends State<StageTouch> {
  EditorSession get c => widget.c;
  double get _s => widget.scale;
  Offset _comp(Offset screen) => screen / _s;
  Offset _screen(Offset comp) => comp * _s;

  Offset? _start;
  String? _handle;
  List<int> _ids = const [];
  Map<String, dynamic>? _pending;
  bool _sending = false;
  Future<void> _drained = Future.value();

  @override
  void initState() {
    super.initState();
    c.slice('stageTouch', const ['layers', 'selectedId', 'selectedIds', 'spatialGizmo']).addListener(_changed);
  }

  @override
  void dispose() {
    c.slice('stageTouch', const ['layers', 'selectedId', 'selectedIds', 'spatialGizmo']).removeListener(_changed);
    super.dispose();
  }

  void _changed() => setState(() {});

  static List<Offset> _corners(Map<String, dynamic> l) => [
        for (final v in (l['corners'] as List? ?? const []))
          if (v is List && v.length >= 2) Offset((v[0] as num).toDouble(), (v[1] as num).toDouble()),
      ];

  /// The front face of a layer's frame: four corners clockwise from the top left.
  static List<Offset> _face(Map<String, dynamic> l) {
    final raw = _corners(l);
    return raw.length == 8 ? [raw[0], raw[1], raw[3], raw[2]] : raw;
  }

  static bool _inside(List<Offset> poly, Offset p) {
    var inside = false;
    for (var i = 0, j = poly.length - 1; i < poly.length; j = i++) {
      final a = poly[i], b = poly[j];
      if ((a.dy > p.dy) != (b.dy > p.dy) && p.dx < (b.dx - a.dx) * (p.dy - a.dy) / (b.dy - a.dy) + a.dx) inside = !inside;
    }
    return inside;
  }

  bool _grabbable(Map<String, dynamic> l) => l['hidden'] != true && l['locked'] != true && l['kind'] != 'Camera';
  bool _spatial(Map<String, dynamic> l) => const ['3D', 'ThreeD'].contains('${l['projection']}');

  /// Handle name -> screen point, for the active 2D / 2.5D layer.
  Map<String, Offset> _handles() {
    final l = c.activeLayer;
    if (l == null || !_grabbable(l) || _spatial(l)) return const {};
    final f = _face(l);
    if (f.length != 4) return const {};
    final out = <String, Offset>{for (var i = 0; i < 4; i++) const ['nw', 'ne', 'se', 'sw'][i]: _screen(f[i])};
    for (var i = 0; i < 4; i++) {
      out[const ['n', 'e', 's', 'w'][i]] = _screen((f[i] + f[(i + 1) % 4]) / 2);
    }
    final centre = f.reduce((a, b) => a + b) / 4, top = (f[0] + f[1]) / 2, up = top - centre;
    out['rotation'] = _screen(top) + (up.distance > 0 ? up / up.distance : const Offset(0, -1)) * 22;
    return out;
  }

  /// The host's gizmo triangles for the active 3D layer, in composition pixels. The host's colours are meanings:
  /// white is the gizmo at rest, black the part the pointer is on; anything else is a linear colour.
  (List<Offset>, List<Color>, List<int>)? _mesh() {
    final l = c.activeLayer;
    if (l == null || !_spatial(l)) return null;
    final raw = EditorSession.map(c.state['spatialGizmo']);
    final v = raw['vertices'] as List?, col = raw['colors'] as List?, idx = raw['indices'] as List?;
    if (v == null || col == null || idx == null || v.isEmpty) return null;
    Color tone(Object? k) {
      if (k is! List || k.length < 4) return const Color(0x00000000);
      final rgb = [for (var i = 0; i < 3; i++) ((k[i] as num).toDouble()).clamp(0.0, 1.0)];
      final a = (((k[3] as num).toDouble()).clamp(0.0, 1.0) * 255).round();
      if (rgb.every((x) => x >= .999)) return H.text2.withAlpha(a);
      if (rgb.every((x) => x <= .001)) return H.playhead.withAlpha(a);
      int byte(double x) => (math.pow(x, 1 / 2.2) * 255).round();
      return Color.fromARGB(a, byte(rgb[0]), byte(rgb[1]), byte(rgb[2]));
    }

    final points = [for (final p in v) Offset(((p as List)[0] as num).toDouble(), (p[1] as num).toDouble())];
    return (points, [for (var k = 0; k < points.length; k++) tone(k < col.length ? col[k] : null)], [for (final i in idx) (i as num).toInt()]);
  }

  bool _onMesh(Offset comp) {
    final m = _mesh();
    if (m == null) return false;
    final (v, _, idx) = m;
    final slack = 6 / _s;
    for (var i = 0; i + 2 < idx.length; i += 3) {
      final tri = [v[idx[i]], v[idx[i + 1]], v[idx[i + 2]]];
      if (_inside(tri, comp)) return true;
      for (final p in tri) {
        if ((p - comp).distance <= slack) return true;
      }
    }
    return false;
  }

  Map<String, dynamic> _gesture(String phase, Offset point) => {
        'phase': phase,
        'view': widget.view,
        'mode': switch (_handle) { 'spatial' => 'spatial', 'body' => 'move', 'rotation' => 'rotate', _ => 'scale' },
        'ids': _ids,
        'start': [_start!.dx, _start!.dy],
        'point': [point.dx, point.dy],
        'handle': _handle,
        'viewScale': _s,
        'shift': HardwareKeyboard.instance.isShiftPressed,
        'alt': HardwareKeyboard.instance.isAltPressed,
        'snap': HardwareKeyboard.instance.isMetaPressed,
      };

  Future<void> _down(Offset screen) async {
    if (!c.supports('stageGesture')) return;
    final p = _comp(screen);
    String? handle;
    for (final e in _handles().entries) {
      if ((e.value - screen).distance <= 7) handle = e.key;
    }
    final active = c.activeLayer;
    if (handle == null && active != null && _onMesh(p)) handle = 'spatial';
    var ids = [...c.selectedIds];
    if (handle == null) {
      final hit = [for (final l in c.layers) if (_grabbable(l) && _inside(_face(l), p)) l].firstOrNull;
      if (hit == null) {
        c.command('select', {'ids': <int>[], 'keys': []});
        return;
      }
      final id = hit['id'] as int;
      if (HardwareKeyboard.instance.isShiftPressed) {
        ids = ids.contains(id) ? [for (final i in ids) if (i != id) i] : [...ids, id];
        c.command('select', {'ids': ids});
        return;
      }
      if (!ids.contains(id)) {
        ids = [id];
        await c.command('select', {'ids': ids});
      }
      handle = 'body';
    }
    _start = p;
    _handle = handle;
    _ids = ids;
    _drained = c.command('stageGesture', _gesture('begin', p));
  }

  void _move(Offset screen) {
    if (_start == null) return;
    _pending = _gesture('update', _comp(screen));
    if (_sending) return;
    _sending = true;
    final before = _drained;
    _drained = () async {
      try {
        await before;
        while (_pending != null) {
          final args = _pending!;
          _pending = null;
          await c.command('stageGesture', args);
        }
      } finally {
        _sending = false;
      }
    }();
  }

  Future<void> _up(Offset screen, {bool cancel = false}) async {
    if (_start == null) return;
    if (cancel) _pending = null;
    await _drained;
    await c.command('stageGesture', _gesture(cancel ? 'cancel' : 'commit', _comp(screen)));
    _start = null;
    _handle = null;
  }

  Offset _last = Offset.zero;

  @override
  Widget build(BuildContext context) => Listener(
        behavior: HitTestBehavior.opaque,
        onPointerDown: (e) {
          if (e.buttons != kPrimaryButton) return;
          _last = e.localPosition;
          _down(e.localPosition);
        },
        onPointerMove: (e) {
          _last = e.localPosition;
          _move(e.localPosition);
        },
        onPointerUp: (_) => _up(_last),
        onPointerCancel: (_) => _up(_last, cancel: true),
        child: CustomPaint(painter: _Overlay(c.activeLayer == null ? const [] : [for (final p in _face(c.activeLayer!)) _screen(p)], _handles(), _mesh(), _s), size: Size.infinite),
      );
}

/// The selected layer's frame and handles, and a 3D layer's gizmo as the host sent it.
class _Overlay extends CustomPainter {
  _Overlay(this.frame, this.handles, this.mesh, this.scale);
  final List<Offset> frame;
  final Map<String, Offset> handles;
  final (List<Offset>, List<Color>, List<int>)? mesh;
  final double scale;

  @override
  void paint(Canvas cv, Size s) {
    final m = mesh;
    if (m != null) {
      final (v, col, idx) = m;
      if (v.length == col.length) {
        cv.drawVertices(ui.Vertices(ui.VertexMode.triangles, [for (final p in v) p * scale], colors: col, indices: idx), BlendMode.srcOver, Paint());
      }
      return;
    }
    if (frame.length == 4) {
      cv.drawPath(Path()..addPolygon(frame, true), Paint()..color = H.playhead..style = PaintingStyle.stroke..strokeWidth = 1);
    }
    for (final e in handles.entries) {
      final r = Rect.fromCenter(center: e.value, width: 7, height: 7);
      if (e.key == 'rotation') {
        cv.drawCircle(e.value, 4, Paint()..color = H.text);
        cv.drawCircle(e.value, 4, Paint()..color = H.playhead..style = PaintingStyle.stroke..strokeWidth = 1);
      } else {
        cv.drawRect(r, Paint()..color = H.text);
        cv.drawRect(r, Paint()..color = H.playhead..style = PaintingStyle.stroke..strokeWidth = 1);
      }
    }
  }

  @override
  bool shouldRepaint(_Overlay o) => true;
}
