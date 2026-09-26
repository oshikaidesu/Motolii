
import '../../../hf/insp/transform_model.dart';
import '../../../session/editor_session.dart';
import '../../../session/read_model.dart';

/// The Transform Instrument's store over the real session: it reads the layers' rows the way the Classic Inspector
/// does and writes through the same operations (previewProperties, commitPreview, toggleKey, anchor, setAttrs).
/// The Instrument's own rules (a drag is relative across the selection, a typed number is absolute, locked layers
/// refuse) stay in [TransformStore]; this class only connects it to the document.
class SessionTransformStore extends TransformStore {
  SessionTransformStore(this.c)
      : super(_read(c), selection: _selection(c), active: _active(c)) {
    animating = c.animating;
    anchorPreview.addListener(_anchorHover);
  }

  final EditorSession c;

  static List<TLayer> _read(EditorSession c) => [for (final l in c.liveLayers()) layerOf(l)];
  static List<int> _selection(EditorSession c) {
    final ids = c.selectedIds;
    return ids.isEmpty ? [_active(c)] : ids;
  }

  static int _active(EditorSession c) => (c.activeLayer?['id'] as int?) ?? (c.layers.first['id'] as int);

  /// One layer as the Instrument models it.
  static TLayer layerOf(Map<String, dynamic> l) {
    final rows = {for (final r in panelRows(l['properties'])) '${r['id']}': r};
    double number(String id, double fallback) => (rows[id]?['value'] as num?)?.toDouble() ?? fallback;
    List<double> pair(String id, List<double> fallback) {
      final v = rows[id]?['value'];
      return v is List && v.length >= 2 ? [(v[0] as num).toDouble(), (v[1] as num).toDouble()] : fallback;
    }

    final position = pair('position', [0, 0]);
    final scale = pair('scale', [1, 1]);
    final anchor = l['anchorFraction'];
    final layer = TLayer(
      l['id'] as int,
      '${l['name'] ?? 'Layer'}',
      position: [position[0], position[1], number('position.z', 0)],
      scale: [scale[0], scale[1], number('scale.z', 1)],
      rotation: number('rotation', 0),
      rotX: number('rotation.x', 0),
      rotY: number('rotation.y', 0),
      depth: number('depth', 0),
      opacity: number('opacity', 1),
      anchor: anchor is List && anchor.length >= 2 ? [(anchor[0] as num).toDouble(), (anchor[1] as num).toDouble()] : null,
      projection: '${l['projection'] ?? '2D'}',
      parent: l['parent'] as int?,
      locked: l['locked'] == true,
    );
    for (final row in rows.values) {
      if ((row['keys'] as List?)?.isNotEmpty ?? false) layer.animated.add('${row['id']}');
      if (row['keyedNow'] == true) layer.keyedNow.add('${row['id']}');
    }
    return layer;
  }

  bool _gesture = false;

  /// The document changed under the Instrument (another edit, undo, a new selection). While a gesture is running the
  /// Instrument owns its values, and the last commit reads the document again.
  void absorb() {
    if (_gesture || c.layers.isEmpty) return;
    layers
      ..clear()
      ..addAll(_read(c));
    selection = _selection(c);
    activeId = layers.any((l) => l.id == _active(c)) ? _active(c) : layers.first.id;
    animating = c.animating;
    resync();
  }

  // ---- writing: the operations Classic's Inspector uses --------------------------------------------------------
  @override
  void previewed(String id, Map<int, Object?> values) {
    _gesture = true;
    // One edit and how it spreads: the host puts it to the rest of the selection (a drag keeps each layer's own
    // offset, a typed number sets the axes that changed) and leaves locked layers alone.
    c.command('previewProperties', {
      'edits': [
        {'layer': activeId, 'property': id, 'value': values[activeId] ?? values.values.first, 'spread': typing ? 'typed' : 'offset'},
      ],
    });
  }

  @override
  void committed(String id) {
    _gesture = false;
    c.command('commitPreview');
  }

  @override
  void cancelledGesture(String id) {
    _gesture = false;
    c.command('cancelPreview');
  }

  @override
  void toggleKey(String id) {
    if (!canEdit || !c.supports('toggleKey')) return;
    c.command('toggleKey', {'layer': activeId, 'property': id});
  }

  @override
  void setAnimate(bool on) {
    c.setAnimate(on);
  }

  @override
  void setAnchor(double x, double y) {
    if (!canEdit || !c.supports('anchor')) return;
    c.command('anchor', {'layer': activeId, 'xFraction': x, 'yFraction': y});
  }

  @override
  void setProjection(String p) {
    if (!canEdit) return;
    c.command('setAttrs', {'layers': c.selectedIds.isEmpty ? [activeId] : c.selectedIds, 'patch': {'projection': p}});
  }

  @override
  void setParent(int? id) {
    if (!canEdit) return;
    c.command('setAttrs', {'layers': [activeId], 'patch': {'parent': id}});
  }

  /// A hand-over to a specialist (Depth): the desk drawer or panel of that name comes to the front.
  @override
  void route(String to, [String? from]) {
    super.route(to, from);
    c.placePanel(to, 'show');
  }

  void _anchorHover() => c.anchorPreview.value = anchorPreview.value;

  @override
  void dispose() {
    anchorPreview.removeListener(_anchorHover);
    c.anchorPreview.value = null;
    super.dispose();
  }
}
