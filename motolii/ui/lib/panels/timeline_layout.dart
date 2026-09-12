part of 'timeline.dart';

class _TrackRow {
  _TrackRow(this.layer, [this.property]);
  final Map<String, dynamic> layer;
  final Map<String, dynamic>? property;
  Rect bounds = Rect.zero;
  Rect get disclosureBounds =>
      Rect.fromLTWH(bounds.left, bounds.top, 20, bounds.height);
  int depth = 0;
  List<int> ancestors = [];
  List<Map<String, dynamic>> descendants = [];
  bool groupOpen = true;
  bool lanesOpen = false;
  bool get isGroup => layer['kind'] == 'Group';
  int get id => (layer['id'] as num).toInt();
  String get laneId =>
      '$id:${property == null ? 'layer' : property!['id'] ?? 'content'}';
  List<Map<String, dynamic>> get keys => EditorSession.maps(property?['keys']);

  /// Folded layer row: every key of the layer (properties, content, effect
  /// params) as selection entries, so the bar can show and move them like AE.
  List<Map<String, dynamic>> get allKeys {
    if (property != null) return const [];
    final out = <Map<String, dynamic>>[];
    void add(String? prop, dynamic keys) {
      for (final k in EditorSession.maps(keys))
        out.add({'layer': id, 'property': prop, 'frame': k['frame']});
    }

    for (final p in EditorSession.maps(layer['properties']))
      add(p['id'] as String?, p['keys']);
    if (layer['contentKeys'] is List) add('content', layer['contentKeys']);
    for (final e in EditorSession.maps(layer['effects']))
      for (final p in EditorSession.maps(e['params']))
        add(p['id'] as String?, p['keys']);
    return out;
  }

  // Rows are rebuilt whenever the document moves, so once per row is enough.
  late final List<int> summaryFrames =
      allKeys.map((k) => (k['frame'] as num).toInt()).toSet().toList()..sort();
}

class _LaneContainer {
  _LaneContainer(this.row, this.children);
  final _TrackRow row;
  final List<_LaneContainer> children;
  Rect bounds = Rect.zero;
  void place(double left, double top, double nameWidth, List<_TrackRow> rows) {
    row.bounds = Rect.fromLTWH(left, top, math.max(0, nameWidth - left), 20);
    rows.add(row);
    var bottom = top + 20;
    for (final child in children) {
      child.place(left + 8, bottom, nameWidth, rows);
      bottom = child.bounds.bottom;
    }
    bounds = Rect.fromLTWH(
      left,
      top,
      math.max(0, nameWidth - left),
      bottom - top,
    );
  }
}

class _LaneLayout {
  _LaneLayout(
    List<Map<String, dynamic>> layers,
    Set<int> expanded,
    Set<int> all,
    Set<int> collapsed, {
    double baseNameWidth = 138,
  }) {
    final byId = {
      for (final layer in layers) (layer['id'] as num).toInt(): layer,
    };
    final children = <int, List<Map<String, dynamic>>>{};
    for (final layer in layers) {
      final parent = layer['parent'];
      if (parent is int && byId[parent]?['kind'] == 'Group')
        children.putIfAbsent(parent, () => []).add(layer);
    }
    final visited = <int>{};
    List<Map<String, dynamic>> descendants(int id, Set<int> seen) {
      if (!seen.add(id)) return [];
      return [
        for (final child in children[id] ?? []) ...[
          child,
          ...descendants((child['id'] as num).toInt(), seen),
        ],
      ];
    }

    _LaneContainer? node(Map<String, dynamic> layer, List<int> ancestors) {
      final id = (layer['id'] as num).toInt();
      if (!visited.add(id)) return null;
      final row = _TrackRow(layer)
        ..ancestors = ancestors
        ..depth = ancestors.length
        ..groupOpen = !collapsed.contains(id)
        ..lanesOpen = expanded.contains(id);
      final nested = <_LaneContainer>[];
      if (row.isGroup) row.descendants = descendants(id, {});
      if (expanded.contains(id)) {
        final properties = EditorSession.maps(layer['properties']);
        if (layer['contentKeys'] is List &&
            !properties.any((p) => p['id'] == 'content'))
          properties.add({
            'id': 'content',
            'label': 'Content',
            'keys': layer['contentKeys'],
          });
        for (final property in properties) {
          if (all.contains(id) ||
              EditorSession.maps(property['keys']).isNotEmpty)
            nested.add(
              _LaneContainer(
                _TrackRow(layer, property)
                  ..ancestors = ancestors
                  ..depth = ancestors.length,
                [],
              ),
            );
        }
      }
      if (row.isGroup && row.groupOpen) {
        for (final child in children[id] ?? []) {
          final result = node(child, [...ancestors, id]);
          if (result != null) nested.add(result);
        }
      }
      return _LaneContainer(row, nested);
    }

    for (final layer in layers) {
      if (byId[layer['parent']]?['kind'] == 'Group') continue;
      final result = node(layer, []);
      if (result != null) roots.add(result);
    }
    int depth(_LaneContainer node) => node.children.isEmpty
        ? 0
        : 1 + node.children.map(depth).reduce(math.max);
    indentation = (roots.isEmpty ? 0 : roots.map(depth).reduce(math.max)) * 8.0;
    nameWidth = (baseNameWidth + indentation).clamp(114.0, 162.0 + indentation);
    var top = 0.0;
    for (final root in roots) {
      root.place(0, top, nameWidth, rows);
      top = root.bounds.bottom;
    }
    height = top;
  }
  final roots = <_LaneContainer>[];
  final rows = <_TrackRow>[];
  double height = 0;
  double nameWidth = 138;
  double indentation = 0;
  _LaneContainer? container(int id) {
    _LaneContainer? find(_LaneContainer node) {
      if (node.row.property == null && node.row.id == id) return node;
      for (final child in node.children) {
        final match = find(child);
        if (match != null) return match;
      }
      return null;
    }

    for (final root in roots) {
      final match = find(root);
      if (match != null) return match;
    }
    return null;
  }

  int rowAt(double y) =>
      rows.indexWhere((row) => y >= row.bounds.top && y < row.bounds.bottom);
}
