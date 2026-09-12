import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/timeline.dart';

class RecordingController extends EditorSession {
  final commands = <Map<String, dynamic>>[];

  /// A reply the test releases by hand, to look at the frames in between.
  Completer<void>? hold;
  @override
  Future<void> command(
    String op, [
    Map<String, dynamic> args = const {},
  ]) async {
    commands.add({'op': op, ...args});
    if (op == 'select')
      document.value = {
        ...state,
        'selectedIds': args['ids'],
        'selectedKeys': args['keys'] ?? [],
      };
    if (op == 'moveKeys' && hold != null) await hold!.future;
  }
}

Map<String, dynamic> _layer(int id, int start) => {
  'id': id,
  'name': 'clip $id',
  'kind': 'Shape',
  'start': start,
  'duration': 40,
  'sourceIn': 0,
  'properties': const [],
  'effects': const [],
};

/// Two selected bars, one grabbed and dragged: every preview and the release
/// carry both layers by the same delta from where they started.
void main() {
  testWidgets('a multi-layer drag moves every selected bar by one delta', (
    tester,
  ) async {
    final c = RecordingController();
    c.document.value = {
      'fps': 30,
      'durationFrames': 300,
      'capabilities': [
        'select',
        'setTimings',
        'previewTimings',
        'commitPreview',
      ],
      'selectedIds': [1, 2],
      'selectedKeys': [],
      'layers': [_layer(1, 10), _layer(2, 30)],
    };
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(body: TimelinePanel(controller: c)),
      ),
    );
    // labelWidth 220, 4 px per frame, first row centred at y 64.
    double x(int frame) => 220 + frame * 4.0;
    final pointer = await tester.startGesture(Offset(x(20), 64));
    await tester.pump();
    for (final dx in [8.0, 12.0, 20.0]) {
      await pointer.moveTo(Offset(x(20) + dx, 64));
      await tester.pump();
    }
    final previews = c.commands.where((m) => m['op'] == 'previewTimings');
    expect(previews, isNotEmpty);
    for (final (i, preview) in previews.indexed) {
      final changes = List<Map<String, dynamic>>.from(preview['changes']);
      final starts = {for (final ch in changes) ch['layer']: ch['start']};
      expect(
        starts[2]! - starts[1]!,
        20,
        reason: 'preview $i keeps the bars 20 frames apart',
      );
    }
    await pointer.up();
    await tester.pump();
    final last = List<Map<String, dynamic>>.from(previews.last['changes']);
    expect({for (final ch in last) ch['layer']: ch['start']}, {1: 15, 2: 35});
    expect(c.commands.last['op'], 'commitPreview');
    expect(c.commands.where((m) => m['op'] == 'setTimings'), isEmpty);
  });

  testWidgets(
    'gripped keys stop at frame 0 together and stay drawn until the reply',
    (tester) async {
      final c = RecordingController();
      c.document.value = {
        'fps': 30,
        'durationFrames': 300,
        'capabilities': ['select', 'moveKeys'],
        'selectedIds': [],
        'selectedKeys': [],
        'layers': [
          {
            ..._layer(1, 0),
            'duration': 300,
            'properties': [
              {
                'id': 'position',
                'keys': [
                  {'frame': 5},
                  {'frame': 20},
                ],
              },
            ],
          },
        ],
      };
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: TimelinePanel(controller: c)),
        ),
      );
      double x(int frame) => 220 + frame * 4.0;
      await tester.tapAt(Offset(x(5), 64));
      await tester.pump();
      await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
      await tester.tapAt(Offset(x(20), 64));
      await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
      await tester.pump();
      expect(
        (c.commands.lastWhere((m) => m['op'] == 'select')['keys'] as List)
            .length,
        2,
      );
      c.hold = Completer<void>();
      final pointer = await tester.startGesture(Offset(x(20), 64));
      await tester.pump();
      // 15 frames left would put the first key at -10: the grip stops at -5.
      await pointer.moveTo(Offset(x(20) - 60, 64));
      await tester.pump();
      dynamic lanes() => tester
          .widgetList<CustomPaint>(find.byType(CustomPaint))
          .map((w) => w.painter)
          .firstWhere(
            (p) =>
                p.runtimeType.toString() == '_TimelinePainter' &&
                (p as dynamic).ruler == false,
          );
      expect(lanes().delta, -5);
      await pointer.up();
      await tester.pump();
      expect(c.commands.last, {'op': 'moveKeys', 'deltaFrames': -5});
      // The reply has not landed: the keys are still drawn at their new place.
      expect(lanes().delta, -5);
      expect((lanes().dragKeys as List).length, 2);
      c.hold!.complete();
      await tester.pump();
      expect(lanes().delta, 0);
      expect(lanes().dragKeys, isEmpty);
    },
  );
}
