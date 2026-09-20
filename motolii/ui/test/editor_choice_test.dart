import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_stage5/foundation/metrics.dart';
import 'package:motolii_stage5/foundation/theme.dart';

import 'support/editor_test_theme.dart';

import 'package:motolii_stage5/foundation/leaves.dart';

void main() {
  testWidgets('a choice opens app-height menu rows and reports the pick', (
    tester,
  ) async {
    Object? picked;
    await tester.pumpWidget(
      MaterialApp(
        theme: editorTestTheme,
        home: Scaffold(
          body: SizedBox(
            width: EditorMetrics.s200,
            child: EditorChoice<int>(
              value: 0,
              choices: const [
                MapEntry(0, 'Line'),
                MapEntry(1, 'Circle'),
                MapEntry(2, 'Grid'),
              ],
              onChanged: (v) => picked = v,
            ),
          ),
        ),
      ),
    );
    expect(find.text('Line'), findsOneWidget);
    expect(find.text('Grid'), findsNothing);
    await tester.tap(find.text('Line'));
    await tester.pumpAndSettle();
    expect(find.byType(EditorMenuRow), findsNWidgets(3));
    final row = tester.getSize(find.byType(EditorMenuRow).first);
    expect(row.height, EditorMetrics.row);
    // The sheet and its type are the app's menu, not Material 3's.
    final grid = tester.renderObject<RenderParagraph>(find.text('Grid'));
    expect(grid.text.style?.fontSize, EditorMetrics.font);
    expect(grid.text.style?.color, EditorTheme.chromatic.ink);
    final sheet = tester
        .widgetList<Container>(find.byType(Container))
        .map((c) => c.decoration)
        .whereType<BoxDecoration>()
        .firstWhere((d) => d.color == EditorTheme.chromatic.menu);
    expect(sheet.border?.top.color, EditorTheme.chromatic.menuEdge);
    await tester.tap(find.text('Grid'));
    await tester.pumpAndSettle();
    expect(picked, 2);
    expect(find.byType(EditorMenuRow), findsNothing);
  });

  testWidgets('a disabled choice shows the value and does not open', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        theme: editorTestTheme,
        home: Scaffold(
          body: EditorChoice<int>(
            value: 1,
            choices: const [MapEntry(0, 'Line'), MapEntry(1, 'Circle')],
            onChanged: null,
          ),
        ),
      ),
    );
    expect(find.text('Circle'), findsOneWidget);
    await tester.tap(find.text('Circle'));
    await tester.pumpAndSettle();
    expect(find.byType(EditorMenuRow), findsNothing);
  });
}
