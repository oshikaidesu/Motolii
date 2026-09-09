import 'dart:convert';
import 'dart:ui' show PointerDeviceKind;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/panels/ease_desk.dart';
import '../lib/session/editor_session.dart';

void main() {
  test('sampled motion preserves overshoot and the Hold discontinuity', () {
    expect(
      easeValueAt({
        'kind': 'Hold',
        'samples': [
          [0, 0],
          [1, 1],
        ],
      }, .999),
      0,
    );
    expect(easeValueAt({'kind': 'Hold'}, 1), 1);
    expect(
      easeValueAt({
        'samples': [
          [0, 0],
          [.5, 1.5],
          [1, 1],
        ],
      }, .5),
      1.5,
    );
    expect(
      easeValueAt({
        'samples': [
          [0, 0],
          [.5, 1.5],
          [1, 1],
        ],
      }, .75),
      1.25,
    );
  });

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
      await tester.binding.setSurfaceSize(const Size(320, 600));
      addTearDown(() => tester.binding.setSurfaceSize(null));
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Center(
              child: SizedBox(
                width: double.infinity,
                child: EaseDesk(controller: c),
              ),
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

      final beforeHover = jsonEncode(c.deskWork.value);
      final mouse = await tester.createGesture(kind: PointerDeviceKind.mouse);
      await mouse.addPointer(location: Offset.zero);
      await mouse.moveTo(tester.getCenter(find.byTooltip('Bounce').last));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 600));
      final preview =
          tester
                  .widget<CustomPaint>(
                    find.byKey(const ValueKey('ease-motion')),
                  )
                  .painter!
              as EaseMotionPainter;
      expect(preview.shape['dip'], .4);
      final editing =
          tester
                  .widget<CustomPaint>(
                    find.descendant(
                      of: plot,
                      matching: find.byType(CustomPaint),
                    ),
                  )
                  .painter!
              as EaseCurvePainter;
      expect(editing.shape['dip'], .2);

      expect(preview.time, closeTo(.5, .02));
      expect(commands, isEmpty);
      expect(jsonEncode(c.deskWork.value), beforeHover);
      await mouse.moveTo(const Offset(1, 1));
      await tester.pumpAndSettle();
      final restored =
          tester
                  .widget<CustomPaint>(
                    find.byKey(const ValueKey('ease-motion')),
                  )
                  .painter!
              as EaseMotionPainter;
      expect(restored.shape['dip'], .2);
      expect(restored.time, 0);
      await mouse.removePointer();
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
      for (final size in [
        const Size(240, 240),
        const Size(320, 280),
        const Size(640, 320),
      ]) {
        await tester.binding.setSurfaceSize(size);
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
        expect(tester.getSize(plot).height, lessThanOrEqualTo(220));
        expect(tester.getSize(plot).width, tester.getSize(plot).height);
        await tester.ensureVisible(plot);
        await tester.pumpAndSettle();
        await tester.scrollUntilVisible(
          find.byKey(const ValueKey('ease-preset:0')),
          60,
          scrollable: find.byType(Scrollable).first,
        );
        await tester.pumpAndSettle();
        await tester.tap(find.byTooltip('Bounce').first);
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
        expect(commands.length, 2);
      }
      await tester.ensureVisible(find.byTooltip('Overshoot'));
      await tester.tap(find.byTooltip('Overshoot'));
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
      expect(commands.length, 2);
      await tester.pumpWidget(const SizedBox());
      const kinds = [
        'Hold',
        'Linear',
        'Bezier',
        'Bounce',
        'Elastic',
        'Cyclic',
        'Random',
        'Steps',
        'ElasticSteps',
      ];
      final random = <String, dynamic>{
        'kind': 'Random',
        'seed': 500.0,
        'grain': .15,
        'center_u': .5,
        'center_v': .75,
        'bias': .5,
      };
      c.deskWork.value = {'ease': random};
      c.document.value = {
        'selectedIds': <int>[],
        'easeKinds': [
          for (final kind in kinds) {'kind': kind},
        ],
      };
      for (final size in [const Size(240, 240), const Size(320, 440)]) {
        await tester.binding.setSurfaceSize(size);
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(body: EaseDesk(controller: c)),
          ),
        );
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
        final bounds = tester.getRect(find.byType(EaseDesk));
        expect(tester.getSize(plot).width, tester.getSize(plot).height);
        for (final key in ['seed', 'grain', 'center_u', 'center_v', 'bias']) {
          final field = find.byKey(ValueKey('Random:$key'));
          if (size.height < 420) {
            await tester.scrollUntilVisible(
              field,
              80,
              scrollable: find
                  .descendant(
                    of: find.byKey(const ValueKey('ease-choices-scroll')),
                    matching: find.byType(Scrollable),
                  )
                  .first,
            );
          } else {
            await tester.ensureVisible(field);
          }
          await tester.pumpAndSettle();
          expect(field.hitTestable(), findsOneWidget);
        }
        final graphBounds = tester.getRect(plot);
        for (final kind in kinds) {
          final preset = find.byTooltip(kind);
          await tester.ensureVisible(preset);
          await tester.pumpAndSettle();
          expect(preset.hitTestable(), findsOneWidget);
          expect(tester.getRect(plot), graphBounds);
          expect(plot.hitTestable(), findsOneWidget);
          final thumbnail = find.descendant(
            of: preset,
            matching: find.byType(CustomPaint),
          );
          expect(
            tester.getSize(thumbnail).shortestSide,
            greaterThanOrEqualTo(44),
          );
          final label = find.descendant(
            of: preset,
            matching: find.byType(Text),
          );
          expect(
            tester.renderObject<RenderParagraph>(label).didExceedMaxLines,
            isFalse,
          );
          final rect = tester.getRect(preset);
          expect(bounds.contains(rect.center), isTrue);
        }
        for (final label in ['Copy curve', 'Save preset', 'Overshoot']) {
          await tester.ensureVisible(find.byTooltip(label));
          await tester.pumpAndSettle();
          expect(find.byTooltip(label).hitTestable(), findsOneWidget);
        }
        await tester.pumpWidget(const SizedBox());
      }
      await tester.binding.setSurfaceSize(const Size(320, 440));
      await tester.pumpWidget(
        MaterialApp(
          home: MediaQuery(
            data: const MediaQueryData(disableAnimations: true),
            child: Scaffold(body: EaseDesk(controller: c)),
          ),
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byTooltip('Preview motion'));
      await tester.pump();
      final reduced =
          tester
                  .widget<CustomPaint>(
                    find.byKey(const ValueKey('ease-motion')),
                  )
                  .painter!
              as EaseMotionPainter;
      expect(reduced.time, 1);
      expect(tester.hasRunningAnimations, isFalse);
      expect(commands.length, 2);
      await tester.binding.setSurfaceSize(null);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    },
  );
}
