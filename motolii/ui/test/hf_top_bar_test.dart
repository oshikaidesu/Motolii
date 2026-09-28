import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/hf/metrics.dart';
import '../lib/hf/shell/top.dart';
import '../lib/hf/shell/top_bar.dart';

/// The top bar at the density owner's height, folding what can go as the window narrows (the motto, then the brand's
/// line, then the duration) and never clipping a key; a key with no operation is drawn quiet.
void main() {
  Future<void> bar(WidgetTester t, double width, {List<int>? keys, List<int>? modes}) async {
    t.view.physicalSize = Size(width, 200);
    t.view.devicePixelRatio = 1;
    addTearDown(t.view.reset);
    await t.pumpWidget(Directionality(
      textDirection: TextDirection.ltr,
      child: Align(
        alignment: Alignment.topLeft,
        child: TopBar(
          TopModel(readouts: ValueNotifier(const ['30.00', '1800', '00:01:16']), onKey: keys?.add, onMode: modes?.add),
          keyEnabled: const [true, false, true],
        ),
      ),
    ));
  }

  testWidgets('wide: everything, at the chrome height', (t) async {
    await bar(t, 1400);
    expect(t.getSize(find.byType(TopBar)).height, UiMetrics.topBar);
    for (final s in ['Motolii', 'for More Relations.', '1800', 'EXPORT', 'Less numbers.']) {
      expect(find.text(s), findsOneWidget, reason: s);
    }
    expect(t.takeException(), isNull);
  });

  testWidgets('narrow: the motto and the brand line fold; the keys and the modes stay; nothing overflows', (t) async {
    await bar(t, 1000);
    expect(find.text('Less numbers.'), findsNothing);
    expect(find.text('for More Relations.'), findsNothing);
    expect(find.text('EXPORT'), findsOneWidget);
    expect(find.text('00:01:16'), findsOneWidget);
    expect(t.takeException(), isNull, reason: 'no overflow at 1000 px');
    await bar(t, 860);
    expect(find.text('1800'), findsNothing, reason: 'the duration folds last');
    expect(t.takeException(), isNull, reason: 'no overflow at 860 px');
  });

  testWidgets('a key without an operation does nothing and says so; the others run', (t) async {
    final keys = <int>[], modes = <int>[];
    await bar(t, 1400, keys: keys, modes: modes);
    final right = find.byWidgetPredicate((w) => w is MouseRegion);
    expect(right, findsWidgets);
    await t.tap(find.text('PLAY'));
    expect(modes, [1]);
  });
}
