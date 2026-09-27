import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../session/editor_session.dart';
import 'geometry.dart';

/// Timeline's presentation independent rows and their document-space bounds.
class TrackRow {
  TrackRow(this.layer, [this.property]);
  final Map<String, dynamic> layer;
  final Map<String, dynamic>? property;
  Rect bounds = Rect.zero;
  double disclosureWidth = TimelineGeometry.classic.disclosureWidth;
  Rect get disclosureBounds =>
      Rect.fromLTWH(bounds.left, bounds.top, disclosureWidth, bounds.height);
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

  /// Folded rows expose keys from properties, content, and effect parameters.
  late final List<Map<String, dynamic>> allKeys = _allKeys();
  List<Map<String, dynamic>> _allKeys() {
    if (property != null) return const [];
    final out = <Map<String, dynamic>>[];
    void add(String? prop, dynamic keys) {
      for (final key in EditorSession.maps(keys)) {
        out.add({'layer': id, 'property': prop, 'frame': key['frame']});
      }
    }

    for (final p in EditorSession.maps(layer['properties'])) {
      add(p['id'] as String?, p['keys']);
    }
    if (layer['contentKeys'] is List) add('content', layer['contentKeys']);
    for (final effect in EditorSession.maps(layer['effects'])) {
      for (final p in EditorSession.maps(effect['params'])) {
        add(p['id'] as String?, p['keys']);
      }
    }
    return out;
  }

  late final List<int> summaryFrames =
      allKeys.map((key) => (key['frame'] as num).toInt()).toSet().toList()
        ..sort();
}

class LaneContainer {
  LaneContainer(this.row, this.children);
  final TrackRow row;
  final List<LaneContainer> children;
  Rect bounds = Rect.zero;

  void place(
    double left,
    double top,
    double nameWidth,
    List<TrackRow> rows,
    TimelineGeometry geometry,
  ) {
    row.disclosureWidth = geometry.disclosureWidth;
    row.bounds = Rect.fromLTWH(
      left,
      top,
      math.max(0, nameWidth - left),
      geometry.rowHeight,
    );
    rows.add(row);
    var bottom = top + geometry.rowHeight;
    for (final child in children) {
      child.place(
        left + geometry.indentStep,
        bottom,
        nameWidth,
        rows,
        geometry,
      );
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

class LaneLayout {
  LaneLayout(
    List<Map<String, dynamic>> layers,
    Set<int> expanded,
    Set<int> all,
    Set<int> collapsed, {
    double? baseNameWidth,
    this.geometry = TimelineGeometry.classic,
  }) {
    final byId = {
      for (final layer in layers) (layer['id'] as num).toInt(): layer,
    };
    final children = <int, List<Map<String, dynamic>>>{};
    for (final layer in layers) {
      final parent = layer['parent'];
      if (parent is int && byId[parent]?['kind'] == 'Group') {
        children.putIfAbsent(parent, () => []).add(layer);
      }
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

    LaneContainer? node(Map<String, dynamic> layer, List<int> ancestors) {
      final id = (layer['id'] as num).toInt();
      if (!visited.add(id)) return null;
      final row = TrackRow(layer)
        ..ancestors = ancestors
        ..depth = ancestors.length
        ..groupOpen = !collapsed.contains(id)
        ..lanesOpen = expanded.contains(id);
      final nested = <LaneContainer>[];
      if (row.isGroup) row.descendants = descendants(id, {});
      if (expanded.contains(id)) {
        final properties = EditorSession.maps(layer['properties']).toList();
        if (layer['contentKeys'] is List &&
            !properties.any((property) => property['id'] == 'content')) {
          properties.add({
            'id': 'content',
            'label': 'Content',
            'keys': layer['contentKeys'],
          });
        }
        for (final effect in EditorSession.maps(layer['effects'])) {
          for (final parameter in EditorSession.maps(effect['params'])) {
            properties.add({
              'id': parameter['id'],
              'label':
                  '${effect['name'] ?? effect['id']} · ${parameter['label'] ?? parameter['id']}',
              'keys': parameter['keys'],
            });
          }
        }
        for (final property in properties) {
          if (all.contains(id) ||
              EditorSession.maps(property['keys']).isNotEmpty) {
            nested.add(
              LaneContainer(
                TrackRow(layer, property)
                  ..ancestors = ancestors
                  ..depth = ancestors.length,
                [],
              ),
            );
          }
        }
      }
      if (row.isGroup && row.groupOpen) {
        for (final child in children[id] ?? []) {
          final childNode = node(child, [...ancestors, id]);
          if (childNode != null) nested.add(childNode);
        }
      }
      return LaneContainer(row, nested);
    }

    for (final layer in layers) {
      if (byId[layer['parent']]?['kind'] == 'Group') continue;
      final root = node(layer, []);
      if (root != null) roots.add(root);
    }
    int depth(LaneContainer container) => container.children.isEmpty
        ? 0
        : 1 + container.children.map(depth).reduce(math.max);
    indentation =
        (roots.isEmpty ? 0 : roots.map(depth).reduce(math.max)) *
        geometry.indentStep;
    final base = baseNameWidth ?? geometry.nameWidth;
    nameWidth = (base + indentation)
        .clamp(geometry.nameWidthMin, geometry.nameWidthMax)
        .toDouble();
    var top = 0.0;
    for (final root in roots) {
      root.place(0, top, nameWidth, rows, geometry);
      top = root.bounds.bottom;
    }
    height = top;
  }

  final TimelineGeometry geometry;
  final roots = <LaneContainer>[];
  final rows = <TrackRow>[];
  double height = 0;
  double nameWidth = TimelineGeometry.classic.nameWidth;
  double indentation = 0;

  LaneContainer? container(int id) {
    LaneContainer? find(LaneContainer node) {
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
