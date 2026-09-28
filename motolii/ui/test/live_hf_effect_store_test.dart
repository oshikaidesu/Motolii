import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';

import '../lib/live_hf/adapters/effect_store.dart';
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

  test('a layer-picker parameter names the other layers and writes the chosen one by id', () async {
    final c = EditorSession()
      ..document.value = {
        'capabilities': ['previewProperties', 'commitPreview', 'setProperty'],
        'layers': [
          {
            'id': 1,
            'name': 'Box',
            'kind': 'Shape',
            'effects': [
              {
                'id': 4,
                'name': 'Matte',
                'params': [
                  {'id': 'effect.4.param.source', 'label': 'Source', 'layer': true, 'value': 0},
                ],
              },
            ],
          },
          {'id': 2, 'name': 'Logo', 'kind': 'Image'},
        ],
      };
    final s = SessionEffectStore(c, 1, 4);
    final row = s.row('effect.4.param.source');
    expect(row['refs'], ['Logo']);
    expect(row['value'], isNull);
    s.set('effect.4.param.source', 'Logo');
    await Future<void>.delayed(Duration.zero);
    expect(sent.last, {'op': 'setProperty', 'layer': 1, 'property': 'effect.4.param.source', 'value': 2});
    s.set('effect.4.param.source', null);
    await Future<void>.delayed(Duration.zero);
    expect(sent.last['value'], 0);
    s.dispose();
  });
  test('two layers with one name are still two choices, each written by its own id', () async {
    final c = EditorSession()
      ..document.value = {
        'capabilities': ['previewProperties', 'commitPreview', 'setProperty'],
        'layers': [
          {'id': 1, 'name': 'Box', 'effects': [{'id': 4, 'name': 'Matte', 'params': [{'id': 'effect.4.param.source', 'layer': true, 'value': 0}]}]},
          {'id': 2, 'name': 'Shape'},
          {'id': 3, 'name': 'Shape'},
        ],
      };
    final s = SessionEffectStore(c, 1, 4);
    expect(s.row('effect.4.param.source')['refs'], ['Shape · 2', 'Shape · 3']);
    s.set('effect.4.param.source', 'Shape · 3');
    await Future<void>.delayed(Duration.zero);
    expect(sent.last['value'], 3);
    s.dispose();
  });
}
