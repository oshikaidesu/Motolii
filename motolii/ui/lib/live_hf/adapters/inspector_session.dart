import '../../session/editor_session.dart';
import 'layout_store.dart' show SessionLayoutStore;

/// What the Inspector is about, with no screen in it: nothing, a camera's own instrument, or a layer's sections
/// (Transform always, the Stage margins of a Stage layer, the Layout of a group or laid-out child, its effects).
sealed class InspectorSubject {
  const InspectorSubject();
}

class InspectorEmpty extends InspectorSubject {
  const InspectorEmpty(this.why);

  /// No layers at all, or none picked.
  final String why;
}

class InspectorCamera extends InspectorSubject {
  const InspectorCamera(this.layer);
  final int layer;
}

class InspectorLayer extends InspectorSubject {
  const InspectorLayer({required this.layer, required this.stage, required this.layout, required this.effects});

  /// The live layer (null when several are picked and none is active: Transform alone speaks for them).
  final Map<String, dynamic>? layer;
  final bool stage, layout;
  final List<Map<String, dynamic>> effects;
}

/// The Inspector's meaning over a document session: what is shown for the selection, and what can be done to a
/// layer's effects. One per EditorSession; skins read it and call it.
class InspectorSession {
  InspectorSession._(this.c);
  static final _all = Expando<InspectorSession>();
  static InspectorSession of(EditorSession c) => _all[c] ??= InspectorSession._(c);
  final EditorSession c;

  InspectorSubject get subject {
    final active = c.activeLayer;
    if (active != null && active['kind'] == 'Camera' && c.selectedIds.length <= 1) return InspectorCamera(active['id'] as int);
    final layer = active == null ? null : c.liveLayers().where((l) => l['id'] == active['id']).firstOrNull ?? active;
    if (layer == null && c.layers.isEmpty) return const InspectorEmpty('No layers yet');
    if (layer == null && c.selectedIds.isEmpty) return const InspectorEmpty('Select a layer');
    if (layer == null) return const InspectorLayer(layer: null, stage: false, layout: false, effects: []);
    final single = c.selectedIds.length <= 1;
    return InspectorLayer(
      layer: layer,
      // a Stage layer's margins are its own rows (Classic IN-065)
      stage: single && EditorSession.maps(layer['properties']).any((r) => '${r['id']}'.startsWith('stage.')),
      layout: single && (layer['kind'] == 'Group' || SessionLayoutStore.isChild(layer)),
      effects: EditorSession.maps(layer['effects']).toList(),
    );
  }

  // ---- a layer's effects -----------------------------------------------------------------------------------------
  bool get canMoveEffects => c.supports('moveEffect');
  void moveEffect(int layer, Object effect, int to) {
    if (canMoveEffects) c.command('moveEffect', {'layer': layer, 'id': effect, 'to': to});
  }

  Future<void> expandEffect(int layer, Object effect) => c.command('expandEffect', {'layer': layer, 'id': effect});
  Future<void> removeEffect(int layer, Object effect) => c.command('removeEffect', {'layer': layer, 'id': effect});
  bool get canEnableEffects => c.supports('enableEffect');
  void enableEffect(int layer, Object effect, bool enabled) {
    if (canEnableEffects) c.command('enableEffect', {'layer': layer, 'id': effect, 'enabled': enabled});
  }
}
