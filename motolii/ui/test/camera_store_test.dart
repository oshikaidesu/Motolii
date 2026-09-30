// The Inspector Camera store shows the document; refreshing it must never write the document back.
import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_ui/inspector/camera/card.dart';
import 'package:motolii_ui/session/editor_session.dart';

Map<String, dynamic> row(String id, Object value) => {'id': id, 'value': value, 'kind': 'f32'};

Map<String, dynamic> doc({required List<Map<String, dynamic>> layers, num target = 0, List? gizmoCenter, bool lockedTarget = false}) => {
      'width': 1280,
      'height': 720,
      'capabilities': ['toggleKey', 'reset', 'setProperty'],
      'selectedIds': [1],
      'selectedId': 1,
      if (gizmoCenter != null) 'cameraGizmos': [{'id': 1, 'center': gizmoCenter}],
      'layers': [
        {
          'id': 1,
          'kind': 'Camera',
          'name': 'Cam',
          'properties': [
            row('camera.center', [10.0, 20.0]),
            row('camera.target.z', 5.0),
            row('camera.target', target),
          ],
        },
        ...layers,
      ],
    };

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  final ops = <String>[];
  setUp(() {
    ops.clear();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(EditorSession.channel, (call) async {
      if (call.method == 'request') ops.add(jsonDecode(call.arguments['command'])['op'] as String);
      return <String, dynamic>{};
    });
  });
  tearDown(() => TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(EditorSession.channel, null));

  test('a Target layer that appears after the store is kept, not cleared by the refresh', () async {
    final c = EditorSession();
    c.document.value = doc(layers: const []);
    final store = SessionCameraStore(c, 1);
    // undo / another panel brings layer 7 back and the camera already names it
    c.document.value = doc(layers: [{'id': 7, 'kind': 'Shape', 'name': 'Ball', 'x': 300.0, 'y': 200.0}], target: 7);
    store.absorb();
    await Future<void>.delayed(Duration.zero);
    expect(ops, isEmpty, reason: 'absorb only reads');
    expect(store.targetLayer?.id, 7);
    expect((store.row('camera.target')['choices'] as List), ['None', 'Ball']);
    c.dispose();
  });

  test('a locked centre shows the host-resolved point in the real comp, and its key / reset do not touch the hidden values', () async {
    final c = EditorSession();
    c.document.value = doc(layers: [{'id': 7, 'kind': 'Shape', 'name': 'Ball', 'x': 300.0, 'y': 200.0}], target: 7);
    final store = SessionCameraStore(c, 1);
    // no gizmo yet: the layer's position in the 1280x720 comp, not a 1920x1080 one
    expect(store.row('camera.center')['value'], [300.0 - 640, 200.0 - 360]);
    c.document.value = doc(layers: [{'id': 7, 'kind': 'Shape', 'name': 'Ball', 'x': 300.0, 'y': 200.0}], target: 7, gizmoCenter: [-1.5, 2.5]);
    store.absorb();
    expect(store.row('camera.center')['value'], [-1.5, 2.5]);
    store.toggleKeys(['camera.center', 'camera.target.z']);
    store.resetMany(['camera.center', 'camera.target.z']);
    await Future<void>.delayed(Duration.zero);
    expect(ops, isEmpty);
    c.dispose();
  });
}
