import 'package:flutter/widgets.dart';

import '../../hf/insp/rows.dart';
import '../../session/editor_session.dart';
import '../../session/read_model.dart';
import 'key_menu.dart';

/// One effect's parameters, as rows of a ParamStore over the real session. The rows are the effect's declared params
/// as the document sends them; an edit reaches every selected, unlocked layer that has the same row, a drag relative
/// and a typed number absolute, through previewProperties and commitPreview as the Classic Inspector sends them.
class SessionEffectStore extends ParamStore {
  SessionEffectStore(this.c, this.layerId, Object this.effectId) : prefix = null, super(const []) {
    _load();
  }

  /// A layer's own rows under [prefix] (a Stage layer's `stage.` margins), written the same way.
  SessionEffectStore.layerRows(this.c, this.layerId, String this.prefix) : effectId = null, super(const []) {
    _load();
  }

  final EditorSession c;
  final int layerId;
  final Object? effectId;
  final String? prefix;
  bool _gesture = false;

  Map<String, dynamic>? _layer() => c.liveLayers().where((l) => l['id'] == layerId).firstOrNull;

  Map<String, dynamic>? _effect(Map<String, dynamic> layer) => panelRows(layer['effects']).where((e) => e['id'] == effectId).firstOrNull;

  static bool _picksLayer(Map<String, dynamic> r) => r['layer'] == true || r['kind'] == 'layer';

  List<Map<String, dynamic>> get _others => [for (final l in c.layers) if (l['id'] != layerId) l];

  String? _nameOf(Object? id) =>
      id is num && id != 0 ? _others.where((l) => l['id'] == id.toInt()).map((l) => '${l['name']}').firstOrNull : null;

  void _load() {
    final layer = _layer();
    final effect = layer == null || effectId == null ? null : _effect(layer);
    final source = prefix != null
        ? [for (final r in panelRows(layer?['properties'])) if ('${r['id']}'.startsWith(prefix!)) r]
        : (effect == null ? const <Map<String, dynamic>>[] : panelRows(effect['params']));
    rows
      ..clear()
      ..addAll([
          for (final r in source)
            {
              ...r,
              // a layer picker names the other layers and shows its layer by name (Classic IN-094: None + the others)
              if (_picksLayer(r)) ...{
                'refs': [for (final l in _others) '${l['name']}'],
                'value': _nameOf(r['value']),
              },
              'animated': (r['keys'] as List?)?.isNotEmpty ?? false,
              if (r['default'] == null) ...const {},
            },
      ]);
    // a frozen layer's effects are baked: shown, not editable (Classic IN-141)
    frozen = layer?['locked'] == true || layer?['frozen'] == true || !c.supports('previewProperties') || !c.supports('commitPreview');
  }

  /// The document changed under the sheet (undo, another edit). A running gesture keeps its own values.
  void absorb() {
    if (_gesture) return;
    _load();
    notifyListeners();
  }

  @override
  bool get keyable => c.supports('toggleKey');

  @override
  void toggleKey(String id) {
    if (frozen || !keyable) return;
    c.command('toggleKey', {'layer': layerId, 'property': id});
  }

  @override
  void menu(BuildContext context, String id, int? axis, Offset at) => keyMenu(context, this, id, at);

  @override
  void preview(String id, Object? v) {
    if (frozen) return;
    super.preview(id, v);
    _gesture = true;
    // One edit; the host spreads it over the selection (a drag by offset, a typed number as it is).
    c.command('previewProperties', {
      'edits': [
        {'layer': layerId, 'property': id, 'value': row(id)['value'], 'spread': typing ? 'absolute' : 'offset'},
      ],
    });
  }

  @override
  void commit(String id) {
    if (frozen) return;
    super.commit(id);
    _gesture = false;
    c.command('commitPreview');
  }

  @override
  void cancelled(String id) {
    _gesture = false;
    c.command('cancelPreview');
  }

  @override
  void set(String id, Object? v) {
    if (_picksLayer(row(id))) {
      // the layer, by id (0 = None), in one step as Classic's picker writes it
      if (frozen) return;
      final target = v == null ? 0 : (_others.where((l) => '${l['name']}' == v).map((l) => l['id']).firstOrNull ?? 0);
      row(id)['value'] = v;
      notifyListeners();
      c.command('setProperty', {'layer': layerId, 'property': id, 'value': target});
      return;
    }
    typing = true;
    try {
      super.set(id, v);
    } finally {
      typing = false;
    }
  }

  /// A hand-over to the specialist that owns the value: the colour wheel for a colour, else the panel of that name.
  @override
  void route(String to, [String? from]) {
    super.route(to, from);
    final name = to.toLowerCase();
    if ((name == 'colors' || name == 'color') && from != null && c.supports('focusColor')) {
      c.focusColor({'layer': layerId, 'property': from});
    } else {
      c.placePanel(to, 'show');
    }
  }
}
