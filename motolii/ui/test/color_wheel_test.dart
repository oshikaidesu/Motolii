import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/browser.dart';
import '../lib/panels/browser/color_wheel.dart';

void main() {
  testWidgets('triangle keeps its hue while a held handle crosses white', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(340, 700);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final c = EditorSession();
    addTearDown(c.dispose);
    c.deskWork.value = {'colorShape': 'triangle'};
    final sent = <Map>[];
    Map<String, dynamic> model(List rgba) => {
      'capabilities': ['previewColor', 'setColor', 'applyPalette'],
      'selectedIds': [1],
      'layers': [
        {'id': 1, 'name': 'Green', 'kind': 'Shape'},
      ],
      'colorTarget': {
        'layer': 1,
        'slot': {
          'ShapeFill': {
            'layer': 1,
            'path': [0],
          },
        },
        'rgba': rgba,
      },
      'palette': <Map>[],
    };
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          final raw = (call.arguments as Map?)?['command'];
          if (raw is String) {
            final op = jsonDecode(raw) as Map;
            sent.add(op);
            if (op['op'] == 'previewColor') {
              c.document.value = model(List.of(op['rgba'] as List));
            }
          }
          return <String, dynamic>{};
        });
    const hue = 85.0;
    final initial = HSVColor.fromAHSV(1, hue, .35, .92).toColor();
    c.document.value = model([initial.r, initial.g, initial.b, 1.0]);
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: BrowserPanel(controller: c, fixedTab: 'Colors'),
        ),
      ),
    );
    final paint = find.byWidgetPredicate(
      (w) => w is CustomPaint && w.painter is ColorWheelPainter,
    );
    final painter =
        tester.widget<CustomPaint>(paint).painter! as ColorWheelPainter;
    final origin = tester.getTopLeft(paint);
    final start =
        origin + painter.wheel.innerHandle(HSVColor.fromColor(initial));
    final white = origin + painter.wheel.triangle(hue)[1];
    final pointer = await tester.startGesture(start);
    for (var i = 1; i <= 8; i++) {
      await pointer.moveTo(Offset.lerp(start, white, i / 8)!);
      await tester.pumpAndSettle();
    }
    // Keep holding and move away from white again. The chosen green hue must
    // return; white itself cannot supply a hue for the next event.
    await pointer.moveTo(Offset.lerp(white, start, .2)!);
    await tester.pumpAndSettle();
    final rgba =
        (sent.lastWhere((m) => m['op'] == 'previewColor')['rgba'] as List)
            .cast<num>();
    expect(rgba[1], greaterThan(rgba[0]));
    expect(rgba[1], greaterThan(rgba[2]));
    final shown =
        tester.widget<CustomPaint>(paint).painter! as ColorWheelPainter;
    expect(shown.hue, closeTo(hue, 1));
    await pointer.up();
    await tester.pumpAndSettle();
  });

  for (final ending in ['release', 'escape', 'selection', 'inactive']) {
    testWidgets('held colour handle follows snapshots and ends by $ending', (
      tester,
    ) async {
      tester.view.physicalSize = const Size(340, 700);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.reset);
      final c = EditorSession();
      addTearDown(c.dispose);
      final sent = <Map>[];
      Map<String, dynamic> model(List rgba, {int layer = 1}) => {
        'capabilities': ['previewColor', 'setColor', 'applyPalette'],
        'selectedIds': [layer],
        'layers': [
          {'id': layer, 'name': 'Layer $layer', 'kind': 'Shape'},
        ],
        'colorTarget': {
          'layer': layer,
          'slot': {
            'ShapeFill': {
              'layer': layer,
              'path': [0],
            },
          },
          'rgba': rgba,
        },
        'palette': <Map>[],
      };
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(EditorSession.channel, (call) async {
            final raw = (call.arguments as Map?)?['command'];
            if (raw is String) {
              final op = jsonDecode(raw) as Map;
              sent.add(op);
              if (op['op'] == 'previewColor')
                c.document.value = model(List.of(op['rgba'] as List));
            }
            return <String, dynamic>{};
          });
      c.document.value = model([1.0, 0.0, 0.0, 1.0]);
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: BrowserPanel(controller: c, fixedTab: 'Colors'),
          ),
        ),
      );
      final wheel = find.byWidgetPredicate(
        (w) => w is CustomPaint && w.painter is ColorWheelPainter,
      );
      final center = tester.getCenter(wheel);
      final pointer = await tester.startGesture(center);
      final colours = <List>[];
      for (final x in [20.0, 30.0, 40.0, 30.0, 20.0]) {
        await pointer.moveTo(center + Offset(x, 0));
        await tester.pumpAndSettle();
        final previews = sent.where((s) => s['op'] == 'previewColor').toList();
        expect(previews, isNotEmpty);
        colours.add(List.of(previews.last['rgba'] as List));
        expect(
          sent.where(
            (s) => s['op'] == 'commitPreview' || s['op'] == 'cancelPreview',
          ),
          isEmpty,
          reason: 'a fresh snapshot of the same colour owner cannot end the gesture',
        );
      }
      expect(
        colours.first,
        colours.last,
        reason: 'returning to the same point has no drift',
      );
      expect(colours[1], colours[3]);
      expect(colours[0], isNot(colours[2]));
      final count = sent.where((s) => s['op'] == 'previewColor').length;
      if (ending == 'escape')
        await tester.sendKeyEvent(LogicalKeyboardKey.escape);
      if (ending == 'selection')
        c.document.value = model([0.0, 1.0, 0.0, 1.0], layer: 2);
      if (ending == 'inactive')
        tester.binding.handleAppLifecycleStateChanged(
          AppLifecycleState.inactive,
        );
      await tester.pumpAndSettle();
      if (ending != 'release') {
        await pointer.moveTo(center + const Offset(-20, 0));
        await tester.pumpAndSettle();
        expect(
          sent.where((s) => s['op'] == 'previewColor').length,
          count,
          reason: 'the held pointer cannot start editing a new owner after cancellation',
        );
      }
      await pointer.up();
      await tester.pumpAndSettle();
      expect(
        sent.where((s) => s['op'] == 'commitPreview').length,
        ending == 'release' ? 1 : 0,
      );
      expect(
        sent.where((s) => s['op'] == 'cancelPreview').length,
        ending == 'release' ? 0 : 1,
      );
      expect(
        sent
            .where((s) => s['op'] == 'previewColor')
            .every((s) => s['layer'] == 1),
        isTrue,
      );
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    });
  }

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
      (w) => w is CustomPaint && w.painter is ColorWheelPainter,
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
