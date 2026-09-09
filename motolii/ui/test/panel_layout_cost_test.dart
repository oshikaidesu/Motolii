import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/app/editor_app.dart';
import '../lib/app/editor_window.dart';
import '../lib/foundation/theme.dart';
import '../lib/panels/registry.dart';
import '../lib/session/editor_session.dart';
import '../lib/workspace/layout.dart';

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
    blocks.where((b) => !laidOut(b) && b.name != 'BUILD').fold(0, (sum, b) => sum + b.count),
    blocks,
  );
}

Map<String, dynamic> _number(String id, String label, Object v) => {
  'id': id,
  'label': label,
  'kind': 'number',
  'value': v,
  'keys': const [],
  'keyedNow': false,
  'min': null,
  'max': null,
};

/// A layer as the native side sends it: thirteen animatable rows, the flags
/// the Timeline reads, and one effect.
Map<String, dynamic> _layer(int id, String kind, double x) => {
  'id': id,
  'order': id,
  'name': '$kind $id',
  'kind': kind,
  'locked': false,
  'hidden': false,
  'solo': false,
  'frozen': false,
  'ghostable': true,
  'flatten': false,
  'clipToBelow': false,
  'environment': false,
  'blendMode': 'Normal',
  'projection': '2.5D',
  'start': 0,
  'sourceIn': 0,
  'duration': 180,
  'parent': null,
  'colors': const [],
  'contentKeys': const [],
  'x': x,
  'y': 244.0,
  'properties': [
    {
      'id': 'position',
      'label': 'Position',
      'kind': 'vec2',
      'value': [x, 244.0],
      'keys': const [],
      'keyedNow': false,
      'min': null,
      'max': null,
    },
    _number('position.z', 'Position Z', 0.0),
    {
      'id': 'scale',
      'label': 'Scale',
      'kind': 'vec2',
      'value': const [1.0, 1.0],
      'keys': const [],
      'keyedNow': false,
      'min': null,
      'max': null,
    },
    _number('scale.z', 'Scale Z', 1.0),
    _number('rotation', 'Rotation', 0.0),
    _number('rotation.x', 'Rotation X', 0.0),
    _number('rotation.y', 'Rotation Y', 0.0),
    _number('opacity', 'Opacity', 1.0),
    _number('text_style.0.size', 'Size', 14.0),
    _number('text_style.0.line_height', 'Line height', 144.0),
    _number('text_style.0.tracking', 'Tracking', 0.0),
  ],
  'effects': [
    {
      'id': 'motolii.blur',
      'name': 'Blur',
      'params': [_number('amount', 'Amount', 2.0)],
    },
  ],
};

/// The two documents of the report: the small one the window is judged on and
/// a fifteen-layer one that says how the cost grows.
Map<String, dynamic> _status(int layers, double x) => {
  'layers': [
    for (var i = 0; i < layers; i++)
      _layer(i + 1, i == 0 ? 'Cube' : 'Rectangle', i == 0 ? x : i * 3.0),
  ],
  'selectedId': 1,
  'selectedIds': const [1],
  'selectedKeys': const [],
  'assets': [
    for (var i = 0; i < 6; i++)
      {'id': 'a$i', 'name': 'clip$i.mp4', 'mime': 'video/mp4'},
  ],
  'background': const [0.1, 0.1, 0.1, 1.0],
  'backgrounds': const [],
  'catalog': const [
    {'id': 'motolii.isf_bloom', 'name': 'Bloom'},
    {'id': 'motolii.blur', 'name': 'Blur'},
    {'id': 'motolii.clip', 'name': 'Clip'},
    {'id': 'motolii.colorize', 'name': 'Colorize'},
    {'id': 'motolii.echo', 'name': 'Echo'},
    {'id': 'motolii.glow', 'name': 'Glow'},
    {'id': 'motolii.levels', 'name': 'Levels'},
    {'id': 'motolii.mirror', 'name': 'Mirror'},
    {'id': 'motolii.noise', 'name': 'Noise'},
    {'id': 'motolii.shadow', 'name': 'Shadow'},
    {'id': 'motolii.warp', 'name': 'Warp'},
  ],
  'palette': [
    for (var i = 0; i < 8; i++) {'id': 'p$i', 'hex': '#10101$i'},
  ],
  'easeKinds': const ['Linear', 'Hold', 'Ease', 'EaseIn', 'EaseOut'],
  'fontFamilies': const ['Arial', 'Georgia', 'Hiragino Sans'],
  'importExtensions': const ['png', 'jpg', 'mp4', 'obj'],
  'history': const {
    'head': 0,
    'entries': [
      {'head': 0, 'kind': 'edit', 'label': 'Open', 'detail': ''},
    ],
  },
  'capabilities': const [
    'setFont',
    'preview',
    'animate',
    'placeAsset',
    'previewProperties',
    'commitPreview',
    'cancelPreview',
    'removeAsset',
    'import',
  ],
  'durationFrames': 180,
  'fps': 30.0,
  'width': 1600,
  'height': 1000,
  'frame': 0,
  'stageView': 'free',
  'contentRevision': '$x',
  'documentRevision': '$x',
  'snapshotId': 1,
  'referenceId': 1,
};

/// Panel, and the pixels the default dock gives it inside a 1280x796 window.
const _panels = <String, Size>{
  'Create': Size(260, 537),
  'Media': Size(260, 537),
  'Effects': Size(260, 537),
  'Colors': Size(260, 537),
  'Fonts': Size(260, 537),
  'Stage': Size(712, 537),
  'Inspector': Size(300, 333),
  'Notes': Size(712, 537),
  'Desk': Size(300, 200),
  'Ease': Size(300, 200),
  'Depth': Size(300, 200),
  'Blend': Size(300, 200),
  'History': Size(300, 200),
  'Timeline': Size(1280, 255),
};

/// What one status update and one whole layout may cost a panel:
/// `(widgets rebuilt, layouts on the update, layouts when laid out whole)`.
/// Held so no panel goes back to measuring itself once per card, to
/// rebuilding its bar for every rendered frame, or to redrawing a shelf that
/// the update did not touch.
const _budget = <String, (int, int, int)>{
  'Create': (2, 2, 260),
  'Media': (2, 2, 300),
  'Effects': (2, 2, 270),
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
            return _status(layers, 0);
          default:
            return <String, dynamic>{};
        }
      });
  Future<void> mount() => tester.pumpWidget(
    MaterialApp(
      theme: EditorTheme.data,
      builder: EditorApp.noHover,
      home: const EditorWindow(),
    ),
  );
  await mount();
  await tester.pumpAndSettle();
  final dynamic host = tester.state(find.byType(EditorWindow));
  final c = host.c as EditorSession;
  var step = 0;
  void move() => c.document.value = _status(layers, (++step).toDouble());
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
      report.add(await _window(tester, layers));
      for (final entry in _panels.entries) {
        final c = EditorSession();
        addTearDown(c.dispose);
        c.document.value = _status(layers, 0);
        Future<void> mount(Size size) => tester.pumpWidget(
          MaterialApp(
            theme: EditorTheme.data,
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
        void move() => c.document.value = _status(layers, (++step).toDouble());
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
        expect(
          update.builds,
          lessThanOrEqualTo(rebuilt),
          reason: '${entry.key} rebuilds too much for one status update',
        );
        expect(
          update.calls,
          lessThanOrEqualTo(perUpdate),
          reason: '${entry.key} lays out too much for one status update',
        );
        expect(
          whole.calls,
          lessThanOrEqualTo(perLayout),
          reason: '${entry.key} lays out too much when laid out whole',
        );
        await tester.pumpWidget(const SizedBox());
      }
      debugPrint('LAYOUT CALLS, $layers layers\n${report.join('\n')}');
    });
  }
  _switchMain();
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
            return _status(3, 0);
          default:
            return <String, dynamic>{};
        }
      });
  await tester.pumpWidget(
    MaterialApp(
      theme: EditorTheme.data,
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
  c.document.value = _status(3, 1);
  await tester.pump();
  final pick = await _cost(tester, () async {
    c.document.value = {
      ..._status(3, 1),
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
    for (final entry in _switching.entries) {
      final c = EditorSession();
      addTearDown(c.dispose);
      c.document.value = _status(3, 0);
      await tester.pumpWidget(
        MaterialApp(
          theme: EditorTheme.data,
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
      c.document.value = _status(3, 1);
      await tester.pump();
      final pick = await _cost(tester, () async {
        c.document.value = {
          ..._status(3, 1),
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
      expect(
        pick.builds,
        lessThanOrEqualTo(rebuilt),
        reason: '${entry.key} rebuilds too much on one selection change',
      );
      expect(
        pick.calls,
        lessThanOrEqualTo(laidOut),
        reason: '${entry.key} lays out too much on one selection change',
      );
      await tester.pumpWidget(const SizedBox());
    }
    debugPrint('SELECTION CHANGE\n${report.join('\n')}');
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
