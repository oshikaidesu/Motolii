import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/parts/glyphs.dart';
import 'package:lab_book/workspace/stage.dart';
import 'package:lab_book/workspace/stage_art.dart';
import 'package:lab_book/workspace/ws.dart';

Future<Ws> _pump(WidgetTester tester, Size size) async {
  tester.view.physicalSize = size;
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final ws = Ws();
  await tester.pumpWidget(
    WidgetsApp(
      color: const Color(0xFF000000),
      builder: (_, _) => WsScope(
        ws: ws,
        child: Align(
          alignment: Alignment.topLeft,
          child: SizedBox(width: size.width, height: size.height, child: const WsStage()),
        ),
      ),
    ),
  );
  return ws;
}

/// The screen point of a document point, through the camera, on the composition as laid out.
Offset _at(WidgetTester tester, Ws ws, Offset doc, {bool camera = true}) {
  final comp = tester.getRect(find.byKey(const ValueKey('ws-stage-comp')));
  return comp.topLeft + WsArt.toView(ArtScene.of(ws, camera: camera), doc) * (comp.width / WsArt.comp.width);
}

void main() {
  testWidgets('stage lays out at reference size and selects by clicking the artwork', (tester) async {
    final ws = await _pump(tester, const Size(924, 616));
    expect(tester.takeException(), isNull);
    expect(ws.selected, 'blob');

    final scene = ArtScene.of(ws, camera: true);
    final grid = [
      for (var y = 100.0; y < 1000; y += 20)
        for (var x = 100.0; x < 1840; x += 20) Offset(x, y),
    ];
    Offset spot(String? id) => grid.firstWhere((p) => WsArt.hit(scene, p) == id);
    await tester.tapAt(_at(tester, ws, spot(null)));
    expect(ws.selected, isNull);

    await tester.tapAt(_at(tester, ws, const Offset(260, 700)));
    expect(ws.selected, 'title');

    for (final id in ['ring', 'confetti', 'blob']) {
      await tester.tapAt(_at(tester, ws, spot(id)));
      expect(ws.selected, id);
    }

    final comp = tester.getRect(find.byKey(const ValueKey('ws-stage-comp')));
    await tester.tapAt(comp.topLeft - const Offset(6, 6));
    expect(ws.selected, isNull);
    expect(tester.takeException(), isNull);
  });

  testWidgets('frame changes, view controls and tools repaint without errors', (tester) async {
    final ws = await _pump(tester, const Size(924, 616));
    for (final f in [0, 12, 30, 60, 100, 150, 205, 240]) {
      ws.frame = f;
      await tester.pump();
    }
    expect(find.text('Not on this frame'), findsNothing);
    ws.frame = 230;
    ws.select('title');
    await tester.pump();
    expect(find.text('Not on this frame'), findsOneWidget);

    ws.frame = 100;
    await tester.tap(find.text('100%'));
    await tester.pump();
    await tester.tap(find.text('Grid'));
    await tester.tap(find.text('Safe'));
    await tester.tap(find.text('Camera'));
    await tester.pump();
    expect(find.text('Free'), findsOneWidget);

    await tester.tap(find.text('50%'));
    await tester.pump();
    final cam = WsArt.cameraBox(ArtScene.of(ws, camera: false));
    await tester.tapAt(_at(tester, ws, cam.corners.first + const Offset(400, 0), camera: false));
    expect(ws.selected, 'cam');

    expect(tester.takeException(), isNull);
  });

  testWidgets('tool column switches the options strip', (tester) async {
    await _pump(tester, const Size(924, 616));
    expect(find.text('SELECT'), findsOneWidget);
    for (final (g, name) in const [(G.rect, 'RECTANGLE'), (G.pen, 'PEN'), (G.text, 'TEXT')]) {
      await tester.tap(find.byWidgetPredicate((w) => w is Glyph && w.kind == g));
      await tester.pump();
      expect(find.text(name), findsOneWidget);
    }
  });

  testWidgets('stage fits a narrower, shorter seat', (tester) async {
    await _pump(tester, const Size(640, 400));
    expect(tester.takeException(), isNull);
    await _pump(tester, const Size(520, 300));
    expect(tester.takeException(), isNull);
  });
}
