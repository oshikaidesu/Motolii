import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/app/editor_app.dart';
import '../lib/foundation/theme.dart';
import '../lib/panels/ease_desk.dart';
import '../lib/panels/registry.dart';
import '../lib/panels/timeline.dart';
import '../lib/session/editor_session.dart';

/// The panels below stopped rebuilding themselves for every snapshot; each
/// now wakes on a fingerprint of its own. A fingerprint that misses something
/// fails silently — the panel simply stops following the document — so every
/// one of them is asked here to show a value, take a new one, and show that.
Future<void> _mount(
  WidgetTester tester,
  String panel,
  EditorSession c,
  Size size,
) => tester.pumpWidget(
  MaterialApp(
    theme: EditorTheme.data,
    builder: EditorApp.noHover,
    home: Scaffold(
      body: Align(
        alignment: Alignment.topLeft,
        child: SizedBox.fromSize(
          size: size,
          child: buildPanel(panel, c, const ValueKey('panel')),
        ),
      ),
    ),
  ),
);

Color? _inkOf(WidgetTester tester, Key tile, String label) => tester
    .widgetList<Text>(
      find.descendant(of: find.byKey(tile), matching: find.byType(Text)),
    )
    .firstWhere((t) => t.data == label)
    .style
    ?.color;

void main() {
  setUp(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(
          EditorSession.channel,
          (call) async => <String, dynamic>{},
        );
  });

  testWidgets('the Blend tiles follow the mode the document reports', (
    tester,
  ) async {
    final c = EditorSession();
    addTearDown(c.dispose);
    Map<String, dynamic> snapshot(String mode) => {
      'path': '/work.motolii',
      'layers': [
        {'id': 1, 'name': 'Shape', 'kind': 'Shape', 'blendMode': mode},
      ],
      'selectedIds': const [1],
      'capabilities': const ['setAttrs', 'previewBlend', 'cancelPreview'],
    };
    c.document.value = snapshot('Normal');
    await _mount(tester, 'Blend', c, const Size(300, 400));
    await tester.pumpAndSettle();
    const normal = ValueKey('blend:Normal'), screen = ValueKey('blend:Screen');
    expect(_inkOf(tester, normal, 'Normal'), EditorTheme.accent);
    expect(_inkOf(tester, screen, 'Screen'), EditorTheme.muted);

    c.document.value = snapshot('Screen');
    await tester.pump();
    expect(_inkOf(tester, screen, 'Screen'), EditorTheme.accent);
    expect(_inkOf(tester, normal, 'Normal'), EditorTheme.muted);
  });

  testWidgets('the Desk follows the selection to the tool it needs', (
    tester,
  ) async {
    final c = EditorSession();
    addTearDown(c.dispose);
    Map<String, dynamic> snapshot(List<Map<String, dynamic>> keys) => {
      'layers': [
        {
          'id': 1,
          'name': 'One',
          'kind': 'Shape',
          'locked': false,
          'properties': [
            {
              'id': 'opacity',
              'label': 'Opacity',
              'kind': 'number',
              'value': 1.0,
              'keys': const [
                {
                  'frame': 0,
                  'interp': {'kind': 'Linear'},
                },
                {
                  'frame': 30,
                  'interp': {'kind': 'Linear'},
                },
              ],
              'keyedNow': true,
            },
          ],
          'effects': const [],
        },
      ],
      'selectedId': 1,
      'selectedIds': const [1],
      'selectedKeys': keys,
      'easeKinds': const [],
      'capabilities': const ['ease'],
      'frame': 0,
      'fps': 30.0,
      'durationFrames': 60,
    };
    c.document.value = snapshot(const []);
    await _mount(tester, 'Desk', c, const Size(300, 260));
    await tester.pumpAndSettle();
    expect(find.byType(EaseDesk), findsNothing);

    c.document.value = snapshot(const [
      {'layer': 1, 'property': 'opacity', 'frame': 0},
    ]);
    await tester.pumpAndSettle();
    expect(find.byType(EaseDesk), findsOneWidget);
  });

  testWidgets('the Ease desk follows the head and the curve it is given', (
    tester,
  ) async {
    final c = EditorSession();
    addTearDown(c.dispose);
    Map<String, dynamic> snapshot(String kind) => {
      'layers': [
        {
          'id': 1,
          'name': 'One',
          'kind': 'Shape',
          'locked': false,
          'properties': [
            {
              'id': 'opacity',
              'label': 'Opacity',
              'kind': 'number',
              'value': 1.0,
              'keys': [
                {
                  'frame': 0,
                  'interp': {'kind': kind},
                },
                {
                  'frame': 30,
                  'interp': {'kind': 'Linear'},
                },
              ],
              'keyedNow': true,
            },
          ],
          'effects': const [],
        },
      ],
      'selectedId': 1,
      'selectedIds': const [1],
      'selectedKeys': const [
        {'layer': 1, 'property': 'opacity', 'frame': 0},
      ],
      'easeKinds': const [],
      'capabilities': const ['ease'],
      'frame': 0,
      'fps': 30.0,
      'durationFrames': 60,
    };
    c.document.value = snapshot('Linear');
    await _mount(tester, 'Ease', c, const Size(300, 320));
    await tester.pumpAndSettle();
    String kindOnScreen() =>
        (tester
                    .widgetList<CustomPaint>(
                      find.descendant(
                        of: find.byKey(const ValueKey('ease-plot')),
                        matching: find.byType(CustomPaint),
                      ),
                    )
                    .firstWhere((p) => p.painter is EaseCurvePainter)
                    .painter!
                as EaseCurvePainter)
            .shape['kind']
            .toString();
    String frameOnScreen() => tester
        .widget<Text>(find.byKey(const ValueKey('ease-current-frame')))
        .data!;

    expect(kindOnScreen(), 'Linear');
    expect(frameOnScreen(), startsWith('0 f'));

    // The head moves without a new snapshot.
    c.frame.value = 15;
    await tester.pump();
    expect(frameOnScreen(), startsWith('15 f'));

    // A new curve arrives on the same keys.
    c.document.value = snapshot('Hold');
    await tester.pump();
    expect(kindOnScreen(), 'Hold');
  });

  testWidgets('the Timeline lanes follow the selection', (tester) async {
    final c = EditorSession();
    addTearDown(c.dispose);
    Map<String, dynamic> snapshot(List<int> selected) => {
      'layers': [
        for (var i = 1; i <= 2; i++)
          {
            'id': i,
            'order': i,
            'name': 'Layer $i',
            'kind': 'Shape',
            'locked': false,
            'start': 0,
            'sourceIn': 0,
            'duration': 60,
            'colors': const [],
            'contentKeys': const [],
            'properties': const [],
            'effects': const [],
          },
      ],
      'selectedId': selected.first,
      'selectedIds': selected,
      'selectedKeys': const [],
      'capabilities': const [],
      'frame': 0,
      'fps': 30.0,
      'durationFrames': 60,
    };
    c.document.value = snapshot(const [1]);
    await _mount(tester, 'Timeline', c, const Size(900, 260));
    await tester.pumpAndSettle();
    CustomPainter lanes() => tester
        .widget<CustomPaint>(find.byKey(const ValueKey('timeline-lanes')))
        .painter!;
    final before = lanes();

    c.document.value = snapshot(const [2]);
    await tester.pump();
    final after = lanes();
    expect(identical(before, after), isFalse);
    expect(
      after.shouldRepaint(before),
      isTrue,
      reason: 'the lanes must redraw for the layer that took the selection',
    );

    // 横に流すのは板を建て直さず painter を起こす道なので、その道を確かめる。
    final dynamic panel = tester.state(find.byType(TimelinePanel));
    panel.scrolled.value++;
    await tester.pump();
    expect(identical(after, lanes()), isFalse);
  });
}
