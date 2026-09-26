import '../../../hf/insp/rows.dart';
import '../../../session/editor_session.dart';
import '../../../session/read_model.dart';

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
  final _bases = <String, Object?>{};

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
    frozen = layer?['locked'] == true || !c.supports('previewProperties') || !c.supports('commitPreview');
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

  /// The shown layer and every other selected, unlocked layer that has the same row.
  List<Map<String, dynamic>> _targets(String id) {
    final live = c.liveLayers();
    final ids = c.selectedIds;
    return [
      for (final l in live)
        if ((l['id'] == layerId || ids.contains(l['id'])) && l['locked'] != true && _has(l, id)) l,
    ];
  }

  static bool _has(Map<String, dynamic> layer, String id) {
    for (final e in panelRows(layer['effects'])) {
      if (panelRows(e['params']).any((r) => r['id'] == id)) return true;
    }
    return false;
  }

  static Object? _valueOf(Map<String, dynamic> layer, String id) {
    for (final e in panelRows(layer['effects'])) {
      for (final r in panelRows(e['params'])) {
        if (r['id'] == id) return r['value'];
      }
    }
    return null;
  }

  @override
  void preview(String id, Object? v) {
    if (frozen) return;
    final before = row(id)['value'];
    _bases.putIfAbsent(id, () => before);
    super.preview(id, v);
    final next = row(id)['value'];
    final base = _bases[id];
    final edits = <Map<String, dynamic>>[];
    for (final target in _targets(id)) {
      Object? value = next;
      if (target['id'] != layerId && !typing) {
        final mine = _valueOf(target, id);
        if (next is num && base is num && mine is num) {
          value = mine + (next - base);
        } else if (next is List && base is List && mine is List) {
          value = [
            for (var i = 0; i < next.length; i++)
              i < base.length && i < mine.length && next[i] is num ? (mine[i] as num) + (next[i] as num) - (base[i] as num) : next[i],
          ];
        }
      }
      edits.add({'layer': target['id'], 'property': id, 'value': value});
    }
    if (edits.isEmpty) return;
    _gesture = true;
    c.command('previewProperties', {'edits': edits});
  }

  @override
  void commit(String id) {
    if (frozen) return;
    super.commit(id);
    _bases.remove(id);
    _gesture = false;
    c.command('commitPreview');
  }

  @override
  void cancelled(String id) {
    _bases.remove(id);
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
