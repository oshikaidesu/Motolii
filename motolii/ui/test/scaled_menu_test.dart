import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/foundation/panel_controls.dart';
import '../lib/foundation/metrics.dart';
import '../lib/foundation/theme.dart';

void main() {
  testWidgets('a context menu scales with the app and opens at the pointer', (
    tester,
  ) async {
    final scale = ValueNotifier(2.0);
    late BuildContext pageContext;
    // The bare Scaffold below is what a panel's own test builds: no menu
    // host of any kind, only the MaterialApp everything gets for free.
    await tester.pumpWidget(
      MaterialApp(
        theme: EditorTheme.data,
        builder: (context, child) => EditorScale(
          notifier: scale,
          child: ValueListenableBuilder(
            valueListenable: scale,
            builder: (_, s, __) =>
                EditorScaledViewport(scale: s, child: child!),
          ),
        ),
        home: Builder(
          builder: (context) {
            pageContext = context;
            return const Scaffold(body: SizedBox.expand());
          },
        ),
      ),
    );
    expect(EditorScale.of(pageContext), same(scale));
    const at = Offset(300, 200);
    final picked = showEditorMenu<String>(pageContext, at, [
      const EditorMenuItem(value: 'a', child: Text('Alpha')),
    ]);
    await tester.pump();
    await tester.pump();
    final item = find.byType(EditorMenuItem<String>);
    final rect = tester.getRect(item);
    expect(rect.height, closeTo(EditorMetrics.row * 2, .5));
    final label = tester.getRect(find.text('Alpha'));
    expect(rect.width, lessThan(label.width + EditorMetrics.s8 * 2 * 2 + 4));
    expect(rect.top, greaterThanOrEqualTo(at.dy - 1));
    expect(rect.left, greaterThanOrEqualTo(at.dx - 1));
    expect(rect.top - at.dy, lessThan(EditorMetrics.row * 2));
    Object? got = 'unset';
    picked.then((v) => got = v);
    await tester.tapAt(rect.center);
    await tester.pump();
    await tester.pump();
    expect(got, 'a');
    expect(item, findsNothing);
    final dismissed = showEditorMenu<String>(pageContext, at, [
      const EditorMenuItem(value: 'a', child: Text('Alpha')),
    ]);
    await tester.pump();
    await tester.pump();
    got = 'unset';
    dismissed.then((v) => got = v);
    await tester.tapAt(const Offset(10, 10));
    await tester.pump();
    await tester.pump();
    expect(got, isNull);
    expect(item, findsNothing);
    expect(tester.takeException(), isNull);
  });
}
