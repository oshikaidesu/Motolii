import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/browser.dart';
import '../lib/panels/timeline.dart';

class RecordingController extends EditorSession {
  final placed = <Map<String, dynamic>>[];
  @override
  Future<void> command(
    String op, [
    Map<String, dynamic> args = const {},
  ]) async {
    if (op == 'placeAsset') placed.add(Map.of(args));
  }
}

void main() {
  testWidgets('Dragging a Media card onto a Timeline row places it there', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(900, 900);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final c = RecordingController();
    Map<String, dynamic> layer(int id, String kind, int? parent) => {
      'id': id,
      'name': 'Layer $id',
      'kind': kind,
      'parent': parent,
      'start': 0,
      'duration': 300,
      'properties': [],
    };
    c.document.value = {
      'fps': 30,
      'durationFrames': 300,
      'capabilities': ['moveLayers', 'placeAsset'],
      'selectedIds': [],
      'selectedKeys': [],
      'layers': [layer(1, 'Shape', null), layer(3, 'Group', null)],
      'assets': [
        {
          'id': 'a1',
          'name': 'logo.png',
          'path': '/tmp/logo.png',
          'mime': 'image/png',
          'used': false,
          'missing': false,
          'role': 'material',
        },
      ],
    };
    // These read the grid; Media itself opens on pictures alone.
    c.deskWork.value = {'browserView': 0};
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Column(
            children: [
              SizedBox(height: 300, child: TimelinePanel(controller: c)),
              Expanded(
                child: BrowserPanel(
                  controller: c,
                  fixedTab: 'Media',
                  showTabs: false,
                ),
              ),
            ],
          ),
        ),
      ),
    );
    Future<void> drop(Offset to) async {
      final g = await tester.startGesture(tester.getCenter(find.text('logo')));
      await g.moveBy(const Offset(0, -20));
      await tester.pump();
      await g.moveTo(to);
      await tester.pump();
      await g.up();
      // 札の double-tap 判定の時計を流し切る(test 終了時の pending timer を残さない)
      await tester.pump(const Duration(milliseconds: 400));
    }

    // 行の名前欄へ: 並び順だけ決まり、開始は playhead のまま
    await drop(const Offset(40, 76));
    expect(c.placed.single, {'id': 'a1', 'target': 3, 'placement': 'before'});

    // 時間軸の上へ: 落とした x が開始コマになる
    await drop(const Offset(700, 64));
    expect(c.placed.length, 2);
    expect(c.placed.last['target'], 1);
    expect(c.placed.last['start'], isA<int>());
    expect(c.placed.last['start'], greaterThan(0));
    expect(tester.takeException(), isNull);
    await tester.pumpWidget(const SizedBox());
  });
}
