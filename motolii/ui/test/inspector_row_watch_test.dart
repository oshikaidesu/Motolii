import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/foundation/panel_controls.dart';
import '../lib/foundation/theme.dart';
import '../lib/panels/inspector.dart';
import '../lib/session/editor_session.dart';

/// Every row of the Inspector watches its own value now, so the danger is the
/// opposite of a rebuild storm: a row that quietly stops following the
/// document. One status per row, moving that row alone, and the control that
/// shows it must say the new number.
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

double _wellValue(WidgetTester tester, String id, int axis) => tester
    .widget<EditorNumericField>(
      find.byKey(ValueKey('inspector:$id:$axis')),
    )
    .value;

void main() {
  testWidgets('every row follows its own value', (tester) async {
    tester.view.physicalSize = const Size(420, 1600);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(
          EditorSession.channel,
          (call) async => <String, dynamic>{},
        );
    final c = EditorSession();
    addTearDown(c.dispose);
    c.document.value = _document(const {});
    await tester.pumpWidget(
      MaterialApp(
        theme: EditorTheme.data,
        home: Scaffold(body: InspectorPanel(controller: c)),
      ),
    );
    await tester.pumpAndSettle();

    // The wells, including the ones a percent or an axis rewrites.
    const wells = <String, (String, int, double, double)>{
      'position X': ('position.0', 0, 33.0, 33.0),
      'position Z': ('position.z', 0, 7.0, 7.0),
      'scale X': ('scale.0', 0, 2.0, 200.0),
      'scale Z': ('scale.z', 0, 3.0, 300.0),
      'rotation': ('rotation', 0, 45.0, 45.0),
      'tilt X': ('rotation.x', 0, 12.0, 12.0),
      'tilt Y': ('rotation.y', 0, 34.0, 34.0),
      'opacity': ('opacity', 0, 0.5, 50.0),
      'a plain property': ('roughness', 0, 0.25, 0.25),
      'an effect amount': ('warp.param.amount', 0, 6.0, 6.0),
      'an effect angle': ('warp.param.angle', 0, 90.0, 90.0),
      'an effect seed': ('warp.param.seed', 0, 42.0, 42.0),
      'half of a point': ('warp.param.center_x', 0, 5.0, 5.0),
    };
    for (final entry in wells.entries) {
      final (id, axis, sent, shown) = entry.value;
      c.document.value = _document({id: sent});
      await tester.pump();
      expect(
        _wellValue(tester, id.endsWith('.0') ? id.split('.0').first : id, axis),
        shown,
        reason: '${entry.key} stopped following the document',
      );
    }

    // The scale line's second well appears when the axes stop agreeing.
    c.document.value = _document(const {'scale.1': 4.0});
    await tester.pump();
    expect(_wellValue(tester, 'scale', 1), 400.0);

    // The controls that are not wells.
    c.document.value = _document(const {'rotation': 60.0});
    await tester.pump();
    expect(
      tester
          .widgetList<EditorDial>(find.byType(EditorDial))
          .map((d) => d.degrees),
      contains(60.0),
    );
    c.document.value = _document(const {'warp.param.center_y': 9.0});
    await tester.pump();
    expect(
      tester.widgetList<EditorPad>(find.byType(EditorPad)).map((p) => p.y),
      contains(9.0),
    );
    c.document.value = _document(const {'warp.param.mode': 1.0});
    await tester.pump();
    expect(
      tester
          .widgetList<EditorChoice<dynamic>>(find.byType(EditorChoice<dynamic>))
          .map((e) => e.value),
      contains(1),
    );
    c.document.value = _document(const {'warp.param.tint': 1.0});
    await tester.pump();
    expect(
      tester
          .widgetList<EditorDraftField>(find.byType(EditorDraftField))
          .map((f) => f.value),
      contains('#ff0000ff'),
    );

    // A key on one row lights that row's lamp without a status shaped
    // differently: the row watches the whole of its declaration.
    final keyed = _document(const {});
    ((((keyed['layers'] as List)[0] as Map)['properties'] as List)[7]
        as Map)['keys'] = const [0];
    c.document.value = keyed;
    await tester.pump();
    expect(
      tester.widgetList<EditorLamp>(find.byType(EditorLamp)).map((l) => l.state),
      contains(KeyLamp.keyed),
    );
    await tester.pumpWidget(const SizedBox());
  });
}
