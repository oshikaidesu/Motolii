import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/foundation/panel_controls.dart';
import '../lib/panels/inspector.dart';
import '../lib/session/editor_session.dart';
import 'support/editor_test_theme.dart';
import 'support/window_fixture.dart';

Future<(EditorSession, ValueNotifier<double>)> mountColumns(
  WidgetTester tester,
) async {
  tester.view.physicalSize = const Size(800, 2000);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
      .setMockMethodCallHandler(
        EditorSession.channel,
        (_) async => <String, dynamic>{},
      );
  final c = EditorSession();
  final width = ValueNotifier(320.0);
  addTearDown(c.dispose);
  addTearDown(width.dispose);
  final source = windowLayer(1, 'Shape', 0);
  c.document.value = {
    ...windowStatus(1, 0),
    'layers': [
      {
        ...source,
        'effects': [],
        'properties': [
          ...(source['properties'] as List),
          {
            ...windowNumber('layout.position_type', 'Position Type', 0),
            'choices': ['Relative', 'Absolute'],
          },
          for (final id in [
            'column_start',
            'row_start',
            'column_span',
            'row_span',
          ])
            windowNumber('layout.$id', id, 1),
          for (var i = 0; i < 6; i++)
            windowNumber('layout.sample_$i', 'Sample $i', i.toDouble()),
        ],
      },
    ],
  };
  c.deskWork.value = {'inspectorCell': 88.0};
  await tester.pumpWidget(
    MaterialApp(
      theme: editorTestTheme,
      home: Align(
        alignment: Alignment.topLeft,
        child: ValueListenableBuilder(
          valueListenable: width,
          builder: (context, value, _) => SizedBox(
            width: value,
            height: 1900,
            child: InspectorPanel(
              key: const ValueKey('inspector'),
              controller: c,
            ),
          ),
        ),
      ),
    ),
  );
  await tester.pumpAndSettle();
  return (c, width);
}

Rect well(WidgetTester tester, String id, [int axis = 0]) =>
    tester.getRect(find.byKey(ValueKey('inspector:$id:$axis')));

void main() {
  testWidgets(
    'row spans share both value edges and keep the accessory rail on resize',
    (tester) async {
      final (c, width) = await mountColumns(tester);
      final state = tester.state(
        find.byKey(const ValueKey('inspector:position:0')),
      );
      final document = c.document.value;
      double previousWide = 0;
      for (final size in [180.0, 244.0, 320.0, 560.0]) {
        width.value = size;
        await tester.pumpAndSettle();
        final x = well(tester, 'position'),
            y = well(tester, 'position', 1),
            z = well(tester, 'position.z');
        final parent = tester.getRect(
          find.byKey(const ValueKey('inspector:parent')),
        );
        final blend = tester.getRect(
          find.byKey(const ValueKey('inspector:blend')),
        );
        expect(parent.left, closeTo(x.left, .01));
        expect(parent.right, closeTo(z.right, .01));
        expect(blend.left, closeTo(x.left, .01));
        expect(blend.right, closeTo(z.right, .01));
        expect(y.width, closeTo(x.width, .01));
        expect(z.width, closeTo(x.width, .01));
        expect(well(tester, 'opacity').right, closeTo(y.right, .01));
        final cell = [
          for (final id in [
            'column_start',
            'row_start',
            'column_span',
            'row_span',
          ])
            well(tester, 'layout.$id'),
        ];
        expect(cell.first.left, closeTo(x.left, .01));
        expect(cell.last.right, closeTo(z.right, .01));
        for (final rect in cell)
          expect(rect.width, closeTo(cell.first.width, .01));
        final dial = tester.getRect(find.byType(EditorDial).first);
        expect(dial.right, closeTo(size - EditorCard.contentInset, .01));
        expect(dial.left - z.right, closeTo(y.left - x.right, .01));
        if (size == 320) previousWide = x.width;
        if (size == 560) expect(x.width, greaterThan(previousWide));
        expect(
          tester.state(find.byKey(const ValueKey('inspector:position:0'))),
          same(state),
        );
        expect(c.document.value, same(document));
        expect(tester.takeException(), isNull);
      }
      await tester.pumpWidget(const SizedBox());
    },
  );

  testWidgets(
    'advanced tiles fill their grid and cell-size changes preserve outer edges',
    (tester) async {
      final (c, width) = await mountColumns(tester);
      await tester.tap(find.byType(EditorFold));
      await tester.pumpAndSettle();
      for (final size in [180.0, 320.0, 520.0]) {
        width.value = size;
        await tester.pumpAndSettle();
        final rects = [
          for (var i = 0; i < 6; i++) well(tester, 'layout.sample_$i'),
        ];
        final firstRow = rects
            .where((r) => (r.top - rects.first.top).abs() < .01)
            .toList();
        expect(firstRow.first.left, closeTo(EditorCard.contentInset, .01));
        expect(
          firstRow.last.right,
          closeTo(size - EditorCard.contentInset, .01),
        );
        for (final rect in rects)
          expect(rect.width, closeTo(rects.first.width, .01));
        if (firstRow.length > 1) {
          final x = well(tester, 'position'), y = well(tester, 'position', 1);
          expect(
            firstRow[1].left - firstRow[0].right,
            closeTo(y.left - x.right, .01),
          );
        }
        expect(tester.takeException(), isNull);
      }
      width.value = 320;
      c.deskWork.value = {'inspectorCell': 200.0};
      await tester.pumpAndSettle();
      final single = well(tester, 'layout.sample_0');
      expect(single.left, closeTo(EditorCard.contentInset, .01));
      expect(single.right, closeTo(320 - EditorCard.contentInset, .01));
      await tester.pumpWidget(const SizedBox());
    },
  );
}
