import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_stage5/app/editor_app.dart';
import 'package:motolii_stage5/foundation/metrics.dart';
import 'package:motolii_stage5/foundation/theme.dart';

Future<void> pump(WidgetTester tester, Widget child) => tester.pumpWidget(
  MaterialApp(
    theme: EditorTheme.data,
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
      const Tooltip(message: 'Rotation', child: Icon(Icons.circle)),
    );
    await hover(tester, find.byType(Icon));
    expect(find.text('Rotation'), findsNothing);
  });

  testWidgets('a long press shows no tooltip either', (tester) async {
    await pump(
      tester,
      const Tooltip(message: 'Rotation', child: Icon(Icons.circle)),
    );
    await tester.longPress(find.byType(Icon));
    await tester.pump(const Duration(seconds: 2));
    expect(find.text('Rotation'), findsNothing);
  });

  testWidgets('a tap target is the control, not a 48dp square', (tester) async {
    await pump(
      tester,
      Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Checkbox(value: true, onChanged: (_) {}),
          Switch(value: true, onChanged: (_) {}),
          TextButton(onPressed: () {}, child: const Text('x')),
        ],
      ),
    );
    // Padded tap targets would be 40x40 / 59x48 under the compact density.
    expect(tester.getSize(find.byType(Checkbox)), const Size(32, 32));
    expect(tester.getSize(find.byType(Switch)).height, 40);
    expect(tester.getSize(find.byType(TextButton)), const Size(19, 12));
  });

  testWidgets('an IconButton is exactly its icon, at any size', (tester) async {
    await pump(
      tester,
      Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          for (final size in [12.0, 14.0, 16.0, 17.0])
            IconButton(
              key: ValueKey(size),
              iconSize: size,
              onPressed: () {},
              icon: const Icon(Icons.star),
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
    // The ink is the same square, so no 70px highlight circle can appear.
    final ink = tester.widget<Material>(
      find.descendant(
        of: find.byKey(const ValueKey(14.0)),
        matching: find.byType(Material),
      ),
    );
    expect(ink.shape, const RoundedRectangleBorder());
  });

  testWidgets('M3 defaults do not reshape the editor', (tester) async {
    expect(EditorTheme.data.useMaterial3, isTrue);
    await pump(
      tester,
      Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const SizedBox(width: 100, child: TextField()),
          SizedBox(width: 100, child: Slider(value: .5, onChanged: (_) {})),
          const SizedBox(width: 100, child: Divider(height: 1)),
        ],
      ),
    );
    // A row of text stays one line of type high; M3's 1.5 geometry would
    // have made it 17.
    expect(tester.getSize(find.byType(TextField)).height, EditorMetrics.font);
    // The track keeps its own height; M3's gapped track is 16 high.
    expect(tester.getSize(find.byType(Slider)).height, EditorMetrics.dense);
    expect(tester.getSize(find.byType(Divider)).height, 1);
    final divider = tester.widget<Divider>(find.byType(Divider));
    expect(
      DividerTheme.of(tester.element(find.byType(Divider))).color,
      EditorTheme.line,
    );
    expect(divider.thickness, isNull);
  });
}
