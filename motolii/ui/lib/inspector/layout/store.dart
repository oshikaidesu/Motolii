import 'package:flutter/widgets.dart';

import 'model.dart';
import '../../session/editor_session.dart';
import '../../session/read_model.dart';
import '../key_menu.dart';

/// The Layout Instrument's store over one real layer: its `layout.*` rows are the layer's own, and an edit goes
/// through previewProperties / commitPreview exactly as the Classic Layout card sends it. What the Instrument
/// knows about a row (its unit, its word for zero, its choices) comes from [LayoutStore]; the value, the range the
/// document declares and the key state come from the document.
class SessionLayoutStore extends LayoutStore {
  SessionLayoutStore(this.c, Map<String, dynamic> layer)
      : layerId = layer['id'] as int,
        super(child: isChild(layer), children: childCount(c, layer['id'] as int), frozen: layer['locked'] == true) {
    _load(layer);
  }

  final EditorSession c;
  final int layerId;
  bool _gesture = false;

  /// A layer the layout gives item rows to sits in a laid-out group: it has `layout.position_type`.
  static bool isChild(Map<String, dynamic> layer) => panelRows(layer['properties']).any((r) => r['id'] == 'layout.position_type');

  static int childCount(EditorSession c, int id) {
    final n = c.layers.where((l) => l['parent'] == id).length;
    return n < 1 ? 1 : n;
  }

  void _load(Map<String, dynamic> layer) {
    final real = {for (final r in panelRows(layer['properties'])) if ('${r['id']}'.startsWith('layout.')) '${r['id']}': r};
    final known = {for (final r in rows) '${r['id']}': r};
    rows
      ..clear()
      ..addAll([
        // the Instrument's own rows first, in its order, with the document's value
        for (final e in known.entries)
          if (real.containsKey(e.key) || known[e.key] != null) _merge(e.value, real[e.key]),
        // every other layout row the document declares stays reachable, behind Advanced
        for (final e in real.entries)
          if (!known.containsKey(e.key)) {..._plain(e.value), 'advanced': true},
      ]);
    frozen = layer['locked'] == true;
  }

  static Map<String, dynamic> _plain(Map<String, dynamic> r) => {for (final e in r.entries) if (e.value != null) e.key: e.value};

  static Map<String, dynamic> _merge(Map<String, dynamic> shape, Map<String, dynamic>? real) {
    if (real == null) return Map.of(shape);
    return {...shape, ..._plain(real), if (shape['choices'] != null) 'choices': shape['choices']};
  }

  // Layout values key like every other value (Classic's wells: key lamp and "Key this frame").
  @override
  bool get keyable => c.supports('toggleKey');

  @override
  void toggleKey(String id) {
    if (frozen || !keyable) return;
    c.command('toggleKey', {'layer': layerId, 'property': id});
  }
  @override
  void toggleKeys(List<String> ids) {
    if (frozen || !keyable) return;
    c.command('toggleKey', {'layer': layerId, 'properties': ids});
  }

  @override
  void menu(BuildContext context, String id, int? axis, Offset at) => keyMenu(context, this, id, at);

  /// The document changed under the Instrument (undo, another edit). A running gesture keeps its own values.
  void absorb() {
    if (_gesture) return;
    final layer = c.liveLayers().where((l) => l['id'] == layerId).firstOrNull;
    if (layer == null) return;
    // what [_load] reads: the layer's layout rows and whether it is locked
    final now = [
      [for (final r in panelRows(layer['properties'])) if ('${r['id']}'.startsWith('layout.')) r],
      layer['locked'],
    ];
    if (sameValue(now, _seen)) return;
    _seen = now;
    _load(layer);
    notifyListeners();
  }

  Object? _seen;

  @override
  void preview(String id, Object? v) {
    if (frozen) return;
    super.preview(id, v);
    _gesture = true;
    c.commandDirect('previewProperties', {
      'edits': [
        {'layer': layerId, 'property': id, 'value': row(id)['value']},
      ],
    }, '$layerId:$id');
  }

  /// Several rows as one edit: one preview carrying all of them, then the commit that follows.
  @override
  void applyMany(Map<String, Object?> values) {
    if (frozen) return;
    for (final e in values.entries) {
      super.preview(e.key, e.value);
    }
    _gesture = true;
    c.commandDirect('previewProperties', {
      'edits': [
        for (final e in values.entries) {'layer': layerId, 'property': e.key, 'value': row(e.key)['value']},
      ],
    }, '$layerId:${values.keys.join(',')}');
  }

  @override
  void cancelled(String id) {
    _gesture = false;
    _seen = null;
    c.commandDirect('cancelPreview');
  }

  @override
  void commit(String id) {
    if (frozen) return;
    super.commit(id);
    _gesture = false;
    _seen = null;
    c.commandDirect('commitPreview');
  }
}
