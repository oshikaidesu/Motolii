import 'dart:convert';

import 'package:flutter/widgets.dart';

import '../../hf/insp/transform_model.dart';
import '../../session/editor_session.dart';
import '../../session/read_model.dart';
import 'key_menu.dart';

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

  static List<TLayer> _read(EditorSession c) => _readAll(c);
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
      kind: '${l['kind'] ?? ''}',
      blendMode: '${l['blendMode'] ?? 'Normal'}',
      environment: l['environment'] == true,
      ghostable: l['ghostable'] == true,
      ghost: (l['ghost'] as num?)?.toInt(),
      clipToBelow: l['clipToBelow'] == true,
    );
    for (final row in rows.values) {
      if ((row['keys'] as List?)?.isNotEmpty ?? false) layer.animated.add('${row['id']}');
      if (row['keyedNow'] == true) layer.keyedNow.add('${row['id']}');
      if (row['link'] is Map) layer.links['${row['id']}'] = Map<String, dynamic>.from(row['link'] as Map);
    }
    return layer;
  }

  /// How many rows, over every layer, a row of [layer] drives: the links that point at it, by property.
  static Map<String, int> drivesOf(EditorSession c, int layerId) {
    final out = <String, int>{};
    for (final l in c.layers) {
      for (final r in panelRows(l['properties'])) {
        final link = r['link'];
        if (link is Map && link['layer'] == layerId) out['${link['property']}'] = (out['${link['property']}'] ?? 0) + 1;
      }
    }
    return out;
  }

  static List<TLayer> _readAll(EditorSession c) {
    final names = {for (final l in c.layers) l['id']: '${l['name']}'};
    // what each row drives, counted in one pass over every link (not once per layer)
    final drives = <Object?, Map<String, int>>{};
    for (final l in c.layers) {
      for (final r in panelRows(l['properties'])) {
        final link = r['link'];
        if (link is Map) {
          final of = drives.putIfAbsent(link['layer'], () => <String, int>{});
          of['${link['property']}'] = (of['${link['property']}'] ?? 0) + 1;
        }
      }
    }
    return [
      for (final l in c.liveLayers())
        layerOf(l)
          ..drives.addAll(drives[l['id']] ?? const {})
          ..links.forEach((_, link) => link['name'] = names[link['layer']] ?? 'Relation'),
    ];
  }

  /// What the last absorb read: the same document gives the same Instrument, so nothing is rebuilt or told.
  int? _read_;

  bool _gesture = false;

  @override
  Object? get subject => activeId;

  /// The document changed under the Instrument (another edit, undo, a new selection). While a gesture is running the
  /// Instrument owns its values, and the last commit reads the document again.
  void absorb() {
    if (_gesture || c.layers.isEmpty) return;
    final read = Object.hash(jsonEncode(c.liveLayers()), Object.hashAll(c.selectedIds), c.activeLayer?['id'], c.animating);
    if (read == _read_) return;
    _read_ = read;
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
    c.commandDirect('previewProperties', {
      'edits': [
        {'layer': activeId, 'property': id, 'value': values[activeId] ?? values.values.first, 'spread': typing ? 'typed' : 'offset'},
      ],
    }, '$activeId:$id');
  }

  @override
  void committed(String id) {
    _gesture = false;
    _read_ = null; // a gesture's own values are the Instrument's until the document is read again
    c.commandDirect('commitPreview');
  }

  @override
  void cancelledGesture(String id) {
    _gesture = false;
    _read_ = null; // a gesture's own values are the Instrument's until the document is read again
    c.commandDirect('cancelPreview');
  }

  @override
  void toggleKey(String id) {
    if (!canEdit || !c.supports('toggleKey')) return;
    c.command('toggleKey', {'layer': activeId, 'property': id});
  }

  @override
  void toggleKeys(List<String> ids) {
    if (!canEdit || !c.supports('toggleKey')) return;
    c.command('toggleKey', {'layer': activeId, 'properties': ids});
  }

  /// Back to the defaults the host knows, on every chosen layer that has the row: one step.
  @override
  void resetMany(List<String> ids) {
    if (!canEdit || !c.supports('reset')) return;
    c.command('reset', {'layer': activeId, 'properties': [for (final i in ids) if (rows.any((r) => r['id'] == i)) i], 'spread': true});
  }

  @override
  void reset(String id) => resetMany([id]);


  @override
  void setAnimate(bool on) {
    c.setAnimate(on);
  }
  @override
  void toggleAnimate() => c.toggleAnimate();


  @override
  void setAnchor(double x, double y) {
    if (!canEdit || !c.supports('anchor')) return;
    c.command('anchor', {'layer': activeId, 'xFraction': x, 'yFraction': y});
  }

  @override
  void setProjection(String p) {
    if (!canEdit) return;
    c.command('setAttrs', {'layer': activeId, 'spread': true, 'patch': {'projection': p}});
  }

  @override
  void setParent(int? id) {
    if (!canEdit) return;
    c.command('setAttrs', {'layer': activeId, 'patch': {'parent': id}});
  }

  /// World: the same three operations Classic's `_world()` sends, one layer, one edit each.
  @override
  void setWorldFlag(String key, dynamic value) {
    if (!canEdit) return;
    switch (key) {
      case 'environment':
        c.command('setAttrs', {'layer': activeId, 'patch': {'environment': value}});
      case 'ghost':
        c.command('ghost', {'enabled': value});
      case 'clipToBelow':
        c.command('clip', {'layer': activeId});
    }
  }

  @override
  void openBlend() => c.focusEditing(activeId, 'blendMode');

  @override
  bool worldCan(String op) => c.supports(op);

  /// The row's menu: make a relation from this value (it becomes the source), or go to the one it is in.
  @override
  Future<void> menu(BuildContext context, String id, int? axis, Offset at) async {
    final row = this.row(id);
    final driven = row['link'] is Map;
    final canRelate = c.supports('relate') && !frozen, canUnrelate = c.supports('unrelate') && !frozen;
    final chosen = await keyMenu(context, this, id, at, extra: [
      ('relate', 'Relation…'),
      if (driven || (row['drives'] ?? 0) > 0) ('focus', 'Show relation'),
      if (driven) ('unrelate', 'Remove relation'),
    ], disabled: {if (!canRelate) 'relate', if (!canUnrelate) 'unrelate'});
    if (chosen == 'relate') {
      final label = '${row['label'] ?? id}${axis == null ? '' : ' ${const ['X', 'Y', 'Z'][axis]}'}';
      c.relationDraft.value = {'layer': activeId, 'name': active.name, 'property': id, 'component': axis ?? 0, 'label': label};
      c.relationFocus.value = null;
      await c.placePanel('Relations', 'show');
    } else if (chosen == 'focus') {
      focusRelation(id);
    } else if (chosen == 'unrelate') {
      await c.command('unrelate', {'layers': [activeId], 'property': id});
    }
  }

  @override
  void focusRelation(String id) {
    final link = row(id)['link'];
    c.relationFocus.value = link is Map ? {'layer': link['layer'], 'property': link['property'], 'component': link['component'] ?? 0} : {'layer': activeId, 'property': id, 'component': 0};
    c.relationDraft.value = null;
    c.placePanel('Relations', 'show');
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
