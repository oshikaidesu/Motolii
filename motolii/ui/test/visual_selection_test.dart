import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/foundation/theme.dart';
import '../lib/panels/gradient_inspector.dart';
import '../lib/panels/font_browser.dart';
import '../lib/panels/browser.dart';
import '../lib/session/editor_session.dart';

void main() {
  testWidgets(
    'gradient modes and every stop reach Document and Colors without a popup',
    (tester) async {
      final sent = <Map<String, dynamic>>[];
      final placements = <dynamic>[];
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(EditorSession.channel, (call) async {
            if (call.method == 'placePanel') placements.add(call.arguments);
            if (call.arguments is Map && call.arguments['command'] is String)
              sent.add(jsonDecode(call.arguments['command']));
            return <String, dynamic>{};
          });
      final c = EditorSession();
      final slot = {
        'ShapeFill': {
          'layer': 1,
          'path': [0],
        },
      };
      final points = [
        for (var i = 0; i < 3; i++)
          {
            'offset': i / 2,
            'rgba': [i / 2, 0.0, 1 - i / 2, 1.0],
            'slot': {
              'ShapeGradientPoint': {
                'layer': 1,
                'path': [0],
                'index': i,
              },
            },
          },
      ];
      c.document.value = {
        'capabilities': [
          'setGradient',
          'focusColor',
          'commitPreview',
          'cancelPreview',
        ],
        'layers': [],
      };
      await tester.pumpWidget(
        MaterialApp(
          theme: EditorTheme.data,
          home: Scaffold(
            body: SizedBox(
              width: 300,
              child: GradientInspector(
                controller: c,
                layer: {'id': 1},
                fill: {
                  'kind': 'linear',
                  'slot': slot,
                  'stops': points,
                  'angle': 0,
                },
              ),
            ),
          ),
        ),
      );
      await tester.tap(find.byKey(const ValueKey('fill-mode:radial')));
      await tester.pumpAndSettle();
      expect(sent.last['op'], 'setGradient');
      expect(sent.last['kind'], 'radial');
      await tester.tap(find.byKey(const ValueKey('gradient-stop:1')));
      await tester.pumpAndSettle();
      expect(sent.last, {
        'op': 'focusColor',
        'layer': 1,
        'slot': points[1]['slot'],
      });
      expect(placements.single, {'name': 'Colors', 'placement': 'show'});
      expect(find.byType(MenuAnchor), findsNothing);
      await tester.tap(find.byTooltip('Add stop'));
      await tester.pumpAndSettle();
      expect((sent.last['stops'] as List).length, 4);
      sent.clear();
      final pointer = await tester.startGesture(
        tester.getCenter(find.byKey(const ValueKey('gradient-stop:1'))),
      );
      await pointer.moveBy(const Offset(20, 0));
      await tester.pump();
      await pointer.moveBy(const Offset(35, 0));
      await tester.pump();
      await pointer.up();
      await tester.pumpAndSettle();
      expect(sent.where((r) => r['op'] == 'commitPreview').length, 1);
      final preview = sent.lastWhere(
        (r) => r['op'] == 'setGradient' && r['preview'] == true,
      );
      expect((preview['stops'][1]['offset'] as num), greaterThan(.5));
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    },
  );

  testWidgets('font samples are requested only for visible specimen rows', (
    tester,
  ) async {
    final calls = <Map<String, dynamic>>[];
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          if (call.arguments is Map && call.arguments['command'] is String)
            calls.add(jsonDecode(call.arguments['command']));
          return <String, dynamic>{};
        });
    final c = EditorSession();
    c.document.value = {
      'visualSamples': true,
      'path': 'sample.rrd',
      'layers': [
        {
          'id': 1,
          'kind': 'Text',
          'name': 'Caption',
          'text': {'content': 'Selected words', 'fontFamily': 'Font 0'},
        },
      ],
      'selectedIds': [1],
      'fontFamilies': [for (var i = 0; i < 100; i++) 'Font $i'],
      'capabilities': ['setFont'],
    };
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: SizedBox(
            width: 300,
            height: 300,
            child: FontBrowser(controller: c),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    expect(calls, isNotEmpty);
    expect(calls.length, lessThan(10));
    expect(calls.every((r) => r['op'] == 'visualSample'), isTrue);
    final count = calls.length;
    await tester.pump();
    expect(calls.length, count);
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });

  testWidgets('a saved gradient applies all colors to the current shape fill', (
    tester,
  ) async {
    final sent = <Map<String, dynamic>>[];
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          if (call.arguments is Map && call.arguments['command'] is String)
            sent.add(jsonDecode(call.arguments['command']));
          return <String, dynamic>{};
        });
    final c = EditorSession();
    final slot = {
      'ShapeFill': {
        'layer': 1,
        'path': [0],
      },
    };
    c.document.value = {
      'layers': [
        {
          'id': 1,
          'kind': 'Shape',
          'fill': {'slot': slot},
        },
      ],
      'selectedIds': [1],
      'palette': [],
      'assets': [],
      'capabilities': ['setGradient'],
    };
    c.deskWork.value = {
      'swatches': [
        {
          'name': 'Red blue',
          'stops': [
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
          ],
        },
      ],
    };
    await tester.pumpWidget(
      MaterialApp(
        theme: EditorTheme.data,
        home: Scaffold(
          body: SizedBox(
            width: 400,
            height: 700,
            child: BrowserPanel(
              controller: c,
              fixedTab: 'Colors',
              showTabs: false,
            ),
          ),
        ),
      ),
    );
    final card = find.byKey(const ValueKey('browser:Colors:saved:0'));
    await tester.ensureVisible(card);
    await tester.tap(card);
    await tester.pumpAndSettle();
    expect(
      sent.any(
        (m) => m['op'] == 'setGradient' && (m['stops'] as List).length == 2,
      ),
      isTrue,
    );
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });
}
