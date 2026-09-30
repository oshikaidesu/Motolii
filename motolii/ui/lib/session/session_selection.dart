part of 'editor_session.dart';

/// Selection, as this window knows it. The selection itself is the host's: `Viewer::selected_ids` and `selected_keys`
/// (native/src/viewer.rs) hold it and the `select` command changes it; every reply carries it back, so this window has no
/// selection of its own to keep in step. What lives here is the reading of that selection (which layers, which keys, the layer
/// the Inspector follows) and the one thing this window adds: the key selection before the current one, so a shortcut can
/// bring a motion back without a trip to the Timeline.
mixin SessionSelection on SessionCore {
  /// The selected layers, primary last; empty when nothing is selected.
  List<int> get selectedIds =>
      (state['selectedIds'] as List? ?? [state['selectedId']])
          .whereType<num>()
          .map((n) => n.toInt())
          .toList();

  /// The selected keys, each as `{layer, property, frame}`.
  List<Map<String, dynamic>> get selectedKeys =>
      EditorSession.maps(state['selectedKeys']);

  /// The layer the Inspector and the Stage act on: the primary selection.
  Map<String, dynamic>? get activeLayer {
    if (selectedIds.isEmpty) return null;
    for (final l in layers) {
      if (l['id'] == selectedIds.last) return l;
    }
    return null;
  }

  /// The key selection before the current one, with the layers it sits on.
  Map<String, dynamic>? previousKeys;

  /// A reply is about to replace the key selection: keep the one it replaces.
  void rememberKeySelection(Map<String, dynamic> next) {
    final keys = selectedKeys;
    if (keys.isNotEmpty &&
        next['selectedKeys'] is List &&
        !sameValue(state['selectedKeys'], next['selectedKeys'])) {
      previousKeys = {'ids': selectedIds, 'keys': keys};
    }
  }

  /// Select the keys that were selected before.
  Future<void> reselectKeys() async {
    final back = previousKeys;
    if (back == null) return;
    await command('select', back);
  }
}
