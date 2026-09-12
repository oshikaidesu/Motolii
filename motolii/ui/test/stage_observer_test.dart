import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/stage.dart';
import '../lib/foundation/theme.dart';

/// rerun 3D view の取説: object をダブルクリックで Focus、背景をダブルクリックで Reset view。
class ObserverSession extends EditorSession {
  final commands = <(String, Map<String, dynamic>)>[];
  final attached = <String>[];
  @override
  Future<void> refreshPreview() async {}
  @override
  Future<void> command(
    String op, [
    Map<String, dynamic> args = const {},
  ]) async {
    commands.add((op, args));
  }

  @override
  Future<dynamic> native(
    String method, [
    Map<String, dynamic> args = const {},
  ]) async {
    if (method == 'attach') attached.add('${args['view'] ?? 'Camera'}');
    return <String, dynamic>{};
  }
}

void main() {
  testWidgets(
    'double-click focuses the object under the pointer, background resets',
    (tester) async {
      final c = ObserverSession();
      c.document.value = {
        'width': 400,
        'height': 400,
        'frame': 0,
        'documentRevision': 'r1',
        'stageView': 'User',
        'observer': {
          'front': false,
          'orbit': [10, 20],
        },
        'layers': [
          {
            'id': 7,
            'stageBounds': {
              'corners': [
                [0, 0],
                [200, 0],
                [200, 200],
                [0, 200],
              ],
            },
          },
        ],
        'selectedIds': [],
      };
      await tester.pumpWidget(
        MaterialApp(
          theme: EditorTheme.data,
          home: Scaffold(
            body: SizedBox(
              width: 464,
              height: 480,
              child: StagePanel(controller: c),
            ),
          ),
        ),
      );
      await tester.pump();
      final stage = tester.getRect(find.byType(StagePanel));
      // The 400×400 composition fits the 464×(480−toolbar) viewport with a 16px margin.
      final inside = stage.topLeft + const Offset(60, 80);
      final outside = stage.topLeft + const Offset(440, 440);
      await tester.tapAt(inside);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.tapAt(inside);
      await tester.pump();
      expect(c.commands.last.$1, 'stageView');
      expect(c.commands.last.$2, {'focus': 7});
      await tester.pump(const Duration(seconds: 1));
      await tester.tapAt(outside);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.tapAt(outside);
      await tester.pump();
      expect(c.commands.last.$1, 'stageView');
      expect(c.commands.last.$2, {'reset': true});
      expect(find.text('Front'), findsOneWidget);
      await tester.pumpWidget(const SizedBox());
    },
  );

  /// Boxcam の working comp: 伸ばす許可を入れた時に範囲の物体が無ければ作り、Fit はその範囲へ寄る。
  testWidgets('Extend creates the stage layer once and Fit asks the observer', (
    tester,
  ) async {
    final c = ObserverSession();
    c.document.value = {
      'width': 400,
      'height': 400,
      'stageView': 'User',
      'observer': {'front': true, 'home': true},
      'capabilities': ['create', 'stageView'],
      'layers': [],
      'selectedIds': [],
    };
    await tester.pumpWidget(
      MaterialApp(
        theme: EditorTheme.data,
        home: Scaffold(
          body: SizedBox(
            width: 464,
            height: 480,
            child: StagePanel(controller: c),
          ),
        ),
      ),
    );
    await tester.pump();
    await tester.tap(find.text('Extend'));
    await tester.pump();
    expect(c.deskWork.value['stageExtend'], isTrue);
    expect(c.commands.last.$1, 'create');
    expect(c.commands.last.$2, {'kind': 'stage'});
    c.document.value = {
      ...c.document.value,
      'observer': {
        'front': true,
        'home': true,
        'extent': {
          'layer': 3,
          'margins': [0, 0, 0, 0],
          'points': [
            [0, 0],
            [400, 0],
            [400, 400],
            [0, 400],
          ],
        },
      },
    };
    await tester.pump();
    await tester.tap(find.text('● Extend'));
    await tester.pump();
    await tester.tap(find.text('Extend'));
    await tester.pump();
    expect(
      c.commands.where((e) => e.$1 == 'create').length,
      1,
      reason: 'an existing stage layer is reused',
    );
    await tester.tap(find.text('Fit'));
    await tester.pump();
    expect(c.commands.last.$1, 'stageView');
    expect(c.commands.last.$2, {'fit': true});
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('a hovered anchor is marked on the selected layer', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(900, 700);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final c = ObserverSession();
    c.document.value = {
      'width': 400,
      'height': 400,
      'layers': [
        {
          'id': 7,
          'name': 'a',
          'kind': 'Image',
          'corners': [
            [0, 0],
            [200, 0],
            [200, 200],
            [0, 200],
          ],
          'stageBounds': {
            'corners': [
              [0, 0],
              [200, 0],
              [200, 200],
              [0, 200],
            ],
          },
        },
      ],
      'selectedIds': [7],
    };
    await tester.pumpWidget(
      MaterialApp(
        theme: EditorTheme.data,
        home: Scaffold(
          body: SizedBox(
            width: 464,
            height: 480,
            child: StagePanel(controller: c),
          ),
        ),
      ),
    );
    await tester.pump();
    CustomPaint overlay() => tester
        .widgetList<CustomPaint>(find.byType(CustomPaint))
        .firstWhere((w) => '${w.painter.runtimeType}' == '_StageOverlay');
    expect((overlay().painter as dynamic).anchorPreview, isNull);
    c.anchorPreview.value = [1.0, 0.0];
    await tester.pump();
    final at = (overlay().painter as dynamic).anchorPreview as Offset?;
    expect(at, isNotNull, reason: 'the cross appears while a cell is hovered');
    c.anchorPreview.value = null;
    await tester.pump();
    expect((overlay().painter as dynamic).anchorPreview, isNull);
    await tester.pumpWidget(const SizedBox());
  });

  /// 別タブ 2 枚同時: 見えたタブは自分の view の texture を取り、Stage タブはタブの寸法と
  /// zoom/pan の関心域を窓として native へ置き、隠れると窓を引く。Camera は窓を置かない。
  testWidgets(
    'each shown tab takes its own texture; the Stage places its window',
    (tester) async {
      final c = ObserverSession();
      c.document.value = {
        'width': 400,
        'height': 400,
        'capabilities': ['stageView', 'stageWindow'],
        'layers': [],
        'selectedIds': [],
      };
      var index = 1;
      late StateSetter show;
      await tester.pumpWidget(
        MaterialApp(
          theme: EditorTheme.data,
          home: Scaffold(
            body: SizedBox(
              width: 464,
              height: 480,
              child: StatefulBuilder(
                builder: (context, setState) {
                  show = setState;
                  return IndexedStack(
                    index: index,
                    children: [
                      StagePanel(controller: c, view: 'User'),
                      StagePanel(controller: c, view: 'Camera'),
                    ],
                  );
                },
              ),
            ),
          ),
        ),
      );
      await tester.pump();
      await tester.pump();
      expect(c.attached, ['Camera']);
      expect(c.commands.where((c) => c.$1 == 'stageWindow'), isEmpty);
      show(() => index = 0);
      await tester.pump();
      await tester.pump();
      expect(c.attached, ['Camera', 'User']);
      final placed = c.commands.lastWhere((c) => c.$1 == 'stageWindow').$2;
      // 464×(480−bars) の窓、関心域は fit した comp を中央に含む comp px の矩形。
      expect(placed['width'], greaterThan(0));
      expect(placed['height'], greaterThan(0));
      final roi = (placed['roi'] as List).cast<double>();
      expect(roi[0], lessThan(0));
      expect(roi[2], greaterThan(400));
      show(() => index = 1);
      await tester.pump();
      expect(c.commands.last.$1, 'stageWindow');
      expect(c.commands.last.$2, {'width': 0, 'height': 0});
      await tester.pumpWidget(const SizedBox());
    },
  );
}
