import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/inspector.dart';

void main() {
  testWidgets('A placement effect row offers Expand next to remove', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(900, 1000);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final commands = <String>[];
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          final args = call.arguments;
          if (args is Map && args['command'] is String) {
            commands.add(args['command'] as String);
          }
          return <String, dynamic>{};
        });
    final c = EditorSession();
    Map<String, dynamic> layer(String pluginId, bool placement) => {
      'id': 1,
      'name': 'paper',
      'kind': 'Shape',
      'properties': [],
      'effects': [
        {
          'id': 0,
          'pluginId': pluginId,
          'name': pluginId,
          'placement': placement,
          if (placement)
            'layout': {
              'columns': ['Each', 'Random'],
              'count': 'effect.0.param.count',
              'along': 'effect.0.param.mode',
              'pick': 'effect.0.param.pick',
              'subject': 'effect.0.param.subject',
              'seed': 'effect.0.param.seed',
              'materials': [
                {'id': 'effect.0.param.share.11', 'label': 'circle'},
                {'id': 'effect.0.param.share.12', 'label': 'square'},
              ],
              'shape': [
                {
                  'id': 'effect.0.param.radius',
                  'label': 'Radius',
                  'unit': 'px',
                },
              ],
              'rows': [
                {
                  'label': 'Rotation',
                  'unit': '°',
                  'each': 'effect.0.param.rotation_each',
                  'random': 'effect.0.param.rotation_random',
                },
                {
                  'label': 'Seed',
                  'unit': '',
                  'random': 'effect.0.param.seed',
                  'advanced': true,
                },
              ],
            },
          'params': placement
              ? [
                  {
                    'id': 'effect.0.param.count',
                    'label': 'Count',
                    'value': 12.0,
                  },
                  {
                    'id': 'effect.0.param.mode',
                    'label': 'Along',
                    'value': 1,
                    'choices': ['Line', 'Circle', 'Grid'],
                  },
                  {
                    'id': 'effect.0.param.radius',
                    'label': 'Radius',
                    'value': 200.0,
                  },
                  {
                    'id': 'effect.0.param.rotation_each',
                    'label': 'Rotation',
                    'value': 30.0,
                    'keys': [
                      {'frame': 0},
                    ],
                  },
                  {
                    'id': 'effect.0.param.rotation_random',
                    'label': 'Rotation',
                    'value': 0.0,
                  },
                  {'id': 'effect.0.param.seed', 'label': 'Seed', 'value': 7.0},
                  {
                    'id': 'effect.0.param.pick',
                    'label': 'Pick',
                    'value': 0,
                    'choices': ['Random', 'Iterate'],
                  },
                  {
                    'id': 'effect.0.param.subject',
                    'label': 'Copies',
                    'value': 0,
                    'choices': ['One child', 'Whole group'],
                  },
                  {
                    'id': 'effect.0.param.share.11',
                    'label': 'circle',
                    'value': 300.0,
                  },
                  {
                    'id': 'effect.0.param.share.12',
                    'label': 'square',
                    'value': 100.0,
                  },
                ]
              : [],
        },
      ],
    };
    void show(Map<String, dynamic> one) {
      c.document.value = {
        'layers': [one],
        'selectedIds': [1],
        'selectedKeys': [],
        'capabilities': [
          'expandEffect',
          'removeEffect',
          'animate',
          'setProperty',
        ],
        'easeKinds': [],
      };
    }

    show(layer('motolii.repeat', true));
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(body: InspectorPanel(controller: c)),
      ),
    );
    expect(find.text('Expand'), findsOneWidget);
    expect(find.text('×'), findsOneWidget);
    expect(find.text('Animate'), findsOneWidget);
    await tester.tap(
      find.descendant(
        of: find
            .ancestor(of: find.text('Animate'), matching: find.byType(Row))
            .first,
        matching: find.text('Off'),
      ),
    );
    await tester.pump();
    expect(
      find.text('Circle'),
      findsOneWidget,
      reason: 'the Along toggle shows the chosen shape',
    );
    expect(
      find.text('Radius'),
      findsOneWidget,
      reason: 'shape fields follow the toggle',
    );
    expect(find.text('Each'), findsOneWidget);
    expect(
      find.text('Random'),
      findsWidgets,
      reason: 'column header, and Pick for a group',
    );
    expect(
      find.text('One child'),
      findsOneWidget,
      reason: 'a group Repeater says whether it picks a child or copies the whole group',
    );
    expect(
      find.text('Rotation'),
      findsOneWidget,
      reason: 'one row holds both the Each and Random cells',
    );
    expect(
      find.text('◇'),
      findsOneWidget,
      reason: 'a keyed cell marks its row',
    );
    expect(
      find.text('Seed'),
      findsNothing,
      reason: 'advanced rows start folded',
    );
    expect(
      find.text('•'),
      findsOneWidget,
      reason: 'a non-default advanced value shows as a dot',
    );
    await tester.ensureVisible(find.text('▸'));
    await tester.tap(find.text('▸'));
    await tester.pump();
    expect(find.text('Seed'), findsOneWidget);
    await tester.ensureVisible(find.text('↻'));
    await tester.tap(find.text('↻'));
    await tester.pump();
    expect(
      commands
          .map(jsonDecode)
          .any(
            (m) =>
                m['op'] == 'setProperty' &&
                m['property'] == 'effect.0.param.seed' &&
                m['value'] == 8,
          ),
      isTrue,
      reason: 'the seed button asks for the next seed',
    );
    expect(find.text('Materials'), findsOneWidget);
    expect(find.text('circle'), findsOneWidget);
    expect(
      find.text('75%'),
      findsOneWidget,
      reason: 'shares are shown as a normalized percent',
    );
    expect(find.text('25%'), findsOneWidget);
    expect(find.text('Random'), findsWidgets, reason: 'Pick shows for a group');

    await tester.tap(find.text('Expand'));
    await tester.pump();
    final sent = commands.map(jsonDecode).whereType<Map>().toList();
    expect(
      sent.any((m) => m['op'] == 'animate' && m['enabled'] == true),
      isTrue,
      reason: 'the Animate switch asks native to turn keying on: $commands',
    );
    expect(
      sent.any(
        (m) => m['op'] == 'expandEffect' && m['layer'] == 1 && m['id'] == 0,
      ),
      isTrue,
      reason:
          'Expand dispatches expandEffect for that layer and effect: $commands',
    );

    show(layer('motolii.blur', false));
    await tester.pump();
    expect(find.text('Expand'), findsNothing);
    expect(find.text('×'), findsOneWidget);
  });
}
