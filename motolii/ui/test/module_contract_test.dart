import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/bridge/protocol.dart';
import '../lib/session/editor_session.dart';
import 'package:docking/docking.dart';

import '../lib/input/window_keys.dart';
import '../lib/workspace/dock_workspace.dart';

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
  group('workspace keeps one instance of each panel and survives a save', () {
    DockWorkspace make() => DockWorkspace(
      {
        for (final id in const ['Stage', 'Inspector', 'Timeline', 'Colors'])
          id: PanelDef(id, id, () => const SizedBox()),
      },
      (item) => DockingRow([
        item('Stage'),
        DockingTabs([item('Inspector'), item('Timeline'), item('Colors')]),
      ]),
    );
    List<String> tabs(DockWorkspace w) {
      final ids = <String>[];
      void walk(DockingArea? a) {
        if (a is DockingItem) ids.add('${a.id}');
        if (a is DockingParentArea) a.forEach(walk);
      }
      walk(w.layout.root);
      return ids;
    }

    test('close then show puts a panel back exactly once', () {
      final w = make();
      w.close('Colors');
      expect(w.isOpen('Colors'), isFalse);
      w.activate('Colors');
      w.activate('Colors');
      expect(tabs(w).where((id) => id == 'Colors'), hasLength(1));
      expect(tabs(w).toSet(), {'Stage', 'Inspector', 'Timeline', 'Colors'});
    });
    test('a saved workspace round-trips through JSON, front tab included', () {
      final w = make();
      w.activate('Timeline');
      final saved = jsonDecode(jsonEncode(w.snapshot()));
      final again = make();
      expect(again.restore(saved), isTrue);
      expect(again.snapshot(), w.snapshot());
      expect(again.isShown('Timeline'), isTrue);
      expect(tabs(again)..sort(), tabs(w)..sort());
    });
    test('an unreadable saved workspace leaves the current one alone', () {
      final w = make();
      final before = w.snapshot();
      final good = w.snapshot();
      expect(w.restore(null), isFalse);
      expect(w.restore({...good, 'version': 99}), isFalse);
      expect(w.restore({...good, 'layout': 'not a layout'}), isFalse);
      expect(w.restore({...good, 'layout': (good['layout'] as String).replaceAll('Stage', 'Gone')}), isFalse);
      expect(w.snapshot(), before);
    });
  });
  testWidgets(
    'text input keeps keyboard ownership and repeated view commands notify',
    (tester) async {
      final c = EditorSession();
      final root = FocusNode();
      final text = FocusNode();
      final home = GlobalKey();
      var viewCalls = 0;
      final keys = LiveKeys(c, () => home.currentContext!);
      c.viewCommand.addListener(() {
        if (c.viewCommand.value != null) viewCalls++;
      });
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            key: home,
            body: Focus(
              focusNode: root,
              onKeyEvent: keys.handle,
              child: TextField(focusNode: text),
            ),
          ),
        ),
      );
      text.requestFocus();
      await tester.pump();
      await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.digit0);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
      expect(viewCalls, 0);
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
