import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/stage.dart';
import 'support/editor_test_theme.dart';
import 'support/painters.dart';

/// 3D レイヤーの 3 軸ギズモと、2D の平面ケージは別物。
/// 掴む所は、native が当たり判定に使うのと同じ三角形。
class GestureSession extends EditorSession {
  final commands = <(String, Map<String, dynamic>)>[];
  @override
  Future<void> refreshPreview() async {}
  @override
  Future<void> command(
    String op, [
    Map<String, dynamic> args = const {},
  ]) async {
    commands.add((op, args));
  }
}

Map<String, dynamic> _layer(
  int id,
  String projection, {
  String kind = 'Shape',
  int? parent,
  List<List<int>>? corners,
}) => {
  'id': id,
  'kind': kind,
  'parent': parent,
  'projection': projection,
  'stageBounds': {
    'corners':
        corners ??
        [
          [100, 100],
          [300, 100],
          [300, 300],
          [100, 300],
        ],
  },
};

/// comp 座標 (150,150)–(250,250) を覆う四角の網。
const _mesh = {
  'vertices': [
    [150.0, 150.0],
    [250.0, 150.0],
    [250.0, 250.0],
    [150.0, 250.0],
  ],
  'colors': [
    [1.0, 0.0, 0.5, 1.0],
    [1.0, 0.0, 0.5, 1.0],
    [1.0, 0.0, 0.5, 1.0],
    [1.0, 0.0, 0.5, 1.0],
  ],
  'indices': [0, 1, 2, 0, 2, 3],
};

Map<String, dynamic> _doc({required String projection, bool mesh = true}) => {
  'width': 400,
  'height': 400,
  'frame': 0,
  'documentRevision': 'r1',
  'stageView': 'User',
  'capabilities': ['stageGesture', 'select'],
  'observer': {'front': true},
  'layers': [_layer(7, projection)],
  'selectedId': 7,
  'selectedIds': [7],
  if (mesh) 'stageSpatialGizmo': _mesh,
};

/// comp 座標 → 画面。Stage はタブいっぱいに描くので、合成の矩形は overlay が持つ枠から取る。
Offset Function(double, double) _screen(WidgetTester tester) {
  final overlay = painterCarrying(
    (p) => p.viewport is Rect && p.dimOutside is bool,
  );
  final frame =
      (tester.widget<CustomPaint>(overlay).painter as dynamic).viewport as Rect;
  final rect = frame.shift(tester.getTopLeft(overlay));
  final scale = rect.width / 400;
  return (double x, double y) => rect.topLeft + Offset(x, y) * scale;
}

Future<Rect> _mount(WidgetTester tester, GestureSession c) async {
  await tester.pumpWidget(
    MaterialApp(
      theme: editorTestTheme,
      home: Scaffold(
        body: SizedBox(
          width: 464,
          height: 480,
          // A fresh key per session, so no test inherits another's State.
          child: StagePanel(key: ValueKey(identityHashCode(c)), controller: c),
        ),
      ),
    ),
  );
  await tester.pump();
  return tester.getRect(find.byType(StagePanel));
}

void main() {
  testWidgets('a press on the 3D gizmo starts the spatial gesture', (
    tester,
  ) async {
    final c = GestureSession()..document.value = _doc(projection: '3D');
    await _mount(tester, c);
    final centre = _screen(tester)(200, 200);
    final gesture = await tester.startGesture(centre);
    await tester.pump();
    final begin = c.commands.firstWhere((v) => v.$1 == 'stageGesture');
    expect(begin.$2['mode'], 'spatial');
    expect(begin.$2['phase'], 'begin');
    await gesture.up();
    await tester.pump();
  });

  testWidgets('a press away from the mesh selects instead of grabbing', (
    tester,
  ) async {
    final c = GestureSession()..document.value = _doc(projection: '3D');
    await _mount(tester, c);
    final corner = _screen(tester)(120, 120);
    final gesture = await tester.startGesture(corner);
    await tester.pump();
    expect(c.commands.any((v) => v.$1 == 'stageGesture'), isFalse);
    await gesture.up();
    await tester.pump();
  });

  testWidgets('2D layers keep the planar cage corner handles', (tester) async {
    for (final projection in ['2D', '2.5D']) {
      final c = GestureSession()..document.value = _doc(projection: projection);
      await _mount(tester, c);
      // The cage's north-west corner handle, at comp (100,100).
      final handle = _screen(tester)(100, 100);
      final gesture = await tester.startGesture(handle);
      await tester.pump();
      final begin = c.commands.firstWhere((v) => v.$1 == 'stageGesture');
      expect(begin.$2['mode'], 'scale', reason: projection);
      expect(begin.$2['handle'], 'nw', reason: projection);
      await gesture.up();
      await tester.pumpAndSettle();
    }
  });

  testWidgets('a 2D layer still moves as a plane, mesh or no mesh', (
    tester,
  ) async {
    final c = GestureSession()..document.value = _doc(projection: '2D');
    await _mount(tester, c);
    final centre = _screen(tester)(200, 200);
    final drag = await tester.startGesture(centre);
    await tester.pump();
    await drag.moveBy(const Offset(40, 0));
    await tester.pump();
    final begin = c.commands.firstWhere((v) => v.$1 == 'stageGesture');
    expect(begin.$2['mode'], 'move');
    expect(c.commands.every((v) => v.$2['mode'] != 'spatial'), isTrue);
    await drag.up();
    await tester.pumpAndSettle();
  });

  testWidgets('a group is never what the Stage touches: its child is', (
    tester,
  ) async {
    // The group's box covers the whole comp; its child sits inside.
    final c = GestureSession()
      ..document.value = {
        ..._doc(projection: '2D', mesh: false),
        'layers': [
          _layer(
            1,
            '2D',
            kind: 'Group',
            corners: [
              [0, 0],
              [400, 0],
              [400, 400],
              [0, 400],
            ],
          ),
          _layer(2, '2D', parent: 1),
        ],
        'selectedId': 1,
        'selectedIds': [1],
      };
    await _mount(tester, c);
    final overlay = painterCarrying(
      (p) => p.viewport is Rect && p.dimOutside is bool,
    );
    dynamic painter() => tester.widget<CustomPaint>(overlay).painter;
    // Hovering the child under the selected group: no cage on the group.
    final gesture = await tester.createGesture(kind: PointerDeviceKind.mouse);
    await gesture.addPointer(location: _screen(tester)(200, 200));
    addTearDown(gesture.removePointer);
    await tester.pump();
    await gesture.moveTo(_screen(tester)(200, 200));
    await tester.pump();
    expect(painter().outlines, isEmpty);
    expect(painter().handles, isEmpty);
    // A press there takes the child, not the group.
    await gesture.down(_screen(tester)(200, 200));
    await tester.pump();
    final select = c.commands.lastWhere((v) => v.$1 == 'select');
    expect(select.$2['ids'], [2]);
    await gesture.up();
    await tester.pump();
    // Where only the group lies, nothing is taken: a marquee starts.
    await gesture.down(_screen(tester)(20, 20));
    await tester.pump();
    expect(c.commands.where((v) => v.$1 == 'select').length, 1);
    await gesture.up();
    await tester.pump();
  });
}
