import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';

Future<Doc> mount(WidgetTester tester) async {
  tester.view.physicalSize = const Size(800, 4000);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final doc = Doc(Kind.shape);
  await tester.pumpWidget(
    WidgetsApp(
      color: const Color(0xFF000000),
      builder: (c, _) => ListenableBuilder(
        listenable: doc,
        builder: (c, _) => Ctx(
          cfg: const Cfg(),
          doc: doc,
          child: Pop(
            child: Overlay.wrap(
              child: const SingleChildScrollView(
                child: Align(
                  alignment: Alignment.topLeft,
                  child: SizedBox(width: 372, child: Column(children: [RepeaterCard(), MirrorCard(), BlobTrackCard(), MotionBlurCard()])),
                ),
              ),
            ),
          ),
        ),
      ),
    ),
  );
  await tester.pump();
  return doc;
}

Finder field(String id) => find.byWidgetPredicate((w) => w is NumField && w.id == id);

Future<void> tap(WidgetTester tester, Finder f) async {
  await tester.tap(f);
  await tester.pump(const Duration(milliseconds: 400));
  expect(tester.takeException(), isNull);
}

void main() {
  testWidgets('placement cards lay out and every kind of control writes the Doc', (tester) async {
    final doc = await mount(tester);
    expect(tester.takeException(), isNull);
    expect(doc.get('rep_count'), 3);

    // a horizontal scrub on Count (away from its +/- glyph) lands on a whole number
    final r = tester.getRect(field('rep_count'));
    await tester.dragFrom(r.topLeft + const Offset(16, 30), const Offset(60, 0));
    await tester.pump(const Duration(milliseconds: 400));
    final n = doc.get('rep_count')!;
    expect(n, isNot(3));
    expect(n, n.roundToDouble());

    // Circle shows its own fields and takes the ring's Each default (placement.rs default_for)
    expect(find.text('Radius'), findsNothing);
    await tap(tester, find.text('CIRCLE'));
    expect(doc.get('rep_mode'), 1);
    expect(doc.get('rep_pos_each.x'), 0);
    expect(find.text('Radius'), findsOneWidget);
    expect(find.text('Sweep'), findsOneWidget);
    await tap(tester, find.text('GRID'));
    expect(find.text('Columns'), findsOneWidget);
    expect(find.text('Radius'), findsNothing);
    expect(doc.get('rep_pos_each.x'), 100);

    // the Each / Random matrix: Random never goes below 0
    final rr = tester.getRect(field('rep_rot_rand'));
    await tester.dragFrom(rr.center, const Offset(-80, 0));
    await tester.pump(const Duration(milliseconds: 400));
    expect(doc.get('rep_rot_rand'), 0);
    final re = tester.getRect(field('rep_rot_each'));
    await tester.dragFrom(re.center, const Offset(40, 0));
    await tester.pump(const Duration(milliseconds: 400));
    expect(doc.get('rep_rot_each'), isNot(0));

    // Advanced unfolds Delay and Seed
    expect(field('rep_seed'), findsNothing);
    await tap(tester, find.text('Advanced').first);
    expect(field('rep_seed'), findsOneWidget);

    // Mirror: Segments only for Radial; dragging the picture moves the centre
    expect(find.text('Segments'), findsNothing);
    await tap(tester, find.text('RADIAL'));
    expect(doc.get('mir_mode'), 3);
    expect(find.text('Segments'), findsOneWidget);
    final strip = tester.getRect(find.text('Axis')).topCenter - const Offset(0, 40);
    await tester.dragFrom(strip, const Offset(10, -5));
    await tester.pump();
    expect(doc.get('mir_centre.x'), isNot(0));

    // Blob Track: the layer chooser floats, Color brings its rows
    await tap(tester, find.text('Layers'));
    await tap(tester, find.text('Footage'));
    expect(doc.s2['blob_source'], 'Footage');
    expect(find.text('Tolerance'), findsNothing);
    await tap(tester, find.text('COLOR'));
    expect(doc.get('blob_mode'), 2);
    expect(find.text('Tolerance'), findsOneWidget);

    // Motion Blur: a part switches off, Tune 0 says why nothing blurs
    await tap(tester, find.descendant(of: find.byType(Chip), matching: find.text('Scale')));
    expect(doc.get('mb_scale'), 0);
    doc.set('mb_tune', 0);
    await tester.pump();
    expect(find.text('No blur while Tune is 0'), findsOneWidget);

    // fold and unfold by the title; the folded card summarises its values
    await tap(tester, find.text('Repeater'));
    expect(doc.b['fold.Repeater'], true);
    expect(field('rep_count'), findsNothing);
    expect(find.textContaining('× Grid'), findsOneWidget);
    await tap(tester, find.text('Repeater'));
    expect(doc.b['fold.Repeater'], false);

    // the card's switch bypasses it
    expect(doc.b['rep.on'], true);
    await tap(tester, find.descendant(of: find.byType(OnOff).first, matching: find.byType(GestureDetector)).first);
    expect(doc.b['rep.on'], false);
    expect(doc.b['fold.Repeater'], false);
  });
}
