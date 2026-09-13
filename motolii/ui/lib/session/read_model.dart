import 'editor_session.dart';

Map<String, dynamic> panelMap(dynamic v) => EditorSession.map(v);
List<Map<String, dynamic>> panelRows(dynamic v) =>
    EditorSession.maps(v is List ? v : const []);
bool panelCan(EditorSession c, String op) => c.supports(op);
// A property's name in the window: the core's one table, the same names a script writes.
String panelName(EditorSession c, String id) =>
    '${(c.state['names'] as Map?)?[id] ?? id}';
