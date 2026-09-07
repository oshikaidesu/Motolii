import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/browser.dart';

void main() {
  testWidgets('Square or triangle: tapping the wheel picks a colour', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(320, 600);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final set = <List>[];
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          final args = call.arguments;
          if (args is Map && args['command'] is String) {
            final command = jsonDecode(args['command'] as String) as Map;
            if (command['op'] == 'setColor') set.add(command['rgba'] as List);
          }
          return <String, dynamic>{};
        });
    final c = EditorSession();
    c.document.value = {
      'capabilities': ['applyPalette', 'setColor'],
      'layers': [
        {'id': 'l', 'kind': 'Solid'},
      ],
      'colorTarget': {
        'rgba': [1, 0, 0, 1],
        'layer': 'l',
        'slot': 0,
      },
    };
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: BrowserPanel(controller: c, fixedTab: 'Colors'),
        ),
      ),
    );
    final wheel = find.byWidgetPredicate(
      (w) => w is CustomPaint && '${w.painter.runtimeType}' == '_WheelPainter',
    );
    final center = tester.getCenter(wheel);
    final half = tester.getSize(wheel).width / 2;

    // Square: the centre of the wheel is mid saturation, mid value.
    await tester.tapAt(center);
    await tester.pump(const Duration(seconds: 1));
    expect(set.last[0], closeTo(.5, .05));
    expect(set.last[1], closeTo(.25, .05));

    await tester.tap(find.byKey(const ValueKey('browser:color-shape')));
    await tester.pump(const Duration(seconds: 1));
    expect(c.deskWork.value['colorShape'], 'triangle');

    // Triangle: the hue corner sits at the top for red, so tapping just
    // under the ring there gives nearly pure red.
    await tester.tapAt(center - Offset(0, half - 24));
    await tester.pump(const Duration(seconds: 1));
    expect(set.last[0], greaterThan(.9));
    expect(set.last[1], lessThan(.15));
    // The ring still turns hue with the triangle in place.
    await tester.tapAt(center + Offset(half - 7, 0));
    await tester.pump(const Duration(seconds: 1));
    expect(set.last[1], greaterThan(.9));
  });
}
