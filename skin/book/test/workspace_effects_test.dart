import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/workspace/dock_effects.dart';
import 'package:lab_book/workspace/fx_catalog.dart';
import 'package:lab_book/workspace/stage_art.dart';
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/workspace.dart';
import 'package:lab_book/workspace/ws.dart';

Future<Ws> _pump(WidgetTester t) async {
  t.view.physicalSize = const Size(1600, 960);
  t.view.devicePixelRatio = 1;
  await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => const Workspace()));
  await t.pumpAndSettle();
  final ws = WsScope.read(t.element(find.byType(WsTimeline)))..shelf = 'Effects';
  await t.pumpAndSettle();
  return ws;
}

Finder _row(String name) => find.descendant(of: find.byType(ListView), matching: find.text(name));

Future<void> _pickRow(WidgetTester t, String name) async {
  final list = find.descendant(of: find.byType(EffectsShelf), matching: find.byType(Scrollable)).last;
  t.state<ScrollableState>(list).position.jumpTo(0);
  await t.pumpAndSettle();
  await t.scrollUntilVisible(_row(name), 60, scrollable: list);
  await Scrollable.ensureVisible(t.element(_row(name)), alignment: .5);
  await t.pumpAndSettle();
  await t.tap(_row(name));
  await t.pumpAndSettle();
}

Future<void> _tap(WidgetTester t, Finder f) async {
  await t.ensureVisible(f);
  await t.pumpAndSettle();
  await t.tap(f);
  await t.pumpAndSettle();
}

void main() {
  testWidgets('the filter band narrows by what the shader declares: AND across groups, OR within, Clear', (t) async {
    addTearDown(t.view.reset);
    await _pump(t);
    expect(find.byKey(const ValueKey('browser:filters')), findsNothing);
    await _tap(t, find.byKey(const ValueKey('browser:filters-toggle')));
    expect(find.byKey(const ValueKey('browser:filters')), findsOneWidget);

    final timed = fxCatalog.where((f) => f.time == 'Uses time').length;
    await _tap(t, find.byKey(const ValueKey('browser:filter:Time:Uses time')));
    expect(find.text('$timed · 1 filter'), findsOneWidget);
    expect(_row('Glow'), findsNothing);
    expect(_row('Wave Warp'), findsOneWidget);

    await _tap(t, find.byKey(const ValueKey('browser:filter:Look:Moving')));
    final both = fxCatalog.where((f) => f.time == 'Uses time' && f.looks.contains('Moving')).length;
    expect(find.text('$both · 2 filters'), findsOneWidget);
    expect(_row('Film Grain'), findsNothing);

    await _tap(t, find.byKey(const ValueKey('browser:filter:Time:Static')));
    expect(_row('Film Grain'), findsNothing, reason: 'a plain click replaces the choice within its group');
    expect(_row('Motion Blur'), findsOneWidget);

    await _tap(t, find.byKey(const ValueKey('browser:filters-clear')));
    expect(find.text('${fxCatalog.length}'), findsOneWidget);
    expect(_row('Glow'), findsOneWidget);
    expect(t.takeException(), isNull);
  });

  testWidgets('the user sets the band height: drag the grip, double-click resets', (t) async {
    addTearDown(t.view.reset);
    await _pump(t);
    await _tap(t, find.byKey(const ValueKey('browser:filters-toggle')));
    final band = find.byKey(const ValueKey('browser:filters'));
    final h0 = t.getSize(band).height;
    await t.drag(find.byKey(const ValueKey('browser:filters-grip')), const Offset(0, 120));
    await t.pumpAndSettle();
    expect(t.getSize(band).height, closeTo(h0 + 120, 1));
    await t.drag(find.byKey(const ValueKey('browser:filters-grip')), const Offset(0, -2000));
    await t.pumpAndSettle();
    expect(t.getSize(band).height, 72, reason: 'never smaller than the floor');
    await t.tap(find.byKey(const ValueKey('browser:filters-grip')));
    await t.pump(const Duration(milliseconds: 50));
    await t.tap(find.byKey(const ValueKey('browser:filters-grip')));
    await t.pumpAndSettle();
    expect(t.getSize(band).height, closeTo(h0, 1));
    expect(t.takeException(), isNull);
  });

  testWidgets('picking a row tries it on the stage; Esc drops it, Enter and Add keep it', (t) async {
    addTearDown(t.view.reset);
    final ws = await _pump(t);
    expect(ws.selected, 'blob');

    await _pickRow(t, 'Glow');
    expect(ws.trial, 'Glow');
    expect(find.byKey(const ValueKey('ws-stage-trial')), findsOneWidget);
    expect(ArtScene.of(ws, camera: true).fx['blob'], [FxFamily.light]);

    await t.sendKeyEvent(LogicalKeyboardKey.escape);
    await t.pumpAndSettle();
    expect(ws.trial, isNull);
    expect(ws.fxOf('blob'), isEmpty);
    expect(find.byKey(const ValueKey('ws-stage-trial')), findsNothing);

    await _pickRow(t, 'Glow');
    await t.sendKeyEvent(LogicalKeyboardKey.enter);
    await t.pumpAndSettle();
    expect(ws.fxOf('blob'), ['Glow']);
    expect(find.descendant(of: find.byType(EffectsShelf), matching: find.text('ON')), findsOneWidget);

    await _pickRow(t, 'Gaussian Blur');
    await _tap(t, find.byKey(const ValueKey('ws-trial-add')));
    expect(ws.fxOf('blob'), ['Glow', 'Gaussian Blur']);
    expect(ArtScene.of(ws, camera: true).fx['blob'], [FxFamily.light, FxFamily.blur]);
    expect(t.takeException(), isNull);
  });

  testWidgets('the try-on says what it cannot show by itself; a camera takes no effect', (t) async {
    addTearDown(t.view.reset);
    final ws = await _pump(t);
    await _pickRow(t, 'Frosted Glass');
    expect(find.text('Seen on what is below the layer'), findsOneWidget);
    await _pickRow(t, 'Light Rays');
    expect(find.textContaining('scrub past'), findsOneWidget);

    ws.select('cam');
    await t.pumpAndSettle();
    expect(ws.trial, isNull);
    expect(find.byKey(const ValueKey('ws-stage-trial')), findsNothing);
    expect(t.takeException(), isNull);
  });
}
