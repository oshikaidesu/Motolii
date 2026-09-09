import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_stage5/app/editor_app.dart';
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
}
