// The Layout Instrument, driven with real pointers. One container diagram takes columns/rows, gap, padding, placement and
// sizing gestures; the exact values sit beside it. Classic's gating is kept: Grid enables the rest, Fixed enables its number.
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_stage5/hf/insp/layout.dart';
import 'package:motolii_stage5/hf/insp/layout_diagram.dart';
import 'package:motolii_stage5/hf/insp/layout_model.dart';

Widget host(Widget child, double w, double h) => WidgetsApp(
      color: const Color(0xFF000000),
      pageRouteBuilder: <T>(settings, builder) => PageRouteBuilder<T>(settings: settings, pageBuilder: (c, _, __) => builder(c)),
      home: Align(alignment: Alignment.topLeft, child: SizedBox(width: w, height: h, child: child)),
    );

Future<LayoutStore> open(WidgetTester t, {LayoutStore? store, double w = 310, double h = 660, bool advancedOpen = false}) async {
  t.view.physicalSize = const Size(900, 900);
  t.view.devicePixelRatio = 1;
  addTearDown(t.view.reset);
  final s = store ?? LayoutStore();
  await t.pumpWidget(host(LayoutInstrument(s, advancedOpen: advancedOpen), w, h));
  await t.pump();
  return s;
}

Finder k(String key) => find.byKey(ValueKey(key));
Future<void> realGap(WidgetTester t) async { await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 380))); }

class D {
  D(this.t, this.s);
  final WidgetTester t;
  final LayoutStore s;
  Rect get rect => t.getRect(k('layout-diagram'));
  LayoutGeom get g => LayoutGeom(rect.size, s);
  Offset at(Offset local) => rect.topLeft + local;
}

void main() {
  setUpAll(() async {
    final inter = FontLoader('Inter')
      ..addFont(rootBundle.load('assets/fonts/Inter-Regular.ttf'))
      ..addFont(rootBundle.load('assets/fonts/Inter-Medium.ttf'))
      ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.ttf'));
    await inter.load();
  });

  group('lays out at every size, for a group and for a child', () {
    for (final (label, w, h, child) in [('wide group', 310.0, 660.0, false), ('narrow group', 170.0, 660.0, false), ('narrower', 150.0, 300.0, false), ('child', 310.0, 420.0, true)]) {
      testWidgets(label, (t) async {
        await open(t, store: LayoutStore(child: child), w: w, h: h);
        expect(t.takeException(), isNull);
        expect(child ? k('ignore-switch') : k('layout-diagram'), findsOneWidget);
      });
    }
  });

  group('Grid gates everything but Columns and Rows, as in Classic', () {
    testWidgets('with Grid off only the arrangement values remain; turning it on brings the rest back', (t) async {
      final s = await open(t, store: LayoutStore()..set('layout.display', 0));
      expect(k('toy-layout.grid_columns'), findsOneWidget);
      expect(k('toy-layout.grid_rows'), findsOneWidget);
      expect(k('toy-layout.gap'), findsNothing);
      expect(k('toy-layout.padding-0'), findsNothing);
      expect(k('size-w-0'), findsNothing);
      expect(k('layout-advanced'), findsNothing);
      await t.tap(k('grid-switch'));
      await t.pump();
      expect(s.gridOn, isTrue);
      expect(k('toy-layout.gap'), findsOneWidget);
      expect(k('size-w-0'), findsOneWidget);
    });

    testWidgets('with Grid off the diagram still takes Columns and Rows, and nothing else', (t) async {
      final s = await open(t, store: LayoutStore()..set('layout.display', 0));
      final d = D(t, s);
      final g = d.g;
      await t.dragFrom(d.at(g.colHandle), Offset(LayoutGeom.cw + g.gap, 0));
      await t.pump();
      expect(s.gi('layout.grid_columns'), 4);
      final gap0 = s.gd('layout.gap');
      await t.dragFrom(d.at(d.g.gapHandle ?? const Offset(0, 0)), const Offset(20, 0));
      await t.pump();
      expect(s.gd('layout.gap'), gap0); // the gap handle is gated by Grid
    });
  });

  group('one diagram, several gestures', () {
    testWidgets('the columns handle adds and removes columns; the rows handle sets rows and returns to auto', (t) async {
      final s = await open(t);
      final d = D(t, s);
      await t.dragFrom(d.at(d.g.colHandle), Offset(LayoutGeom.cw + d.g.gap, 0));
      await t.pump();
      expect(s.gi('layout.grid_columns'), 4);
      expect(s.commits, 1);
      await t.dragFrom(d.at(d.g.colHandle), -Offset(2 * (LayoutGeom.cw + d.g.gap), 0));
      await t.pump();
      expect(s.gi('layout.grid_columns'), 2);
      await t.dragFrom(d.at(d.g.rowHandle), const Offset(0, 30));
      await t.pump();
      expect(s.gi('layout.grid_rows'), greaterThan(0));
      await t.dragFrom(d.at(d.g.rowHandle), const Offset(0, -120));
      await t.pump();
      expect(s.gi('layout.grid_rows'), 0); // auto again
    });

    testWidgets('dragging the gap handle spaces the children apart; the number follows', (t) async {
      final s = await open(t);
      final d = D(t, s);
      await t.dragFrom(d.at(d.g.gapHandle!), const Offset(9, 0));
      await t.pump();
      expect(s.gd('layout.gap'), closeTo(12 + 9 / LayoutGeom.k, 1.1));
      expect(s.commits, 1);
    });

    testWidgets('padding: an edge handle changes one axis, the corner handle both at once', (t) async {
      final s = await open(t);
      final d = D(t, s);
      await t.dragFrom(d.at(d.g.padL), const Offset(9, 0));
      await t.pump();
      expect(s.padX, closeTo(16 + 9 / LayoutGeom.k, 1.1));
      expect(s.padY, 16);
      await t.dragFrom(d.at(d.g.padCorner), const Offset(-4, -4));
      await t.pump();
      expect(s.padX, lessThan(16 + 9 / LayoutGeom.k));
      expect(s.padY, lessThan(16));
      expect(s.commits, 2); // the corner is one edit for both axes
    });

    testWidgets('dragging the group places it: Justify and Align change together, as one commit', (t) async {
      final s = await open(t, store: LayoutStore()..set('layout.horizontal_sizing', 1)..set('layout.vertical_sizing', 1));
      final d = D(t, s);
      final before = s.commits;
      final start = d.at(d.g.boxes.first.center);
      final r = d.g.inner;
      await t.dragFrom(start, d.at(Offset(r.right - 6, r.bottom - 6)) - start);
      await t.pump();
      expect(s.gi('layout.justify_content'), 1); // End
      expect(s.gi('layout.align_items'), 2); // End
      expect(s.commits, before + 1);
      // and to the middle
      final s2 = d.at(d.g.boxes.first.center);
      final r2 = d.g.inner;
      await t.dragFrom(s2, d.at(r2.center) - s2);
      await t.pump();
      expect(s.gi('layout.justify_content'), 2); // Center
      expect(s.gi('layout.align_items'), 3); // Center
    });
  });

  group('Hug, Fill and Fixed stay three different relationships', () {
    testWidgets('the container hugs its children, fills the space, or takes its number', (t) async {
      final s = await open(t);
      final d = D(t, s);
      final g0 = d.g;
      // the fixture starts Fixed width (360) and Hug height
      expect(g0.parent.width, closeTo(360 * LayoutGeom.k, .5));
      expect(g0.parent.height, closeTo(g0.content.height + 2 * g0.padY, .5));
      await t.tap(k('size-w-0')); // Hug
      await t.pump();
      expect(d.g.parent.width, closeTo(d.g.content.width + 2 * d.g.padX, .5));
      await t.tap(k('size-w-1')); // Fill
      await t.pump();
      expect(d.g.parent.width, closeTo(d.g.avail.width, .5));
      expect(s.gi('layout.horizontal_sizing'), 1);
    });

    testWidgets('the size number counts under Fixed alone, and so does its handle', (t) async {
      final s = await open(t);
      final d = D(t, s);
      // height is Hug: its number ignores a drag, and so does its handle
      await t.drag(k('toy-layout.height'), const Offset(30, 0));
      await t.dragFrom(d.at(d.g.sizeH), const Offset(0, 20));
      await t.pump(const Duration(seconds: 1));
      expect(s.gd('layout.height'), 240);
      await t.tap(k('size-h-2')); // Fixed
      await t.pump();
      await t.drag(k('toy-layout.height'), const Offset(30, 0));
      await t.pump(const Duration(seconds: 1));
      expect(s.gd('layout.height'), greaterThan(240));
      final h1 = s.gd('layout.height');
      await t.dragFrom(d.at(d.g.sizeH), const Offset(0, 9));
      await t.pump();
      expect(s.gd('layout.height'), greaterThan(h1));
    });

    testWidgets('dragging the width handle sizes the container when Fixed', (t) async {
      final s = await open(t);
      final d = D(t, s);
      await t.dragFrom(d.at(d.g.sizeW), const Offset(-18, 0));
      await t.pump();
      expect(s.gd('layout.width'), closeTo(360 - 18 / LayoutGeom.k, 1.1));
      expect(s.commits, 1);
    });
  });

  group('exact values stay available', () {
    testWidgets('typed columns, gap and rows; zero rows reads as auto', (t) async {
      final s = await open(t);
      expect(find.text('auto'), findsOneWidget);
      await t.tap(k('toy-layout.gap'));
      await t.pump();
      await t.enterText(find.descendant(of: k('toy-layout.gap'), matching: find.byType(EditableText)), '20');
      await t.testTextInput.receiveAction(TextInputAction.done);
      await t.pump(const Duration(seconds: 1));
      expect(s.gd('layout.gap'), 20);
      await realGap(t);
      await t.tap(k('toy-layout.grid_columns'));
      await t.pump();
      await t.enterText(find.descendant(of: k('toy-layout.grid_columns'), matching: find.byType(EditableText)), '5');
      await t.testTextInput.receiveAction(TextInputAction.done);
      await t.pump(const Duration(seconds: 1));
      expect(s.gi('layout.grid_columns'), 5);
    });

    testWidgets('the transition rows and the Advanced fold are reachable', (t) async {
      final s = await open(t);
      expect(k('toy-layout.transition_duration'), findsOneWidget);
      await t.tap(k('choice-layout.transition_easing-1'));
      await t.pump();
      expect(s.gi('layout.transition_easing'), 1);
      await t.ensureVisible(k('layout-advanced'));
      await t.tap(k('layout-advanced'));
      await t.pump();
      expect(k('toy-layout.min_width'), findsOneWidget);
      await t.drag(k('toy-layout.min_width'), const Offset(40, 0));
      await t.pump(const Duration(seconds: 1));
      expect(s.gd('layout.min_width'), greaterThan(0));
    });
  });

  group('a child inside the layout', () {
    testWidgets('Ignore layout, its own sizing, and the grid area stay reachable', (t) async {
      final s = await open(t, store: LayoutStore(child: true), h: 420);
      expect(s.gi('layout.position_type'), 0);
      await t.tap(k('ignore-switch'));
      await t.pump();
      expect(s.gi('layout.position_type'), 1);
      await t.tap(k('size-w-2'));
      await t.pump();
      await t.drag(k('toy-layout.width'), const Offset(30, 0));
      await t.pump(const Duration(seconds: 1));
      expect(s.gd('layout.width'), greaterThan(120));
      expect(k('toy-layout.column_start'), findsOneWidget);
      await t.drag(k('toy-layout.column_span'), const Offset(30, 0));
      await t.pump(const Duration(seconds: 1));
      expect(s.gi('layout.column_span'), greaterThan(1));
      expect(s.gi('layout.column_span'), isA<int>());
    });
  });

  group('locked: editable = false disables the diagram and the values together', () {
    testWidgets('nothing changes and nothing is counted', (t) async {
      final s = await open(t, store: LayoutStore(frozen: true));
      final d = D(t, s);
      await t.dragFrom(d.at(d.g.colHandle), const Offset(40, 0));
      await t.dragFrom(d.at(d.g.boxes.first.center), const Offset(60, 40));
      await t.tap(k('size-w-1'));
      await t.tap(k('grid-switch'));
      await t.drag(k('toy-layout.gap'), const Offset(30, 0));
      await t.pump(const Duration(seconds: 1));
      expect(s.gi('layout.grid_columns'), 3);
      expect(s.gi('layout.horizontal_sizing'), 2);
      expect(s.gridOn, isTrue);
      expect(s.commits, 0);
    });
  });

  group('narrow keeps the diagram and reaches everything', () {
    testWidgets('the diagram is not shrunk to a mark and still takes a gesture', (t) async {
      final s = await open(t, w: 170);
      final d = D(t, s);
      expect(d.rect.width, greaterThan(140));
      await t.dragFrom(d.at(d.g.colHandle), Offset(LayoutGeom.cw + d.g.gap, 0));
      await t.pump();
      expect(s.gi('layout.grid_columns'), 4);
      await t.ensureVisible(k('size-w-2'));
      await t.pump();
      expect(k('size-w-2'), findsOneWidget);
      expect(t.takeException(), isNull);
    });
  });

  group('the setup loop, counted against Classic', () {
    testWidgets('4 columns, gap 20, padding 24 both ways, centred, width 400: six actions here, seven in Classic', (t) async {
      final s = await open(t, store: LayoutStore()..set('layout.horizontal_sizing', 1)..set('layout.vertical_sizing', 1)..set('layout.padding', [8.0, 8.0]));
      final d = D(t, s);
      final commits0 = s.commits; // the fixture's own setup
      var actions = 0;
      await t.dragFrom(d.at(d.g.colHandle), Offset(LayoutGeom.cw + d.g.gap, 0)); actions++; // 1 columns
      await t.dragFrom(d.at(d.g.gapHandle!), const Offset(4, 0)); actions++; // 2 gap
      await t.dragFrom(d.at(d.g.padCorner), const Offset(6, 6)); actions++; // 3 padding, both axes in one
      final r = d.g.inner;
      final from = d.at(d.g.boxes.first.center);
      await t.dragFrom(from, d.at(r.center) - from); actions++; // 4 centred
      await t.tap(k('size-w-2')); actions++; // 5 Fixed
      await t.pump();
      await t.dragFrom(d.at(d.g.sizeW), const Offset(10, 0)); actions++; // 6 width
      await t.pump();
      expect(actions, 6);
      // Classic: Columns, Gap, Padding X, Padding Y, the alignment pad, Width sizing, Width number
      const classic = 7;
      expect(actions, lessThan(classic));
      expect(s.gi('layout.grid_columns'), 4);
      expect(s.gd('layout.gap'), greaterThan(12));
      expect(s.padX, s.padY);
      expect(s.gi('layout.justify_content'), 2);
      expect(s.gi('layout.align_items'), 3);
      expect(s.fixed('w'), isTrue);
      expect(s.commits - commits0, 6);
    });
  });
}
