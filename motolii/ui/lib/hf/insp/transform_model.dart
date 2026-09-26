// The Transform of a layer, as the Classic Inspector owns it: what it is, who else is selected, what a write does.
// Semantics kept from Classic: a drag is relative across the selection (each layer keeps its offset), a typed number is
// absolute; locked layers refuse edits; scale is linked by default; Space applies to the selection, Parent and Anchor to
// the shown layer; Depth and the Z axes exist only outside 2D; Animate turns touched values into keys.
import 'dart:math' as math;
import 'package:flutter/foundation.dart';
import 'rows.dart';

class TLayer {
  TLayer(this.id, this.name, {List<double>? position, List<double>? scale, this.rotation = 0, this.rotX = 0, this.rotY = 0, this.depth = 0, this.opacity = 1, List<double>? anchor, this.projection = '2D', this.parent, this.locked = false})
      : position = position ?? [0, 0, 0],
        scale = scale ?? [1, 1, 1],
        anchor = anchor ?? [.5, .5];
  final int id;
  String name;
  List<double> position, scale, anchor; // scale is a ratio: 1 is 100 %
  double rotation, rotX, rotY, depth, opacity;
  String projection;
  int? parent;
  bool locked;
  final animated = <String>{}, keyedNow = <String>{};
}

class TransformStore extends ParamStore {
  TransformStore(this.layers, {List<int>? selection, int? active})
      : selection = selection ?? [layers.first.id],
        activeId = active ?? (selection ?? [layers.first.id]).first,
        super(const []) {
    linked.add('scale'); // Keep the shape is the default
    _sync();
  }

  final List<TLayer> layers;
  List<int> selection;
  int activeId;
  bool animating = false;
  final anchorPreview = ValueNotifier<List<double>?>(null); // where the Stage would show the pivot if the pointer chose it
  final focusRequest = ValueNotifier<String?>(null);
  final _touched = <String>{};
  Map<int, Object?>? _bases; // each target's value where the gesture began
  String? _gestureId;

  TLayer get active => layers.firstWhere((l) => l.id == activeId);
  bool get multiple => selection.length > 1;
  bool get _outside2D => active.projection != '2D';
  bool get canEdit => !active.locked;

  void select(List<int> ids, {int? active}) {
    selection = ids;
    activeId = active ?? ids.first;
    _sync();
    notifyListeners();
  }

  /// Every selected, unlocked layer, or the shown one alone when it is not part of the selection.
  List<TLayer> get targets {
    final t = [for (final l in layers) if (selection.contains(l.id) && !l.locked) l];
    return t.any((l) => l.id == activeId) ? t : [active];
  }

  // ---- rows: what the shown layer declares -------------------------------------------------------------------
  void _sync() {
    final l = active;
    frozen = l.locked;
    Map<String, dynamic> k(String id, Map<String, dynamic> r) => {'id': id, ...r, if (l.animated.contains(id)) 'animated': true, if (l.keyedNow.contains(id)) 'keyedNow': true};
    rows
      ..clear()
      ..addAll([
        k('position', {'label': 'Position', 'kind': 'vec2', 'value': [l.position[0], l.position[1]], 'default': [0.0, 0.0], 'unit': 'px'}),
        if (_outside2D) k('position.z', {'label': 'Position Z', 'kind': 'f32', 'value': l.position[2], 'default': 0.0, 'unit': 'px'}),
        k('scale', {'label': 'Scale', 'kind': 'vec2', 'value': [l.scale[0], l.scale[1]], 'default': [1.0, 1.0], 'percent': true}),
        if (_outside2D) k('scale.z', {'label': 'Scale Z', 'kind': 'f32', 'value': l.scale[2], 'default': 1.0, 'percent': true}),
        k('rotation', {'label': 'Rotation', 'kind': 'f32', 'value': l.rotation, 'default': 0.0, 'subtype': 'ANGLE'}),
        if (_outside2D) k('rotation.x', {'label': 'Rotation X', 'kind': 'f32', 'value': l.rotX, 'default': 0.0, 'subtype': 'ANGLE'}),
        if (_outside2D) k('rotation.y', {'label': 'Rotation Y', 'kind': 'f32', 'value': l.rotY, 'default': 0.0, 'subtype': 'ANGLE'}),
        if (_outside2D) k('depth', {'label': 'Depth', 'kind': 'f32', 'value': l.depth, 'default': 0.0, 'unit': 'px', 'route': 'Depth'}),
        k('opacity', {'label': 'Opacity', 'kind': 'f32', 'value': l.opacity, 'default': 1.0, 'min': 0.0, 'max': 1.0, 'subtype': 'OPACITY'}),
      ]);
  }

  Object? _get(TLayer l, String id) => switch (id) {
        'position' => [l.position[0], l.position[1]],
        'position.z' => l.position[2],
        'scale' => [l.scale[0], l.scale[1]],
        'scale.z' => l.scale[2],
        'rotation' => l.rotation,
        'rotation.x' => l.rotX,
        'rotation.y' => l.rotY,
        'depth' => l.depth,
        'opacity' => l.opacity,
        _ => null,
      };

  void _put(TLayer l, String id, Object? v) {
    switch (id) {
      case 'position': l.position[0] = (v as List)[0] as double; l.position[1] = v[1] as double;
      case 'position.z': l.position[2] = v as double;
      case 'scale': l.scale[0] = (v as List)[0] as double; l.scale[1] = v[1] as double;
      case 'scale.z': l.scale[2] = v as double;
      case 'rotation': l.rotation = v as double;
      case 'rotation.x': l.rotX = v as double;
      case 'rotation.y': l.rotY = v as double;
      case 'depth': l.depth = v as double;
      case 'opacity': l.opacity = math.max(0, math.min(1, v as double));
    }
  }

  static Object? _copy(Object? v) => v is List ? List<double>.of(v.cast<double>()) : v;
  static List<double> _dbl(List v) => [for (final e in v) (e as num).toDouble()];

  // ---- writing ----------------------------------------------------------------------------------------------------
  @override
  void preview(String id, Object? v) {
    if (!canEdit || _get(active, id) == null) return;
    final next = v is List ? _dbl(v) : (v as num).toDouble();
    if (_bases == null || _gestureId != id) {
      _gestureId = id;
      _bases = {for (final l in targets) l.id: _copy(_get(l, id))};
    }
    final base = _bases![activeId];
    for (final l in targets) {
      final own = _bases![l.id];
      Object? value = next;
      if (typing && l.id != activeId && next is List<double> && base is List && _bases![l.id] is List) {
        // a typed number is absolute, but only for the axes it changed: the others keep their own values
        final own = _bases![l.id] as List;
        value = [for (var i = 0; i < next.length; i++) next[i] != (base[i] as double) ? next[i] : own[i] as double];
      }
      if (!typing && l.id != activeId) {
        // relative: this layer keeps its own offset from the shown one
        if (next is double && base is double && own is double) {
          value = own + (next - base);
        } else if (next is List<double> && base is List && own is List) {
          value = [for (var i = 0; i < next.length; i++) (own[i] as double) + (next[i] - (base[i] as double))];
        }
      }
      if (value is double && id == 'opacity') value = math.max(0, math.min(1, value));
      _put(l, id, value);
    }
    _touched.add(id);
    previews++;
    _sync();
    notifyListeners();
  }

  @override
  void commit(String id) {
    if (!canEdit) return;
    if (animating) {
      for (final l in targets) { l.animated.add(id); l.keyedNow.add(id); }
    }
    commits++;
    _bases = null;
    _gestureId = null;
    _touched.clear();
    _sync();
    notifyListeners();
  }

  @override
  void set(String id, Object? v) {
    if (id == 'projection' || id == 'parent') return; // attributes have their own entry points
    typing = true;
    try { super.set(id, v); } finally { typing = false; }
  }

  @override
  void reset(String id) {
    final r = row(id);
    if (!r.containsKey('default')) return;
    final d = r['default'];
    set(id, d is List ? List<double>.of(d.cast<double>()) : d);
  }

  @override
  bool mixed(String id, int? axis) {
    if (!multiple) return false;
    final mine = _get(active, id);
    for (final l in layers) {
      if (l.id == activeId || !selection.contains(l.id) || l.locked) continue;
      final o = _get(l, id);
      if (mine is List && o is List) {
        if (axis == null ? !listEquals(mine, o) : mine[axis] != o[axis]) return true;
      } else if (o != mine) {
        return true;
      }
    }
    return false;
  }

  /// Key this frame / remove the key, on the shown layer (as Classic's key lamp does).
  void toggleKey(String id) {
    if (!canEdit) return;
    final l = active;
    if (l.keyedNow.contains(id)) { l.keyedNow.remove(id); } else { l.animated.add(id); l.keyedNow.add(id); }
    _sync();
    notifyListeners();
  }

  void setAnimate(bool on) {
    animating = on;
    notifyListeners();
  }

  /// Anchor: the shown layer alone, one of nine places. Locked layers refuse.
  void setAnchor(double x, double y) {
    if (!canEdit) return;
    active.anchor = [x, y];
    commits++;
    _sync();
    notifyListeners();
  }

  /// Space applies to every selected layer.
  void setProjection(String p) {
    if (!canEdit) return;
    for (final l in layers) { if (selection.contains(l.id)) l.projection = p; }
    commits++;
    _sync();
    notifyListeners();
  }

  /// Parent: the shown layer alone.
  void setParent(int? id) {
    if (!canEdit) return;
    active.parent = id;
    commits++;
    notifyListeners();
  }

  /// Ask the Instrument to reveal a property: show the mode that owns it and focus its number.
  void focusProperty(String id) => focusRequest.value = id;

  // Scale hand-over: linked axes keep their ratio, exactly as typed or scrubbed numbers do.
  bool get scaleEven => (active.scale[0] - active.scale[1]).abs() < 1e-9;
}
