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
    expect(sent().last['from'], isTrue);
    expect(sent().last['shape'], EditorSession.easyEase);

    c.document.value = {'animate': true};
    c.deskWork.value = {'animateFrom': false};
    await tester.sendKeyEvent(LogicalKeyboardKey.keyA);
    await tester.pump();
    expect(sent().last['enabled'], isFalse);
    expect(sent().last['from'], isFalse);

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

  testWidgets('Shift+Cmd+D brings the previous key selection back', (
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
    final span = [
      {'layer': 1, 'property': 'position', 'frame': 30},
      {'layer': 1, 'property': 'position', 'frame': 60},
    ];
    c.absorb({
      'selectedIds': [1],
      'selectedKeys': span,
    });
    expect(c.previousKeys, isNull);
    c.absorb({
      'selectedIds': [2],
      'selectedKeys': [],
    });
    expect(c.previousKeys, {
      'ids': [1],
      'keys': span,
    });
    await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
    await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
    await tester.sendKeyEvent(LogicalKeyboardKey.keyD);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
    await tester.pump();
    expect(sent().last['op'], 'select');
    expect(sent().last['ids'], [1]);
    expect(sent().last['keys'], span);
    await tester.pumpWidget(const SizedBox());
    root.dispose();
  });

  testWidgets('Settings owns the start-key choice, on by default', (
    tester,
  ) async {
    final c = EditorSession();
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(body: PanelSettings(controller: c)),
      ),
    );
    expect(c.animateFrom, isTrue);
    await tester.tap(find.byKey(const ValueKey('settings:animateFrom')));
    await tester.pump();
    expect(c.deskWork.value['animateFrom'], isFalse);
    expect(c.animateFrom, isFalse);
  });

  test('only the key accent flips its hue while Animate is on', () {
    EditorTheme.animating.value = false;
    expect(EditorTheme.keyAccent, EditorTheme.accent);
    EditorTheme.animating.value = true;
    expect(EditorTheme.keyAccent, EditorTheme.animate);
    expect(EditorTheme.accent, const Color(0xffffaa61));
    expect(
      HSLColor.fromColor(EditorTheme.animate).lightness,
      closeTo(HSLColor.fromColor(EditorTheme.accent).lightness, .01),
    );
    expect(
      (HSLColor.fromColor(EditorTheme.animate).hue -
              HSLColor.fromColor(EditorTheme.accent).hue)
          .abs(),
      closeTo(180, 1),
    );
    EditorTheme.animating.value = false;
  });
}

final _commands = <String>[];
