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
}
