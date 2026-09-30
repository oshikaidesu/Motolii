import 'package:flutter/widgets.dart';

import '../../hf/insp/camera.dart';
import '../../hf/insp/camera_model.dart';
import '../../session/editor_session.dart';
import '../../session/read_model.dart';
import 'key_menu.dart';
import 'inspector_session.dart';

/// The Camera instrument over the active camera layer: its `camera.*` rows read from the layer, written the way every
/// value is (a drag previews and commits, a choice or a reset sets, a diamond keys); Target is a layer id, 0 for none.
class SessionCameraStore extends CameraStore {
  SessionCameraStore(this.c, this.layer) : super(layers: _others(c, layer)) {
    absorb();
  }
  final EditorSession c;
  final int layer;
  bool _gesture = false;

  static List<CamLayer> _others(EditorSession c, int self) => [
        for (final l in c.layers)
          if (l['id'] != self && l['kind'] != 'Camera')
            CamLayer(l['id'] as int, '${l['name'] ?? 'Layer'}', (l['x'] as num? ?? 0).toDouble(), (l['y'] as num? ?? 0).toDouble(), 0),
      ];

  Map<String, Map<String, dynamic>> get _rows {
    final l = c.layers.where((l) => l['id'] == layer).firstOrNull;
    return {for (final r in panelRows(l?['properties'])) '${r['id']}': r};
  }

  Object? _seen;

  void absorb() {
    if (_gesture) return;
    // what this reads: the camera layer itself, the layers a Target may name (name and place), the comp's size and the
    // host's resolved centre; a change elsewhere in the document leaves the instrument as it stands
    final now = [
      c.layers.where((l) => l['id'] == layer).firstOrNull,
      for (final l in c.layers) if (l['id'] != layer && l['kind'] != 'Camera') [l['id'], l['name'], l['x'], l['y']],
      c.state['width'],
      c.state['height'],
      EditorSession.maps(c.state['cameraGizmos']).where((g) => g['id'] == layer).firstOrNull?['center'],
    ];
    if (sameValue(now, _seen)) return;
    _seen = now;
    // a locked camera shows its values and refuses changes, as every locked layer does
    frozen = c.layers.where((l) => l['id'] == layer).firstOrNull?['locked'] == true;
    // the layers a Target may name and the comp it sits in are the document's now, not as they were when this store began
    useLayers(_others(c, layer));
    compW = (c.state['width'] as num?)?.toDouble() ?? compW;
    compH = (c.state['height'] as num?)?.toDouble() ?? compH;
    final seen = EditorSession.maps(c.state['cameraGizmos']).where((g) => g['id'] == layer).firstOrNull?['center'];
    resolvedCenter = seen is List && seen.length == 2 && seen.every((v) => v is num) ? [for (final v in seen) (v as num).toDouble()] : null;
    final rows = _rows;
    animated.clear();
    keyedNow.clear();
    for (final r in this.rows) {
      final id = '${r['id']}', live = rows[id];
      if (live == null) continue;
      if (id == 'camera.target') {
        final i = layers.indexWhere((l) => l.id == live['value']);
        r['value'] = i + 1;
      } else {
        r['value'] = live['value'] is List ? [for (final v in live['value'] as List) (v as num).toDouble()] : (live['value'] as num).toDouble();
        if (id == 'camera.center') authoredCenter = [for (final v in r['value'] as List) v as double];
        if (id == 'camera.target.z') authoredZ = r['value'] as double;
      }
      if ((live['keys'] as List?)?.isNotEmpty ?? false) animated.add(id);
      if (live['keyedNow'] == true) keyedNow.add(id);
    }
    // the instrument derives what a Target layer overrides from these values (shown, never written back)
    reveal();
  }

  @override
  bool get keyable => c.supports('toggleKey');

  @override
  void preview(String id, Object? v) {
    super.preview(id, v);
    if (frozen) return;
    _gesture = true;
    c.commandDirect('previewProperties', {
      'edits': [
        {'layer': layer, 'property': id, 'value': row(id)['value']},
      ],
    }, '$layer:$id');
  }

  @override
  void commit(String id) {
    super.commit(id);
    if (!_gesture) return;
    _gesture = false;
    _seen = null;
    c.commandDirect('commitPreview');
  }

  @override
  void cancelled(String id) {
    _gesture = false;
    _seen = null;
    c.commandDirect('cancelPreview');
  }

  @override
  void set(String id, Object? v) {
    if (id != 'camera.target') return super.set(id, v);
    super.set(id, v);
    if (_rows[id] == null) return;
    final i = (v as num).round() - 1;
    final target = i >= 0 && i < layers.length ? layers[i].id : 0;
    if (_rows[id]!['value'] != target) c.command('setProperty', {'layer': layer, 'property': id, 'value': target});
  }

  @override
  void menu(BuildContext context, String id, int? axis, Offset at) => keyMenu(context, this, id, at);

  @override
  void toggleKey(String id) {
    if (!keyable || frozen) return;
    c.command('toggleKey', {'layer': layer, 'property': id});
  }
  @override
  void toggleKeys(List<String> ids) {
    // a Target layer's point is not the camera's to key or reset: the authored Center / Target Z stay as they are
    final own = [for (final i in ids) if (!held(i)) i];
    if (frozen || !keyable || own.isEmpty) return;
    c.command('toggleKey', {'layer': layer, 'properties': own});
  }

  /// Back to the defaults the host knows: one step.
  @override
  void resetMany(List<String> ids) {
    final own = [for (final i in ids) if (!held(i) && rows.any((r) => r['id'] == i)) i];
    if (frozen || !c.supports('reset') || own.isEmpty) return;
    c.command('reset', {'layer': layer, 'properties': own});
  }

  @override
  void reset(String id) => resetMany([id]);

  @override
  void route(String to, [String? from]) {
    super.route(to, from);
    // Opens the real Dock panel (Depth is no longer a fixed seat, so this is the only way there since the
    // desks stopped defaulting into the workspace) - the same call transform_store.dart's own `route` makes.
    c.placePanel(to, 'show');
  }
}

/// The Camera instrument on the session's active camera layer.
class LiveCamera extends StatefulWidget {
  const LiveCamera({super.key, required this.c, required this.layer});
  final EditorSession c;
  final int layer;
  @override
  State<LiveCamera> createState() => _LiveCameraState();
}

class _LiveCameraState extends State<LiveCamera> {
  SessionCameraStore get store => InspectorSession.of(widget.c).camera(widget.layer);

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    // the head draws the layer's name and the Animate switch; a camera value preview changes neither
    listenable: widget.c.slice('liveCameraHead:${widget.layer}', const ['animate'], derived: () => widget.c.layers.where((l) => l['id'] == widget.layer).firstOrNull?['name']),
    builder: (context, _) => CameraInstrument(
      store,
      // the layer's own name, not the word "Camera" (Classic IN-002)
      title: '${widget.c.layers.where((l) => l['id'] == widget.layer).firstOrNull?['name'] ?? 'Camera'}',
      animating: widget.c.supports('animate') ? widget.c.animating : null,
      onAnimate: () => widget.c.toggleAnimate(),
    ),
  );
}
