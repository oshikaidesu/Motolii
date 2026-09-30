part of 'editor_session.dart';

/// snapshot を配る: taking a reply into the document, handing each panel the
/// slice it listens to, and the reading of the layers as of the drawn frame.
mixin SessionSnapshot on SessionCore {
  final _slices = <String, DocumentSlice>{};
  Map<String, dynamic> _spread = const {};

  /// The status a panel reads, as its own listenable. Named so the panels that
  /// share a reading share one slice.
  DocumentSlice slice(
    String name,
    List<String> keys, {
    Object? Function()? derived,
  }) => _slices[name] ??= DocumentSlice._(this, keys.toSet(), derived, name);

  /// What a layer *is*, without the values a hand is moving: names, kinds, timing, flags, which properties and effects it
  /// has and where their keys sit. A preview of Position, an effect parameter or a colour changes values only, so this
  /// stays equal, and a panel that lays out layers (the Timeline rows, which Inspector is shown) listens to it instead of
  /// to `layers`. A panel that shows a value listens to `layers`.
  List<Object?> layerShape() {
    final source = state['layers'];
    if (identical(source, _shapeSource) && _shapeCache != null) return _shapeCache!;
    _shapeSource = source;
    Map<String, dynamic> property(Map<String, dynamic> p) => {
          for (final e in p.entries)
            if (e.key != 'value' && e.key != 'keys') e.key: e.value,
          'keys': [
            for (final k in EditorSession.maps(p['keys'])) {'frame': k['frame'], 'interp': k['interp']},
          ],
        };
    const values = {'properties', 'effects', 'x', 'y', 'bounds', 'stageBounds', 'corners', 'anchorFraction'};
    return _shapeCache = [
      for (final l in EditorSession.maps(source))
        {
          for (final e in l.entries)
            if (!values.contains(e.key)) e.key: e.value,
          'properties': [for (final p in EditorSession.maps(l['properties'])) property(p)],
          'effects': [
            for (final fx in EditorSession.maps(l['effects']))
              {
                for (final e in fx.entries)
                  if (e.key != 'params') e.key: e.value,
                'params': [for (final p in EditorSession.maps(fx['params'])) property(p)],
              },
          ],
        },
    ];
  }

  Object? _shapeSource;
  List<Object?>? _shapeCache;

  void _spreadDocument() {
    final was = _spread, now = state;
    _spread = now;
    if (_slices.isEmpty) return;
    final changed = <String>{};
    for (final key in was.keys) {
      if (!now.containsKey(key) || !sameValue(was[key], now[key]))
        changed.add(key);
    }
    for (final key in now.keys) {
      if (!was.containsKey(key)) changed.add(key);
    }
    for (final slice in _slices.values.toList()) {
      slice._consider(changed);
    }
  }

  void absorb(Map<String, dynamic> next) {
    rememberKeySelection(next);
    EditorSession.take(document, next);
  }

  /// Whether the last rendered frame describes the current frame of this document.
  bool get renderedIsFresh {
    final r = rendered.value;
    return r.isNotEmpty &&
        r['frame'] == frame.value &&
        (r['contentRevision'] == null ||
            state['contentRevision'] == null ||
            r['contentRevision'] == state['contentRevision']) &&
        (r['documentRevision'] == null ||
            r['documentRevision'] == state['documentRevision']);
  }

  /// Layers as of the last rendered frame. A playback frame carries only the
  /// values that move (liveLayers); they are laid over the document layers by id.
  List<Map<String, dynamic>> liveLayers() {
    if (!renderedIsFresh) return layers;
    final r = rendered.value, s = state;
    if (identical(r, _liveFromRendered) && identical(s, _liveFromState))
      return _liveCache!;
    _liveFromRendered = r;
    _liveFromState = s;
    if (r['layers'] is List)
      return _liveCache = EditorSession.maps(r['layers']);
    final live = {
      for (final l in EditorSession.maps(r['liveLayers'])) l['id']: l,
    };
    return _liveCache = [
      for (final l in layers)
        if (live[l['id']] case final o?) overlayLayer(l, o) else l,
    ];
  }

  /// One overlay per (document, frame): Stage and Inspector read the same list.
  List<Map<String, dynamic>>? _liveCache;
  Map<String, dynamic>? _liveFromRendered, _liveFromState;

  static Map<String, dynamic> overlayLayer(
    Map<String, dynamic> base,
    Map<String, dynamic> live,
  ) {
    final out = {...base, ...live};
    out['properties'] = _overlayRows(
      EditorSession.maps(base['properties']),
      EditorSession.maps(live['properties']),
    );
    final liveEffects = {
      for (final e in EditorSession.maps(live['effects'])) e['id']: e,
    };
    out['effects'] = [
      for (final e in EditorSession.maps(base['effects']))
        if (liveEffects[e['id']] case final o?)
          {
            ...e,
            'params': _overlayRows(
              EditorSession.maps(e['params']),
              EditorSession.maps(o['params']),
            ),
          }
        else
          e,
    ];
    return out;
  }

  static List<Map<String, dynamic>> _overlayRows(
    List<Map<String, dynamic>> base,
    List<Map<String, dynamic>> live,
  ) {
    final byId = {for (final r in live) r['id']: r};
    return [
      for (final r in base)
        if (byId[r['id']] case final o?) {...r, ...o} else r,
    ];
  }
}
