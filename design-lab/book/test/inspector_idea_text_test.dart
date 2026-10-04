import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';

/// (text, size, fill) per run, read straight from the encoded document.
List<(String, double, String)> runs(Doc d) => [
  for (final r in d.s2['tx_runs']!.split('\u001e'))
    if (r.split('\u001f') case [final s, _, final f, ...final t]) (t.join('\u001f'), double.parse(s), f),
];

double sizeAt(Doc d, int i) {
  var at = 0;
  for (final (t, s, _) in runs(d)) {
    if (i < at + t.length) return s;
    at += t.length;
  }
  return -1;
}

void main() {
  Future<Doc> pump(WidgetTester tester) async {
    tester.view.physicalSize = const Size(600, 1200);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final doc = Doc(Kind.text);
    await tester.pumpWidget(
      WidgetsApp(
        color: const Color(0xFF000000),
        builder: (c, _) => ListenableBuilder(
          listenable: doc,
          builder: (_, _) => Ctx(
            cfg: const Cfg(),
            doc: doc,
            child: Pop(
              child: Overlay.wrap(
                child: const SingleChildScrollView(
                  child: Align(
                    alignment: Alignment.topLeft,
                    child: SizedBox(width: 372, child: TextIdeas()),
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    return doc;
  }

  Finder nf(String id) => find.byWidgetPredicate((w) => w is NumField && w.id == id);

  testWidgets('the palette styles the selected letters only; typing keeps styles', (tester) async {
    final doc = await pump(tester);
    expect(tester.takeException(), isNull);
    expect(runs(doc).map((r) => r.$1).join(), 'Hero title\nSmall move changes everything.');

    final edit = find.byType(EditableText);
    await tester.tap(edit);
    await tester.pump();
    tester.widget<EditableText>(edit).controller.selection = const TextSelection(baseOffset: 17, extentOffset: 21); // "move"
    await tester.pumpAndSettle();
    expect(find.text('Styling “move”'), findsOneWidget);

    await tester.drag(nf('tx_sz'), const Offset(60, 0));
    await tester.pumpAndSettle();
    expect(sizeAt(doc, 17), greaterThan(36));
    expect(sizeAt(doc, 20), sizeAt(doc, 17));
    expect(sizeAt(doc, 12), 36, reason: 'letters outside the selection keep their size');

    await tester.enterText(edit, 'Hero title!\nSmall move changes everything.');
    await tester.pumpAndSettle();
    expect(runs(doc).first.$1, 'Hero ');
    expect(runs(doc).map((r) => r.$1).join(), startsWith('Hero title!'));
    expect(tester.takeException(), isNull);
  });

  testWidgets('with nothing selected the palette styles all text; font and fill send to their panels', (tester) async {
    final doc = await pump(tester);
    expect(find.textContaining('Styling all text'), findsOneWidget);
    await tester.drag(nf('tx_w'), const Offset(80, 0));
    await tester.pumpAndSettle();
    expect(doc.s2['tx_runs'], isNot(contains('\u001f400.0\u001f')));
    await tester.tap(find.byKey(const ValueKey('link:fonts:')));
    await tester.pumpAndSettle();
    expect(doc.s2['route'], 'Browser › Fonts');
    await tester.tap(find.byKey(const ValueKey('link:colors:Fill')));
    await tester.pumpAndSettle();
    expect(doc.s2['route'], 'Browser › Colors');
    await tester.tap(find.byKey(const ValueKey('tx_align_1')));
    await tester.pumpAndSettle();
    expect(doc.get('tx_align'), 1);
    expect(tester.takeException(), isNull);
  });
}
