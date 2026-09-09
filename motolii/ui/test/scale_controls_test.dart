import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/foundation/panel_controls.dart';

void main() {
  testWidgets('scaled viewport fills its bounds and hit tests the far corner', (
    tester,
  ) async {
    for (final scale in [.5, .89, 1.0, 1.01, 2.0]) {
      var taps = 0;
      await tester.pumpWidget(
        MaterialApp(
          home: EditorScaledViewport(
            scale: scale,
            child: Stack(
              children: [
                Positioned(
                  right: 0,
                  bottom: 0,
                  child: GestureDetector(
                    onTap: () => taps++,
                    child: Container(
                      key: const ValueKey('corner'),
                      width: 40,
                      height: 40,
                      color: Colors.red,
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      );
      final rect = tester.getRect(find.byKey(const ValueKey('corner')));
      expect(rect.right, closeTo(800, .001));
      expect(rect.bottom, closeTo(600, .001));
      await tester.tapAt(const Offset(799, 599));
      expect(taps, 1);
      expect(tester.takeException(), isNull);
    }
  });
  testWidgets(
    'panel size advances one percent and accepts an exact percentage',
    (tester) async {
      var size = 88.0;
      await tester.pumpWidget(
        MaterialApp(
          home: StatefulBuilder(
            builder: (context, setState) => Scaffold(
              body: Center(
                child: EditorZoomBar(
                  value: size,
                  min: 48,
                  max: 200,
                  base: 88,
                  keyPrefix: 'size',
                  onChanged: (v) => setState(() => size = v),
                ),
              ),
            ),
          ),
        ),
      );
      await tester.tap(find.byKey(const ValueKey('size-larger')));
      await tester.pump();
      expect(size, closeTo(88.88, .0001));
      await tester.tap(find.byKey(const ValueKey('size-smaller')));
      await tester.pump();
      expect(size, 88);
      final field = find.byType(EditorNumericField);
      await tester.tap(field);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.tap(field);
      await tester.pumpAndSettle();
      await tester.enterText(find.byType(TextField), '137');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pumpAndSettle();
      expect(size, closeTo(120.56, .0001));
      expect(tester.takeException(), isNull);
    },
  );
}
