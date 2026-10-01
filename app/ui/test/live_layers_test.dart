import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  test('a playback frame lays its live values over the document layers', () {
    final c = EditorSession();
    c.document.value = {
      'documentRevision': 'r1',
      'layers': [
        {
          'id': 1,
          'name': 'paper',
          'corners': [
            [0, 0],
            [1, 0],
            [1, 1],
            [0, 1],
          ],
          'properties': [
            {
              'id': 'position',
              'label': 'Position',
              'value': [0.0, 0.0],
              'keys': [
                {'frame': 0},
              ],
              'keyedNow': true,
            },
            {'id': 'opacity', 'label': 'Opacity', 'value': 1.0, 'keys': []},
          ],
          'effects': [
            {
              'id': 0,
              'pluginId': 'motolii.repeat',
              'layout': {'rows': []},
              'params': [
                {
                  'id': 'effect.0.param.count',
                  'label': 'Count',
                  'value': 3.0,
                  'choices': null,
                },
              ],
            },
          ],
        },
        {'id': 2, 'name': 'still', 'properties': [], 'effects': []},
      ],
    };
    c.frame.value = 12;
    c.rendered.value = {
      'frame': 12,
      'documentRevision': 'r1',
      'liveLayers': [
        {
          'id': 1,
          'corners': [
            [5, 5],
            [6, 5],
            [6, 6],
            [5, 6],
          ],
          'properties': [
            {
              'id': 'position',
              'value': [40.0, 0.0],
              'keyedNow': false,
            },
          ],
          'effects': [
            {
              'id': 0,
              'params': [
                {'id': 'effect.0.param.count', 'value': 7.0, 'keyedNow': false},
              ],
            },
          ],
        },
      ],
    };
    final live = c.liveLayers();
    expect(
      live.length,
      2,
      reason: 'layers the frame did not mention stay as they are',
    );
    final paper = live.first;
    expect(paper['name'], 'paper', reason: 'static fields survive the overlay');
    expect(paper['corners'], [
      [5, 5],
      [6, 5],
      [6, 6],
      [5, 6],
    ]);
    final position = EditorSession.maps(paper['properties']).first;
    expect(position['value'], [40.0, 0.0]);
    expect(position['label'], 'Position');
    expect(position['keys'], [
      {'frame': 0},
    ], reason: 'keys are static and come from the document');
    final count = EditorSession.maps(
      EditorSession.maps(paper['effects']).first['params'],
    ).first;
    expect(count['value'], 7.0);
    expect(count['label'], 'Count');
    expect(EditorSession.maps(paper['effects']).first['layout'], {'rows': []});

    c.frame.value = 13;
    expect(c.liveLayers().first['corners'], [
      [0, 0],
      [1, 0],
      [1, 1],
      [0, 1],
    ], reason: 'a stale frame is ignored');
  });

  test('a typed reply is read through views, not copies', () {
    final c = EditorSession();
    // What the channel hands over: untyped maps and lists, as StandardMessageCodec decodes them.
    final reply = <Object?, Object?>{
      'status': <Object?, Object?>{
        'documentRevision': 'r1',
        'frame': 0,
        'layers': <Object?>[
          <Object?, Object?>{
            'id': 1,
            'name': 'paper',
            'properties': <Object?>[
              <Object?, Object?>{
                'id': 'position',
                'value': <Object?>[1, 2],
              },
            ],
            'effects': <Object?>[],
          },
          <Object?, Object?>{
            'id': 2,
            'name': 'still',
            'properties': <Object?>[],
            'effects': <Object?>[],
          },
        ],
      },
    };
    final typed = EditorSession.typed(reply) as Map<String, dynamic>;
    final status = typed['status'] as Map<String, dynamic>;
    expect(status['layers'], isA<List<Map<String, dynamic>>>());
    expect(
      identical(EditorSession.maps(status['layers']), status['layers']),
      isTrue,
      reason: 'maps() is a view over a typed list',
    );
    final first = EditorSession.maps(status['layers']).first;
    expect(
      identical(EditorSession.map(first), first),
      isTrue,
      reason: 'map() is a view over a typed map',
    );
    expect(
      identical(EditorSession.typed(typed), typed),
      isTrue,
      reason: 'typing an already typed reply allocates nothing',
    );
    // An untyped list still comes back as a fresh typed copy.
    expect(
      EditorSession.maps(reply['status']),
      isA<List<Map<String, dynamic>>>(),
    );

    c.document.value = status;
    c.frame.value = 0;
    c.rendered.value = <String, dynamic>{
      'frame': 0,
      'documentRevision': 'r1',
      'liveLayers': <Map<String, dynamic>>[
        <String, dynamic>{
          'id': 1,
          'properties': <Map<String, dynamic>>[
            <String, dynamic>{
              'id': 'position',
              'value': <Object?>[5, 6],
            },
          ],
        },
      ],
    };
    final live = c.liveLayers();
    expect(
      identical(live, c.liveLayers()),
      isTrue,
      reason:
          'one overlay per (document, frame), shared by Stage and Inspector',
    );
    expect(
      identical(live[1], EditorSession.maps(status['layers'])[1]),
      isTrue,
      reason: 'a layer the frame did not mention is the document object itself',
    );
    expect(EditorSession.maps(live[0]['properties']).first['value'], [5, 6]);
    expect(EditorSession.maps(status['layers'])[0]['properties'][0]['value'], [
      1,
      2,
    ], reason: 'the overlay never writes into the document');
    c.rendered.value = <String, dynamic>{...c.rendered.value};
    expect(
      identical(live, c.liveLayers()),
      isFalse,
      reason: 'a new frame, a new overlay',
    );
  });
}
