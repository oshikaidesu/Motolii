import 'dart:convert';

import 'package:flutter/gestures.dart' show kDoubleTapMinTime;
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/foundation/panel_controls.dart';
import '../lib/input/editor_shortcuts.dart';
import '../lib/session/editor_session.dart';

/// 打ち始めた瞬間に別の数へ化けない。見えていた桁と単位のまま、枡の中で打つ。
Future<void> _open(WidgetTester tester) async {
  final at = tester.getCenter(find.byType(EditorNumericField));
  await tester.tapAt(at);
  await tester.pump(kDoubleTapMinTime);
  await tester.tapAt(at);
  await tester.pumpAndSettle();
}

void main() {
  Widget well(double value, {int decimals = 2, String? unit}) => MaterialApp(
    home: Scaffold(
      body: Center(
        child: SizedBox(
          width: 100,
          child: EditorNumericField(
            value: value,
            label: 'Rotation',
            decimals: decimals,
            unit: unit,
            onPreview: (_) async {},
            onCommit: (_) async {},
            onFinish: () async {},
            onCancel: () async {},
          ),
        ),
      ),
    ),
  );

  testWidgets('typing starts from the digits the well was showing', (
    tester,
  ) async {
    await tester.pumpWidget(well(-18.999999999, decimals: 0, unit: 'px'));
    expect(find.text('-19'), findsOneWidget);

    await _open(tester);

    final field = tester.widget<TextField>(find.byType(TextField));
    expect(field.controller!.text, '-19');
    // 単位の札は打っている間も居る。
    expect(find.text('px'), findsOneWidget);
  });

  testWidgets('the well keeps its box and right edge while typing', (
    tester,
  ) async {
    await tester.pumpWidget(well(0));
    final resting = tester.getRect(find.byType(EditorNumericField));

    await _open(tester);

    expect(tester.getRect(find.byType(EditorNumericField)), resting);
    final field = tester.widget<TextField>(find.byType(TextField));
    expect(field.controller!.text, '0.00');
    expect(field.textAlign, TextAlign.right);
  });

  testWidgets(
    'typing owns keys until Enter; idle Cmd-Z reaches document undo',
    (tester) async {
      final commands = <String>[];
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(EditorSession.channel, (call) async {
            final command = (call.arguments as Map?)?['command'];
            if (command is String)
              commands.add(jsonDecode(command)['op'] as String);
            return <String, dynamic>{};
          });
      final session = EditorSession();
      addTearDown(session.dispose);
      final shortcuts = EditorShortcuts(
        session,
        onMenu: (_) {},
        hasSheet: () => false,
        closeSheet: () {},
        showComposition: () {},
        showInspector: () {},
      );
      final committed = <double>[];
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Focus(
              onKeyEvent: shortcuts.handle,
              child: Center(
                child: SizedBox(
                  width: 100,
                  child: EditorNumericField(
                    value: 960,
                    label: 'Position X',
                    onPreview: (_) async {},
                    onCommit: (value) async {
                      committed.add(value);
                    },
                    onFinish: () async {},
                    onCancel: () async {},
                  ),
                ),
              ),
            ),
          ),
        ),
      );
      await _open(tester);
      expect(find.byType(TextField), findsOneWidget);
      await tester.sendKeyEvent(LogicalKeyboardKey.keyA);
      await tester.pump();
      expect(commands, isEmpty, reason: 'typing A must not enable Animate');
      await tester.enterText(find.byType(TextField), '1100');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pump();
      expect(committed, [1100]);
      expect(find.byType(TextField), findsNothing);
      await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.keyZ);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
      await tester.pumpAndSettle();
      expect(commands.where((op) => op == 'undo'), ['undo']);
    },
  );
}
