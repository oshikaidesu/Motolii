import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/app/editor_app.dart';
import '../lib/app/editor_window.dart';
import '../lib/panels/registry.dart';
import '../lib/session/editor_session.dart';
import '../lib/workspace/layout.dart';
import 'support/editor_test_theme.dart';
import 'support/window_fixture.dart';

/// What layout costs, panel by panel, on the two moves the window is judged
/// on: taking one status update, and being laid out whole.
///
/// `debugProfileLayoutsEnabled` names every `RenderObject.layout` call after
/// its type and [FlutterTimeline.debugCollect] adds them up. The count is the
/// same on every machine, so the budgets below hold counts, not milliseconds.
class _Cost {
  const _Cost(this.calls, this.builds, this.byType);
  final int calls, builds;
  final List<AggregatedTimedBlock> byType;
  List<AggregatedTimedBlock> get heaviest =>
      byType.toList()..sort((a, b) => b.count - a.count);
  String get top => heaviest
      .where((b) => b.name != 'BUILD')
      .take(4)
      .map((b) => '${b.name.replaceFirst('Render', '')} ${b.count}')
      .join(' ');

  /// The render objects that were laid out, heaviest first: what a spike on
  /// the first frame of a move is actually made of.
  String get laid => heaviest
      .where((b) => b.name.startsWith('Render') || b.name.startsWith('_Render'))
      .take(6)
      .map((b) => '${b.name.replaceFirst('Render', '')} ${b.count}')
      .join(' ');
}

Future<_Cost> _cost(WidgetTester tester, Future<void> Function() act) async {
  debugProfileBuildsEnabled = true;
  debugProfileLayoutsEnabled = true;
  FlutterTimeline.debugCollectionEnabled = true;
  await act();
  final blocks = FlutterTimeline.debugCollect().aggregatedBlocks;
  FlutterTimeline.debugCollectionEnabled = false;
  debugProfileLayoutsEnabled = false;
  debugProfileBuildsEnabled = false;
  bool laidOut(AggregatedTimedBlock b) =>
      b.name.startsWith('Render') || b.name.startsWith('_Render');
  return _Cost(
    blocks.where(laidOut).fold(0, (sum, b) => sum + b.count),
    blocks
        .where((b) => !laidOut(b) && b.name != 'BUILD')
        .fold(0, (sum, b) => sum + b.count),
    blocks,
  );
}

/// What one status update and one whole layout may cost a panel:
/// `(widgets rebuilt, layouts on the update, layouts when laid out whole)`.
/// Held so no panel goes back to measuring itself once per card, to
/// rebuilding its bar for every rendered frame, or to redrawing a shelf that
/// the update did not touch. A Browser card carries one pointer listener so
/// the pick lands on the press; that is one proxy box per card.
const _budget = <String, (int, int, int)>{
  'Create': (2, 2, 271),
  'Media': (2, 2, 300),
  'Effects': (2, 2, 292),
  'Colors': (2, 2, 320),
  'Fonts': (2, 2, 120),
  'Stage': (60, 8, 80),
  'Inspector': (120, 8, 400),
  'Notes': (2, 2, 50),
  'Desk': (2, 2, 110),
  'Ease': (2, 2, 150),
  'Depth': (2, 2, 50),
  'Blend': (2, 2, 340),
  'History': (2, 2, 60),
  'Timeline': (130, 4, 60),
};

/// The default dock, whole: five panels and the three Browser tabs the dock
/// keeps behind the front one.
Future<String> _window(WidgetTester tester, int layers) async {
  TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
      .setMockMethodCallHandler(EditorSession.channel, (call) async {
        switch (call.method) {
          case 'windowInfo':
            return {'id': 'main', 'main': true};
          case 'readSettings':
            return {'dock': initialDock().json()};
          case 'attach':
          case 'render':
            return windowStatus(layers, 0);
          default:
            return <String, dynamic>{};
        }
      });
  Future<void> mount() => tester.pumpWidget(
    MaterialApp(
      theme: editorTestTheme,
      builder: EditorApp.noHover,
      home: const EditorWindow(),
    ),
  );
  await mount();
  await tester.pumpAndSettle();
  final dynamic host = tester.state(find.byType(EditorWindow));
  final c = host.c as EditorSession;
  var step = 0;
  void move() => c.document.value = windowStatus(layers, (++step).toDouble());
  move();
  await tester.pump();
  final update = await _cost(tester, () async {
    move();
    await tester.pump();
  });
  final whole = await _cost(tester, () async {
    tester.view.physicalSize = const Size(1287, 803);
    await tester.pump();
  });
  tester.view.physicalSize = const Size(1280, 796);
  await tester.pumpWidget(const SizedBox());
  return '${'Window'.padRight(10)} '
      'update ${update.calls.toString().padLeft(5)}   '
      'builds ${update.builds.toString().padLeft(5)}   '
      'whole ${whole.calls.toString().padLeft(5)}   '
      '${whole.top}';
}

void main() {
  setUp(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(
          EditorSession.channel,
          (call) async => <String, dynamic>{},
        );
  });

  for (final layers in [3, 15]) {
    testWidgets('layout cost, $layers layers', (tester) async {
      tester.view.physicalSize = const Size(1280, 796);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.reset);
      final report = <String>[];
      // Every panel is measured and printed before any is judged, so one run
      // names every panel over budget.
      final over = <String>[];
      report.add(await _window(tester, layers));
      for (final entry in dockPanels.entries) {
        final c = EditorSession();
        addTearDown(c.dispose);
        c.document.value = windowStatus(layers, 0);
        Future<void> mount(Size size) => tester.pumpWidget(
          MaterialApp(
            theme: editorTestTheme,
            builder: EditorApp.noHover,
            home: Scaffold(
              body: Align(
                alignment: Alignment.topLeft,
                child: SizedBox.fromSize(
                  size: size,
                  child: buildPanel(entry.key, c, const ValueKey('panel')),
                ),
              ),
            ),
          ),
        );
        await mount(entry.value);
        await tester.pumpAndSettle();
        var step = 0;
        void move() =>
            c.document.value = windowStatus(layers, (++step).toDouble());
        // Warm: first update settles whatever mounting deferred.
        move();
        await tester.pump();
        final update = await _cost(tester, () async {
          move();
          await tester.pump();
        });
        // Whole: a size the panel has not seen leaves nothing to early-return.
        final whole = await _cost(
          tester,
          () => mount(Size(entry.value.width + 7, entry.value.height + 7)),
        );
        report.add(
          '${entry.key.padRight(10)} '
          'update ${update.calls.toString().padLeft(5)}   '
          'builds ${update.builds.toString().padLeft(5)}   '
          'whole ${whole.calls.toString().padLeft(5)}   '
          '${whole.top}',
        );
        final (rebuilt, perUpdate, perLayout) = _budget[entry.key]!;
        if (update.builds > rebuilt)
          over.add(
            '${entry.key} rebuilds ${update.builds} > $rebuilt '
            'for one status update',
          );
        if (update.calls > perUpdate)
          over.add(
            '${entry.key} lays out ${update.calls} > $perUpdate '
            'for one status update',
          );
        if (whole.calls > perLayout)
          over.add(
            '${entry.key} lays out ${whole.calls} > $perLayout '
            'when laid out whole',
          );
        await tester.pumpWidget(const SizedBox());
      }
      debugPrint('LAYOUT CALLS, $layers layers\n${report.join('\n')}');
      expect(over, isEmpty);
    });
  }
  _switchMain();
  _shelfMain();
}

/// The Browser tile under the pointer, by asset id.
Finder _tile(String id) => find.byKey(ValueKey('browser:Media:$id'));

/// One click in a shelf of five hundred: the frame between the press and the
/// second press of a double-click. The shelf may redraw the tile that lost
/// the pick and the one that took it, and the count that names it — not the
/// list, not the other tiles, and nothing it decodes.
void _shelfMain() {
  testWidgets('the first frame of a click in a shelf of 500', (tester) async {
    tester.view.physicalSize = const Size(1280, 796);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final c = EditorSession();
    addTearDown(c.dispose);
    c.document.value = {
      ...windowStatus(3, 0),
      'assets': [
        for (var i = 0; i < 500; i++)
          {
            'id': 'a$i',
            'name': 'clip$i.mp4',
            'mime': 'video/mp4',
            'path': '/m/clip$i.mp4',
          },
      ],
    };
    await tester.pumpWidget(
      MaterialApp(
        theme: editorTestTheme,
        builder: EditorApp.noHover,
        home: Scaffold(
          body: Align(
            alignment: Alignment.topLeft,
            child: SizedBox.fromSize(
              size: dockPanels['Media'],
              child: buildPanel('Media', c, const ValueKey('panel')),
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    // Warm: the first pick has no tile to unpick.
    await tester.tap(_tile('a1'));
    await tester.pump(const Duration(milliseconds: 400));
    final click = await _cost(tester, () async {
      await tester.tap(_tile('a2'));
      await tester.pump();
    });
    // The pick is on screen in the frame of the press, not after the
    // double-tap window.
    expect(find.byKey(const ValueKey('browser:band:a2')), findsOneWidget);
    expect(find.byKey(const ValueKey('browser:band:a1')), findsNothing);
    // The double-tap window closes before the test does.
    await tester.pump(const Duration(milliseconds: 400));
    debugPrint(
      'SHELF CLICK  builds ${click.builds}   layouts ${click.calls}   '
      '${click.top}',
    );
    expect(
      click.builds,
      lessThanOrEqualTo(100),
      reason: 'a click rebuilds more than the two tiles it touches',
    );
    expect(
      click.calls,
      lessThanOrEqualTo(20),
      reason: 'a click lays out more than the two tiles it touches',
    );
    await tester.pumpWidget(const SizedBox());
  });
}

/// The panels that read the selection, and the pixels the dock gives them.
/// The spike the window is judged on is the first frame after a click on the
/// Stage, so the move measured here is one selection change.
const _switching = <String, Size>{
  'Inspector': Size(300, 333),
  'Timeline': Size(1280, 255),
  'Ease': Size(300, 200),
  'Desk': Size(300, 200),
  'Blend': Size(300, 200),
  'Stage': Size(712, 537),
};

/// What one selection change may cost a panel: `(widgets rebuilt, layouts)`.
const _switchBudget = <String, (int, int)>{
  'Inspector': (1100, 20),
  'Timeline': (130, 6),
  'Ease': (2, 2),
  'Desk': (2, 2),
  'Blend': (2, 2),
  'Stage': (40, 8),
};

/// The default dock, mounted and settled, with its session.
Future<EditorSession> _mounted(WidgetTester tester) async {
  TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
      .setMockMethodCallHandler(EditorSession.channel, (call) async {
        switch (call.method) {
          case 'windowInfo':
            return {'id': 'main', 'main': true};
          case 'readSettings':
            return {'dock': initialDock().json()};
          case 'attach':
          case 'render':
            return windowStatus(3, 0);
          default:
            return <String, dynamic>{};
        }
      });
  await tester.pumpWidget(
    MaterialApp(
      theme: editorTestTheme,
      builder: EditorApp.noHover,
      home: const EditorWindow(),
    ),
  );
  await tester.pumpAndSettle();
  final dynamic host = tester.state(find.byType(EditorWindow));
  return host.c as EditorSession;
}

/// The whole dock, taking one selection change: the frame the window is
/// judged on when a layer is clicked on the Stage.
Future<String> _windowSwitch(WidgetTester tester) async {
  final c = await _mounted(tester);
  c.document.value = windowStatus(3, 1);
  await tester.pump();
  final pick = await _cost(tester, () async {
    c.document.value = {
      ...windowStatus(3, 1),
      'selectedId': 2,
      'selectedIds': const [2],
    };
    await tester.pump();
  });
  await tester.pumpWidget(const SizedBox());
  expect(
    pick.calls,
    lessThanOrEqualTo(40),
    reason: 'the window lays out too much on one selection change',
  );
  expect(
    pick.builds,
    lessThanOrEqualTo(1250),
    reason: 'the window rebuilds too much on one selection change',
  );
  return '${'Window'.padRight(10)} '
      'select builds ${pick.builds.toString().padLeft(5)}   '
      'layouts ${pick.calls.toString().padLeft(5)}   '
      '${pick.laid}';
}

void _switchMain() {
  testWidgets('the first frame of a selection change', (tester) async {
    tester.view.physicalSize = const Size(1280, 796);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final report = <String>[await _windowSwitch(tester)];
    final over = <String>[];
    for (final entry in _switching.entries) {
      final c = EditorSession();
      addTearDown(c.dispose);
      c.document.value = windowStatus(3, 0);
      await tester.pumpWidget(
        MaterialApp(
          theme: editorTestTheme,
          builder: EditorApp.noHover,
          home: Scaffold(
            body: Align(
              alignment: Alignment.topLeft,
              child: SizedBox.fromSize(
                size: entry.value,
                child: buildPanel(entry.key, c, const ValueKey('panel')),
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      // Warm: one move of the values, so nothing mounting deferred is counted.
      c.document.value = windowStatus(3, 1);
      await tester.pump();
      final pick = await _cost(tester, () async {
        c.document.value = {
          ...windowStatus(3, 1),
          'selectedId': 2,
          'selectedIds': const [2],
        };
        await tester.pump();
      });
      report.add(
        '${entry.key.padRight(10)} '
        'select builds ${pick.builds.toString().padLeft(5)}   '
        'layouts ${pick.calls.toString().padLeft(5)}   '
        '${pick.laid}',
      );
      final (rebuilt, laidOut) = _switchBudget[entry.key]!;
      if (pick.builds > rebuilt)
        over.add(
          '${entry.key} rebuilds ${pick.builds} > $rebuilt '
          'on one selection change',
        );
      if (pick.calls > laidOut)
        over.add(
          '${entry.key} lays out ${pick.calls} > $laidOut '
          'on one selection change',
        );
      await tester.pumpWidget(const SizedBox());
    }
    debugPrint('SELECTION CHANGE\n${report.join('\n')}');
    expect(over, isEmpty);
  });

  testWidgets('the first frame of a pane taking the ring', (tester) async {
    tester.view.physicalSize = const Size(1280, 796);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    await _mounted(tester);
    // Press in one pane, then in another: the ring moves from the Stage to
    // the Inspector, and that is all the window may do about it.
    final first = await tester.startGesture(const Offset(640, 300));
    await tester.pump();
    await first.up();
    await tester.pump();
    final second = await tester.startGesture(const Offset(1150, 300));
    final ring = await _cost(tester, () => tester.pump());
    await second.up();
    await tester.pump();
    debugPrint(
      'RING       press  builds ${ring.builds}   layouts ${ring.calls}',
    );
    expect(
      ring.builds,
      lessThanOrEqualTo(80),
      reason: 'the ring must not rebuild the panels it is drawn around',
    );
    expect(ring.calls, lessThanOrEqualTo(20));
    await tester.pumpWidget(const SizedBox());
  });
}
