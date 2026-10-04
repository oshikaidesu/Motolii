import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';

Future<Doc> mount(WidgetTester tester) async {
  tester.view.physicalSize = const Size(600, 2400);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final doc = Doc(Kind.shape);
  await tester.pumpWidget(
    WidgetsApp(
      color: const Color(0xFF000000),
      builder: (c, _) => Ctx(
        cfg: const Cfg(),
        doc: doc,
        child: Pop(
          child: Overlay.wrap(
            child: const SingleChildScrollView(
              child: SizedBox(width: 372, child: Column(children: [ParticlesCard()])),
            ),
          ),
        ),
      ),
    ),
  );
  await tester.pump();
  return doc;
}

void main() {
  testWidgets('every table row has a default and the card lays out at 372 px', (tester) async {
    final doc = await mount(tester);
    expect(tester.takeException(), isNull);
    expect(doc.get('pt_rate'), 40);
    expect(doc.get('pt_life'), 2);
    expect(doc.get('pt_direction'), -90);
    expect(doc.get('pt_turbulence_size'), 120);
    expect(doc.get('pt_line_opacity'), 60);
    expect(doc.s2['pt_color'], '#FFFFFF');
    for (final s in ['Rate', 'Life', 'Direction', 'Spread', 'Speed', 'Gravity', 'Wind X', 'Bounce', 'End Color']) {
      expect(find.text(s), findsOneWidget, reason: s);
    }
    expect(find.text('Floor'), findsNothing);
    await tester.tap(find.byKey(const ValueKey('adv:pt.adv')));
    await tester.pumpAndSettle();
    expect(find.text('Seed'), findsOneWidget);
  });

  testWidgets('scrubbing Rate sideways writes the doc', (tester) async {
    final doc = await mount(tester);
    await tester.drag(find.text('Rate'), const Offset(60, 0));
    await tester.pump();
    expect(doc.get('pt_rate'), isNot(40));
    expect(find.textContaining('alive'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  testWidgets('scrubbing Spread sideways writes the doc', (tester) async {
    final doc = await mount(tester);
    await tester.drag(find.text('Spread'), const Offset(-20, 0));
    await tester.pump();
    expect(doc.get('pt_spread'), isNot(30));
    expect(doc.get('pt_direction'), -90);
  });

  testWidgets('Turbulence and Links fold open; dependants follow their switch', (tester) async {
    final doc = await mount(tester);
    expect(find.text('Amount'), findsNothing);
    await tester.tap(find.text('Turbulence'));
    await tester.pump();
    expect(doc.b['pt_fold.turb'], isTrue);
    expect(find.text('Amount'), findsOneWidget);
    expect(find.text('Scale'), findsNothing);
    doc.set('pt_turbulence', 40);
    await tester.pump();
    expect(find.text('Scale'), findsOneWidget);
    await tester.tap(find.text('Links'));
    await tester.pump();
    expect(find.text('Distance'), findsOneWidget);
    doc.setMany({'pt_connect': 80, 'pt_bounce': 50});
    await tester.pump();
    expect(find.text('Line Width'), findsOneWidget);
    expect(find.text('Floor'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  testWidgets('the card folds to its brief', (tester) async {
    final doc = await mount(tester);
    await tester.tap(find.text('Particles'));
    await tester.pump();
    expect(doc.b['fold.Particles'], isTrue);
    expect(find.text('40/s'), findsOneWidget);
    expect(find.text('2.0 s'), findsOneWidget);
    expect(find.text('-90°'), findsOneWidget);
    expect(find.text('Rate'), findsNothing);
  });

  testWidgets('a colour sends to Browser › Colors', (tester) async {
    final doc = await mount(tester);
    await tester.tap(find.byKey(const ValueKey('link:colors:pt_color')));
    await tester.pump();
    expect(doc.s2['route'], 'Browser › Colors');
    expect(tester.takeException(), isNull);
  });
}
