import 'package:flutter/widgets.dart';

import '../../hf/insp/rows.dart';
import '../../session/editor_session.dart';
import '../../session/read_model.dart';
import 'key_menu.dart';

/// One effect's parameters, as rows of a ParamStore over the real session. The rows are the effect's declared params
/// as the document sends them; an edit reaches every selected, unlocked layer that has the same row, a drag relative
/// and a typed number absolute, through previewProperties and commitPreview as the Classic Inspector sends them.
class SessionEffectStore extends ParamStore {
  SessionEffectStore(this.c, this.layerId, this.effectId) : super(const []) {
    _load();
  }

  final EditorSession c;
  final int layerId;
  final Object effectId;
  bool _gesture = false;

  Map<String, dynamic>? _layer() => c.liveLayers().where((l) => l['id'] == layerId).firstOrNull;

  Map<String, dynamic>? _effect(Map<String, dynamic> layer) => panelRows(layer['effects']).where((e) => e['id'] == effectId).firstOrNull;

  void _load() {
    final layer = _layer();
    final effect = layer == null ? null : _effect(layer);
    rows
      ..clear()
      ..addAll([
        if (effect != null)
          for (final r in panelRows(effect['params']))
            {
              ...r,
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
