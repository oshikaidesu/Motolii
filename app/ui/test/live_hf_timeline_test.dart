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
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          final args = call.arguments;
          if (args is Map && args['command'] is String)
            sent.add(jsonDecode(args['command'] as String) as Map);
          return <String, dynamic>{};
        });
  });

  testWidgets(
    'a body dragged is retimed from where the drag began; a name selects its layer',
    (tester) async {
      tester.view.physicalSize = const Size(1178, 291);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.reset);
      final c = EditorSession()
        ..document.value = {
          'fps': 30,
          'durationFrames': 300,
          'capabilities': [
            'moveKeys',
            'setTiming',
            'setTimings',
            'previewTimings',
            'commitPreview',
            'moveLayers',
            'setAttrs',
            'toggleKey',
            'clip',
          ],
          'layers': [
            {
              'id': 5,
              'name': 'Title',
              'kind': 'Text',
              'start': 30,
              'duration': 60,
              'sourceIn': 0,
              'properties': [],
            },
          ],
        };
      await tester.pumpWidget(
        Directionality(
          textDirection: TextDirection.ltr,
          child: Align(
            alignment: Alignment.topLeft,
            child: SizedBox(
              width: 1178,
              height: 291,
              child: LiveTimeline(c: c),
            ),
          ),
        ),
      );

      // the body runs from 1 s to 3 s: grab it at 2 s and move it one second (one major tick) right
      final y = tlTop + tlPitch * 0 + tlRowH / 2 - 703;
      final from = Offset(tlX(2) - 344, y);
      final g = await tester.startGesture(from);
      await g.moveBy(const Offset(tlUnit / 2, 0));
      await tester.pump();
      // the host answers the preview with the moved start; the next preview must not add to it
      c.document.value = {
        ...c.state,
        'layers': [
          {...EditorSession.maps(c.state['layers']).first, 'start': 45},
        ],
      };
      await tester.pump();
      await g.moveBy(const Offset(tlUnit / 2, 0));
      await g.up();
      for (var i = 0; i < 5; i++) {
        await tester.pump();
      }
      final retimes = [
        for (final m in sent)
          if (m['op'] == 'previewTimings' || m['op'] == 'setTimings') m,
      ];
      expect(sent.last['op'], 'commitPreview');
      expect((retimes.last['changes'] as List).single, {
        'layer': 5,
        'start': 60,
        'duration': 60,
        'sourceIn': 0,
      });
      expect(
        sent.any((m) => m['op'] == 'select' && (m['ids'] as List).contains(5)),
        isTrue,
      );
    },
  );

  testWidgets(
    'HF Timeline opens shared property and effect lanes from folded key summaries',
    (tester) async {
      tester.view.physicalSize = const Size(1178, 291);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.reset);
      final c = EditorSession()
        ..document.value = {
          'fps': 30,
          'durationFrames': 90,
          'layers': [
            {
              'id': 9,
              'name': 'Shape',
              'kind': 'Shape',
              'start': 0,
              'duration': 90,
              'properties': [
                {
                  'id': 'opacity',
                  'label': 'Opacity',
                  'keys': [
                    {'frame': 10},
                    {'frame': 20},
                  ],
                },
              ],
              'effects': [
                {
                  'id': 4,
                  'name': 'Blur',
                  'params': [
                    {
                      'id': 'effect.4.param.radius',
                      'label': 'Radius',
                      'keys': [
                        {'frame': 15},
                      ],
                    },
                  ],
                },
              ],
            },
          ],
        };
      await tester.pumpWidget(
        Directionality(
          textDirection: TextDirection.ltr,
          child: Align(
            alignment: Alignment.topLeft,
            child: SizedBox(
              width: 1178,
              height: 291,
              child: LiveTimeline(c: c),
            ),
          ),
        ),
      );
      final dynamic state = tester.state(find.byType(LiveTimeline));
      expect(state.tracks.first.summaryFrames, [10, 15, 20]);
      expect(state.tracks, hasLength(1));
      await tester.tapAt(const Offset(134, 83));
      await tester.pump();
      expect(
        state.tracks.map((row) => row.property?['id']).whereType<String>(),
        ['opacity', 'effect.4.param.radius'],
      );
    },
  );

  testWidgets(
    'the HF M control toggles layer state through the shared Timeline input',
    (tester) async {
      tester.view.physicalSize = const Size(1178, 291);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.reset);
      final c = EditorSession()
        ..document.value = {
          'fps': 30,
          'durationFrames': 90,
          'capabilities': ['setAttrs'],
          'layers': [
            {
              'id': 5,
              'name': 'Title',
              'kind': 'Text',
              'start': 0,
              'duration': 90,
              'properties': [],
            },
          ],
        };
      await tester.pumpWidget(
        Directionality(
          textDirection: TextDirection.ltr,
          child: Align(
            alignment: Alignment.topLeft,
            child: SizedBox(
              width: 1178,
              height: 291,
              child: LiveTimeline(c: c),
            ),
          ),
        ),
      );
      await tester.tapAt(const Offset(151, 83));
      await tester.pump();
      expect(sent.single, {
        'op': 'setAttrs',
        'layers': [5],
        'patch': {'hidden': true},
      });
    },
  );
}
