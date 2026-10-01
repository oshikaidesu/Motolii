import 'dart:convert';

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/live_hf/adapters/stage_touch.dart';
import '../lib/session/editor_session.dart';

void main() {
  final sent = <Map>[];
  setUp(() {
    sent.clear();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(EditorSession.channel, (call) async {
      final args = call.arguments;
      if (args is Map && args['command'] is String) sent.add(jsonDecode(args['command'] as String) as Map);
      return <String, dynamic>{};
    });
  });

  testWidgets('a press on a layer selects it, a drag moves it through stageGesture in composition pixels', (tester) async {
    final c = EditorSession()
      ..document.value = {
        'capabilities': ['stageGesture', 'select'],
        'layers': [
          {
            'id': 3,
            'kind': 'Shape',
            'projection': '2D',
            'corners': [[100, 100], [300, 100], [300, 200], [100, 200]],
          },
        ],
        'selectedIds': <int>[],
      };
    await tester.pumpWidget(Directionality(
      textDirection: TextDirection.ltr,
      child: Align(alignment: Alignment.topLeft, child: SizedBox(width: 400, height: 300, child: StageTouch(c: c, view: 'Camera', scale: .5))),
    ));
    final g = await tester.startGesture(const Offset(100, 75)); // composition (200, 150): inside the layer
    for (var i = 0; i < 5; i++) {
      await tester.pump();
    }
    await g.moveTo(const Offset(110, 75));
    for (var i = 0; i < 5; i++) {
      await tester.pump();
    }
    await g.up();
    for (var i = 0; i < 5; i++) {
      await tester.pump();
    }
    expect(sent.first, {'op': 'select', 'ids': [3]});
    final gestures = [for (final m in sent) if (m['op'] == 'stageGesture') m];
    expect([for (final m in gestures) m['phase']], ['begin', 'update', 'commit']);
    expect(gestures[1]['point'], [220.0, 150.0]);
    expect(gestures[1]['mode'], 'move');
    expect(gestures[1]['ids'], [3]);
  });
}
