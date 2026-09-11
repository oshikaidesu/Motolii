import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/foundation/panel_controls.dart';
import '../lib/foundation/theme.dart';
import '../lib/panels/inspector.dart';
import '../lib/session/editor_session.dart';

Map<String, dynamic> _number(
  String id,
  String label,
  Object value, {
  num? min,
  num? max,
  Object? choices,
  String? kind,
}) => {
  'id': id,
  'label': label,
  if (kind != null) 'kind': kind,
  'value': value,
  'keys': const [],
  'keyedNow': false,
  'min': min,
  'max': max,
  if (choices != null) 'choices': choices,
};

Map<String, dynamic> _document(Map<String, double> moved) {
  double at(String id, double rest) => moved[id] ?? rest;
  return {
    'layers': [
      {
        'id': 1,
        'name': 'Cube',
        'kind': 'Shape',
        'locked': false,
        'colors': const [],
        'blendMode': 'Normal',
        'projection': '2.5D',
        'properties': [
          _number('position', 'Position', [at('position.0', 10), 20.0],
              kind: 'vec2'),
          _number('position.z', 'Position Z', at('position.z', 0)),
          _number('scale', 'Scale', [at('scale.0', 1), at('scale.1', 1)],
              kind: 'vec2'),
          _number('scale.z', 'Scale Z', at('scale.z', 1)),
          _number('rotation', 'Rotation', at('rotation', 0)),
          _number('rotation.x', 'Rotation X', at('rotation.x', 0)),
          _number('rotation.y', 'Rotation Y', at('rotation.y', 0)),
          _number('opacity', 'Opacity', at('opacity', 1)),
          _number('depth', 'Depth', at('depth', 0), min: 0, max: 100000),
          _number('roughness', 'Roughness', at('roughness', 0.5),
              min: 0, max: 1),
        ],
        'effects': [
          {
            'id': 'motolii.warp',
            'name': 'Warp',
            'enabled': true,
            'params': [
              _number('warp.param.amount', 'Amount', at('warp.param.amount', 2),
                  min: 0, max: 10),
              _number('warp.param.angle', 'Angle', at('warp.param.angle', 0)),
              _number('warp.param.seed', 'Seed', at('warp.param.seed', 1),
                  min: 0, max: 9999),
              _number(
                'warp.param.center_x',
                'Center X',
                at('warp.param.center_x', 0),
              ),
              _number(
                'warp.param.center_y',
                'Center Y',
                at('warp.param.center_y', 0),
              ),
              _number('warp.param.mode', 'Mode', at('warp.param.mode', 0),
                  choices: const ['Push', 'Pull']),
              {
                'id': 'warp.param.tint',
                'label': 'Tint',
                'kind': 'color',
                'value': [at('warp.param.tint', 0), 0.0, 0.0, 1.0],
                'keys': const [],
              },
            ],
          },
        ],
      },
    ],
    'selectedId': 1,
    'selectedIds': const [1],
    'selectedKeys': const [],
    'capabilities': const [
      'previewProperties',
      'commitPreview',
      'cancelPreview',
      'toggleKey',
    ],
    'durationFrames': 60,
    'fps': 30.0,
    'contentRevision': moved.toString(),
  };
}

void main() {
  testWidgets('repro: trackpad pan on an inspector well', (tester) async {
    tester.view.physicalSize = const Size(420, 1600);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final ops = <String>[];
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
      ops.add('${call.method} ${call.arguments}');
      return <String, dynamic>{};
    });
    final c = EditorSession();
    addTearDown(c.dispose);
    c.document.value = _document(const {});
    await tester.pumpWidget(MaterialApp(
      theme: EditorTheme.data,
      home: Scaffold(body: InspectorPanel(controller: c)),
    ));
    await tester.pumpAndSettle();
    ops.clear();
    final well = find.byKey(const ValueKey('inspector:position:0'));
    final at = tester.getCenter(well);
    final fingers = await tester.createGesture(kind: PointerDeviceKind.trackpad);
    await fingers.panZoomStart(at);
    for (var n = 1; n <= 4; n++) {
      await fingers.panZoomUpdate(at, pan: Offset(-10.0 * n, 0));
      await tester.pump(const Duration(milliseconds: 16));
    }
    await fingers.panZoomEnd();
    await tester.pumpAndSettle();
    for (final o in ops) debugPrint('OP $o');
    debugPrint('shown ${tester.widget<EditorNumericField>(well).value}');
  });
}
