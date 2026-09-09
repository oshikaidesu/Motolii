import 'dart:convert';
import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/foundation/theme.dart';
import '../lib/panels/blend_panel.dart';
import '../lib/session/editor_session.dart';

void main() {
  testWidgets('the desk shows samples, previews on hover and commits once', (
    tester,
  ) async {
    final c = EditorSession();
    final sent = <Map<String, dynamic>>[];
    var mode = 'Normal';
    var selection = [1, 2, 3];
    Map<String, dynamic> snapshot() => {
      'path': '/work.motolii',
      'layers': [
        {
          'id': 1,
          'name': 'Shape',
          'kind': 'Shape',
          'blendMode': mode,
          'blendPreviews': {
            for (final name in BlendPanelState.modes)
              name: [
                [0.1, 0.1, 0.1],
                [0.3, 0.3, 0.3],
                [0.6, 0.6, 0.6],
                [0.9, 0.9, 0.9],
                [0.2, 0.4, 0.8],
                [0.9, 0.5, 0.2],
              ],
          },
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
      'capabilities': ['setAttrs', 'previewBlend', 'cancelPreview'],
    };
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          if (call.method == 'request') {
            final command = Map<String, dynamic>.from(
              jsonDecode(call.arguments['command']),
            );
            sent.add(command);
            if (command['op'] == 'setAttrs')
              mode = command['patch']['blendMode'];
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
            // The narrow drawer size a Desk gives this panel.
            child: SizedBox(
              width: 260,
              height: 200,
              child: BlendPanel(controller: c),
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    // Every mode is reachable in a 260-wide panel, and each tile carries the
    // sample the document produced rather than a paragraph about the mode.
    for (final name in BlendPanelState.modes) {
      final tile = find.byKey(ValueKey('blend:$name'));
      expect(tile, findsOneWidget, reason: name);
      await tester.scrollUntilVisible(tile, 40);
    }
    expect(
      find.descendant(
        of: find.byKey(const ValueKey('blend:Screen')),
        matching: find.byType(ColoredBox),
      ),
      findsNWidgets(6),
    );
    // The only words are the mode names themselves.
    final words = tester
        .widgetList<Text>(
          find.descendant(
            of: find.byType(BlendPanel),
            matching: find.byType(Text),
          ),
        )
        .map((t) => t.data);
    expect(words.toSet().difference(BlendPanelState.modes.toSet()), isEmpty);

    await tester.scrollUntilVisible(
      find.byKey(const ValueKey('blend:Screen')),
      -40,
    );
    final screen = find.byKey(const ValueKey('blend:Screen'));
    final mouse = await tester.createGesture(kind: PointerDeviceKind.mouse);
    await mouse.addPointer(location: const Offset(700, 400));

    // Hover previews the last selected editable layer only — never the camera
    // and never the locked layer.
    await mouse.moveTo(tester.getCenter(screen));
    await tester.pumpAndSettle();
    expect(sent.last, {'op': 'previewBlend', 'layer': 1, 'mode': 'Screen'});
    expect(sent.where((s) => s['op'] == 'setAttrs'), isEmpty);

    // Leaving puts the document back.
    await mouse.moveTo(const Offset(700, 400));
    await tester.pumpAndSettle();
    expect(sent.last['op'], 'cancelPreview');

    // One click, one setAttrs; clicking the mode it already has adds nothing.
    await tester.tap(screen);
    await tester.pumpAndSettle();
    final applied = sent.where((s) => s['op'] == 'setAttrs').toList();
    expect(applied.single['layers'], [1]);
    expect(applied.single['patch'], {'blendMode': 'Screen'});
    // Clicking the mode the layer already has records nothing, and still
    // hands the document back instead of leaving the preview standing.
    await mouse.moveTo(tester.getCenter(screen));
    await tester.pumpAndSettle();
    expect(sent.last['op'], 'previewBlend');
    await tester.tap(screen);
    await tester.pumpAndSettle();
    expect(sent.where((s) => s['op'] == 'setAttrs').length, 1);
    expect(sent.last['op'], 'cancelPreview');
    await mouse.moveTo(const Offset(700, 400));
    await tester.pumpAndSettle();

    // A selection change while hovering releases the preview it owns.
    await mouse.moveTo(tester.getCenter(screen));
    await tester.pumpAndSettle();
    selection = [2, 3];
    c.document.value = snapshot();
    await tester.pumpAndSettle();
    expect(sent.last['op'], 'cancelPreview');
    await tester.tap(screen);
    await tester.pumpAndSettle();
    expect(sent.where((s) => s['op'] == 'setAttrs').length, 1);

    expect(tester.takeException(), isNull);
    await mouse.removePointer();
    await tester.pumpWidget(const SizedBox());
    await tester.pumpAndSettle();
    c.dispose();
  });
}
