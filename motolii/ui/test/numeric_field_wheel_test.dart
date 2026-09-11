import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/foundation/panel_controls.dart';

void main() {
  var value = 10.0;
  var begins = 0, finishes = 0;
  final previews = <double>[];
  late StateSetter rebuild;

  Future<void> mount(WidgetTester tester) async {
    value = 10;
    begins = finishes = 0;
    previews.clear();
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
  }

  Future<void> scroll(WidgetTester tester, Offset delta) async {
    await tester.sendEventToBinding(
      PointerScrollEvent(
        position: tester.getCenter(find.byType(EditorNumericField)),
        scrollDelta: delta,
        kind: PointerDeviceKind.mouse,
      ),
    );
    await tester.pump();
  }

  testWidgets('two-finger horizontal scroll scrubs without a press', (
    tester,
  ) async {
    await mount(tester);
    await scroll(tester, const Offset(15, 2));
    await scroll(tester, const Offset(10, -1));
    expect(begins, 1);
    expect(previews.last, closeTo(35, .001));
    expect(finishes, 0);
    // The fingers rest: the session settles into one finish.
    await tester.pump(const Duration(milliseconds: 350));
    expect(finishes, 1);
  });

  testWidgets('two fingers sideways scrub the number, not the list', (
    tester,
  ) async {
    value = 10;
    begins = finishes = 0;
    previews.clear();
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: StatefulBuilder(
            builder: (context, setState) {
              rebuild = setState;
              return ListView(
                children: [
                  SizedBox(
                    width: 100,
                    child: EditorNumericField(
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
                  const SizedBox(height: 3000),
                ],
              );
            },
          ),
        ),
      ),
    );
    final at = tester.getCenter(find.byType(EditorNumericField));
    final fingers = await tester.createGesture(
      kind: PointerDeviceKind.trackpad,
    );
    await fingers.panZoomStart(at);
    await fingers.panZoomUpdate(at, pan: const Offset(-30, 2));
    await tester.pump();
    await fingers.panZoomUpdate(at, pan: const Offset(-45, 3));
    await tester.pump();
    expect(begins, 1);
    expect(previews.last, closeTo(value, .001));
    expect(value, greaterThan(10));
    final list = tester.state<ScrollableState>(find.byType(Scrollable));
    expect(list.position.pixels, 0);
    await fingers.panZoomEnd();
    await tester.pump();
    expect(finishes, 1);

    // Up and down is still the list's.
    final list2 = await tester.createGesture(kind: PointerDeviceKind.trackpad);
    await list2.panZoomStart(at);
    await list2.panZoomUpdate(at, pan: const Offset(0, -60));
    await tester.pump();
    await list2.panZoomUpdate(at, pan: const Offset(0, -120));
    await tester.pump();
    await list2.panZoomEnd();
    await tester.pump();
    expect(list.position.pixels, greaterThan(0));
    expect(finishes, 1);
  });

  testWidgets('the wheel steps one unit per notch only while pressed', (
    tester,
  ) async {
    await mount(tester);
    await scroll(tester, const Offset(0, -120));
    expect(begins, 0);
    expect(previews, isEmpty);

    final pointer = await tester.startGesture(
      tester.getCenter(find.byType(EditorNumericField)),
    );
    await scroll(tester, const Offset(0, -120));
    await scroll(tester, const Offset(0, -120));
    await scroll(tester, const Offset(0, 120));
    expect(begins, 1);
    expect(previews.last, closeTo(11, .001));
    await tester.pump(const Duration(milliseconds: 350));
    expect(finishes, 0, reason: 'held: only the release finishes');
    await pointer.up();
    await tester.pump(const Duration(milliseconds: 50));
    expect(finishes, 1);
  });
}
