import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';

void main() {
  testWidgets('scene lane cards build and edit the Doc', (tester) async {
    tester.view.physicalSize = const Size(600, 4000);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final doc = Doc(Kind.shape);
    // Ctx always notifies, so pumping the same tree again shows the Doc's latest values.
    Future<void> pump() async {
      await tester.pumpWidget(
        WidgetsApp(
          color: const Color(0xFF000000),
          builder: (c, _) => Ctx(
            cfg: const Cfg(),
            doc: doc,
            child: Pop(
              child: Overlay.wrap(
                child: const SingleChildScrollView(
                  child: SizedBox(width: 372, child: Column(children: [SceneCard(), HostCameraCard(), ExtrudeCard(), BevelCard(), OverlayCard()])),
                ),
              ),
            ),
          ),
        ),
      );
      await tester.pump(const Duration(milliseconds: 400));
      expect(tester.takeException(), isNull);
    }

    await pump();
    expect(doc.get('sc_projection'), 2);
    expect(doc.get('sc_flatten'), 0);

    await tester.tap(find.text('2.5D'));
    await pump();
    expect(doc.get('sc_projection'), 1);

    await tester.drag(find.text('Depth'), const Offset(80, 0));
    await pump();
    expect(doc.get('ex_depth')!, greaterThan(20));

    final c = tester.getCenter(find.byKey(const ValueKey('hc-orbit')));
    await tester.dragFrom(c + const Offset(0, 20), const Offset(30, -10));
    await pump();
    expect(doc.get('hc_pan'), isNot(0));

    expect(find.text('Segments'), findsOneWidget);
    await tester.tap(find.text('CHAMFER'));
    await pump();
    expect(doc.get('bv_profile'), 1);
    expect(find.text('Segments'), findsNothing);

    await tester.tap(find.text('PHYSICS'));
    await pump();
    expect(doc.get('ov_kind'), 3);
    expect(doc.get('ov_method'), 2);
    expect(doc.get('ov_well'), 1);
    expect(doc.s2['ov_marker_color'], '#EDFA8C');
    expect(find.text('Physics Trace'), findsOneWidget);

    await tester.tap(find.text('Scene'));
    await pump();
    expect(doc.b['fold.Scene'], true);
    expect(find.text('2.5D'), findsOneWidget);
  });
}
