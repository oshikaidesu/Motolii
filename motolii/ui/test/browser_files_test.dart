import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import '../lib/session/editor_session.dart';
import '../lib/panels/browser.dart';

void main() {
  late Directory root;
  setUp(() {
    root = Directory.systemTemp.createTempSync('motolii-files');
    Directory('${root.path}/clips').createSync();
    File('${root.path}/still.png').writeAsBytesSync(const [0x89, 0x50, 0x4e, 0x47]);
    File('${root.path}/notes.txt').writeAsStringSync('no');
    File('${root.path}/.hidden.png').writeAsBytesSync(const [0]);
    File('${root.path}/clips/take1.mp4').writeAsBytesSync(const [0]);
  });
  tearDown(() => root.deleteSync(recursive: true));

Future<void> settle(WidgetTester tester) async {
    await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 300)));
    await tester.pumpAndSettle();
  }

  Future<EditorSession> mount(WidgetTester tester) async {
    tester.view.physicalSize = const Size(400, 600);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final calls = <String>[];
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          calls.add('${call.method}:${call.arguments}');
          return <String, dynamic>{};
        });
    final c = EditorSession();
    c.document.value = {
      'capabilities': ['import'],
      'importExtensions': ['png', 'mp4'],
      'assets': [],
    };
    c.deskWork.value = {'browserView': 0};
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: BrowserPanel(
            controller: c,
            fixedTab: 'Files',
            showTabs: false,
            initialFolder: root.path,
          ),
        ),
      ),
    );
    await settle(tester);
    return c;
  }


  testWidgets('A folder shows its folders and the files the shelf can take', (
    tester,
  ) async {
    await mount(tester);
    expect(find.text('clips'), findsOneWidget);
    expect(find.text('still'), findsOneWidget);
    expect(find.text('notes'), findsNothing);
    expect(find.text('.hidden'), findsNothing);
    expect(find.text('PNG'), findsOneWidget);
    // Folders come first.
    expect(
      tester.getTopLeft(find.text('clips')).dx,
      lessThan(tester.getTopLeft(find.text('still')).dx),
    );
    expect(find.byKey(const ValueKey('browser:path')), findsOneWidget);
  });

  testWidgets('Double-clicking a folder walks in; Up and Back walk out', (
    tester,
  ) async {
    await mount(tester);
    await tester.tap(find.byKey(ValueKey('browser:Files:${root.path}/clips')));
    await tester.pump(const Duration(milliseconds: 50));
    await tester.tap(find.byKey(ValueKey('browser:Files:${root.path}/clips')));
    await settle(tester);
    expect(find.text('take1'), findsOneWidget);
    expect(find.text('MP4'), findsOneWidget);
    await tester.tap(find.byIcon(Icons.arrow_upward));
    await settle(tester);
    expect(find.text('clips'), findsOneWidget);
    expect(find.text('take1'), findsNothing);
  });
}
