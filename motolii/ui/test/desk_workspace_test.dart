import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/registry.dart';

void main() {
  testWidgets(
    'desk drafts and presets work without targets and survive tool changes',
    (tester) async {
      Map<String, dynamic> settings = {};
      final calls = <String>[];
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(EditorSession.channel, (call) async {
            calls.add(call.method);
            if (call.method == 'readSettings') return settings;
            if (call.method == 'writeSettings') {
              settings = Map<String, dynamic>.from(call.arguments as Map);
              return true;
            }
            throw StateError('Unexpected document action ${call.method}');
          });
      final c = EditorSession();
      c.document.value = {
        'layers': [],
        'selectedIds': [],
        'selectedKeys': [],
        'assets': [],
        'markers': [],
        'easeKinds': [],
      };
      c.deskDrawer.value = 'Ease';
      await tester.pumpWidget(
        MaterialApp(
          home: SizedBox(
            width: 600,
            height: 600,
            child: buildPanel('Desk', c, null),
          ),
        ),
      );
      await tester.tap(find.text('Save preset'));
      await tester.pumpAndSettle();
      expect((c.deskWork.value['easePresets'] as List).length, 1);
      // Blend keeps no draft of its own: with nothing selected its tiles are
      // inert, and visiting it must not disturb the drafts of other tools.
      c.deskDrawer.value = 'Blend';
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const ValueKey('blend:Multiply')));
      await tester.pumpAndSettle();
      expect(c.deskWork.value.containsKey('blends'), isFalse);
      c.deskDrawer.value = 'Ease';
      await tester.pumpAndSettle();
      expect((c.deskWork.value['easePresets'] as List).length, 1);
      expect(((settings['deskWork'] as Map)['easePresets'] as List).length, 1);
      expect(calls.where((v) => v == 'request'), isEmpty);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    },
  );
}
