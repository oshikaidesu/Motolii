// The lasso relations try (lib/sets/panels/lasso_set.dart) by hand: drawn round, Around made, its centre carried, one
// taken out, the strength to 0, released; nothing jumps when a relation goes. Pictures in LASSO_SHOTS if given.
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/rendering.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/sets/panels/lasso_set.dart';

/// The stage's pixels (the card at the right left out).
Future<List<int>> stage(WidgetTester t, GlobalKey key) async {
  late List<int> out;
  await t.runAsync(() async {
    final image = await (key.currentContext!.findRenderObject()! as RenderRepaintBoundary).toImage();
    final b = (await image.toByteData())!;
    out = [for (var y = 0; y < image.height; y++) for (var x = 0; x < image.width - 260; x++) b.getUint32((y * image.width + x) * 4)];
  });
  return out;
}

int differ(List<int> a, List<int> b) => [for (var i = 0; i < a.length; i++) if (a[i] != b[i]) i].length;

void main() {
  testWidgets('a lasso makes Around; taking out and releasing keep every thing where it stands', (t) async {
    t.view.physicalSize = const Size(1100, 520);
    t.view.devicePixelRatio = 1;
    addTearDown(t.view.reset);
    final key = GlobalKey();
    await t.pumpWidget(WidgetsApp(
      color: const Color(0xFF000000),
      builder: (c, _) => RepaintBoundary(key: key, child: Builder(builder: (c) => lassoSet().useCases.first.builder(c))),
    ));
    Future<void> shot(String name) async {
      final dir = Platform.environment['LASSO_SHOTS'];
      if (dir == null) return;
      await t.runAsync(() async {
        final image = await (key.currentContext!.findRenderObject()! as RenderRepaintBoundary).toImage();
        final png = await image.toByteData(format: ui.ImageByteFormat.png);
        File('$dir/$name.png').writeAsBytesSync(png!.buffer.asUint8List());
      });
    }

    await shot('0_start');
    // (drawn round the whole stage, from the top left, clockwise)
    final g = await t.startGesture(const Offset(30, 20));
    for (final p in const [Offset(820, 20), Offset(820, 500), Offset(30, 500), Offset(30, 25)]) {
      await g.moveTo(p);
      await t.pump();
    }
    await g.up();
    await t.pump();
    expect(find.text('Around'), findsOneWidget);
    await t.tap(find.text('Around'));
    await t.pump();
    await shot('1_around');
    expect(find.text('Around · 8'), findsOneWidget);

    // (one taken out: it stays where it stands; only its mark of being in a relation goes. The card let go first, so none is dimmed)
    await t.tap(find.text('Around · 8'));
    await t.pump();
    final inside = await stage(t, key);
    await t.tap(find.text('1 ×'));
    await t.pump();
    expect(find.text('Around · 7'), findsOneWidget);
    expect(differ(inside, await stage(t, key)), lessThan(120));
    await shot('2_one_out');
    // (released: every one stays where it stands)
    final held = await stage(t, key);
    await t.tap(find.text('Release'));
    await t.pump();
    expect(find.text('Around · 7'), findsNothing);
    expect(differ(held, await stage(t, key)), lessThan(2500)); // (the ring, two handles and seven marks go; no thing moves)
    await shot('3_released');
  });
}
