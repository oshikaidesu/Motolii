import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/browser.dart';
import '../lib/panels/gradient_inspector.dart';
import '../lib/foundation/panel_controls.dart';

void main() {
  testWidgets('HEX owns navigation and Enter instead of the palette', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(340, 700);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final commands = <Map>[];
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          final command = (call.arguments as Map?)?['command'];
          if (command is String) commands.add(jsonDecode(command) as Map);
          return <String, dynamic>{};
        });
    final c = EditorSession();
    addTearDown(c.dispose);
    c.document.value = {
      'capabilities': ['setColor', 'applyPalette'],
      'selectedIds': [1],
      'layers': [
        {'id': 1, 'kind': 'Shape', 'name': 'Rectangle'},
      ],
      'colorTarget': {
        'layer': 1,
        'slot': {
          'ShapeFill': {
            'layer': 1,
            'path': [0],
          },
        },
        'rgba': [1.0, 0.0, 0.0, 1.0],
      },
      'palette': [
        {
          'hex': '#ffffff',
          'rgba': [1, 1, 1, 1],
        },
        {
          'hex': '#0000ff',
          'rgba': [0, 0, 1, 1],
        },
      ],
    };
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: BrowserPanel(controller: c, fixedTab: 'Colors'),
        ),
      ),
    );
    final hex = find.descendant(
      of: find.byWidgetPredicate(
        (w) => w is EditorDraftField && w.label == 'hex',
      ),
      matching: find.byType(TextField),
    );
    await tester.tap(hex);
    await tester.enterText(hex, '#00ff00');
    await tester.sendKeyEvent(LogicalKeyboardKey.home);
    await tester.sendKeyEvent(LogicalKeyboardKey.end);
    await tester.pump();
    expect(tester.widget<TextField>(hex).focusNode!.hasFocus, isTrue);
    expect(find.text('HEX · 3 OR 6 DIGITS'), findsOneWidget);
    await tester.sendKeyEvent(LogicalKeyboardKey.enter);
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pumpAndSettle();
    expect(commands.where((m) => m['op'] == 'applyPalette'), isEmpty);
    expect(commands.singleWhere((m) => m['op'] == 'setColor')['rgba'], [
      0.0,
      1.0,
      0.0,
      1.0,
    ]);
  });

  for (final ending in ['release', 'escape', 'selection', 'drop', 'scaled']) {
    testWidgets('held stop keeps its position across rebuilds and $ending', (
      tester,
    ) async {
      final c = EditorSession();
      addTearDown(c.dispose);
      c.document.value = {
        'capabilities': [
          'setGradient',
          'previewProperties',
          'commitPreview',
          'cancelPreview',
        ],
      };
      var layer = 1;
      List<Map<String, dynamic>> rows = [
        for (final (id, offset) in [(2, 0.1), (7, 0.5), (9, 0.9)])
          {
            'id': id,
            'offset': offset,
            'rgba': [1.0, 0.0, 0.0, 1.0],
            'positionProperty': 'fill.stop.$id.offset',
          },
      ];
      final commands = <Map>[];
      late StateSetter rebuild;
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(EditorSession.channel, (call) async {
            final raw = (call.arguments as Map?)?['command'];
            if (raw is String) {
              final op = jsonDecode(raw) as Map;
              commands.add(op);
              if (op['op'] == 'previewProperties') {
                final value = op['edits'][0]['value'];
                rebuild(
                  () => rows = [
                    for (final row in rows)
                      {...row, if (row['id'] == 7) 'offset': value},
                  ],
                );
              }
            }
            return <String, dynamic>{};
          });
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: StatefulBuilder(
              builder: (context, setState) {
                rebuild = setState;
                return Center(
                  child: Transform.scale(
                    scale: ending == 'scaled' ? .65 : 1,
                    child: SizedBox(
                      width: 300,
                      child: GradientInspector(
                        controller: c,
                        layer: {'id': layer},
                        fill: {
                          'slot': {
                            'ShapeFill': {
                              'layer': layer,
                              'path': [0],
                            },
                          },
                          'kind': 'linear',
                          'stops': rows,
                        },
                      ),
                    ),
                  ),
                );
              },
            ),
          ),
        ),
      );
      final handle = find.byKey(const ValueKey('gradient-stop:1'));
      final origin = tester.getCenter(handle);
      final right = tester
          .getCenter(find.byKey(const ValueKey('gradient-stop:2')))
          .dx;
      final pointer = await tester.startGesture(origin);
      final positions = <double>[];
      for (final dx in [25.0, 50.0, 75.0, 50.0, 25.0]) {
        await pointer.moveTo(origin + Offset(dx, 0));
        await tester.pumpAndSettle();
        positions.add(
          (commands.lastWhere(
                    (m) => m['op'] == 'previewProperties',
                  )['edits'][0]['value']
                  as num)
              .toDouble(),
        );
        expect(
          commands.where(
            (m) => m['op'] == 'commitPreview' || m['op'] == 'cancelPreview',
          ),
          isEmpty,
        );
        expect(
          tester.getCenter(handle).dx,
          closeTo((origin.dx + dx).clamp(double.negativeInfinity, right), .01),
        );
      }
      expect(positions.first, positions.last);
      if (ending == 'escape')
        await tester.sendKeyEvent(LogicalKeyboardKey.escape);
      if (ending == 'selection') rebuild(() => layer = 2);
      if (ending == 'drop') await pointer.moveTo(origin + const Offset(25, 60));
      await tester.pumpAndSettle();
      final count = commands
          .where((m) => m['op'] == 'previewProperties')
          .length;
      if (ending == 'escape' || ending == 'selection') {
        await pointer.moveTo(origin + const Offset(80, 0));
        await tester.pumpAndSettle();
        expect(
          commands.where((m) => m['op'] == 'previewProperties').length,
          count,
        );
      }
      await pointer.up();
      await tester.pumpAndSettle();
      expect(
        commands.where((m) => m['op'] == 'commitPreview').length,
        (ending == 'release' || ending == 'scaled') ? 1 : 0,
      );
      expect(
        commands.where((m) => m['op'] == 'cancelPreview').length,
        (ending == 'release' || ending == 'scaled') ? 0 : 1,
      );
      expect(
        commands
            .where((m) => m['op'] == 'previewProperties')
            .every(
              (m) =>
                  m['edits'][0]['layer'] == 1 &&
                  m['edits'][0]['property'] == 'fill.stop.7.offset',
            ),
        isTrue,
      );
      if (ending == 'drop')
        expect(
          commands.singleWhere((m) => m['op'] == 'setGradient')['removeStop'],
          7,
        );
    });
  }

  for (final solid in [true, false]) {
    testWidgets(
      '${solid ? 'solid' : 'gradient'} stop focuses its color owner',
      (tester) async {
        // editor/gradient.rs::model: solids use the fill slot; gradients carry
        // a ShapeGradientPoint slot on each stop.
        final fillSlot = {
          'ShapeFill': {
            'layer': 1,
            'path': [0],
          },
        };
        final stopSlot = {
          'ShapeGradientPoint': {
            'layer': 1,
            'path': [0],
            'index': 0,
          },
        };
        final commands = <Map>[];
        final placements = <Object?>[];
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(EditorSession.channel, (call) async {
              if (call.method == 'placePanel') placements.add(call.arguments);
              final command = (call.arguments as Map?)?['command'];
              if (command is String) commands.add(jsonDecode(command) as Map);
              return <String, dynamic>{};
            });
        final c = EditorSession();
        addTearDown(c.dispose);
        c.document.value = {
          'capabilities': ['focusColor', 'setGradient'],
        };
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: SizedBox(
                width: 300,
                child: GradientInspector(
                  controller: c,
                  layer: const {'id': 1},
                  fill: {
                    'slot': fillSlot,
                    'kind': solid ? 'solid' : 'linear',
                    'stops': [
                      {
                        'offset': 0.0,
                        'rgba': [1.0, 1.0, 1.0, 1.0],
                        if (!solid) 'slot': stopSlot,
                      },
                    ],
                  },
                ),
              ),
            ),
          ),
        );
        await tester.tap(
          find.byKey(ValueKey(solid ? 'gradient-bar' : 'gradient-stop:0')),
        );
        await tester.pumpAndSettle();
        expect(
          commands.singleWhere((m) => m['op'] == 'focusColor')['slot'],
          solid ? fillSlot : stopSlot,
        );
        expect(c.browserTab.value, 'Colors');
        expect(placements, [
          {'name': 'Colors', 'placement': 'show'},
        ]);
        expect(commands.where((m) => m['op'] == 'setGradient'), isEmpty);
      },
    );
  }

  testWidgets('the current fill saves directly without a second stop editor', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(320, 600);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(
          EditorSession.channel,
          (call) async => <String, dynamic>{},
        );
    final c = EditorSession();
    final fillSlot = {
      'ShapeFill': {
        'layer': 1,
        'path': [0],
      },
    };
    c.document.value = {
      'capabilities': ['applyPalette', 'setColor', 'setGradient'],
      'selectedIds': [1],
      'layers': [
        {
          'id': 1,
          'kind': 'Shape',
          'fill': {
            'kind': 'linear',
            'blend': 'oklab',
            'slot': fillSlot,
            'stops': [
              {
                'rgba': [1, 0, 0, 1],
              },
              {
                'rgba': [0, 0, 1, 1],
              },
            ],
          },
        },
      ],
      'colorTarget': {
        'rgba': [1, 0, 0, 1],
        'layer': 1,
        'slot': fillSlot,
      },
      'palette': [
        {
          'hex': '#00ff00',
          'rgba': [0, 1, 0, 1],
        },
      ],
    };
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: BrowserPanel(controller: c, fixedTab: 'Colors'),
        ),
      ),
    );
    // Cards carry no label any more.
    expect(find.text('#00ff00'), findsNothing);

    expect(find.byKey(const ValueKey('browser:stop-add')), findsNothing);
    expect(find.text('FILL TYPE · LINEAR'), findsOneWidget);
    expect(find.text('BLEND · OKLAB'), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('browser:save-current-color')));
    await tester.pump(const Duration(seconds: 1));
    final saved = c.deskWork.value['swatches'] as List;
    expect(saved.length, 1);
    expect((saved.single['stops'] as List).length, 2);
    expect(find.text('Save current color'), findsOneWidget);

    // The saved tile carries the current gradient and its blend.
    final card = find.byKey(const ValueKey('browser:Colors:saved:0'));
    expect(card, findsOneWidget);
    await tester.tap(card);
    await tester.pump(const Duration(seconds: 1));
    expect(saved.single['blend'], 'oklab');
  });
}
