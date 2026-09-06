import 'editor_session.dart';

Map<String, dynamic> panelMap(dynamic v) => EditorSession.map(v);
List<Map<String, dynamic>> panelRows(dynamic v) =>
    EditorSession.maps(v is List ? v : const []);
bool panelCan(EditorSession c, String op) => c.supports(op);
