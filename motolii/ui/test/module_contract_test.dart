import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/bridge/protocol.dart';
import '../lib/session/editor_session.dart';
import '../lib/workspace/layout.dart';
import '../lib/workspace/panel_ids.dart';
import '../lib/input/editor_shortcuts.dart';

void main() {
  test(
    'document operations agree with native and cannot override their tag',
    () {
      final native = File('native/src/port.rs').readAsStringSync();
      final list = RegExp(
        r'CAPABILITIES[^=]*=\s*&\[(.*?)\]',
        dotAll: true,
      ).firstMatch(native)!.group(1)!;
      final names = RegExp(r'"([^"]+)"')
          .allMatches(list)
          .map((m) => m.group(1)!)
          .toSet();
      expect(DocumentOperation.values.map((op) => op.wireName).toSet(), names);
      for (final op in DocumentOperation.values) {
        expect(jsonDecode(op.encode({'layer': 7})), {
          'op': op.wireName,
          'layer': 7,
        });
        expect(() => op.encode({'op': 'delete'}), throwsArgumentError);
      }
      expect(
        () => DocumentOperation.parse('notAnOperation'),
        throwsArgumentError,
      );
    },
  );
  test(
    'workspace move keeps one panel instance and survives serialization',
    () {
      final layout = WorkspaceLayout();
      for (final name in paneNames) {
        layout.show(name);
      }
      final target = layout.root.leaves.firstWhere(
        (n) => n.tabs.contains('Stage'),
      );
      layout.move('Colors', target, 'right');
      var tabs = layout.root.leaves.expand((n) => n.tabs).toList();
      expect(tabs.toSet(), paneNames.toSet());
      expect(tabs.length, paneNames.length);
      layout.root = DockNode.read(jsonDecode(jsonEncode(layout.root.json())));
      layout.close('Colors');
      layout.show('Colors');
      tabs = layout.root.leaves.expand((n) => n.tabs).toList();
      expect(tabs.where((n) => n == 'Colors').length, 1);
      expect(tabs.toSet(), paneNames.toSet());
    },
  );
  testWidgets(
    'text input keeps keyboard ownership and repeated view commands notify',
    (tester) async {
      final c = EditorSession();
      final root = FocusNode();
      final text = FocusNode();
      var menuCalls = 0, viewCalls = 0;
      final shortcuts = EditorShortcuts(
        c,
        onMenu: (_) => menuCalls++,
        hasSheet: () => false,
        closeSheet: () {},
        showComposition: () {},
        showInspector: () {},
      );
      c.viewCommand.addListener(() {
        if (c.viewCommand.value != null) viewCalls++;
      });
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Focus(
              focusNode: root,
              onKeyEvent: shortcuts.handle,
              child: TextField(focusNode: text),
            ),
          ),
        ),
      );
      text.requestFocus();
      await tester.pump();
      await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.keyS);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
      expect(menuCalls, 0);
      root.requestFocus();
      await tester.pump();
      await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.digit0);
      await tester.sendKeyEvent(LogicalKeyboardKey.digit0);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
      expect(viewCalls, 2);
      await tester.pumpWidget(const SizedBox());
      root.dispose();
      text.dispose();
    },
  );
}
