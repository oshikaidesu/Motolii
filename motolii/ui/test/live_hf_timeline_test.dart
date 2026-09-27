import 'dart:convert';

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/hf/shell/timeline.dart';
import '../lib/live_hf/adapters/timeline.dart';
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

  testWidgets('a body dragged is retimed from where the drag began; a name selects its layer', (tester) async {
    tester.view.physicalSize = const Size(1178, 291);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final c = EditorSession()
      ..document.value = {
        'fps': 30,
        'durationFrames': 300,
        'layers': [
          {'id': 5, 'name': 'Title', 'kind': 'Text', 'start': 30, 'duration': 60, 'sourceIn': 0, 'properties': []},
        ],
      };
    await tester.pumpWidget(Directionality(textDirection: TextDirection.ltr, child: Align(alignment: Alignment.topLeft, child: SizedBox(width: 1178, height: 291, child: LiveTimeline(c: c)))));

    // the body runs from 1 s to 3 s: grab it at 2 s and move it one second (one major tick) right
    final y = tlTop + tlPitch * 0 + tlRowH / 2 - 703;
    final from = Offset(tlX(2) - 344, y);
    final g = await tester.startGesture(from);
    await g.moveBy(const Offset(tlUnit / 2, 0));
    await tester.pump();
    // the host answers the preview with the moved start; the next preview must not add to it
    c.document.value = {...c.state, 'layers': [{...EditorSession.maps(c.state['layers']).first, 'start': 45}]};
    await tester.pump();
    await g.moveBy(const Offset(tlUnit / 2, 0));
    await g.up();
    for (var i = 0; i < 5; i++) {
      await tester.pump();
    }
    final retimes = [for (final m in sent) if (m['op'] == 'previewTimings' || m['op'] == 'setTimings') m];
    expect(retimes.last['op'], 'setTimings');
    expect((retimes.last['changes'] as List).single, {'layer': 5, 'start': 60, 'duration': 60, 'sourceIn': 0});
    expect(sent.any((m) => m['op'] == 'select' && (m['ids'] as List).contains(5)), isTrue);
  });
}
