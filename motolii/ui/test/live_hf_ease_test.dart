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
  test('ghost mode: several picked layers and no keys spread delays by the curve (previewSequence, then sequence)', () async {
    final c = EditorSession()
      ..document.value = {
        'capabilities': ['sequence', 'previewSequence', 'cancelPreview'],
        'selectedIds': [5, 3, 9],
        'selectedKeys': [],
        'layers': [
          {'id': 3, 'ghostable': true},
          {'id': 5, 'ghostable': true},
          {'id': 9, 'ghostable': false},
        ],
        'easeKinds': [
          {'kind': 'Linear', 'samples': [[0, 0], [1, 1]]},
          {'kind': 'Bezier', 'x1': .42, 'y1': 0, 'x2': .58, 'y2': 1, 'samples': [[0, 0], [1, 1]]},
        ],
        'easeIntervals': [],
      };
    final host = LiveEaseHost(c);
    expect(host.sequence, 2, reason: 'the layer that cannot carry a ghost is left out');
    final seg = host.intervals.single;
    seg.setValues([.2, .1, .8, .9]);
    host.preview(0, seg);
    await Future<void>.delayed(Duration.zero);
    expect(sent.last, {'op': 'previewSequence', 'layers': [5, 3], 'shape': {'kind': 'Bezier', 'x1': .2, 'y1': .1, 'x2': .8, 'y2': .9}});
    await host.apply(0, seg);
    expect(sent.last, {'op': 'sequence', 'layers': [5, 3], 'shape': {'kind': 'Bezier', 'x1': .2, 'y1': .1, 'x2': .8, 'y2': .9}});
    expect(sent.where((m) => m['op'] == 'ease'), isEmpty);

    // keys picked: back to shaping their intervals
    c.document.value = {...c.state, 'selectedKeys': [{'layer': 3, 'property': 'opacity', 'frame': 0}]};
    await Future<void>.delayed(Duration.zero);
    expect(host.sequence, 0);
    host.dispose();
  });
}
