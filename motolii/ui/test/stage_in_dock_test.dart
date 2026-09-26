// The production Stage inside the dock: a press lands on the same composition point wherever the dock puts it.
import 'package:docking/docking.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/app/new/dock_theme.dart';
import '../lib/panels/stage.dart';
import '../lib/session/editor_session.dart';
import '../lib/workspace/dock_workspace.dart';
import 'support/editor_test_theme.dart';
import 'support/painters.dart';

class GestureSession extends EditorSession {
  final commands = <(String, Map<String, dynamic>)>[];
  final attached = <String>[];
  @override
  Future<void> attachView(String view) async {
    attached.add(view);
  }

  @override
  Future<void> refreshPreview() async {}
  @override
  Future<void> command(String op, [Map<String, dynamic> args = const {}]) async {
    commands.add((op, args));
  }
}

const _doc = {
  'width': 400,
  'height': 400,
  'frame': 0,
  'documentRevision': 'r1',
  'stageView': 'User',
  'capabilities': ['stageGesture', 'select'],
  'observer': {'front': true},
  'layers': [
    {
      'id': 7,
      'kind': 'Shape',
      'parent': null,
      'projection': '2D',
      'stageBounds': {
        'corners': [[100, 100], [300, 100], [300, 300], [100, 300]],
      },
    },
  ],
  'selectedId': 7,
  'selectedIds': [7],
};

/// Composition point to screen, from the overlay the Stage draws its frame with.
Offset Function(double, double) screen(WidgetTester tester) {
  final overlay = painterCarrying((p) => p.viewport is Rect && p.dimOutside is bool);
  final frame = (tester.widget<CustomPaint>(overlay).painter as dynamic).viewport as Rect;
  final rect = frame.shift(tester.getTopLeft(overlay));
  final scale = rect.width / 400;
  return (double x, double y) => rect.topLeft + Offset(x, y) * scale;
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

void ignoreSqueezedTabChips() {
  final report = FlutterError.onError;
  FlutterError.onError = (details) {
    final text = details.toString(minLevel: DiagnosticLevel.debug);
    final chip = text.contains('RenderFlex overflowed') &&
        RegExp(r'constraints: BoxConstraints\(w=(\d+\.\d+), h=\d+\.\d+\)').allMatches(text).any((m) => double.parse(m.group(1)!) < 40);
    if (!chip) report?.call(details);
  };
}

void main() {
  testWidgets('a press on the cage corner lands on the same composition point wherever the dock puts the Stage', (tester) async {
    tester.view.physicalSize = const Size(1500, 1000);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    ignoreSqueezedTabChips();
    final c = GestureSession()..document.value = _doc;
    Widget filler(String name) => ColoredBox(color: const Color(0xFF202020), child: Center(child: Text(name)));
    final ws = DockWorkspace(
      {
        'Browser': PanelDef('Browser', 'BROWSER', () => filler('browser-body'), minSize: 200),
        'Stage': PanelDef('Stage', 'STAGE', () => StagePanel(key: const ValueKey('stage'), controller: c), minSize: 260),
        'Camera': PanelDef('Camera', 'CAMERA', () => filler('camera-body'), minSize: 200),
        'Timeline': PanelDef('Timeline', 'TIMELINE', () => filler('timeline-body'), minSize: 120),
      },
      (item) => DockingRow([
        item('Browser', weight: .2),
        DockingColumn([
          DockingTabs([item('Stage'), item('Camera')], weight: .7),
          item('Timeline', weight: .3),
        ], weight: .8),
      ]),
    );
    await tester.pumpWidget(MaterialApp(
      theme: editorTestTheme,
      home: Scaffold(
        body: TabbedViewTheme(data: shellTabs(), child: MultiSplitViewTheme(data: shellSplit(), child: ws.view())),
      ),
    ));
    await tester.pumpAndSettle();

    var presses = 0;

    /// Press the north-west handle of the selected layer's cage; return what the Stage told native, and where.
    Future<void> pressCorner(String when) async {
      c.commands.clear();
      // The handle is grabbed within 7 px; the Stage counts two presses within 6 px in quick succession as a double tap,
      // so each press lands on the other side of the handle.
      final side = presses.isEven ? 3.2 : -3.2;
      presses++;
      final at = screen(tester)(100, 100) + Offset(side, 0);
      final press = await tester.startGesture(at);
      await tester.pump();
      final begin = c.commands.where((v) => v.$1 == 'stageGesture').toList();
      if (begin.isEmpty) print('STAGE $when: commands ${c.commands.map((v) => v.$1).toList()}, at $at, stage rect ${tester.getRect(find.byType(StagePanel))}, hit ${find.byType(StagePanel).hitTestable().evaluate().length}');
      expect(begin, isNotEmpty, reason: '$when: the press reached the Stage');
      expect(begin.first.$2['mode'], 'scale', reason: when);
      expect(begin.first.$2['handle'], 'nw', reason: when);
      await press.up();
      await tester.pumpAndSettle();
      print('STAGE $when: at $at, stage ${tester.getRect(find.byType(StagePanel)).size}');
    }

    await pressCorner('as placed');

    // A divider dragged: the Stage is narrower and elsewhere.
    final before = ws.rectOf('Stage')!;
    await drag(tester, Offset(before.left - 2, 500), Offset(before.left + 140, 500));
    expect(ws.rectOf('Stage')!.left, greaterThan(before.left + 60));
    await pressCorner('after the Browser divider moved');

    // Another tab in front and back again.
    await tester.tap(find.text('CAMERA'));
    await tester.pumpAndSettle();
    expect(find.byType(StagePanel).hitTestable(), findsNothing, reason: 'behind the Camera tab it is not on the face');
    await tester.tap(find.text('STAGE'));
    await tester.pumpAndSettle();
    await pressCorner('after another tab was in front');

    // Dragged out of its strip to sit above the Timeline.
    final timeline = ws.rectOf('Timeline')!;
    await drag(tester, tester.getCenter(find.text('STAGE').first), Offset(timeline.center.dx, timeline.top + timeline.height * .12));
    expect(ws.rectOf('Stage')!.bottom, lessThanOrEqualTo(timeline.top + 60));
    await pressCorner('after moving above the Timeline');

    // Docked as a tab beside the Timeline.
    final tl = tester.getCenter(find.text('TIMELINE').first);
    await drag(tester, tester.getCenter(find.text('STAGE').first), tl + const Offset(70, 0));
    await pressCorner('after joining the Timeline strip');
    expect(find.byType(StagePanel), findsOneWidget, reason: 'one Stage, not a copy left behind');
    expect(tester.takeException(), isNull);
  });
}
