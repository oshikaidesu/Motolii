import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/timeline.dart';

class RecordingController extends EditorSession {
  final commands = <Map<String, dynamic>>[];
  @override
  Future<void> command(
    String op, [
    Map<String, dynamic> args = const {},
  ]) async {
    commands.add({'op': op, ...args});
    if (op == 'select')
      document.value = {
        ...state,
        'selectedIds': args['ids'],
        'selectedKeys': args['keys'] ?? [],
      };
  }
}

void main() {
  testWidgets('A folded layer row shows every key and moves them together', (
    tester,
  ) async {
    final c = RecordingController();
    c.document.value = {
      'fps': 30,
      'durationFrames': 300,
      'capabilities': ['moveKeys', 'select'],
      'selectedIds': [],
      'selectedKeys': [],
      'layers': [
        {
          'id': 1,
          'name': 'paper',
          'kind': 'Shape',
          'start': 0,
          'duration': 300,
          'properties': [
            {
              'id': 'position',
              'keys': [
                {'frame': 10},
                {'frame': 20},
              ],
            },
            {
              'id': 'opacity',
              'keys': [
                {'frame': 10},
              ],
            },
          ],
          'effects': [
            {
              'id': 0,
              'pluginId': 'motolii.blur',
              'params': [
                {
                  'id': 'effect.0.param.radius',
                  'keys': [
                    {'frame': 30},
                  ],
                },
              ],
            },
          ],
        },
      ],
    };
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(body: TimelinePanel(controller: c)),
      ),
    );
    // labelWidth 220, 4 px per frame, first row centred at y 64 (lanes folded).
    double x(int frame) => 220 + frame * 4.0;

    await tester.tapAt(Offset(x(10), 64));
    await tester.pump();
    final select = c.commands.lastWhere((m) => m['op'] == 'select');
    final keys = List<Map<String, dynamic>>.from(select['keys'] as List);
    expect(keys.map((k) => '${k['property']}@${k['frame']}').toSet(), {
      'position@10',
      'opacity@10',
    }, reason: 'one summary diamond stands for every key at that frame');

    final pointer = await tester.startGesture(Offset(x(30), 64));
    await pointer.moveTo(Offset(x(30) + 40, 64));
    await tester.pump();
    await pointer.up();
    await tester.pump();
    final move = c.commands.lastWhere((m) => m['op'] == 'moveKeys');
    expect(move['deltaFrames'], 10);
    final picked =
        c.commands.where((m) => m['op'] == 'select').last['keys'] as List;
    expect(picked.map((k) => '${k['property']}@${k['frame']}').toSet(), {
      'effect.0.param.radius@30',
    }, reason: 'effect parameter keys sit on the folded row too');
  });
}
