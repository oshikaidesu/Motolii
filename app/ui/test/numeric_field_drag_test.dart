import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/foundation/panel_controls.dart';

void main() {
  testWidgets('numeric drag keeps one pointer origin across rebuilds', (
    tester,
  ) async {
    var value = 10.0;
    var begins = 0;
    var finishes = 0;
    late StateSetter rebuild;
    final previews = <double>[];

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: StatefulBuilder(
            builder: (context, setState) {
              rebuild = setState;
              return Center(
                child: SizedBox(
                  width: 100,
                  child: EditorNumericField(
                    key: const ValueKey('number'),
                    value: value,
                    label: 'Position X',
                    onBegin: () => begins++,
                    onPreview: (next) async {
                      previews.add(next);
                      value = next;
                      rebuild(() {});
                    },
                    onCommit: (_) async {},
                    onFinish: () async => finishes++,
                    onCancel: () async {},
                  ),
                ),
              );
            },
          ),
        ),
      ),
    );

    final pointer = await tester.startGesture(
      tester.getCenter(find.byType(EditorNumericField)),
    );
    // Leave the 100px field: the original hit-test route must keep the drag.
    await pointer.moveBy(const Offset(130, 0));
    await tester.pump();
    await pointer.moveBy(const Offset(-110, 0));
    await tester.pump();
    await pointer.up();
    // GestureDetector's double-tap recognizer keeps a 40ms arbitration timer.
    await tester.pump(const Duration(milliseconds: 50));

    expect(begins, 1);
    expect(finishes, 1);
    expect(previews, isNotEmpty);
    expect(previews.last, closeTo(30, .001));
  });
}
