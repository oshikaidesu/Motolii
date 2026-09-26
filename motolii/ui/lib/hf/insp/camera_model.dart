// A camera layer's rig, as the native side defines it (traced from code, not from labels):
// the target is one 3D point (Center X, Center Y, Target Z), or a layer's origin when Target is set (the layer wins);
// the eye sits on a sphere around it: Orbit (pitch, yaw) gives the direction, Distance the length (a multiple of the
// distance at which the frame height fills a 55 degree view); Roll turns the picture about that ray; Zoom is the lens only.
// Distance and Zoom meet in one number at the target plane: magnification = Zoom / Distance.
import 'dart:math' as math;
import 'rows.dart';

class CamLayer {
  const CamLayer(this.id, this.name, this.x, this.y, this.z);
  final int id;
  final String name;
  final double x, y, z; // the layer's world origin, composition pixels
}

class CameraStore extends ParamStore {
  CameraStore({this.layers = const [], int target = 0, bool frozen = false}) : super(const [], frozen: frozen) {
    rows.addAll([
      {'id': 'camera.center', 'label': 'Center', 'kind': 'vec2', 'value': [0.0, 0.0], 'default': [0.0, 0.0], 'unit': 'px', 'speed': 1.0},
      {'id': 'camera.target.z', 'label': 'Target Z', 'kind': 'f32', 'value': 0.0, 'default': 0.0, 'unit': 'px', 'speed': 1.0},
      {'id': 'camera.target', 'label': 'Target', 'kind': 'enum', 'choices': ['None', for (final l in layers) l.name], 'value': target, 'default': 0},
      {'id': 'camera.orbit', 'label': 'Orbit', 'kind': 'vec2', 'value': [0.0, 0.0], 'default': [0.0, 0.0], 'speed': 1.0},
      {'id': 'camera.distance', 'label': 'Distance', 'kind': 'f32', 'value': 1.0, 'default': 1.0, 'min': .01, 'max': 100.0},
      {'id': 'camera.zoom', 'label': 'Zoom', 'kind': 'f32', 'value': 1.0, 'default': 1.0, 'min': .01, 'max': 100.0},
      {'id': 'camera.roll', 'label': 'Roll', 'kind': 'f32', 'value': 0.0, 'default': 0.0, 'subtype': 'ANGLE'},
    ]);
    _derive();
  }

  final List<CamLayer> layers;
  static const compW = 1920.0, compH = 1080.0;
  final animated = <String>{}, keyedNow = <String>{};
  // what the document stores for Center and Target Z; a Target layer overrides them for display and drawing, never overwrites them
  List<double> authoredCenter = [0, 0];
  double authoredZ = 0;

  CamLayer? get targetLayer {
    final i = ((row('camera.target')['value'] as num).round()) - 1;
    return i >= 0 && i < layers.length ? layers[i] : null;
  }

  /// With a Target layer set the layer decides the point; Center and Target Z show it and are not written.
  bool get targetLocked => targetLayer != null;

  void _derive() {
    final l = targetLayer;
    row('camera.center')['value'] = l != null ? [l.x - compW / 2, l.y - compH / 2] : List<double>.of(authoredCenter);
    row('camera.target.z')['value'] = l != null ? l.z : authoredZ;
    for (final r in rows) {
      r['animated'] = animated.contains(r['id']) ? true : null;
      r['keyedNow'] = keyedNow.contains(r['id']) ? true : null;
    }
  }

  @override
  void preview(String id, Object? v) {
    if (frozen) return;
    if (targetLocked && (id == 'camera.center' || id == 'camera.target.z')) return;
    if (id == 'camera.distance' || id == 'camera.zoom') v = math.max(.01, math.min(100.0, (v as num).toDouble()));
    super.preview(id, v);
    if (id == 'camera.center') authoredCenter = [for (final e in (row(id)['value'] as List)) (e as num).toDouble()];
    if (id == 'camera.target.z') authoredZ = (row(id)['value'] as num).toDouble();
    _derive();
    notifyListeners();
  }

  @override
  void commit(String id) {
    if (frozen) return;
    super.commit(id);
  }

  @override
  void set(String id, Object? v) {
    if (id == 'camera.target') {
      if (frozen) return;
      row(id)['value'] = v;
      _derive();
      commits++;
      notifyListeners();
      return;
    }
    super.set(id, v);
  }

  void toggleKey(String id) {
    if (frozen) return;
    if (keyedNow.contains(id)) { keyedNow.remove(id); } else { animated.add(id); keyedNow.add(id); }
    _derive();
    notifyListeners();
  }

  // ---- the numbers, as the geometry sees them ----------------------------------------------------------------
  double get pitch => ((row('camera.orbit')['value'] as List)[0] as num).toDouble();
  double get yaw => ((row('camera.orbit')['value'] as List)[1] as num).toDouble();
  double get distance => (row('camera.distance')['value'] as num).toDouble();
  double get zoom => (row('camera.zoom')['value'] as num).toDouble();
  double get roll => (row('camera.roll')['value'] as num).toDouble();
  double get cx => ((row('camera.center')['value'] as List)[0] as num).toDouble();
  double get cy => ((row('camera.center')['value'] as List)[1] as num).toDouble();

  /// On-axis magnification at the target plane: the one number Distance and Zoom both move.
  double get magnification => zoom / distance;

  /// The eye's direction from the target (unit): Ry(yaw) * Rx(pitch) * (0, 0, -1).
  List<double> get eyeDirection {
    final p = pitch * math.pi / 180, y = yaw * math.pi / 180;
    return [-math.cos(p) * math.sin(y), math.sin(p), -math.cos(p) * math.cos(y)];
  }
}
