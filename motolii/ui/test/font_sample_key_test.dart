import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/panels/font_browser.dart';
import '../lib/session/editor_session.dart';

Map<String, dynamic> _style({
  double size = 20,
  List<double> fill = const [1, 1, 1, 1],
}) => {
  'id': 0,
  'font': {'path': '/a.ttf', 'family': 'Arial', 'style': 'Regular'},
  'size': size,
  'fill': fill,
  'line_height': null,
  'tracking': 0.0,
  'stroke_color': null,
  'stroke_width': 0.0,
  'stroke_over_fill': false,
  'axes': [],
  'features': [],
};

Map<String, dynamic> _text({
  String content = 'One two three',
  double size = 20,
  List<double> fill = const [1, 1, 1, 1],
}) => {
  'content': content,
  'fontFamily': 'Arial',
  'size': size,
  'classes': ['word'],
  'runs': [
    {'style': 0, 'start': 0, 'end': 13},
  ],
  'styles': [_style(size: size, fill: fill)],
};

void main() {
  test('a specimen key holds what the renderer draws with, and nothing else', () {
    final plain = fontSampleKey(_text());
    // The renderer overrides fill and stroke, so colour is not in the picture.
    expect(fontSampleKey(_text(fill: const [1, 0, 0, 1])), plain);
    // Every style is scaled so the first lands at a fixed size.
    expect(fontSampleKey(_text(size: 60)), plain);
    // Only the first line's opening words are drawn.
    expect(
      fontSampleKey(_text(content: 'One two three\nfourth line')),
      plain,
    );
    expect(
      fontSampleKey(_text(content: 'One two four')),
      isNot(plain),
    );
  });

  testWidgets('touching a format redraws no specimen the shelf already holds', (
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
    Map<String, dynamic> document(String name, List<double> fill) => {
      'visualSamples': true,
      'path': 'sample.rrd',
      'layers': [
        {
          'id': 1,
          'kind': 'Text',
          'name': name,
          'text': _text(fill: fill),
        },
      ],
      'selectedIds': [1],
      'fontFamilies': [for (var i = 0; i < 24; i++) 'Font $i'],
      'capabilities': ['setFont'],
    };
    c.document.value = document('Caption', const [1, 1, 1, 1]);
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
    Future<void> settle() async {
      for (var i = 0; i < 8; i++) {
        await tester.pump(const Duration(milliseconds: 200));
      }
      await tester.pumpAndSettle();
    }

    Future<void> readTheShelf() async {
      for (var step = 0; step < 8; step++) {
        await tester.drag(find.byType(ListView), const Offset(0, -200));
        await settle();
      }
      for (var step = 0; step < 8; step++) {
        await tester.drag(find.byType(ListView), const Offset(0, 200));
        await settle();
      }
    }

    await settle();
    await readTheShelf();
    final drawn = calls.length;
    expect(drawn, greaterThan(8), reason: 'the shelf was read end to end');
    calls.clear();
    // The shelf is rebuilt for its own reasons; only the colour moved.
    c.document.value = document('Renamed', const [1, 0, 0, 1]);
    await settle();
    await readTheShelf();
    debugPrint('PROBE room=font-samples drawn=$drawn redrawn=${calls.length}');
    expect(calls, isEmpty);
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });
}
