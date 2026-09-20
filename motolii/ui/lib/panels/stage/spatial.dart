part of '../stage.dart';

/// Whether a press lands on the 3D gizmo. Native hands over the very
/// triangles it hit-tests against, so grabbing and drawing cannot drift;
/// this mixin only parses them and asks "is the point on one of these?".
mixin _StageSpatial on State<StagePanel>, _StageView {
  /// 3D layer gizmo: the native side hands over the very triangles it hit-tests
  /// against, in comp coordinates. Grabbing and drawing therefore cannot drift.
  /// 2D and 2.5D layers never reach here; their cage lives in [_handles].
  /// Native sends the same map object until the gizmo changes, so identity
  /// is the cache key: the mesh is parsed once per status, not once per build.
  Object? _meshRaw;
  EditorTheme? _meshTheme;
  SpatialMesh? _meshParsed;
  SpatialMesh? get _mesh {
    final raw = _state[_userStage ? 'stageSpatialGizmo' : 'spatialGizmo'];
    final colors = EditorTheme.of(context);
    if (!identical(raw, _meshRaw) || colors != _meshTheme) {
      _meshTheme = colors;
      _meshRaw = raw;
      _meshParsed = SpatialMesh.parse(raw, colors);
      _screenMeshKey = null;
    }
    return _meshParsed;
  }

  /// The mesh on screen, rebuilt only when the mesh or the view moves.
  (Object, Offset, double)? _screenMeshKey;
  ui.Vertices? _screenMeshCache;
  ui.Vertices? _screenMesh() {
    final mesh = _mesh;
    if (mesh == null) return null;
    final key = (mesh, _origin, _scale);
    if (_screenMeshKey != key) {
      _screenMeshKey = key;
      _screenMeshCache = mesh.toScreen(_origin, _scale);
    }
    return _screenMeshCache;
  }

  /// Only a 3D layer carries the three-axis gizmo. 2D and 2.5D layers keep the
  /// planar cage and are never reached from here.
  bool get _spatialActive {
    final layer = _active;
    if (layer == null || layer['locked'] == true) return false;
    return ['3D', 'ThreeD'].contains('${layer['projection']}');
  }

  /// What you see is what you grab: the press counts as a gizmo grab only when
  /// it lands on a drawn triangle. The native side then picks the axis.
  bool _spatialHit(Offset screen) {
    if (!_spatialActive) return false;
    final mesh = _mesh;
    if (mesh == null) return false;
    final vertices = mesh.vertices;
    final indices = mesh.indices;
    final p = _toComp(screen);
    final slack = 6 / _scale;
    for (var i = 0; i + 2 < indices.length; i += 3) {
      final a = vertices[indices[i]],
          b = vertices[indices[i + 1]],
          v = vertices[indices[i + 2]];
      if (!a.dx.isFinite || !b.dx.isFinite || !v.dx.isFinite) continue;
      if (inTriangle(p, a, b, v)) return true;
      if (segmentDistance(p, a, b) < slack ||
          segmentDistance(p, b, v) < slack ||
          segmentDistance(p, v, a) < slack)
        return true;
    }
    return false;
  }
}
