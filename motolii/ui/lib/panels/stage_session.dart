import 'dart:math' as math;
import 'dart:ui' show Offset, Rect, Size;

import 'package:flutter/foundation.dart';

import '../session/editor_session.dart';
import 'stage/geometry.dart';

/// The Stage as a person works it, with no screen in it: where each view (the User stage, the Camera) is looking —
/// its zoom and the composition point at the middle of its tab — in composition coordinates. A skin brings its own
/// tab size and turns it into pixels; a skin can be thrown away or swapped and the view stays where it was.
class StageSession extends ChangeNotifier {
  StageSession._(this.c, this.view);
  static final _all = Expando<Map<String, StageSession>>();
  static StageSession of(EditorSession c, String view) => (_all[c] ??= {})[view] ??= StageSession._(c, view);

  final EditorSession c;

  /// 'User' (the Stage tab) or 'Camera'.
  final String view;

  double get width => (numOf(c.state['width'], 1920)).clamp(1, double.infinity).toDouble();
  double get height => (numOf(c.state['height'], 1080)).clamp(1, double.infinity).toDouble();

  // ---- where it looks ------------------------------------------------------------------------------------------------
  /// Pixels a composition pixel takes (null: fit the tab), and the composition point at the tab's middle (null: the
  /// composition's middle).
  double? zoom;
  Offset? centre;

  double scaleIn(Size tab) => zoom ?? math.min(math.max(1, tab.width - 32) / width, math.max(1, tab.height - 32) / height);
  Offset originIn(Size tab) => tab.center(Offset.zero) - (centre ?? Offset(width / 2, height / 2)) * scaleIn(tab);
  Offset toComp(Offset screen, Size tab) => (screen - originIn(tab)) / scaleIn(tab);
  Offset toScreen(Offset comp, Size tab) => originIn(tab) + comp * scaleIn(tab);

  void fit() {
    zoom = null;
    centre = null;
    notifyListeners();
    if (view == 'User' && c.supports('stageView')) c.command('stageView', {'fit': true});
  }

  void actual() {
    zoom = 1;
    centre = null;
    notifyListeners();
  }

  /// Zoom to [next] keeping the composition point under [anchor] (a point on the tab) where it is.
  void zoomAt(double next, Offset anchor, Size tab) {
    final point = toComp(anchor, tab);
    final scale = next.clamp(.02, 16.0).toDouble();
    zoom = scale;
    centre = point - (anchor - tab.center(Offset.zero)) / scale;
    notifyListeners();
  }

  /// The picture carried by [delta] on the tab (a hand-pan).
  void panBy(Offset delta, Size tab) {
    centre = (centre ?? Offset(width / 2, height / 2)) - delta / scaleIn(tab);
    notifyListeners();
  }

  // ---- what it reads -----------------------------------------------------------------------------------------------
  bool get userStage => view == 'User';

  /// The document with the rendered frame laid over it: one copy per (document, frame).
  Map<String, dynamic>? _stateCache, _stateFrom, _renderedFrom;
  Map<String, dynamic> get state {
    if (!c.renderedIsFresh) return c.state;
    final st = c.state, rendered = c.rendered.value;
    if (!identical(st, _stateFrom) || !identical(rendered, _renderedFrom)) {
      _stateFrom = st;
      _renderedFrom = rendered;
      _stateCache = {...st, ...rendered, 'layers': c.liveLayers()};
    }
    return _stateCache!;
  }

  List<Map<String, dynamic>>? _layersCache;
  Map<String, dynamic>? _layersFrom;
  List<Map<String, dynamic>> get layers {
    final st = state;
    if (!identical(st, _layersFrom)) {
      _layersFrom = st;
      _layersCache = EditorSession.maps(st['layers']);
    }
    return _layersCache!;
  }

  Map<String, dynamic>? get active {
    for (final layer in layers) {
      if (c.selectedIds.isNotEmpty && layer['id'] == c.selectedIds.last) return layer;
    }
    return null;
  }

  List<Map<String, dynamic>> get cameras => userStage ? EditorSession.maps(state['cameraGizmos']) : [];
  Map<String, dynamic> get observer => EditorSession.map(state['observer']);
  bool get front => !userStage || observer['front'] != false;
  bool get home => !userStage || observer['home'] != false;
  Map<String, dynamic>? get extent => userStage && observer['extent'] is Map ? EditorSession.map(observer['extent']) : null;
  bool get extending => c.deskWork.value['stageExtend'] == true;

  /// A layer's outline on this view, in composition coordinates.
  List<Offset> rawCorners(Map<String, dynamic> layer) {
    final values = EditorSession.map(layer[userStage ? 'stageBounds' : 'bounds'])['corners'] ?? layer['corners'];
    if (values is! List) return [];
    return [
      for (final v in values)
        if (v is List && v.length >= 2 ? Offset(numOf(v[0], double.nan), numOf(v[1], double.nan)) : v is Map ? Offset(numOf(v['x'], double.nan), numOf(v['y'], double.nan)) : null
            case final p? when p.dx.isFinite && p.dy.isFinite)
          p,
    ];
  }

  List<Offset> corners(Map<String, dynamic> layer) => hull(rawCorners(layer));
  List<Map<String, dynamic>> get visible => layers.where((l) => l['hidden'] != true).toList();

  /// What a hand can take on the picture: a Group is reached from the layer lists alone (2026-09-19, user).
  static bool grabbable(Map<String, dynamic> layer) => layer['locked'] != true && layer['kind'] != 'Group';

  /// The layer a press at [comp] takes.
  Map<String, dynamic>? layerAt(Offset comp) {
    for (final layer in visible) {
      if (grabbable(layer) && polygonContains(corners(layer), comp)) return layer;
    }
    return null;
  }

  /// A camera the box can author (front on, looking at its own target).
  Map<String, dynamic>? get selectedCamera {
    for (final camera in cameras) {
      if (c.selectedIds.contains(camera['id']) && camera['authorable'] != false) return camera;
    }
    return null;
  }

  /// A camera's box on the composition plane (all four corners, or none).
  List<Offset> cameraBox(Map<String, dynamic> camera) {
    final pts = [for (final p in (camera['points'] as List? ?? const [])) p is List && p.length >= 2 ? Offset(numOf(p[0]), numOf(p[1])) : null];
    return pts.length == 4 && pts.every((p) => p != null) ? pts.cast<Offset>() : const [];
  }

  // ---- the gesture in flight, in composition coordinates -----------------------------------------------------------
  StGesture? gesture;
  Offset? _start;
  List<int> _ids = [];
  String _mode = 'move', _handle = 'body';
  bool _additive = false, _finishing = false;

  /// P / R / S held (AE): the gizmo narrows to position, rotation or scale while the key is down.
  String? held;

  /// The marquee, in composition coordinates.
  Rect? marquee;

  bool get busy => gesture != null || _finishing;

  /// A press on [target] at [comp].
  void press(StTarget target, Offset comp, StMods mods) {
    if (busy) return;
    _start = comp;
    _additive = mods.add;
    switch (target) {
      case StPick():
        _pickColor(comp);
      case StFocus(:final layer):
        c.command('stageView', layer == null ? {'reset': true} : {'focus': layer});
      case StOrbit():
        gesture = StGesture.orbit;
      case StPan():
        gesture = StGesture.pan;
      case StExtentEdge(:final side):
        _carry(StGesture.extent, 'extent', const [], const ['top', 'right', 'bottom', 'left'][side], comp, mods);
      case StCameraHandle(:final camera, :final handle):
        if (handle == 'center') c.command('select', {'ids': [camera['id']]});
        _carry(StGesture.camera, 'camera', [(camera['id'] as num).toInt()], handle, comp, mods);
      case StHandle(:final handle):
        _begin(comp, c.selectedIds, handle, mods);
      case StLayer(:final id):
        final ids = List<int>.of(c.selectedIds);
        if (_additive) {
          ids.contains(id) ? ids.remove(id) : ids.add(id);
        } else if (!ids.contains(id)) {
          ids
            ..clear()
            ..add(id);
        }
        // what the drag guard expects; the host decides what is chosen
        _ids = ids;
        if (c.supports('select')) c.command('select', _additive ? {'toggle': id} : {'ids': ids});
        // a click selects; a drag begins once the pointer leaves the skin's slop
        _handle = 'body';
        gesture = StGesture.pending;
      case StEmpty():
        _ids = _additive ? List.of(c.selectedIds) : [];
        marquee = Rect.fromPoints(comp, comp);
        gesture = StGesture.marquee;
    }
    notifyListeners();
  }

  /// The pointer moved with the press held; [beyondSlop]: the skin says it has left its own dead zone.
  void drag(Offset comp, {required bool beyondSlop, required StMods mods, required double viewScale}) {
    switch (gesture) {
      case StGesture.marquee:
        marquee = Rect.fromPoints(_start!, comp);
        notifyListeners();
      case StGesture.pending when beyondSlop && _ids.isNotEmpty && c.supports('stageGesture'):
        _begin(_start!, _ids, 'body', mods, viewScale: viewScale);
        _update(comp, mods, viewScale);
      case StGesture.layer || StGesture.camera || StGesture.extent:
        _update(comp, mods, viewScale);
      default:
        break;
    }
  }

  /// The observer turned by [pitch] and [yaw] degrees (the skin decides how far a pixel turns it).
  List<double>? _orbit;
  void orbitBy(double pitch, double yaw) {
    final base = _orbit ?? (observer['orbit'] as List?)?.map((v) => (v as num).toDouble()).toList() ?? [0, 0];
    _orbit = [(base[0] + pitch).clamp(-85.0, 85.0), base[1] + yaw];
    _orbitPending = List.of(_orbit!);
    _sendOrbit();
  }

  void release(Offset comp, StMods mods, double viewScale) {
    switch (gesture) {
      case StGesture.orbit:
        _orbit = null;
        gesture = null;
      case StGesture.pan:
        gesture = null;
      case StGesture.marquee:
        final box = marquee!;
        if (c.supports('select')) {
          final hits = visible.where((l) => l['locked'] != true && polygonIntersects(corners(l), box)).map((l) => (l['id'] as num).toInt());
          c.command('select', {'ids': {..._ids, ...hits}.toList()});
        }
        _finish(false, comp, mods, viewScale);
      default:
        _finish(false, comp, mods, viewScale);
    }
    notifyListeners();
  }

  void cancel(Offset comp, StMods mods, double viewScale) {
    _finish(true, comp, mods, viewScale);
  }

  // ---- the layer gesture, streamed to the host ---------------------------------------------------------------------
  Map<String, dynamic>? _pending;
  Future<void> _drained = Future<void>.value();
  bool _sending = false;

  void _begin(Offset comp, List<int> ids, String handle, StMods mods, {double viewScale = 1}) {
    gesture = StGesture.layer;
    _ids = List.of(ids);
    _handle = handle;
    _mode = handle == 'spatial' ? 'spatial' : handle == 'body' ? 'move' : handle == 'rotation' ? 'rotate' : 'scale';
    _drained = c.command('stageGesture', _gestureArgs('begin', comp, mods, viewScale));
  }

  /// A camera box handle or a working-area edge, carried by the host like any Stage gesture: the host turns the
  /// pointer into the camera's or the area's values. The view's scale is left as it is.
  void _carry(StGesture kind, String mode, List<int> ids, String handle, Offset comp, StMods mods) {
    if (!c.supports('stageGesture')) return;
    gesture = kind;
    _ids = ids;
    _handle = handle;
    _mode = mode;
    _drained = c.command('stageGesture', _gestureArgs('begin', comp, mods, null));
  }

  Map<String, dynamic> _gestureArgs(String phase, Offset point, StMods mods, double? viewScale) => {
        'phase': phase,
        'view': view,
        'mode': _mode,
        // a layer drag carries what the host has chosen; a camera box names its camera
        if (_mode == 'camera') 'ids': _ids,
        'start': [_start!.dx, _start!.dy],
        'point': [point.dx, point.dy],
        'handle': _handle,
        if (viewScale != null) 'viewScale': viewScale,
        'shift': mods.shift,
        'alt': mods.alt,
        // Cmd while moving: snap to other boxes' edges and centres (After Effects)
        'snap': mods.cmd,
        'held': held,
      };

  void _update(Offset comp, StMods mods, double viewScale) {
    _pending = _gestureArgs('update', comp, mods, viewScale);
    if (_sending) return;
    _sending = true;
    final previous = _drained;
    _drained = () async {
      try {
        await previous;
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

  Future<void> _finish(bool cancel, Offset comp, StMods mods, double viewScale) async {
    if (_finishing) return;
    _finishing = true;
    final wasDragging = gesture == StGesture.layer || gesture == StGesture.camera || gesture == StGesture.extent;
    if (cancel) _pending = null;
    try {
      if (wasDragging) {
        await _drained;
        await c.command('stageGesture', _gestureArgs(cancel ? 'cancel' : 'commit', comp, mods, viewScale));
      }
    } finally {
      _start = null;
      marquee = null;
      gesture = null;
      _finishing = false;
      notifyListeners();
    }
  }

  /// The 3D gizmo lights the part under the pointer: the host hears the pointer while it is on the mesh, and once when
  /// it leaves. One in flight at a time.
  Map<String, dynamic>? _hoverPending;
  bool _hoverSending = false;
  void hover(Offset? comp, double viewScale) {
    _hoverPending = {'phase': 'hover', 'view': view, 'point': comp == null ? null : [comp.dx, comp.dy], 'viewScale': viewScale, 'held': held};
    if (_hoverSending) return;
    _hoverSending = true;
    () async {
      try {
        while (_hoverPending != null) {
          final args = _hoverPending!;
          _hoverPending = null;
          await c.command('stageGesture', args);
        }
      } finally {
        _hoverSending = false;
      }
    }();
  }

  // ---- the observer's orbit ----------------------------------------------------------------------------------------
  List<double>? _orbitPending;
  bool _orbitSending = false;
  Future<void> _sendOrbit() async {
    if (_orbitSending) return;
    _orbitSending = true;
    try {
      while (_orbitPending != null) {
        final angles = _orbitPending!;
        _orbitPending = null;
        await c.command('stageView', {'orbit': angles});
      }
    } finally {
      _orbitSending = false;
    }
  }

  /// The working area switched on (a Stage layer is made when there is none) or off.
  Future<void> toggleExtend() async {
    final on = !extending;
    await c.storeDesk('stageExtend', on);
    if (on && extent == null && c.supports('create')) await c.command('create', {'kind': 'stage'});
  }

  // ---- the eyedropper ------------------------------------------------------------------------------------------------
  Future<void> _pickColor(Offset comp) async {
    c.eyedropper.value = false;
    if (!c.supports('pickColor')) return;
    // picked and applied by the host in one request: a pick that fails applies nothing
    await c.command('pickColor', {'x': comp.dx, 'y': comp.dy, 'apply': true});
  }
}

enum StGesture { pending, layer, marquee, camera, extent, orbit, pan }

/// What a skin found under a press on the Stage, in the Stage's own terms.
sealed class StTarget {
  const StTarget();
}

/// A handle of the selection's cage: nw ne se sw n e s w, or rotation — or the 3D gizmo ('spatial').
class StHandle extends StTarget {
  const StHandle(this.handle);
  final String handle;
}

class StLayer extends StTarget {
  const StLayer(this.id);
  final int id;
}

class StEmpty extends StTarget {
  const StEmpty();
}

/// A camera box's handle (zoom0..zoom3, roll) or its edge ('center').
class StCameraHandle extends StTarget {
  const StCameraHandle(this.camera, this.handle);
  final Map<String, dynamic> camera;
  final String handle;
}

/// A working-area edge: top 0, right 1, bottom 2, left 3.
class StExtentEdge extends StTarget {
  const StExtentEdge(this.side);
  final int side;
}

class StOrbit extends StTarget {
  const StOrbit();
}

class StPan extends StTarget {
  const StPan();
}

/// A double click: look at [layer] (null: back to front).
class StFocus extends StTarget {
  const StFocus(this.layer);
  final Object? layer;
}

/// The eyedropper's press.
class StPick extends StTarget {
  const StPick();
}

class StMods {
  const StMods({this.add = false, this.shift = false, this.alt = false, this.cmd = false});
  final bool add, shift, alt, cmd;
}
