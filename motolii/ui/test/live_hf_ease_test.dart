import 'dart:convert';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/hf/desk/ease.dart';
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

    // one picked key has no interval to shape: still the ghost mode (Classic's rule)
    c.document.value = {...c.state, 'selectedKeys': [{'layer': 3, 'property': 'opacity', 'frame': 0}]};
    await Future<void>.delayed(Duration.zero);
    expect(host.sequence, 2);
    // an interval to shape: back to shaping it
    c.document.value = {
      ...c.state,
      'easeIntervals': [
        {'layer': 3, 'property': 'opacity', 'frame': 0, 'end': 10, 'shape': {'kind': 'Linear'}},
      ],
    };
    await Future<void>.delayed(Duration.zero);
    expect(host.sequence, 0);
    host.dispose();
  });
  test('kept curves are the desk settings Classic keeps: copied curve, saved presets, new-key shape', () async {
    final c = EditorSession()
      ..document.value = {
        'easeKinds': [
          {'kind': 'Linear', 'samples': [[0, 0], [1, 1]]},
          {'kind': 'Bezier', 'x1': .42, 'y1': 0, 'x2': .58, 'y2': 1, 'samples': [[0, 0], [1, 1]]},
        ],
      };
    final host = LiveEaseHost(c);
    expect(host.saved, isEmpty, reason: 'no demo curves pretend to be saved');
    final seg = Seg(1, 1, host.kinds)..setValues([.1, .2, .3, .4]);
    host.copyCurve(seg);
    host.savePreset(seg);
    host.useForNewKeys(seg);
    expect(c.deskWork.value['curveClip'], {'kind': 'Bezier', 'x1': .1, 'y1': .2, 'x2': .3, 'y2': .4});
    expect(c.deskWork.value['easePresets'], hasLength(1));
    expect(c.deskWork.value['newKeyShape'], {'kind': 'Bezier', 'x1': .1, 'y1': .2, 'x2': .3, 'y2': .4});
    expect(host.saved, hasLength(2));
    host.clearSaved();
    expect(host.saved, hasLength(1), reason: 'the bin clears the saved presets, the copied curve stays');
    host.dispose();
  });
  test('the desk says what it edits, follows the playhead, and keeps a workspace curve without applying it', () async {
    final c = EditorSession()
      ..document.value = {
        'capabilities': ['ease'],
        'layers': [
          {'id': 3, 'name': 'Title'},
        ],
        'easeKinds': [
          {'kind': 'Linear', 'samples': [[0, 0], [1, 1]]},
          {'kind': 'Bezier', 'x1': .42, 'y1': 0, 'x2': .58, 'y2': 1, 'samples': [[0, 0], [1, 1]]},
        ],
        'easeIntervals': [
          {'layer': 3, 'property': 'opacity', 'frame': 0, 'end': 10, 'shape': {'kind': 'Linear'}},
          {'layer': 3, 'property': 'opacity', 'frame': 10, 'end': 20, 'shape': {'kind': 'Bezier'}},
        ],
      };
    final host = LiveEaseHost(c);
    expect(host.target, 'Title · opacity · 0–10 · 2 intervals · Mixed');
    c.frame.value = 15;
    expect(host.current, 1);
    expect(host.playhead(1), closeTo(.5, 1e-9));

    c.document.value = {...c.state, 'easeIntervals': []};
    await Future<void>.delayed(Duration.zero);
    expect(host.target, 'No interval · Workspace');
    expect(host.caption, 'Workspace');
    final seg = host.intervals.single..setValues([.3, 0, .7, 1]);
    sent.clear();
    await host.apply(0, seg);
    expect(sent.where((m) => m['op'] == 'ease'), isEmpty, reason: 'the workspace curve is kept, not applied');
    expect(c.deskWork.value['ease'], {'kind': 'Bezier', 'x1': .3, 'y1': 0, 'x2': .7, 'y2': 1});
    host.dispose();
  });
  testWidgets('a kind other than Bezier is grabbed by its own handles: the host remodels it, release writes it', (tester) async {
    final asked = <Map>[];
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(EditorSession.channel, (call) async {
      final args = call.arguments;
      if (call.method == 'easeModel') {
        asked.add(args as Map);
        return <String, dynamic>{'kind': 'Bounce', 'bounces': 5.0, 'handles': [[.5, .9]], 'samples': [[0, 0], [1, 1]]};
      }
      if (args is Map && args['command'] is String) sent.add(jsonDecode(args['command'] as String) as Map);
      return <String, dynamic>{};
    });
    final bounce = {'kind': 'Bounce', 'bounces': 3.0, 'handles': [[.5, .5]], 'samples': [[0, 0], [1, 1]]};
    final c = EditorSession()
      ..document.value = {
        'capabilities': ['ease', 'select'],
        'selectedIds': [3],
        'layers': [{'id': 3, 'name': 'Title'}],
        'easeKinds': [
          {'kind': 'Linear', 'samples': [[0, 0], [1, 1]]},
          {'kind': 'Bezier', 'x1': .42, 'y1': 0, 'x2': .58, 'y2': 1, 'samples': [[0, 0], [1, 1]]},
          bounce,
        ],
        'easeIntervals': [
          {'layer': 3, 'property': 'opacity', 'frame': 0, 'end': 10, 'shape': bounce},
        ],
      };
    tester.view.physicalSize = const Size(420, 760);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    await tester.pumpWidget(Directionality(textDirection: TextDirection.ltr, child: LiveEase(c: c)));
    await tester.pumpAndSettle();
    final plot = find.byKey(const ValueKey('ease-plot'));
    final box = tester.getRect(plot);
    final handle = box.topLeft + easeAt(box.size, .5, .5);
    final g = await tester.startGesture(handle);
    await g.moveBy(const Offset(0, -20));
    await tester.pump();
    await tester.pump();
    expect(asked, isNotEmpty);
    expect(asked.last['handle'], 0);
    expect((asked.last['shape'] as Map)['kind'], 'Bounce');
    await g.up();
    await tester.pumpAndSettle();
    final ease = sent.lastWhere((m) => m['op'] == 'ease');
    expect(ease['kind'], 'Bounce');
    expect(ease['bounces'], 5.0);
    await tester.pumpWidget(const SizedBox());
  });
  testWidgets('a curve number is typed (or scrubbed) and written as the curve', (tester) async {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(EditorSession.channel, (call) async {
      final args = call.arguments;
      if (args is Map && args['command'] is String) sent.add(jsonDecode(args['command'] as String) as Map);
      return <String, dynamic>{};
    });
    final c = EditorSession()
      ..document.value = {
        'capabilities': ['ease', 'select'],
        'selectedIds': [3],
        'layers': [{'id': 3, 'name': 'Title'}],
        'easeKinds': [
          {'kind': 'Linear', 'samples': [[0, 0], [1, 1]]},
          {'kind': 'Bezier', 'x1': .42, 'y1': 0, 'x2': .58, 'y2': 1, 'samples': [[0, 0], [1, 1]]},
        ],
        'easeIntervals': [
          {'layer': 3, 'property': 'opacity', 'frame': 0, 'end': 10, 'shape': {'kind': 'Bezier', 'x1': .42, 'y1': 0, 'x2': .58, 'y2': 1}},
        ],
      };
    tester.view.physicalSize = const Size(420, 900);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    await tester.pumpWidget(Directionality(textDirection: TextDirection.ltr, child: LiveEase(c: c)));
    await tester.pumpAndSettle();
    final x1 = find.byKey(const ValueKey('ease-value-X1'));
    await tester.ensureVisible(x1);
    await tester.tap(x1);
    await tester.pump();
    await tester.enterText(find.descendant(of: x1, matching: find.byType(EditableText)), '0.3');
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pumpAndSettle();
    final ease = sent.lastWhere((m) => m['op'] == 'ease');
    expect(ease['kind'], 'Bezier');
    expect(ease['x1'], closeTo(.3, 1e-9));
    await tester.pumpWidget(const SizedBox());
  });
  test('a kept curve of another kind keeps its own numbers when it is read back and written again', () async {
    final bounce = {'kind': 'Bounce', 'bounces': 3.0, 'handles': [[.5, .5]], 'samples': [[0, 0], [1, 1]]};
    final c = EditorSession()
      ..document.value = {
        'capabilities': ['ease'],
        'easeKinds': [
          {'kind': 'Linear', 'samples': [[0, 0], [1, 1]]},
          bounce,
        ],
        'easeIntervals': [],
      };
    c.deskWork.value = {'ease': {'kind': 'Bounce', 'bounces': 5.0}};
    final host = LiveEaseHost(c);
    final seg = host.intervals.single;
    expect(seg.model!['bounces'], 5.0, reason: 'the kept number, not the kind default');
    await host.apply(0, seg);
    expect(c.deskWork.value['ease'], {'kind': 'Bounce', 'bounces': 5.0});
    host.dispose();
  });
}
