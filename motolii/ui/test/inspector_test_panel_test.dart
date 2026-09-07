import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/foundation/panel_controls.dart';
import '../lib/foundation/theme.dart';
import '../lib/panels/inspector_test.dart';
import '../lib/session/editor_session.dart';

/// The Test tab builds every control from the declaration: each kind the
/// snapshot can send gets its part, and a Vism's params need no hand layout.
void main() {
  testWidgets('every declared kind becomes a control, effects included', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(900, 1400);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final commands = <Map<String, dynamic>>[];
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          final args = call.arguments;
          if (args is Map && args['command'] is String) {
            commands.add(Map<String, dynamic>.from(args));
          }
          return <String, dynamic>{};
        });
    final c = EditorSession();
    Map<String, dynamic> row(
      String id,
      String label,
      dynamic value, {
      String kind = 'number',
      num? min,
      num? max,
      List<String>? choices,
      List<Map<String, dynamic>> keys = const [],
      bool keyedNow = false,
    }) => {
      'id': id,
      'label': label,
      'kind': kind,
      'value': value,
      'min': min,
      'max': max,
      if (choices != null) 'choices': choices,
      'keys': keys,
      'keyedNow': keyedNow,
    };
    c.document.value = {
      'layers': [
        {
          'id': 1,
          'name': 'paper',
          'kind': 'Image',
          'projection': '2.5D',
          'anchorFraction': [.5, .5],
          'ghostable': true,
          'properties': [
            row('position', 'Position', [10.0, 20.0], kind: 'vec2'),
            row('position.z', 'Z', 0.0),
            row('scale', 'Scale', [1.0, 1.0], kind: 'vec2'),
            row(
              'rotation',
              'Rotation',
              30.0,
              keys: [
                {'frame': 0},
              ],
              keyedNow: true,
            ),
            row('rotation.x', 'Tilt X', 0.0),
            row('rotation.y', 'Tilt Y', 0.0),
            row('opacity', 'Opacity', .5),
          ],
          'effects': [
            {
              'id': 0,
              'pluginId': 'motolii.new_vism',
              'name': 'New Vism',
              'params': [
                row('effect.0.param.mix', 'Mix', .3, min: 0, max: 1),
                row('effect.0.param.radius', 'Radius', 12.0),
                row('effect.0.param.angle', 'rotation_of', 45.0),
                row(
                  'effect.0.param.mode',
                  'Mode',
                  1,
                  choices: ['Line', 'Circle', 'Grid'],
                ),
                row('effect.0.param.offset', 'Offset', [
                  1.0,
                  2.0,
                ], kind: 'vec2'),
              ],
            },
          ],
        },
      ],
      'selectedIds': [1],
      'selectedKeys': [],
      'capabilities': [
        'previewProperties',
        'commitPreview',
        'cancelPreview',
        'setProperty',
        'setAttrs',
        'anchor',
        'ghost',
        'clip',
        'animate',
        'removeEffect',
      ],
      'easeKinds': [],
    };
    await tester.pumpWidget(
      MaterialApp(
        theme: EditorTheme.data,
        home: Scaffold(body: InspectorTestPanel(controller: c)),
      ),
    );
    // Transform: position x/y/z, scale (locked: one well), rotation + tilts,
    // opacity. Effect: mix, radius, angle, offset x/y.
    expect(find.byType(EditorNumericField), findsNWidgets(13));
    expect(find.byType(EditorDial), findsOneWidget);
    expect(find.byType(EditorAnchorGrid), findsOneWidget);
    expect(find.byType(EditorChoice<dynamic>), findsNWidgets(2));
    // Scale link, Environment, Ghost, Clip, Animate.
    expect(find.byType(EditorSwitch), findsNWidgets(5));
    // The rotation well carries the key lamp; opacity does not.
    final lamps = tester
        .widgetList<EditorLamp>(find.byType(EditorLamp))
        .map((l) => l.state)
        .toList();
    expect(lamps.where((s) => s == KeyLamp.now).length, 1);
    expect(find.text('New Vism'.toUpperCase()), findsOneWidget);
    expect(find.text('50 %'), findsNothing, reason: 'unit is its own rider');
    expect(find.text('50'), findsOneWidget, reason: 'opacity shown in percent');
    expect(find.text('%'), findsWidgets);

    // Picking an anchor cell goes through the layer's own anchor route.
    await tester.tap(find.byType(EditorAnchorGrid));
    await tester.pumpAndSettle();
    expect(
      commands.any((m) => '${m['command']}'.contains('"op":"anchor"')),
      isTrue,
      reason: 'error: ${c.error.value}; sent: $commands',
    );
  });
}
