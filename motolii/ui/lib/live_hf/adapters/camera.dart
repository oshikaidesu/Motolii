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

  void absorb() {
    if (_gesture) return;
    // a locked camera shows its values and refuses changes, as every locked layer does
    frozen = c.layers.where((l) => l['id'] == layer).firstOrNull?['locked'] == true;
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
    // the instrument derives what a Target layer overrides from these values
    set('camera.target', row('camera.target')['value']);
    commits--;
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
    c.commandDirect('commitPreview');
  }

  @override
  void cancelled(String id) {
    _gesture = false;
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
    if (frozen || !keyable) return;
    c.command('toggleKey', {'layer': layer, 'properties': ids});
  }

  /// Back to the defaults the host knows: one step.
  @override
  void resetMany(List<String> ids) {
    if (frozen || !c.supports('reset')) return;
    c.command('reset', {'layer': layer, 'properties': [for (final i in ids) if (rows.any((r) => r['id'] == i)) i]});
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
