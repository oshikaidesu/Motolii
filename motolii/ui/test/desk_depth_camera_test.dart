import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_stage5/panels/depth_desk.dart';
import 'package:motolii_stage5/session/editor_session.dart';

/// Depth: 注視点が原点、カメラは eye に居て、掴んで回すと Orbit の yaw と Distance が届く。
void main() {
  testWidgets('dragging the camera around the target authors orbit and distance', (
    tester,
  ) async {
    final c = EditorSession();
    final edits = <Map<String, dynamic>>[];
    final calls = <String>[];
    Map<String, dynamic> state() => {
      'selectedIds': [2],
      'selectedKeys': [],
      'layers': [
        {'id': 1, 'kind': 'Null', 'name': 'Aim'},
        {'id': 2, 'kind': 'Camera', 'name': 'Camera'},
      ],
      'depthLayout': {
        'items': [
          {
            'id': 1,
            'name': 'Aim',
            'point': [0.0, 0.0],
            'local': [0.0, 0.0, 0.0],
            'inverseX': [1.0, 0.0, 0.0],
            'inverseZ': [0.0, 0.0, 1.0],
          },
        ],
        'camera': {
          'point': [0.0, -1000.0],
          'layer': 2,
          'target': 1,
          'orbit': [0.0, 0.0],
          'distance': 1.0,
          'baseDistance': 1000.0,
        },
        'halfFov': 0.9,
      },
    };
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          if (call.method == 'request') {
            final cmd = jsonDecode((call.arguments as Map)['command']);
            calls.add(cmd['op']);
            if (cmd['op'] == 'previewProperties') {
              edits
                ..clear()
                ..addAll((cmd['edits'] as List).cast<Map<String, dynamic>>());
            }
          }
          return state();
        });
    c.document.value = state();
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: SizedBox(
            width: 400,
            height: 400,
            child: DepthDesk(controller: c),
          ),
        ),
      ),
    );
    expect(find.text('Aim'), findsOneWidget, reason: 'the target is named');
    final box = tester.getRect(
      find.byWidgetPredicate(
        (w) => w is CustomPaint && '${w.painter.runtimeType}' == '_DepthGrid',
      ),
    );
    final origin = box.center;
    // range = 1000 * 1.25、scale = (min(w,h) - 36) / (2 * range)
    final scale = (box.shortestSide - 36) / 2500;
    final eye = origin + Offset(0, 1000 * scale);
    final drag = await tester.startGesture(eye);
    await tester.pumpAndSettle();
    expect(calls, contains('select'));
    // 右へ 90° 回して距離を 2 倍に: eye が (x=+2000, z=0) → yaw = atan2(-x, -z) = -90°
    await drag.moveTo(origin + Offset(2000 * scale, 0));
    await tester.pumpAndSettle();
    await drag.up();
    await tester.pumpAndSettle();
    expect(calls.last, 'commitPreview');
    final orbit = edits.firstWhere((e) => e['property'] == 'camera.orbit');
    final distance = edits.firstWhere(
      (e) => e['property'] == 'camera.distance',
    );
    expect(orbit['layer'], 2);
    expect((orbit['value'] as List)[0], 0.0);
    expect((orbit['value'] as List)[1], closeTo(-90, 0.5));
    expect(distance['value'], closeTo(2.0, 0.02));
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });
}
