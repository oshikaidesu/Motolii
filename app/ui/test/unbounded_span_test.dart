import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/timeline.dart';

void main() {
  testWidgets('Timeline publishes its visible span and create carries it', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(900, 500);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final sent = <Map<String, dynamic>>[];
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          final args = call.arguments;
          if (args is Map && args['command'] is String)
            sent.add(
              Map<String, dynamic>.from(
                jsonDecode(args['command'] as String) as Map,
              ),
            );
          return <String, dynamic>{};
        });
    final c = EditorSession();
    c.document.value = {
      'fps': 30,
      'durationFrames': 300,
      'capabilities': ['create'],
      'selectedIds': [],
      'selectedKeys': [],
      'layers': [],
    };
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(body: TimelinePanel(controller: c)),
      ),
    );
    await tester.pump();
    final visible = c.visibleFrames.value;
    expect(visible, isNotNull);
    expect(visible, greaterThan(0));

    await c.command('create', {'kind': 'rectangle'});
    final create = sent.singleWhere((m) => m['op'] == 'create');
    expect(create['visibleFrames'], visible);
    await tester.pumpWidget(const SizedBox());
  });
}
