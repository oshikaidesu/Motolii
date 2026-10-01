import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/browser.dart';

void main() {
  testWidgets('Importing opens Media and selects what came in', (tester) async {
    tester.view.physicalSize = const Size(1600, 900);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final commands = <Map<String, dynamic>>[];
    final asset = {
      'id': 'a1',
      'name': 'logo.png',
      'path': '/tmp/logo.png',
      'mime': 'image/png',
      'used': false,
      'missing': false,
      'role': 'material',
    };
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          final args = call.arguments;
          if (args is Map && args['command'] is String) {
            final command = jsonDecode(args['command'] as String) as Map;
            if (command['op'] == 'import') {
              commands.add(Map<String, dynamic>.from(command));
              return {
                'status': {
                  'assets': [asset],
                },
              };
            }
          }
          return <String, dynamic>{};
        });
    final c = EditorSession();
    c.document.value = {
      'capabilities': ['import', 'placeAsset'],
      'importExtensions': ['png', 'mp4'],
      'assets': [],
    };
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(body: BrowserPanel(controller: c)),
      ),
    );
    expect(find.text('logo'), findsNothing);

    await c.importPaths(['/tmp/logo.png', '/tmp/photo.heic']);
    await tester.pump();

    expect(commands.single['paths'], ['/tmp/logo.png']);
    expect(c.error.value, 'Not supported: photo.heic');
    expect(c.importedAssets.value, ['a1']);
    expect(find.text('logo'), findsOneWidget);
  });
}
