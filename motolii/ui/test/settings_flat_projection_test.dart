import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/panel_settings.dart';

/// Settings「New layers」: 選ぶと desk に残り、native へ preferences として 1 回だけ届く。
void main() {
  testWidgets('choosing 3D for new layers reaches native once', (tester) async {
    final commands = <Map>[];
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          final args = call.arguments;
          if (args is Map && args['command'] is String) {
            commands.add(jsonDecode(args['command'] as String) as Map);
          }
          return <String, dynamic>{};
        });
    final c = EditorSession();
    c.deskWork.value = {'flatProjection': '2.5D'};
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(body: PanelSettings(controller: c)),
      ),
    );
    await tester.pumpAndSettle();
    expect(c.flatProjection, '2.5D');
    c.deskWork.value = {...c.deskWork.value, 'flatProjection': '3D'};
    c.deskWork.value = {...c.deskWork.value, 'unrelated': 1};
    await tester.pumpAndSettle();
    final prefs = commands.where((m) => m['op'] == 'preferences').toList();
    expect(prefs.map((m) => m['flatProjection']).toList(), ['2.5D', '3D']);
    expect(c.flatProjection, '3D');
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });
}
