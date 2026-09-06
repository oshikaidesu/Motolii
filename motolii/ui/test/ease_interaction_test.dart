import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/panels/ease_desk.dart';
import '../lib/session/editor_session.dart';

void main() {
  testWidgets(
    'legacy handles commit once, cancel on interruption, and presets edit intervals',
    (tester) async {
      Map<String, dynamic> shape(double dip) => {
        'kind': 'Bounce',
        'first_dip': .27,
        'dip': dip,
        'samples': [
          [0.0, 0.0],
          [.27, dip],
          [1.0, 1.0],
        ],
        'handles': [
          [.27, dip],
        ],
        'overshoots': false,
      };
      final c = EditorSession();
      final commands = <Map<String, dynamic>>[];
      var current = shape(.2);
      Map<String, dynamic> snapshot() => {
        'selectedIds': [1],
        'selectedKeys': [
          {'layer': 1, 'property': 'opacity', 'frame': 0},
          {'layer': 1, 'property': 'opacity', 'frame': 30},
        ],
        'capabilities': ['ease'],
        'layers': [
          {
            'id': 1,
            'name': 'Rectangle',
            'properties': [
              {
                'id': 'opacity',
                'keys': [
                  {'frame': 0, 'interp': current},
                  {
                    'frame': 30,
                    'interp': {'kind': 'Linear'},
                  },
                ],
              },
            ],
          },
        ],
        'easeKinds': [shape(.2), shape(.4)],
      };
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(EditorSession.channel, (call) async {
            final args = Map<String, dynamic>.from(
              call.arguments as Map? ?? {},
            );
            if (call.method == 'easeModel') {
              final source = Map<String, dynamic>.from(args['shape']);
              return shape(
                args['point'] == null
                    ? (source['dip'] as num).toDouble()
                    : (args['point'][1] as num).toDouble(),
              );
            }
            if (call.method == 'request') {
              final command = Map<String, dynamic>.from(
                jsonDecode(args['command']),
              );
              commands.add(command);
              current = shape((command['dip'] as num).toDouble());
              return snapshot();
            }
            if (call.method == 'render') return snapshot();
            if (call.method == 'readSettings') return {};
            if (call.method == 'writeSettings') return true;
            return {};
          });
      c.document.value = snapshot();
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Center(
              child: SizedBox(width: 320, child: EaseDesk(controller: c)),
            ),
          ),
        ),
      );
      final plot = find.byKey(const ValueKey('ease-plot'));
      Offset handle() {
        final size = tester.getSize(plot);
        final painter = EaseCurvePainter(shape: current, handles: true);
        return tester.getTopLeft(plot) +
            painter.toPixel(
              Offset(.27, (current['dip'] as num).toDouble()),
              size,
            );
      }

      final p = handle();
      final drag = await tester.startGesture(p, pointer: 1);
      await drag.moveBy(const Offset(0, -30));
      await tester.pumpAndSettle();
      expect(commands, isEmpty);
      await drag.up();
      await tester.pumpAndSettle();
      expect(commands.length, 1);
      expect(commands.single['op'], 'ease');
      expect(commands.single['dip'], greaterThan(.2));
      final cancel = await tester.startGesture(handle(), pointer: 2);
      await cancel.moveBy(const Offset(0, -20));
      await tester.pumpAndSettle();
      await tester.sendKeyEvent(LogicalKeyboardKey.escape);
      await cancel.up();
      await tester.pumpAndSettle();
      expect(commands.length, 1);
      final lost = await tester.startGesture(handle(), pointer: 3);
      await lost.moveBy(const Offset(0, -20));
      await tester.pumpAndSettle();
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.inactive);
      await lost.up();
      await tester.pumpAndSettle();
      expect(commands.length, 1);
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      await tester.tap(find.byTooltip('Bounce').last);
      await tester.pumpAndSettle();
      expect(commands.length, 2);
      expect(commands.last['dip'], .4);
      c.document.value = {...snapshot(), 'selectedKeys': [], 'selectedIds': []};
      await tester.pumpAndSettle();
      await tester.tap(find.byTooltip('Bounce').first);
      await tester.pumpAndSettle();
      expect(commands.length, 2);
      expect(c.deskWork.value['ease'], isNotNull);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    },
  );
}
