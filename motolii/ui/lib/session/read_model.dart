import 'editor_session.dart';

Map<String, dynamic> panelMap(dynamic v) => EditorSession.map(v);
List<Map<String, dynamic>> panelRows(dynamic v) =>
    EditorSession.maps(v is List ? v : const []);
bool panelCan(EditorSession c, String op) => c.supports(op);

/// The layers a blend applies to: the host says which (`blendTargets` in the status), and this only finds their rows.
List<Map<String, dynamic>> blendTargets(EditorSession c) {
  final ids = (c.state['blendTargets'] as List? ?? const []).toSet();
  return [
    for (final l in (c.state['layers'] as List? ?? const []).whereType<Map>())
      if (ids.contains(l['id'])) Map<String, dynamic>.from(l),
  ];
}

/// Everything the blend tiles show: the mode in force and the specimens for it.
Object blendReading(EditorSession c) => [
  c.selectedIds,
  for (final l in blendTargets(c)) [l['id'], l['blendMode'], l['blendPreviews']],
  c.activeLayer?['blendPreviews'],
];
