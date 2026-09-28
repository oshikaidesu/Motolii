import 'dart:convert';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/hf/bp/colors.dart' show WheelPainter;
import '../lib/live_hf/adapters/colors.dart';
import '../lib/session/editor_session.dart';

void main() {
  final sent = <Map>[];
  setUp(() {
    sent.clear();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(EditorSession.channel, (call) async {
      final args = call.arguments;
      if (args is Map && args['command'] is String) sent.add(jsonDecode(args['command'] as String) as Map);
      return <String, dynamic>{};
    });
  });

  Future<EditorSession> mount(WidgetTester tester) async {
    final c = EditorSession()
      ..document.value = {
        'capabilities': ['setColor', 'previewColor', 'commitPreview', 'cancelPreview'],
        'layers': [
          {
            'id': 3,
            'kind': 'Shape',
            'fill': {
              'slot': 'fill',
              'stops': [
                {'slot': 'fill', 'rgba': [1, 0, 0, 1]},
              ],
            },
          },
        ],
        'selectedId': 3,
        'selectedIds': [3],
      };
    await tester.pumpWidget(Directionality(
      textDirection: TextDirection.ltr,
      child: Align(alignment: Alignment.topLeft, child: LiveColorInstrument(c: c, wheel: 160)),
    ));
    return c;
  }

  testWidgets('the live wheel edits the selection fill: previews while dragging, one write on release', (tester) async {
    await mount(tester);
    final wheel = tester.getRect(find.byKey(const ValueKey('hf-color-wheel')));
    final g = await tester.startGesture(wheel.center);
    await g.moveBy(const Offset(-20, 20));
    await tester.pump();
    final previews = sent.where((m) => m['op'] == 'previewColor').toList();
    expect(previews, isNotEmpty);
    expect(previews.last['layer'], 3);
    expect(previews.last['slot'], 'fill');
    await g.up();
    await tester.pumpAndSettle();
    expect(sent.last['op'], 'commitPreview');
  });

  testWidgets('Esc during a drag returns the colour (cancelPreview)', (tester) async {
    await mount(tester);
    final bar = tester.getRect(find.byKey(const ValueKey('hf-color-value')));
    final g = await tester.startGesture(bar.center);
    await g.moveBy(const Offset(0, 30));
    await tester.pump();
    await tester.sendKeyEvent(LogicalKeyboardKey.escape);
    await tester.pumpAndSettle();
    expect(sent.last['op'], 'cancelPreview');
    await g.up();
    await tester.pumpAndSettle();
    expect(sent.where((m) => m['op'] == 'commitPreview'), isEmpty);
  });

  testWidgets('a typed hex is written at once, and the glyph by it arms the Stage eyedropper', (tester) async {
    final c = await mount(tester);
    expect(find.text('#FF0000'), findsOneWidget, reason: 'the target colour, not the reference placeholder');
    await tester.tap(find.byKey(const ValueKey('hf-color-hex')));
    await tester.pump();
    await tester.enterText(find.byKey(const ValueKey('hf-color-hex')), '00ff00');
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pumpAndSettle();
    final set = sent.lastWhere((m) => m['op'] == 'setColor');
    expect(set['rgba'], [0.0, 1.0, 0.0, 1.0]);
    expect(set['slot'], 'fill');
    await tester.tap(find.byKey(const ValueKey('hf-eyedropper')));
    await tester.pump();
    expect(c.eyedropper.value, isTrue);
    await tester.sendKeyEvent(LogicalKeyboardKey.escape);
    await tester.pump();
    expect(c.eyedropper.value, isFalse);
  });
  testWidgets('with the triangle wheel (the desk colorShape), a press at the white corner previews white', (tester) async {
    final c = await mount(tester);
    c.deskWork.value = {'colorShape': 'triangle'};
    await tester.pump();
    final wheel = tester.getRect(find.byKey(const ValueKey('hf-color-wheel')));
    final corners = WheelPainter.triangleAt(wheel.size, 0); // the target is red: hue 0
    final white = wheel.topLeft + Offset.lerp(wheel.size.center(Offset.zero), corners[1], .97)!;
    final g = await tester.startGesture(white);
    await tester.pump(const Duration(milliseconds: 150)); // the press lands (tap-down deadline)
    final rgba = (sent.lastWhere((m) => m['op'] == 'previewColor')['rgba'] as List).cast<num>();
    for (final v in rgba.take(3)) expect(v.toDouble(), greaterThan(.9));
    await g.up();
    await tester.pumpAndSettle();
  });
}
