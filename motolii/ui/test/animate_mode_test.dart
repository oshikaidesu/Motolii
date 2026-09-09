import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/input/editor_shortcuts.dart';
import '../lib/foundation/theme.dart';
import '../lib/panels/panel_settings.dart';

void main() {
  List<Map> sent() => _commands.map(jsonDecode).whereType<Map>().toList();
  setUp(() {
    _commands.clear();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          final args = call.arguments;
          if (args is Map && args['command'] is String) {
            _commands.add(args['command'] as String);
          }
          return <String, dynamic>{};
        });
  });

  testWidgets('A alone toggles Animate; Shift+A still reaches Anchor', (
    tester,
  ) async {
    final c = EditorSession();
    final root = FocusNode();
    final shortcuts = EditorShortcuts(
      c,
      onMenu: (_) {},
      hasSheet: () => false,
      closeSheet: () {},
      showComposition: () {},
      showInspector: () {},
    );
    await tester.pumpWidget(
      MaterialApp(
        home: Focus(
          focusNode: root,
          onKeyEvent: shortcuts.handle,
          child: const SizedBox(),
        ),
      ),
    );
    root.requestFocus();
    await tester.pump();
    await tester.sendKeyEvent(LogicalKeyboardKey.keyA);
    await tester.pump();
    expect(sent().last['op'], 'animate');
    expect(sent().last['enabled'], isTrue);
    expect(sent().last['from'], isFalse);

    c.document.value = {'animate': true};
    c.deskWork.value = {'animateFrom': true};
    await tester.sendKeyEvent(LogicalKeyboardKey.keyA);
    await tester.pump();
    expect(sent().last['enabled'], isFalse);
    expect(sent().last['from'], isTrue);

    final before = sent().length;
    await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
    await tester.sendKeyEvent(LogicalKeyboardKey.keyA);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
    await tester.pump();
    expect(sent().length, before);
    expect(c.focusProperty.value, 'anchor');
    await tester.pumpWidget(const SizedBox());
    root.dispose();
  });

  testWidgets('Settings owns the start-key choice', (tester) async {
    final c = EditorSession();
    await tester.pumpWidget(
      MaterialApp(home: Scaffold(body: PanelSettings(controller: c))),
    );
    expect(c.deskWork.value['animateFrom'], isNull);
    await tester.tap(find.byKey(const ValueKey('settings:animateFrom')));
    await tester.pump();
    expect(c.deskWork.value['animateFrom'], isTrue);
  });

  test('the accent flips its hue while Animate is on', () {
    EditorTheme.animating.value = false;
    expect(EditorTheme.accent, EditorTheme.design);
    EditorTheme.animating.value = true;
    expect(EditorTheme.accent, EditorTheme.animate);
    expect(
      HSLColor.fromColor(EditorTheme.animate).lightness,
      closeTo(HSLColor.fromColor(EditorTheme.design).lightness, .01),
    );
    expect(
      (HSLColor.fromColor(EditorTheme.animate).hue -
              HSLColor.fromColor(EditorTheme.design).hue)
          .abs(),
      closeTo(180, 1),
    );
    EditorTheme.animating.value = false;
  });
}

final _commands = <String>[];
