// Generic Inspector, Phase 1. Kind guarantees access; unknown is generic, never an error.
// The Toys are driven with real pointer and key events.
import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_stage5/hf/bp/search.dart';
import 'package:motolii_stage5/hf/insp/fixtures.dart';
import 'package:motolii_stage5/hf/insp/panel.dart';
import 'package:motolii_stage5/hf/insp/rows.dart';
import 'package:motolii_stage5/hf/insp/tones.dart';

Widget host(Widget child, double w, double h) => WidgetsApp(
      color: const Color(0xFF000000),
      pageRouteBuilder: <T>(settings, builder) => PageRouteBuilder<T>(settings: settings, pageBuilder: (c, _, __) => builder(c)),
      home: Align(alignment: Alignment.topLeft, child: SizedBox(width: w, height: h, child: child)),
    );

Future<ParamStore> open(WidgetTester t, {List<Map<String, dynamic>>? rows, bool frozen = false, double w = 310, double h = 640, bool advancedOpen = false, SearchCapability? search}) async {
  t.view.physicalSize = const Size(900, 900);
  t.view.devicePixelRatio = 1;
  addTearDown(t.view.reset);
  final store = ParamStore(rows ?? unknownEffect(), frozen: frozen);
  await t.pumpWidget(host(InspectorBody(store: store, subject: 'Test', advancedOpen: advancedOpen, search: search), w, h));
  await t.pump();
  return store;
}

// A click and a second click within a third of a second are a double click, in real time, so tests that edit one Toy twice wait.
Future<void> realGap(WidgetTester t) async { await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 380))); }

Finder k(String key) => find.byKey(ValueKey(key));
// The list builds only what is near the viewport, so reaching a row means scrolling to it.
Future<void> reach(WidgetTester t, String key) async {
  final list = find.descendant(of: k('insp-list'), matching: find.byType(Scrollable)).first;
  await t.scrollUntilVisible(k(key), 300, scrollable: list, maxScrolls: 200);
  await t.pump();
}
dynamic val(ParamStore s, String id) => s.row(id)['value'];

void main() {
  setUpAll(() async {
    final inter = FontLoader('Inter')
      ..addFont(rootBundle.load('assets/fonts/Inter-Regular.ttf'))
      ..addFont(rootBundle.load('assets/fonts/Inter-Medium.ttf'))
      ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.ttf'));
    await inter.load();
  });

  group('kind decision (no widgets)', () {
    PKind of(Map<String, dynamic> r) => kindOf(r);
    test('every declared type lands on a control kind', () {
      expect(of({'kind': 'f32', 'value': 1.0}), PKind.scalar);
      expect(of({'kind': 'f32', 'value': 1.0, 'min': 0, 'max': 1}), PKind.bounded);
      expect(of({'kind': 'i32', 'value': 1}), PKind.integer);
      expect(of({'kind': 'u32', 'value': 1}), PKind.integer);
      expect(of({'kind': 'bool', 'value': true}), PKind.toggle);
      expect(of({'kind': 'enum', 'choices': ['a'], 'value': 0}), PKind.choice);
      expect(of({'kind': 'vec2', 'value': [0, 0]}), PKind.vec2);
      expect(of({'kind': 'vec3', 'value': [0, 0, 0]}), PKind.vec3);
      expect(of({'kind': 'text', 'value': 'x'}), PKind.text);
      expect(of({'layer': true, 'value': null}), PKind.reference);
      expect(of({'kind': 'color', 'value': '#fff'}), PKind.route);
    });
    test('unknown is generic: a stranger kind is still reachable by its value', () {
      expect(of({'kind': 'wat', 'value': 3}), PKind.scalar);
      expect(of({'kind': 'wat', 'value': 'abc'}), PKind.text);
      expect(of({'kind': 'wat', 'value': true}), PKind.toggle);
      expect(of({'kind': 'wat', 'value': [1, 2]}), PKind.vec2);
      expect(of({'kind': 'wat', 'value': {'a': 1}}), PKind.raw);
      expect(of({'kind': 'wat', 'value': null}), PKind.raw);
    });
    test('the label never decides the kind', () {
      expect(of({'id': 'p.param.angle', 'label': 'Angle', 'kind': 'f32', 'value': 10.0}), PKind.scalar);
    });
    test('a hard range is not a reach: only a tight range earns a track', () {
      expect(tight({'value': .5, 'default': .5, 'min': 0.0, 'max': 1.0}), isTrue);
      expect(tight({'value': 4, 'default': 4, 'min': 1, 'max': 8}), isTrue);
      expect(tight({'value': 5.0, 'default': 5.0, 'min': 0.0, 'max': 100000.0}), isFalse);
      expect(tight({'value': 0.0, 'default': 0.0, 'min': -1e9, 'max': 1e9}), isFalse);
      expect(tight({'value': 1.0, 'default': 1.0}), isFalse);
    });
    test('heroes: declared, else the first four; never a cap on what exists', () {
      final rows = stress(9);
      expect(heroIds(rows), {for (var i = 0; i < 4; i++) 'p.param.p$i'});
      expect(heroIds(stress(3)), isEmpty);
      final declared = [for (final r in unknownEffect()) r];
      expect(heroIds(declared), {'fx.param.foo', 'fx.param.mix', 'fx.param.iterations', 'fx.param.invert'});
    });
    test('layout: heroes, sections, then the advanced fold; nothing dropped', () {
      final e = layoutOf(unknownEffect(), advancedOpen: true);
      final ids = [for (final x in e) if (x is PCells) ...[for (final r in x.rows) r['id']]];
      expect(ids.length, unknownEffect().length);
      expect(ids.take(4).toSet(), {'fx.param.foo', 'fx.param.mix', 'fx.param.iterations', 'fx.param.invert'});
      expect(e.whereType<PFold>().single.count, 3);
      expect(layoutOf(unknownEffect(), advancedOpen: false).whereType<PCells>().expand((c) => c.rows).length, unknownEffect().length - 3);
      expect(layoutOf(stress(64), advancedOpen: false, narrow: true).whereType<PCells>().every((c) => c.rows.length == 1), isTrue);
    });
  });

  group('the value Toy', () {
    testWidgets('horizontal drag scrubs; a drag is many previews and one commit', (t) async {
      final s = await open(t);
      final before = val(s, 'fx.param.foo') as double;
      await t.drag(k('toy-fx.param.foo'), const Offset(60, 0));
      await t.pump(const Duration(seconds: 1));
      expect((val(s, 'fx.param.foo') as double) - before, closeTo(.6, .02));
      expect(s.commits, 1);
      expect(s.previews, greaterThanOrEqualTo(1));
    });

    testWidgets('Shift makes the drag fine', (t) async {
      final s = await open(t);
      await t.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
      final before = val(s, 'fx.param.foo') as double;
      await t.drag(k('toy-fx.param.foo'), const Offset(60, 0));
      await t.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
      await t.pump(const Duration(seconds: 1));
      expect((val(s, 'fx.param.foo') as double) - before, closeTo(.06, .005));
    });

    testWidgets('hard min and max clamp silently; the value stays inside', (t) async {
      final s = await open(t);
      await t.drag(k('toy-fx.param.mix'), const Offset(800, 0));
      await t.pump(const Duration(seconds: 1));
      expect(val(s, 'fx.param.mix'), 1.0);
    });

    testWidgets('only a tight range draws a track; an open or guarded value never does', (t) async {
      await open(t);
      expect(k('track-fx.param.mix'), findsOneWidget);
      expect(k('track-fx.param.iterations'), findsOneWidget);
      expect(k('track-fx.param.foo'), findsNothing);
      expect(k('track-fx.param.bar'), findsNothing); // -1e9..1e9 is a guard
      expect(k('track-fx.param.bins'), findsNothing);
    });

    testWidgets('whole numbers stay whole while scrubbing', (t) async {
      final s = await open(t);
      await t.drag(k('toy-fx.param.iterations'), const Offset(45, 0));
      await t.pump(const Duration(seconds: 1));
      expect(val(s, 'fx.param.iterations'), isA<int>());
      expect(val(s, 'fx.param.iterations'), greaterThan(6));
    });

    testWidgets('a huge negative value is shown whole, and a tiny one does not read as zero', (t) async {
      await open(t, advancedOpen: true);
      await reach(t, 'toy-fx.param.bar');
      expect(find.text('-12840.5'), findsOneWidget);
      await reach(t, 'toy-fx.param.epsilon');
      expect(find.text('1.00e-6'), findsOneWidget);
    });

    testWidgets('click opens exact input; Enter applies it as one commit; Esc cancels', (t) async {
      final s = await open(t);
      await t.tap(k('toy-fx.param.foo'));
      await t.pump();
      final field = find.descendant(of: k('toy-fx.param.foo'), matching: find.byType(EditableText));
      expect(field, findsOneWidget);
      await t.enterText(field, '12.5');
      await t.testTextInput.receiveAction(TextInputAction.done);
      await t.pump(const Duration(seconds: 1));
      expect(val(s, 'fx.param.foo'), 12.5);
      expect(s.commits, 1);
      // Esc leaves without applying
      await t.pump(const Duration(seconds: 1));
      await t.tap(k('toy-fx.param.gain'));
      await t.pump();
      await t.enterText(find.descendant(of: k('toy-fx.param.gain'), matching: find.byType(EditableText)), '99');
      await t.sendKeyEvent(LogicalKeyboardKey.escape);
      await t.pump(const Duration(seconds: 1));
      expect(val(s, 'fx.param.gain'), 1.0);
    });

    testWidgets('double click returns to the default', (t) async {
      final s = await open(t);
      expect(val(s, 'fx.param.foo'), .35);
      await t.tap(k('toy-fx.param.foo'));
      await t.tap(k('toy-fx.param.foo'));
      await t.pump(const Duration(seconds: 1));
      expect(val(s, 'fx.param.foo'), .5);
      expect(find.descendant(of: k('toy-fx.param.foo'), matching: find.byType(EditableText)), findsNothing);
    });

    testWidgets('keyboard: arrows nudge, Shift nudges finely, Delete resets', (t) async {
      final s = await open(t);
      await t.tap(k('toy-fx.param.foo'));
      await t.pump();
      await t.sendKeyEvent(LogicalKeyboardKey.escape); // leave exact input, keep focus on the Toy
      await t.pump(const Duration(seconds: 1));
      final v0 = val(s, 'fx.param.foo') as double;
      await t.sendKeyEvent(LogicalKeyboardKey.arrowUp);
      final v1 = val(s, 'fx.param.foo') as double;
      expect(v1, greaterThan(v0));
      await t.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
      await t.sendKeyEvent(LogicalKeyboardKey.arrowUp);
      await t.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
      final v2 = val(s, 'fx.param.foo') as double;
      expect(v2 - v1, lessThan((v1 - v0) / 5));
      await t.sendKeyEvent(LogicalKeyboardKey.delete);
      expect(val(s, 'fx.param.foo'), .5);
    });
  });

  group('the other Toys', () {
    testWidgets('toggle', (t) async {
      final s = await open(t);
      expect(val(s, 'fx.param.invert'), true);
      await t.tap(k('toy-fx.param.invert'));
      await t.pump();
      expect(val(s, 'fx.param.invert'), false);
      expect(s.commits, 1);
    });

    testWidgets('choice: chips for a short list, a stepper for a long one', (t) async {
      final s = await open(t);
      await reach(t, 'choice-fx.param.mode-2');
      await t.tap(k('choice-fx.param.mode-2'));
      await t.pump();
      expect(val(s, 'fx.param.mode'), 2);
      await reach(t, 'choice-fx.param.pattern-next');
      await t.tap(k('choice-fx.param.pattern-next'));
      await t.pump();
      expect(val(s, 'fx.param.pattern'), 5);
      expect(find.text('Noise'), findsOneWidget);
      // it wraps
      for (var i = 0; i < 3; i++) { await t.tap(k('choice-fx.param.pattern-next')); await t.pump(); }
      expect(val(s, 'fx.param.pattern'), 0);
    });

    testWidgets('vectors are their axes: dragging X changes X only', (t) async {
      final s = await open(t);
      await reach(t, 'toy-fx.param.center-0');
      await t.drag(k('toy-fx.param.center-0'), const Offset(50, 0));
      await t.pump(const Duration(seconds: 1));
      final c = val(s, 'fx.param.center') as List;
      expect(c[0], isNot(.5));
      expect(c[1], .5);
      await reach(t, 'toy-fx.param.tint-2');
      await t.drag(k('toy-fx.param.tint-2'), const Offset(-30, 0));
      await t.pump(const Duration(seconds: 1));
      expect((val(s, 'fx.param.tint') as List)[2], lessThan(.2));
      expect((val(s, 'fx.param.tint') as List)[0], 1.0);
    });

    testWidgets('text', (t) async {
      final s = await open(t);
      await reach(t, 'text-fx.param.label');
      await t.enterText(k('text-fx.param.label'), 'world');
      await t.testTextInput.receiveAction(TextInputAction.done);
      await t.pump();
      expect(val(s, 'fx.param.label'), 'world');
    });

    testWidgets('reference picks among declared candidates and can be empty', (t) async {
      final s = await open(t);
      await reach(t, 'ref-fx.param.matte');
      await t.tap(k('ref-fx.param.matte'));
      await t.pump();
      expect(val(s, 'fx.param.matte'), 'Layer 3');
      await t.tap(k('ref-fx.param.matte'));
      await t.pump();
      expect(val(s, 'fx.param.matte'), isNull);
      expect(find.text('None'), findsOneWidget);
    });

    testWidgets('a value the host has a specialist for routes there and is not edited here', (t) async {
      final s = await open(t);
      await reach(t, 'route-fx.param.glow');
      await t.tap(k('route-fx.param.glow'));
      await t.pump();
      expect(s.routes, ['Colors']);
      expect(val(s, 'fx.param.glow'), '#F5C94A');
      expect(find.descendant(of: k('route-fx.param.glow'), matching: find.byType(EditableText)), findsNothing);
    });

    testWidgets('a declared action is a button, composed with the Value (seed = Value + Reroll)', (t) async {
      final s = await open(t);
      await reach(t, 'action-fx.param.seed-Reroll');
      await t.tap(k('action-fx.param.seed-Reroll'));
      await t.pump();
      expect(s.actions, ['fx.param.seed:Reroll']);
      expect(k('toy-fx.param.seed'), findsOneWidget);
    });

    testWidgets('an unrecognised value is still editable, as text that must parse', (t) async {
      final s = await open(t, advancedOpen: true);
      await reach(t, 'text-fx.param.weird');
      expect(find.text('{"a":1}'), findsOneWidget);
      await t.enterText(k('text-fx.param.weird'), '{"a":2}');
      await t.testTextInput.receiveAction(TextInputAction.done);
      await t.pump();
      expect(val(s, 'fx.param.weird'), {'a': 2});
      await t.enterText(k('text-fx.param.weird'), 'oops{');
      await t.testTextInput.receiveAction(TextInputAction.done);
      await t.pump();
      expect(val(s, 'fx.param.weird'), {'a': 2}); // not applied, not lost
    });
  });

  group('marks and structure', () {
    testWidgets('modified and animated state, and reset', (t) async {
      final s = await open(t);
      expect(k('mod-fx.param.foo'), findsOneWidget);
      expect(k('mod-fx.param.mix'), findsOneWidget);
      expect(k('anim-fx.param.foo'), findsOneWidget);
      expect(k('anim-fx.param.mix'), findsNothing);
      await t.tap(k('reset-fx.param.foo'));
      await t.pump();
      expect(val(s, 'fx.param.foo'), .5);
      expect(k('mod-fx.param.foo'), findsNothing);
      expect(k('reset-fx.param.foo'), findsNothing); // nothing to go back to
    });

    testWidgets('advanced rows are behind a fold and open with it', (t) async {
      await open(t);
      expect(k('cell-fx.param.gamma'), findsNothing);
      await reach(t, 'advanced-fold');
      await t.tap(k('advanced-fold'));
      await t.pump();
      await reach(t, 'cell-fx.param.gamma');
      expect(k('cell-fx.param.gamma'), findsOneWidget);
    });

    testWidgets('the filter finds a parameter inside a closed fold', (t) async {
      final search = SearchCapability();
      await open(t, search: search);
      search.controller.text = 'epsilon';
      await t.pump();
      expect(k('cell-fx.param.epsilon'), findsOneWidget);
      expect(k('cell-fx.param.foo'), findsNothing);
      search.controller.text = 'zzz-none';
      await t.pump();
      expect(find.text('No parameter matches'), findsOneWidget);
    });

    testWidgets('frozen: nothing edits, nothing resets', (t) async {
      final s = await open(t, frozen: true);
      await t.drag(k('toy-fx.param.foo'), const Offset(60, 0));
      await t.tap(k('toy-fx.param.invert'));
      await t.pump(const Duration(seconds: 1));
      expect(val(s, 'fx.param.foo'), .35);
      expect(val(s, 'fx.param.invert'), true);
      expect(s.commits, 0);
      expect(k('reset-fx.param.foo'), findsNothing);
      expect(find.text('Frozen'), findsOneWidget);
    });

    testWidgets('a narrow panel folds to one column and keeps every control', (t) async {
      final s = await open(t, w: 200);
      expect(t.takeException(), isNull);
      await t.drag(k('toy-fx.param.foo'), const Offset(40, 0));
      await t.pump(const Duration(seconds: 1));
      expect(val(s, 'fx.param.foo'), isNot(.35));
    });
  });

  group('colour identifies a group, not a widget kind', () {
    test('the same thing is dealt the same colours every time', () {
      final a = Tones('Unknown effect', unknownEffect()), b = Tones('Unknown effect', unknownEffect());
      for (final r in unknownEffect()) { expect(a.of(r), b.of(r)); }
    });
    test('different kinds in one group share its colour; neighbouring groups differ', () {
      final t = Tones('fx', unknownEffect());
      final numbers = [for (final r in unknownEffect()) if (r['section'] == 'Numbers') t.of(r)];
      expect(numbers.toSet().length, 1); // f32, u32, seed, bar... one colour
      final order = <String>[];
      for (final r in unknownEffect()) { final k = groupKey(r); if (!order.contains(k)) order.add(k); }
      for (var i = 0; i + 1 < order.length; i++) { expect(t.ofGroup(order[i]), isNot(t.ofGroup(order[i + 1]))); }
      expect(t.advanced, isNot(t.ofGroup('Numbers')));
    });
    test('unknown effects get colours too, with no section at all', () {
      final t = Tones('x', stress(3));
      expect(t.of(stress(3).first), isNotNull);
    });
    test('the palette is small and a big list does not run out', () {
      final t = Tones('big', stress(256));
      final colours = {for (final r in stress(256)) t.of(r)};
      expect(colours.length, lessThanOrEqualTo(kTonePalette.length));
    });
    testWidgets('the section mark wears the group colour; frozen dims it away from a touchable colour', (t) async {
      await open(t);
      Color mark(String sec) => ((t.widget<Container>(k('tone-$sec')).decoration) as BoxDecoration).color!;
      final tones = Tones('Test', unknownEffect());
      await reach(t, 'tone-Numbers');
      expect(mark('Numbers'), tones.ofGroup('Numbers'));
      expect(dimTone(tones.ofGroup('Numbers')), isNot(tones.ofGroup('Numbers')));
    });
  });

  group('no ceiling on parameters', () {
    for (final n in [1, 8, 64, 256]) {
      testWidgets('$n declared parameters: same body, same architecture', (t) async {
        final s = await open(t, rows: stress(n));
        expect(find.text('$n'), findsWidgets);
        expect(t.takeException(), isNull);
        expect(find.byType(ParamCell).evaluate().length, lessThan(60)); // lazily built, not all at once
        // the last one is reachable and editable
        final last = 'p.param.p${n - 1}';
        await t.scrollUntilVisible(k('cell-$last'), 400, scrollable: find.descendant(of: k('insp-list'), matching: find.byType(Scrollable)).first);
        await t.pump();
        expect(k('cell-$last'), findsOneWidget);
        final before = s.commits;
        final row = s.row(last);
        final kind = kindOf(row);
        if (kind == PKind.toggle) {
          await t.tap(k('toy-$last'));
        } else if (kind == PKind.scalar || kind == PKind.bounded || kind == PKind.integer) {
          await t.drag(k('toy-$last'), const Offset(50, 0));
        } else if (kind == PKind.vec2) {
          await t.drag(k('toy-$last-0'), const Offset(50, 0));
        } else if (kind == PKind.choice) {
          await t.tap(k('choice-$last-2'));
        }
        await t.pump(const Duration(seconds: 1));
        if (kind != PKind.text) expect(s.commits, before + 1, reason: '$last ($kind) did not take an edit');
      });
    }
  });

  group('Phase 2: declared meaning refines the Value, it does not replace the Toy', () {
    Future<ParamStore> native(WidgetTester t, {SearchCapability? search}) => open(t, rows: nativeLike(), search: search);
    Finder unitIn(String id, String u) => find.descendant(of: k('cell-$id'), matching: find.text(u));

    test('only a declaration gives a row a character; a name alone gives none', () {
      expect(characterOf({'id': 'x', 'label': 'Angle', 'value': 1.0}), Character.none);
      expect(characterOf({'id': 'x', 'value': 1.0, 'subtype': 'ANGLE'}), Character.angle);
      expect(characterOf({'id': 'x', 'value': 1.0, 'character': 'seed'}), Character.seed);
      // finite opacity needs the declared range to be 0..1
      expect(characterOf({'id': 'x', 'value': .5, 'subtype': 'OPACITY'}), Character.none);
      expect(characterOf({'id': 'x', 'value': .5, 'subtype': 'OPACITY', 'min': 0.0, 'max': 1.0}), Character.opacity);
      // strangers stay generic
      expect(characterOf({'id': 'x', 'value': 'text', 'subtype': 'ANGLE'}), Character.none);
      expect(characterOf({'id': 'x', 'value': 1.0, 'subtype': 'WHATEVER'}), Character.none);
    });

    test('two rows of one declared group are one pair; nothing is paired by name', () {
      final folded = foldPairs(nativeLike());
      expect(folded.where((r) => r['kind'] == 'pair').map((r) => r['id']), ['position', 'scale']);
      expect(folded.firstWhere((r) => r['id'] == 'scale')['linkable'], true);
      final byName = foldPairs([
        {'id': 'a.param.thing_x', 'value': 1.0},
        {'id': 'a.param.thing_y', 'value': 2.0},
      ]);
      expect(byName.length, 2);
    });

    testWidgets('angle: degrees, whole-degree scrub, Shift fine, exact input keeps turns, reset', (t) async {
      final s = await native(t);
      expect(unitIn('l.param.rotation', '°'), findsOneWidget);
      await t.drag(k('toy-l.param.rotation'), const Offset(37, 0));
      await t.pump(const Duration(seconds: 1));
      expect(val(s, 'l.param.rotation'), 69.0); // 32 + 37 whole degrees
      await t.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
      await t.drag(k('toy-l.param.rotation'), const Offset(40, 0));
      await t.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
      await t.pump(const Duration(seconds: 1));
      expect((val(s, 'l.param.rotation') as double) - 69.0, closeTo(4.0, .11)); // a tenth of a degree per pixel
      await t.pump(const Duration(seconds: 1));
      await t.tap(k('toy-l.param.rotation'));
      await t.pump();
      await t.enterText(find.descendant(of: k('toy-l.param.rotation'), matching: find.byType(EditableText)), '720');
      await t.testTextInput.receiveAction(TextInputAction.done);
      await t.pump(const Duration(seconds: 1));
      expect(val(s, 'l.param.rotation'), 720.0); // turns are kept, not wrapped away
      await t.tap(k('reset-l.param.rotation'));
      await t.pump();
      expect(val(s, 'l.param.rotation'), 0.0);
      expect(k('anim-l.param.rotation'), findsOneWidget); // the keyed state is still shown
    });

    testWidgets('a row that is only called "Angle" stays a plain Value', (t) async {
      final s = await native(t);
      await reach(t, 'toy-l.param.angle');
      expect(unitIn('l.param.angle', '°'), findsNothing);
      await t.drag(k('toy-l.param.angle'), const Offset(30, 0));
      await t.pump(const Duration(seconds: 1));
      final v = val(s, 'l.param.angle') as double;
      expect(v, greaterThan(45.5));
      expect(v, isNot(v.roundToDouble())); // not snapped to whole degrees
    });

    testWidgets('count is never fractional and never negative, by drag, key and exact input', (t) async {
      final s = await native(t);
      await t.drag(k('toy-l.param.count'), const Offset(37, 0));
      await t.pump(const Duration(seconds: 1));
      var c = (val(s, "l.param.count") as num).toDouble();
      expect(c, c.roundToDouble());
      expect(c, greaterThan(12));
      await t.tap(k('toy-l.param.count'));
      await t.pump();
      final field = find.descendant(of: k('toy-l.param.count'), matching: find.byType(EditableText));
      await t.enterText(field, '2.7');
      await t.testTextInput.receiveAction(TextInputAction.done);
      await t.pump(const Duration(seconds: 1));
      expect(val(s, 'l.param.count'), 3);
      await t.pump(const Duration(seconds: 1));
      await realGap(t);
      await t.tap(k('toy-l.param.count'));
      await t.pump();
      await t.enterText(field, '-5');
      await t.testTextInput.receiveAction(TextInputAction.done);
      await t.pump(const Duration(seconds: 1));
      expect(val(s, 'l.param.count'), 0);
      await t.pump(const Duration(seconds: 1));
      await realGap(t);
      await t.tap(k('toy-l.param.count'));
      await t.pump();
      await t.sendKeyEvent(LogicalKeyboardKey.escape);
      await t.sendKeyEvent(LogicalKeyboardKey.arrowUp);
      c = (val(s, "l.param.count") as num).toDouble();
      expect(c, 1.0);
    });

    testWidgets('seed is a Value with a Reroll, and rerolls to a whole number', (t) async {
      final s = await native(t);
      expect(k('toy-l.param.seed'), findsOneWidget);
      await reach(t, 'action-l.param.seed-Reroll');
      final before = val(s, 'l.param.seed');
      await t.tap(k('action-l.param.seed-Reroll'));
      await t.pump();
      expect(s.actions, ['l.param.seed:Reroll']);
      expect(val(s, 'l.param.seed'), isA<int>());
      expect(val(s, 'l.param.seed'), isNot(before));
      expect(s.commits, 1);
      // nothing else grew a Reroll
      expect(find.text('Reroll'), findsOneWidget);
    });

    testWidgets('opacity: a true 0..1 domain, shown as a percent, clamps, and the bar and the number agree', (t) async {
      final s = await native(t);
      expect(k('track-l.param.opacity'), findsOneWidget);
      expect(unitIn('l.param.opacity', '%'), findsOneWidget);
      expect(find.text('72'), findsOneWidget);
      await t.drag(k('toy-l.param.opacity'), const Offset(800, 0));
      await t.pump(const Duration(seconds: 1));
      expect(val(s, 'l.param.opacity'), 1.0);
      expect(find.text('100'), findsOneWidget);
      await t.drag(k('toy-l.param.opacity'), const Offset(-2000, 0));
      await t.pump(const Duration(seconds: 1));
      expect(val(s, 'l.param.opacity'), 0.0);
      await t.pump(const Duration(seconds: 1));
      await realGap(t);
      await t.tap(k('toy-l.param.opacity'));
      await t.pump();
      final field = find.descendant(of: k('toy-l.param.opacity'), matching: find.byType(EditableText));
      await t.enterText(field, '150');
      await t.testTextInput.receiveAction(TextInputAction.done);
      await t.pump(const Duration(seconds: 1));
      expect(val(s, 'l.param.opacity'), 1.0);
      await t.pump(const Duration(seconds: 1));
      await realGap(t);
      await t.tap(k('toy-l.param.opacity'));
      await t.pump();
      await t.enterText(field, '30');
      await t.testTextInput.receiveAction(TextInputAction.done);
      await t.pump(const Duration(seconds: 1));
      expect(val(s, 'l.param.opacity'), closeTo(.3, 1e-9));
    });

    testWidgets('an OPACITY declaration whose range is not 0..1 is not trusted: it stays a plain Value', (t) async {
      await native(t);
      await reach(t, 'toy-l.param.opacity_boost');
      expect(k('track-l.param.opacity_boost'), findsNothing);
      expect(unitIn('l.param.opacity_boost', '%'), findsNothing);
    });

    testWidgets('position is one family of exact Values with a way to Depth; it does not pretend to be a pad', (t) async {
      final s = await native(t);
      expect(k('cell-position'), findsOneWidget);
      expect(k('toy-l.param.position_x'), findsOneWidget);
      expect(k('toy-l.param.position_y'), findsOneWidget);
      expect(find.byType(CustomPaint).evaluate().where((e) => e.widget.key == const ValueKey('pad')), isEmpty);
      await t.tap(k('toy-l.param.position_x'));
      await t.pump();
      await t.enterText(find.descendant(of: k('toy-l.param.position_x'), matching: find.byType(EditableText)), '250');
      await t.testTextInput.receiveAction(TextInputAction.done);
      await t.pump(const Duration(seconds: 1));
      expect(val(s, 'l.param.position_x'), 250.0);
      expect(val(s, 'l.param.position_y'), 180.0);
      await t.pump(const Duration(seconds: 1));
      // one reset for the family, and the route beside it
      await t.tap(k('reset-position'));
      await t.pump();
      expect(val(s, 'l.param.position_x'), 0.0);
      expect(val(s, 'l.param.position_y'), 0.0);
      await t.tap(k('route-acc-position'));
      await t.pump();
      expect(s.routes, ['Depth']);
      expect(s.routeFrom, ['position']);
    });

    testWidgets('scale link: apart by default, together when linked, one commit per drag, apart again when unlinked', (t) async {
      final s = await native(t);
      await reach(t, 'link-scale');
      await t.drag(k('toy-l.param.scale_x'), const Offset(30, 0));
      await t.pump(const Duration(seconds: 1));
      expect(val(s, 'l.param.scale_x'), isNot(100.0));
      expect(val(s, 'l.param.scale_y'), 100.0);
      await t.tap(k('toy-l.param.scale_x')); // back to a clean 100 for the ratio check
      await t.tap(k('toy-l.param.scale_x'));
      await t.pump(const Duration(seconds: 1));
      expect(val(s, 'l.param.scale_x'), 100.0);
      final commits0 = s.commits;
      await t.tap(k('link-scale'));
      await t.pump();
      await t.drag(k('toy-l.param.scale_x'), const Offset(30, 0));
      await t.pump(const Duration(seconds: 1));
      final x = val(s, 'l.param.scale_x') as double, y = val(s, 'l.param.scale_y') as double;
      expect(x, closeTo(y, 1e-6));
      expect(x, greaterThan(100));
      expect(s.commits, commits0 + 1); // the linked drag is one operation
      await t.tap(k('link-scale'));
      await t.pump();
      await t.drag(k('toy-l.param.scale_y'), const Offset(-20, 0));
      await t.pump(const Duration(seconds: 1));
      expect(val(s, 'l.param.scale_x'), x); // independent again
      expect(val(s, 'l.param.scale_y'), lessThan(y));
    });

    testWidgets('routes show the exact value here, hand over to the specialist, and leave everything as it was', (t) async {
      final search = SearchCapability(query: '');
      final s = await native(t, search: search);
      for (final (id, to, shown) in [('l.param.fill', 'Colors', '#F5C94A'), ('l.param.font', 'Fonts', 'Inter'), ('l.param.blend', 'Blend', 'Multiply'), ('l.param.ease', 'Ease', 'Ease In Out')]) {
        await reach(t, 'route-$id');
        expect(find.descendant(of: k('route-$id'), matching: find.text(shown)), findsOneWidget); // the current value stays readable
        final offset = t.state<ScrollableState>(find.descendant(of: k('insp-list'), matching: find.byType(Scrollable)).first).position.pixels;
        await t.tap(k('route-$id'));
        await t.pump();
        expect(s.routes.last, to);
        expect(s.routeFrom.last, id);
        expect(val(s, id), shown);
        // the Inspector is where it was: same scroll, same filter, same values
        expect(t.state<ScrollableState>(find.descendant(of: k('insp-list'), matching: find.byType(Scrollable)).first).position.pixels, offset);
        expect(search.query, '');
      }
      expect(s.routes, ['Colors', 'Fonts', 'Blend', 'Ease']);
      expect(s.commits, 0);
    });

    testWidgets('frozen keeps the meaning and refuses every change', (t) async {
      final s = await open(t, rows: nativeLike(), frozen: true);
      await t.drag(k('toy-l.param.rotation'), const Offset(50, 0));
      await reach(t, 'link-scale');
      await t.tap(k('link-scale'));
      await t.pump(const Duration(seconds: 1));
      expect(val(s, 'l.param.rotation'), 32.0);
      expect(s.linked, isEmpty);
      expect(s.commits, 0);
    });
  });
}
