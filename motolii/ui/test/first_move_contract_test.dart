// The Dart half of the first-move contract: the first non-zero move of an Inspector row goes to native as one
// previewProperties, and the render for it follows at once, with nothing else between them and nothing waiting on a
// second input. (The Stage pixels themselves need the real window: integration_test/first_preview_test.dart.)
import 'dart:convert';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_ui/inspector/camera/card.dart';
import 'package:motolii_ui/session/editor_session.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  final calls = <String>[];
  setUp(() {
    calls.clear();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(EditorSession.channel, (call) async {
      if (call.method == 'request') {
        final op = jsonDecode(call.arguments['command'])['op'] as String;
        calls.add('request:$op');
        return {'needsRender': true};
      }
      calls.add(call.method);
      return <String, dynamic>{'frameReady': true};
    });
  });
  tearDown(() => TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(EditorSession.channel, null));

  test('first camera move: previewProperties, then render, nothing between', () async {
    final c = EditorSession();
    c.document.value = {
      'width': 1920,
      'height': 1080,
      'capabilities': ['previewProperties', 'commitPreview'],
      'selectedIds': [1],
      'selectedId': 1,
      'layers': [
        {
          'id': 1,
          'kind': 'Camera',
          'name': 'Cam',
          'properties': [
            {'id': 'camera.roll', 'value': 0.0, 'kind': 'f32'},
          ],
        },
      ],
    };
    final store = SessionCameraStore(c, 1);
    store.preview('camera.roll', 5.0);
    await Future<void>.delayed(const Duration(milliseconds: 50));
    expect(calls.take(2), ['request:previewProperties', 'render']);
    store.commit('camera.roll');
    await Future<void>.delayed(const Duration(milliseconds: 50));
    expect(calls, contains('request:commitPreview'));
    c.dispose();
  });
}
