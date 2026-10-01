import 'dart:convert';

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/hf/bp/effects.dart';
import '../lib/live_hf/adapters/browser.dart';
import '../lib/session/editor_session.dart';
import '../lib/hf/shell/place.dart';

void main() {
  final sent = <Map<String, dynamic>>[];
  setUp(() {
    sent.clear();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          if (call.method == 'request') {
            final args = EditorSession.map(call.arguments);
            final command =
                jsonDecode('${args['command']}') as Map<String, dynamic>;
            final op = command['op'];
            sent.add(command);
            if (op == 'fontFacts') {
              return {
                'facts': [
                  {
                    'family': 'Test Sans',
                    'styles': 2,
                    'weights': 2,
                    'monospaced': false,
                    'axes': [],
                    'scripts': ['Latin'],
                    'color': false,
                  },
                ],
              };
            }
            if (op == 'visualSample') return {'image': null};
            return {'ok': true, 'needsRender': false};
          }
          if (call.method == 'readSettings')
            return <String, dynamic>{'deskWork': {}};
          if (call.method == 'writeSettings')
            return <String, dynamic>{'ok': true};
          return <String, dynamic>{'ok': true};
        });
  });

  testWidgets(
    'the five live Browser surfaces keep HF faces and bind real host operations',
    (tester) async {
      tester.view.physicalSize = const Size(800, 700);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.reset);
      final c = EditorSession()
        ..document.value = {
          'capabilities': [
            'create',
            'applyEffect',
            'applyPalette',
            'setGradient',
            'setFont',
            'placeAsset',
            'import',
            'setAttrs',
            'moveKeys',
            'split',
          ],
          'createKinds': [
            {'id': 'text', 'name': 'Text'},
            {'id': 'rectangle', 'name': 'Rectangle'},
          ],
          'catalog': [
            {'id': 'motolii.blur', 'name': 'Blur', 'stage': 'Pass'},
            {
              'id': 'motolii.motion_blur',
              'name': 'Motion Blur',
              'stage': 'Pass',
            },
          ],
          'palette': [
            {
              'id': 'red',
              'hex': '#FF0000',
              'rgba': [1.0, 0.0, 0.0, 1.0],
              'used': true,
            },
          ],
          'fontFamilies': ['Test Sans'],
          'visualSamples': true,
          'assets': [
            {
              'id': 'asset-a',
              'name': 'clip.mov',
              'mime': 'video/mp4',
              'path': '/missing/clip.mov',
            },
          ],
          'layers': [
            {
              'id': 7,
              'name': 'Title',
              'kind': 'Text',
              'locked': false,
              'text': {'fontFamily': 'Test Sans', 'content': 'Hello'},
              'properties': [],
            },
          ],
          'selectedId': 7,
          'selectedIds': [7],
        };
      final scene = await EffectScene.build();
      await tester.pumpWidget(
        Directionality(
          textDirection: TextDirection.ltr,
          child: DefaultTextStyle(
            style: H.s(12),
            child: SizedBox(
              width: 760,
              height: 640,
              child: LiveBrowser(c: c, scene: scene),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('Text'), findsWidgets);
      expect(
        find.text('Motion Blur'),
        findsNothing,
        reason: 'host-only effects stay out of Create',
      );
      await tester.tap(find.text('Effects').last);
      await tester.pumpAndSettle();
      expect(find.text('Motion Blur'), findsWidgets);
      await tester.tap(find.text('Motion Blur').last);
      await tester.pumpAndSettle();
      expect(
        sent.any(
          (command) =>
              command['op'] == 'applyEffect' && command['pluginIds'] != null,
        ),
        isTrue,
      );

      await tester.tap(find.text('Colors').last);
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const ValueKey('hf-color:#FF0000')));
      await tester.pumpAndSettle();
      expect(sent.any((command) => command['op'] == 'applyPalette'), isTrue);

      await tester.tap(find.text('Fonts').last);
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const ValueKey('hf-font:Test Sans')));
      await tester.pumpAndSettle();
      expect(
        sent.any(
          (command) =>
              command['op'] == 'setFont' && command['family'] == 'Test Sans',
        ),
        isTrue,
      );
      expect(
        sent.any(
          (command) =>
              command['op'] == 'visualSample' && command['kind'] == 'font',
        ),
        isTrue,
      );

      await tester.tap(find.text('Media').last);
      await tester.pumpAndSettle();
      expect(find.text('clip.mov'), findsOneWidget);
      final asset = find.text('clip.mov');
      await tester.tap(asset);
      await tester.pump(const Duration(milliseconds: 40));
      await tester.tap(asset);
      await tester.pumpAndSettle();
      expect(
        sent.any(
          (command) =>
              command['op'] == 'placeAsset' && command['id'] == 'asset-a',
        ),
        isTrue,
      );

      await tester.pumpWidget(const SizedBox());
      c.dispose();
    },
  );
}
