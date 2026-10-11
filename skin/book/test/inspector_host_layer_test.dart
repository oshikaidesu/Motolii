import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';

void main() {
  testWidgets('Layer data cards build, fold and write to Doc', (tester) async {
    tester.view.physicalSize = const Size(600, 3200);
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
                  child: SizedBox(width: 372, child: Column(children: [MasksCard(), TimeCard(), HostLayoutCard(), LinksCard()])),
                ),
              ),
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    expect(tester.takeException(), isNull);
    expect(find.text('Masks'), findsOneWidget);
    expect(find.text('Time'), findsOneWidget);
    expect(find.text('Layout'), findsOneWidget);
    expect(find.text('Links'), findsOneWidget);

    // a horizontal scrub on Speed moves the value
    final speed = find.ancestor(of: find.text('Speed'), matching: find.byType(NumField));
    await tester.drag(speed, const Offset(60, 0));
    await tester.pumpAndSettle();
    expect(doc.get('time_speed'), greaterThan(100));

    // Display: Grid -> Flex swaps Columns/Rows for Direction/Wrap
    expect(find.text('Columns'), findsOneWidget);
    await tester.tap(find.text('FLEX'));
    await tester.pumpAndSettle();
    expect(doc.s2['hl_display'], 'Flex');
    expect(find.text('Columns'), findsNothing);
    expect(find.text('Direction'), findsOneWidget);

    // adding a mask appends to the list and seeds its rows
    await tester.tap(find.text('+  Add mask'));
    await tester.pumpAndSettle();
    expect(doc.s2['mask_list'], '1,2,3');
    expect(doc.get('mask_3.op'), 100);

    // the mode chooser floats its list and stores the pick
    await tester.tap(find.text('Add').first);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Intersect').last);
    await tester.pumpAndSettle();
    expect(doc.s2['mask_1.mode'], 'Intersect');

    // Loop off hides its length; Ghost on shows its delay
    expect(find.text('Length'), findsOneWidget);
    await tester.tap(find.text('Loop'));
    await tester.pumpAndSettle();
    expect(doc.b['time_loop'], isFalse);
    expect(find.text('Length'), findsNothing);
    await tester.tap(find.text('Ghost'));
    await tester.pumpAndSettle();
    expect(find.text('Ghost delay'), findsOneWidget);

    // every Advanced fold opens without overflow
    for (var i = 0; i < 2; i++) {
      await tester.tap(find.text('Advanced').at(i));
      await tester.pumpAndSettle();
    }
    expect(find.text('Stagger'), findsOneWidget);
    expect(find.text('Align self'), findsOneWidget);

    // the card folds to its brief
    await tester.tap(find.text('Links'));
    await tester.pumpAndSettle();
    expect(doc.b['fold.Links'], isTrue);
    expect(find.text('2 links'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });
}
