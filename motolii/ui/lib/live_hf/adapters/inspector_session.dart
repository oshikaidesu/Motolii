import '../../session/editor_session.dart';
import 'package:flutter/foundation.dart';

import 'camera.dart' show SessionCameraStore;
import '../../effects/store.dart';
import 'layout_store.dart' show SessionLayoutStore;
import 'transform_store.dart';

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
class InspectorSession extends ChangeNotifier {
  InspectorSession._(this.c) {
    c.slice('inspectorSession', _watched).addListener(absorb);
    c.rendered.addListener(absorb);
    c.focusProperty.addListener(_focus);
    absorb();
  }
  static final _all = Expando<InspectorSession>();
  static InspectorSession of(EditorSession c) => _all[c] ??= InspectorSession._(c);
  final EditorSession c;
  static const _watched = ['layers', 'selectedId', 'selectedIds', 'animate', 'capabilities', 'documentRevision'];

  // ---- the stores: the rows of what is shown and every edit made to them, kept here so a skin can be thrown away or
  // swapped without losing an edit in flight -------------------------------------------------------------------------
  /// Transform for the selection (null with nothing to transform). Listeners hear when it appears or goes.
  SessionTransformStore? transform;
  final _effects = <(int, Object), SessionEffectStore>{};
  final _rows = <(int, String), SessionEffectStore>{};
  final _layouts = <int, SessionLayoutStore>{};
  final _cameras = <int, SessionCameraStore>{};

  SessionEffectStore effect(int layer, Object id) => _effects[(layer, id)] ??= SessionEffectStore(c, layer, id);
  SessionEffectStore layerRows(int layer, String prefix) => _rows[(layer, prefix)] ??= SessionEffectStore.layerRows(c, layer, prefix);
  SessionLayoutStore layout(Map<String, dynamic> layer) => _layouts[layer['id'] as int] ??= SessionLayoutStore(c, layer);
  SessionCameraStore camera(int layer) => _cameras[layer] ??= SessionCameraStore(c, layer);

  void absorb() {
    final had = transform != null;
    if (c.layers.isEmpty || c.activeLayer == null) {
      transform?.dispose();
      transform = null;
    } else if (transform == null) {
      transform = SessionTransformStore(c);
    } else {
      transform!.absorb();
    }
    final live = {for (final l in c.layers) l['id'] as int: l};
    bool gone(int layer) => !live.containsKey(layer);
    void sweep<K, S extends ChangeNotifier>(Map<K, S> m, int Function(K) layerOf, bool Function(K) alive, void Function(S) absorbIt) {
      m.removeWhere((k, st) {
        if (gone(layerOf(k)) || !alive(k)) {
          st.dispose();
          return true;
        }
        absorbIt(st);
        return false;
      });
    }

    sweep<(int, Object), SessionEffectStore>(_effects, (k) => k.$1, (k) => EditorSession.maps(live[k.$1]!['effects']).any((e) => e['id'] == k.$2), (st) => st.absorb());
    sweep<(int, String), SessionEffectStore>(_rows, (k) => k.$1, (_) => true, (st) => st.absorb());
    sweep<int, SessionLayoutStore>(_layouts, (k) => k, (_) => true, (st) => st.absorb());
    sweep<int, SessionCameraStore>(_cameras, (k) => k, (_) => true, (st) => st.absorb());
    if (had != (transform != null)) notifyListeners();
  }

  void _focus() {
    final id = c.focusProperty.value;
    if (id != null) transform?.focusProperty(id);
  }

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

  /// What the Inspector *draws* of [subject], apart from the numbers the rows' stores hold: which subject, and for a
  /// layer its id, name, kind, frozen / locked, which sections it has and each effect's id, name, bypass and
  /// placement. A slice compares this, so a Position or effect-parameter preview leaves the seat's cards standing
  /// (their rows redraw from the stores) and only a change in this list rebuilds the seat.
  Object? get shape => switch (subject) {
    InspectorEmpty(:final why) => ['empty', why],
    InspectorCamera(:final layer) => ['camera', layer],
    InspectorLayer(:final layer, :final stage, :final layout, :final effects) => [
      'layer',
      layer?['id'],
      layer?['name'],
      layer?['kind'],
      layer?['frozen'],
      layer?['locked'],
      stage,
      layout,
      for (final e in effects) [e['id'], e['name'], e['enabled'], e['placement']],
    ],
  };

  // ---- a layer's effects -----------------------------------------------------------------------------------------
  bool get canMoveEffects => c.supports('moveEffect');
  void moveEffect(int layer, Object effect, int to) {
    if (canMoveEffects) c.command('moveEffect', {'layer': layer, 'id': effect, 'to': to});
  }

  Future<void> expandEffect(int layer, Object effect) => c.command('expandEffect', {'layer': layer, 'id': effect});
  Future<void> removeEffect(int layer, Object effect) => c.command('removeEffect', {'layer': layer, 'id': effect});
  /// Earlier (-1) / later (+1) from where the host has it now.
  void stepEffect(int layer, Object effect, int step) {
    if (canMoveEffects) c.command('moveEffect', {'layer': layer, 'id': effect, 'step': step});
  }

  /// The bypass switch: the host flips what the document holds.
  void flipEffect(int layer, Object effect) {
    if (canEnableEffects) c.command('enableEffect', {'layer': layer, 'id': effect});
  }

  bool get canEnableEffects => c.supports('enableEffect');
  void enableEffect(int layer, Object effect, bool enabled) {
    if (canEnableEffects) c.command('enableEffect', {'layer': layer, 'id': effect, 'enabled': enabled});
  }
}
