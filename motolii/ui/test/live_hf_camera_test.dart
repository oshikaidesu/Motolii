import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';

import '../lib/live_hf/adapters/camera.dart';
import '../lib/session/editor_session.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  final sent = <Map>[];
  setUp(() {
    sent.clear();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(EditorSession.channel, (call) async {
      final args = call.arguments;
      if (args is Map && args['command'] is String) sent.add(jsonDecode(args['command'] as String) as Map);
      return <String, dynamic>{};
    });
  });

  test('a camera layer\'s rows; Target is written as the chosen layer\'s id', () async {
    final c = EditorSession()
      ..document.value = {
        'capabilities': ['toggleKey', 'setProperty'],
        'layers': [
          {'id': 2, 'name': 'Title', 'kind': 'Text', 'x': 960, 'y': 540},
          {
            'id': 9,
            'name': 'Camera',
            'kind': 'Camera',
            'properties': [
              {'id': 'camera.orbit', 'value': [10, 20], 'keys': [{}]},
              {'id': 'camera.distance', 'value': 2},
              {'id': 'camera.target', 'value': 0},
            ],
          },
        ],
      };
    final s = SessionCameraStore(c, 9);
    expect(s.pitch, 10);
    expect(s.yaw, 20);
    expect(s.distance, 2);
    expect(s.animated, contains('camera.orbit'));
    expect(s.targetLayer, isNull);
    expect(s.frozen, isFalse);

    s.set('camera.target', 1);
    for (var i = 0; i < 5; i++) {
      await Future<void>.delayed(Duration.zero);
    }
    expect(sent.single, {'op': 'setProperty', 'layer': 9, 'property': 'camera.target', 'value': 2});
    expect(s.targetLocked, isTrue);
    s.dispose();
  });
  test('a locked camera refuses changes', () {
    final c = EditorSession()
      ..document.value = {
        'layers': [
          {'id': 9, 'name': 'Camera', 'kind': 'Camera', 'locked': true, 'properties': [{'id': 'camera.distance', 'value': 2}]},
        ],
      };
    expect(SessionCameraStore(c, 9).frozen, isTrue);
  });
}
