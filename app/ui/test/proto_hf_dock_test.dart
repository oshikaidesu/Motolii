// Ordinary docking behaviour of the off-the-shelf `docking` package, with Motolii's registry/preset adapter.
import 'package:docking/docking.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_stage5/proto_hf/dock/workspace.dart';
import 'package:motolii_stage5/proto_hf/main_dock.dart' show motoliiTabs;

class Body extends StatefulWidget {
  const Body(this.name, {super.key});
  final String name;
  @override
  State<Body> createState() => _BodyState();
}

class _BodyState extends State<Body> {
  int n = 0;
  @override
  Widget build(BuildContext context) => GestureDetector(onTap: () => setState(() => n++), child: ColoredBox(color: const Color(0xFF191919), child: Center(child: Text('${widget.name}:$n'))));
}

Workspace make() => Workspace(
      {
        for (final e in {'create': 'Create', 'effects': 'Effects', 'stage': 'Stage', 'timeline': 'Timeline', 'transform': 'Transform', 'layer': 'Layer'}.entries)
          e.key: PanelDef(e.key, e.value, () => Body(e.value), minSize: e.key == 'timeline' ? 120 : e.key == 'stage' ? 260 : 220),
      },
      (item) => DockingRow([
        DockingTabs([item('create'), item('effects')], weight: .2),
        DockingColumn([
          DockingRow([item('stage', weight: .7), DockingTabs([item('transform'), item('layer')], weight: .3)]),
          item('timeline', weight: .3),
        ], weight: .8),
      ]),
    );

bool stockTheme = const bool.fromEnvironment('STOCK_THEME');

Future<void> pumpDock(WidgetTester tester, Workspace ws, {Size size = const Size(1536, 1024)}) async {
  tester.view.physicalSize = size;
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  await tester.pumpWidget(Directionality(
    textDirection: TextDirection.ltr,
    child: DefaultTextStyle(
      style: const TextStyle(fontSize: 12, color: Color(0xFFEEEEEE)),
      child: Overlay(initialEntries: [
        OverlayEntry(builder: (_) => TabbedViewTheme(data: stockTheme ? TabbedViewThemeData.dark() : motoliiTabs(), child: MultiSplitViewTheme(data: MultiSplitViewThemeData(dividerThickness: 4), child: ws.view()))),
      ]),
    ),
  ));
  await tester.pumpAndSettle();
}

Future<void> drag(WidgetTester tester, Offset from, Offset to) async {
  final g = await tester.startGesture(from);
  await g.moveBy(const Offset(0, -24));
  await tester.pump(const Duration(milliseconds: 50));
  for (var k = 1; k <= 14; k++) {
    await g.moveTo(Offset.lerp(from, to, k / 14)!);
    await tester.pump(const Duration(milliseconds: 16));
  }
  await g.up();
  await tester.pumpAndSettle();
}

Offset tabOf(WidgetTester tester, String title) => tester.getCenter(find.text(title).first);
String shape(Workspace ws) => ws.layout.stringify(parser: const _P());

class _P extends LayoutParser with LayoutParserMixin {
  const _P();
}

void main() {
  testWidgets('default workspace: four seats, every panel present', (tester) async {
    final ws = make();
    await pumpDock(tester, ws);
    for (final t in ['Stage', 'Timeline', 'Transform']) {
      expect(find.text(t), findsWidgets, reason: t);
    }
    expect(find.text('Create:0'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  testWidgets('resize: Browser width and Timeline height by dragging dividers', (tester) async {
    final ws = make();
    await pumpDock(tester, ws);
    final stage0 = ws.rectOf('stage')!, timeline0 = ws.rectOf('timeline')!;
    // the divider between Browser and the rest sits at the Browser's right edge
    await drag(tester, Offset(stage0.left - 2, 400), Offset(stage0.left + 118, 400));
    final stage1 = ws.rectOf('stage')!;
    // the divider between the upper row and the Timeline
    await drag(tester, Offset(700, timeline0.top - 42), Offset(700, timeline0.top - 162));
    final timeline1 = ws.rectOf('timeline')!;
    print('DOCK stage left ${stage0.left} -> ${stage1.left}; timeline top ${timeline0.top} -> ${timeline1.top}');
    expect(stage1.left, greaterThan(stage0.left + 60));
    expect(timeline1.top, lessThan(timeline0.top - 60));
  });

  testWidgets('dock the Inspector tab beside the Stage (edge drop)', (tester) async {
    final ws = make();
    await pumpDock(tester, ws);
    final before = shape(ws);
    final stage = ws.rectOf('stage')!;
    await drag(tester, tabOf(tester, 'Transform'), Offset(stage.left + stage.width * .08, stage.center.dy));
    print('DOCK edge drop changed the layout: ${before != shape(ws)}');
    expect(before != shape(ws), isTrue);
    expect(ws.rectOf('stage'), isNotNull);
  });

  testWidgets('tab-stack Browser panel with Inspector (centre drop on a tab strip)', (tester) async {
    final ws = make();
    await pumpDock(tester, ws);
    final before = shape(ws);
    final layerTab = tabOf(tester, 'Layer');
    await drag(tester, tabOf(tester, 'Create'), layerTab + const Offset(60, 0));
    final tabs = ws.layout.findDockingTabsWithItem('create')!;
    print('DOCK create now shares a strip with: ${[for (var i = 0; i < tabs.childrenCount; i++) tabs.childAt(i).id]}');
    expect(before != shape(ws), isTrue);
  });

  testWidgets('move the Timeline to another region', (tester) async {
    final ws = make();
    await pumpDock(tester, ws);
    final before = shape(ws), stage = ws.rectOf('stage')!;
    await drag(tester, tabOf(tester, 'Timeline'), Offset(stage.right - stage.width * .08, stage.center.dy));
    print('DOCK timeline moved: ${before != shape(ws)}; stage rect ${stage.size} -> ${ws.rectOf('stage')?.size}');
    expect(before != shape(ws), isTrue);
  });

  testWidgets('close, reopen, activate (tab click and programmatic)', (tester) async {
    final ws = make();
    await pumpDock(tester, ws);
    String step(String what) {
      final e = tester.takeException();
      final line = 'DOCK step "$what": ${e == null ? 'clean' : e.toString().split('\n').first}';
      print(line);
      return line;
    }

    await tester.tap(find.text('Layer').first);
    await tester.pumpAndSettle();
    step('tab click');
    expect(find.text('Layer:0').hitTestable(), findsOneWidget, reason: 'tab click activates');
    ws.activate('transform');
    await tester.pumpAndSettle();
    step('programmatic activate');
    expect(find.text('Transform:0').hitTestable(), findsOneWidget, reason: 'programmatic activation');
    ws.close('effects');
    await tester.pumpAndSettle();
    step('close a tab in a strip');
    expect(ws.isOpen('effects'), isFalse);
    ws.activate('effects');
    await tester.pumpAndSettle();
    step('reopen it');
    expect(ws.isOpen('effects'), isTrue);
    ws.close('timeline');
    await tester.pumpAndSettle();
    step('close a lone panel');
    expect(ws.isOpen('timeline'), isFalse);
    ws.activate('timeline');
    await tester.pumpAndSettle();
    step('reopen the lone panel');
    expect(ws.isOpen('timeline'), isTrue);
  });

  testWidgets('a body keeps its state through a dock move', (tester) async {
    final ws = make();
    await pumpDock(tester, ws);
    await tester.tap(find.text('Stage:0'));
    await tester.pump();
    expect(find.text('Stage:1'), findsOneWidget);
    final tl = ws.rectOf('timeline')!;
    await drag(tester, tabOf(tester, 'Stage'), Offset(tl.center.dx, tl.top + tl.height * .12));
    print('DOCK stage state after move: ${find.text('Stage:1').evaluate().isNotEmpty}');
    expect(find.text('Stage:1'), findsOneWidget);
  });

  testWidgets('narrow window 900x600: where the overflow comes from', (tester) async {
    final ws = make();
    await pumpDock(tester, ws, size: const Size(900, 600));
    final e = tester.takeException();
    print('DOCK 900px exception: ${e.toString().split('\n').first}');
    for (final id in ['create', 'stage', 'timeline', 'transform']) {
      print('DOCK 900px $id rect ${ws.rectOf(id)}');
    }
  });

  testWidgets('layout state is readable: tree in, tree out', (tester) async {
    final ws = make();
    await pumpDock(tester, ws);
    final s = shape(ws);
    final again = DockingLayout();
    again.load(layout: s, parser: const _P(), builder: const _B());
    expect(again.stringify(parser: const _P()), s);
  });
}

class _B extends AreaBuilder with AreaBuilderMixin {
  const _B();
  @override
  DockingItem buildDockingItem({required id, required double? weight, required bool maximized}) => DockingItem(id: id, name: '$id', widget: const SizedBox(), weight: weight);
  @override
  DockingTabs buildDockingTabs({required id, required double? weight, required bool maximized, required List<DockingItem> children}) => DockingTabs(children, id: id, weight: weight);
}
