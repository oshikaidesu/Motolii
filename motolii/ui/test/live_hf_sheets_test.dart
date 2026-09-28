import 'dart:convert';

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/live_hf/adapters/sheets.dart';
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

  testWidgets('the Composition rows write through composition', (tester) async {
    final c = EditorSession()..document.value = {'width': 1920, 'height': 1080, 'fpsNum': 30, 'fpsDen': 1, 'durationFrames': 300};
    final s = CompositionStore(c);
    expect(s.row('fps')['value'], 4);
    s.set('width', 1280);
    s.set('fps', 2);
    await tester.pump();
    expect(sent, [
      {'op': 'composition', 'width': 1280},
      {'op': 'composition', 'fpsNum': 25, 'fpsDen': 1},
    ]);
    s.dispose();
  });

  testWidgets('Background offers Classic\'s presets; a custom colour highlights none', (tester) async {
    final c = EditorSession()..document.value = {'background': [0.3, 0.1, 0.2, 1.0]};
    final s = CompositionStore(c);
    expect(s.row('background')['choices'], ['Black', 'Dark', 'Grey', 'White']);
    expect(s.row('background')['value'], -1);
    s.set('background', 3);
    await tester.pump();
    expect(sent.last, {'op': 'composition', 'background': [1.0, 1.0, 1.0, 1.0]});
    c.document.value = {'background': [0.5, 0.5, 0.5, 0.0]};
    final t = CompositionStore(c);
    expect(t.row('background')['value'], 2);
    t.set('background', 0);
    await tester.pump();
    expect(sent.last, {'op': 'composition', 'background': [0.0, 0.0, 0.0, 0.0]}, reason: 'a transparent ground stays transparent');
    t.dispose();
    s.dispose();
  });

  testWidgets('the Export sheet shows the frame, the range and the job', (tester) async {
    tester.view.physicalSize = const Size(1200, 900);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final c = EditorSession()
      ..document.value = {
        'width': 1920,
        'height': 1080,
        'fps': 30,
        'durationFrames': 90,
        'markers': [],
        'capabilities': ['export', 'cancelExport'],
        'export': {'phase': 'running', 'done': 12, 'total': 90},
      };
    late BuildContext ctx;
    await tester.pumpWidget(WidgetsApp(
      color: const Color(0xFF000000),
      pageRouteBuilder: <T>(s, b) => PageRouteBuilder<T>(settings: s, pageBuilder: (context, _, __) => b(context)),
      home: Builder(builder: (context) {
        ctx = context;
        return const SizedBox.expand();
      }),
    ));
    showExportSheet(ctx, c);
    await tester.pump();
    expect(find.text('1920 × 1080 · 30.00 fps · MP4'), findsOneWidget);
    expect(find.text('Frames 0 – 90  (3.00 s)'), findsOneWidget);
    expect(find.text('Writing 12 / 90'), findsOneWidget);
    await tester.tap(find.text('Cancel'));
    await tester.pump();
    expect(sent.last, {'op': 'cancelExport'});
  });
}
