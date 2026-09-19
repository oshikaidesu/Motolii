import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_stage5/app/editor_app.dart';
import 'package:motolii_stage5/foundation/glyphs.dart';
import 'package:motolii_stage5/foundation/leaves.dart';
import 'package:motolii_stage5/foundation/metrics.dart';
import 'package:motolii_stage5/foundation/panel_controls.dart';
import 'package:motolii_stage5/foundation/theme.dart';

import 'support/editor_test_theme.dart';

Future<void> pump(WidgetTester tester, Widget child) => tester.pumpWidget(
  MaterialApp(
    theme: editorTestTheme,
    builder: EditorApp.noHover,
    home: Scaffold(body: Center(child: child)),
  ),
);

Future<void> hover(WidgetTester tester, Finder target) async {
  final mouse = await tester.createGesture(kind: PointerDeviceKind.mouse);
  await mouse.addPointer(location: Offset.zero);
  addTearDown(mouse.removePointer);
  await tester.pump();
  await mouse.moveTo(tester.getCenter(target));
  await tester.pump(const Duration(seconds: 2));
}

void main() {
  testWidgets('hovering a tooltip shows no panel', (tester) async {
    await pump(
      tester,
      const EditorTooltip(message: 'Rotation', child: Icon(Glyph.circle)),
    );
    await hover(tester, find.byType(Icon));
    expect(find.text('Rotation'), findsNothing);
    // Off means off: no hover region, no long-press detector, no semantics.
    expect(find.byType(RawTooltip), findsNothing);
  });

  testWidgets('a long press shows no tooltip either', (tester) async {
    await pump(
      tester,
      const EditorTooltip(message: 'Rotation', child: Icon(Glyph.circle)),
    );
    await tester.longPress(find.byType(Icon));
    await tester.pump(const Duration(seconds: 2));
    expect(find.text('Rotation'), findsNothing);
  });

  testWidgets('where tooltips are on, hovering shows the sheet', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        theme: editorTestTheme,
        home: const Scaffold(
          body: Center(
            child: EditorTooltip(
              message: 'Rotation',
              child: Icon(Glyph.circle),
            ),
          ),
        ),
      ),
    );
    await hover(tester, find.byType(Icon));
    expect(find.text('Rotation'), findsOneWidget);
  });

  testWidgets('a tap target is the control, not a 48dp square', (tester) async {
    await pump(
      tester,
      Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          EditorTextButton(onPressed: () {}, child: const Text('x')),
          const EditorSwitch(
            on: true,
            glyph: Glyph.circle,
            label: 'on',
            onChanged: null,
          ),
        ],
      ),
    );
    // The Material button under the compact density measured 19 × 12; the
    // switch is its 22 × 12 track and the glyph beside it.
    expect(tester.getSize(find.byType(EditorTextButton)), const Size(19, 12));
    expect(tester.getSize(find.byType(EditorSwitch)).height, EditorMetrics.s14);
  });

  testWidgets('an IconButton is exactly its icon, at any size', (tester) async {
    await pump(
      tester,
      Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          for (final size in [12.0, 14.0, 16.0, 17.0])
            EditorIconButton(
              key: ValueKey(size),
              iconSize: size,
              onPressed: () {},
              icon: const Icon(Glyph.star),
            ),
          EditorIconButton(
            key: const ValueKey('themed'),
            onPressed: () {},
            icon: const Icon(Glyph.star),
          ),
        ],
      ),
    );
    for (final size in [12.0, 14.0, 16.0, 17.0]) {
      expect(
        tester.getSize(find.byKey(ValueKey(size))),
        Size(size, size),
        reason: 'iconSize $size',
      );
    }
    // Unsized, the button is the icon theme's 14, as Material's was.
    expect(
      tester.getSize(find.byKey(const ValueKey('themed'))),
      const Size(EditorMetrics.s14, EditorMetrics.s14),
    );
  });

  testWidgets('the leaves keep the editor\'s own geometry', (tester) async {
    await pump(
      tester,
      Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const SizedBox(width: 100, child: EditorTextField()),
          SizedBox(
            width: 100,
            height: EditorMetrics.dense,
            child: EditorSlider(value: .5, onChanged: (_) {}),
          ),
          const SizedBox(width: 100, child: EditorRule(height: 1)),
        ],
      ),
    );
    // A row of text stays one line of type high; M3's 1.5 geometry would
    // have made it 17.
    expect(
      tester.getSize(find.byType(EditorTextField)).height,
      EditorMetrics.font,
    );
    expect(
      tester.getSize(find.byType(EditorSlider)).height,
      EditorMetrics.dense,
    );
    expect(tester.getSize(find.byType(EditorRule)).height, 1);
    final rule = tester.widget<EditorRule>(find.byType(EditorRule));
    expect(rule.color, EditorTheme.line);
    expect(rule.thickness, 0);
  });
}
