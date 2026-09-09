import 'dart:convert';
import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/foundation/theme.dart';
import '../lib/panels/blend_panel.dart';
import '../lib/session/editor_session.dart';

void main() {
  testWidgets(
    'fixed specimens never follow frame changes; hover cancels, click commits',
    (tester) async {
      final c = EditorSession();
      final sent = <Map<String, dynamic>>[];
      final samples = <Map<String, dynamic>>[];
      var mode = 'Normal';
      var selection = [1, 2, 3];
      int? owner;
      String? interaction;
      Map<String, dynamic> snapshot() => {
        'layers': [
          {
            'id': 1,
            'name': 'Shape',
            'kind': 'Shape',
            'blendMode': mode,
          },
          {'id': 2, 'name': 'Camera', 'kind': 'Camera', 'blendMode': 'Normal'},
          {
            'id': 3,
            'name': 'Locked',
            'kind': 'Shape',
            'locked': true,
            'blendMode': 'Normal',
          },
        ],
        'selectedIds': selection,
        'visualSamples': true,
        'previewOwner': owner,
        'previewInteraction': interaction,
        'preview': owner != null,
        'capabilities': [
          'setAttrs',
          'previewBlend',
          'cancelPreview',
        ],
      };
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(EditorSession.channel, (call) async {
            if (call.method == 'request') {
              final command = Map<String, dynamic>.from(
                jsonDecode(call.arguments['command']),
              );
              if (command['op'] == 'visualSample') {
                samples.add(command);
                return {
                  'image': 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aD1sAAAAASUVORK5CYII=',
                };
              }
              sent.add(command);
              switch (command['op']) {
                case 'previewBlend':
                  owner = 7;
                  interaction = command['interaction'];
                case 'cancelPreview':
                  owner = null;
                  interaction = null;
                case 'setAttrs':
                  mode = command['patch']['blendMode'];
              }
            }
            return snapshot();
          });
      c.document.value = snapshot();
      await tester.pumpWidget(
        MaterialApp(
          theme: EditorTheme.data,
          home: Scaffold(
            body: Align(
              alignment: Alignment.topLeft,
              child: SizedBox(
                width: 600,
                height: 600,
                child: BlendPanel(controller: c),
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      final specimens = samples.length;
      expect(specimens, greaterThanOrEqualTo(4));
      expect(
        samples.every(
          (s) => s['kind'] == 'blendBehavior' && !s.containsKey('layers'),
        ),
        isTrue,
      );
      c.frame.value = 24;
      c.document.value = {...snapshot(), 'documentRevision': 'edited'};
      await tester.pumpAndSettle();
      expect(samples.length, specimens);
      // Shapes only: the desk shows no words or numbers.
      expect(
        find.descendant(
          of: find.byType(BlendPanel),
          matching: find.byWidgetPredicate((w) => w is Text || w is TextField),
        ),
        findsNothing,
      );
      final screen = find.byKey(const ValueKey('blend:Screen'));
      expect(screen.hitTestable(), findsOneWidget);
      final mouse = await tester.createGesture(kind: PointerDeviceKind.mouse);
      await mouse.addPointer(location: const Offset(700, 300));
      await mouse.moveTo(tester.getCenter(screen));
      await tester.pumpAndSettle();
      expect(sent.last['op'], 'previewBlend');
      expect(sent.last['layers'], [1]);
      expect(sent.where((s) => s['op'] == 'setAttrs'), isEmpty);
      await mouse.moveTo(const Offset(700, 300));
      await tester.pumpAndSettle();
      expect(sent.last, {'op': 'cancelPreview', 'owner': 7});
      await tester.tap(screen);
      await tester.pumpAndSettle();
      expect(sent.where((s) => s['op'] == 'setAttrs').single['patch'], {
        'blendMode': 'Screen',
      });
      await tester.tap(screen);
      await tester.pumpAndSettle();
      expect(sent.where((s) => s['op'] == 'setAttrs').length, 1);
      // Changing selection while hovering cancels the owned preview.
      await mouse.moveTo(tester.getCenter(screen));
      await tester.pumpAndSettle();
      selection = [2, 3];
      c.document.value = snapshot();
      await tester.pumpAndSettle();
      expect(sent.last['op'], 'cancelPreview');
      await tester.tap(screen);
      await tester.pumpAndSettle();
      expect(sent.where((s) => s['op'] == 'setAttrs').length, 1);
      expect(samples.length, specimens);
      expect(tester.takeException(), isNull);
      await mouse.removePointer();
      await tester.pumpWidget(const SizedBox());
      await tester.pumpAndSettle();
      c.dispose();
    },
  );
}
