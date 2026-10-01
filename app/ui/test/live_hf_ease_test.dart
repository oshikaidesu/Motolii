import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';

import '../lib/live_hf/adapters/ease.dart';
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

  test('the host kinds and the picked keys\' intervals; one interval is eased through its first key', () async {
    final c = EditorSession()
      ..document.value = {
        'selectedIds': [3],
        'selectedKeys': [
          {'layer': 3, 'property': 'opacity', 'frame': 0},
          {'layer': 3, 'property': 'opacity', 'frame': 10},
        ],
        'easeKinds': [
          {'kind': 'Linear', 'samples': [[0, 0], [1, 1]]},
          {'kind': 'Bezier', 'x1': .42, 'y1': 0, 'x2': .58, 'y2': 1, 'samples': [[0, 0], [1, 1]]},
        ],
        'easeIntervals': [
          {'layer': 3, 'property': 'opacity', 'frame': 0, 'end': 10, 'shape': {'kind': 'Linear'}},
          {'layer': 3, 'property': 'opacity', 'frame': 10, 'end': 16, 'shape': {'kind': 'Bezier', 'x1': .1, 'y1': .2, 'x2': .3, 'y2': .4}},
        ],
      };
    final host = LiveEaseHost(c);
    expect([for (final k in host.kinds) k.name], ['Linear', 'Bezier']);
    final segs = host.intervals;
    expect([for (final s in segs) s.frames], [10, 6]);
    expect(segs[1].values, [.1, .2, .3, .4]);

    segs[1].x1 = .5;
    await host.apply(1, segs[1]);
    expect([for (final m in sent) m['op']], ['select', 'ease', 'select']);
    expect(sent[0]['keys'], [
      {'layer': 3, 'property': 'opacity', 'frame': 10},
    ]);
    expect(sent[1], {'op': 'ease', 'kind': 'Bezier', 'x1': .5, 'y1': .2, 'x2': .3, 'y2': .4});
    expect(sent[2]['keys'], hasLength(2));
    host.dispose();
  });
}
