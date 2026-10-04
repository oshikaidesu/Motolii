import '../../../hf/insp/rows.dart';
import '../../../session/editor_session.dart';
import '../../../session/read_model.dart';

/// A layer's own property rows, filtered, as a ParamStore over the real session — the same shape
/// [SessionEffectStore] gives an effect's params, for the rows a content card (Text, Fill) owns directly on the
/// layer rather than inside an effect. An edit reaches the layer through previewProperties/commitPreview, exactly
/// as Classic's Inspector sends it.
class SessionLayerRowsStore extends ParamStore {
  SessionLayerRowsStore(this.c, this.layerId, this.matches) : super(const []) {
    _load();
  }

  final EditorSession c;
  final int layerId;
  final bool Function(Map<String, dynamic> row) matches;
  bool _gesture = false;

  Map<String, dynamic>? _layer() => c.liveLayers().where((l) => l['id'] == layerId).firstOrNull;

  void _load() {
    final layer = _layer();
    rows
      ..clear()
      ..addAll([if (layer != null) for (final r in panelRows(layer['properties'])) if (matches(r)) r]);
    frozen = layer?['locked'] == true || !c.supports('previewProperties') || !c.supports('commitPreview');
  }

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
  void preview(String id, Object? v) {
    if (frozen) return;
    super.preview(id, v);
    _gesture = true;
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
