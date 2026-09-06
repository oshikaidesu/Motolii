import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/timeline.dart';

class RecordingController extends EditorSession {
  final moves = <Map<String, dynamic>>[];
  @override
  Future<void> command(
    String op, [
    Map<String, dynamic> args = const {},
  ]) async {
    if (op == 'select')
      document.value = {
        ...state,
        'selectedIds': args['ids'],
        'selectedKeys': args['keys'] ?? [],
      };
    if (op == 'moveLayers') moves.add(Map.of(args));
  }
}

void main() {
  testWidgets(
    'row move previews then commits inside, before, root-end; cycle rejected',
    (tester) async {
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
        'capabilities': ['moveLayers'],
        'selectedIds': [],
        'selectedKeys': [],
        'layers': [
          layer(1, 'Shape', null),
          layer(3, 'Group', null),
          layer(4, 'Shape', 3),
        ],
      };
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: TimelinePanel(controller: c)),
        ),
      );
      Future<void> drag(Offset from, Offset to) async {
        final pointer = await tester.startGesture(from);
        await pointer.moveTo(to);
        await tester.pump();
        final count = c.moves.length;
        await pointer.up();
        await tester.pump();
        expect(c.moves.length, count + 1);
      }

      await drag(const Offset(40, 64), const Offset(40, 84));
      expect(c.moves.last, {
        'layers': [1],
        'target': 3,
        'placement': 'inside',
      });
      await drag(const Offset(40, 64), const Offset(40, 76));
      expect(c.moves.last['placement'], 'before');
      await drag(const Offset(40, 64), const Offset(40, 160));
      expect(c.moves.last, {
        'layers': [1],
        'target': null,
        'placement': 'rootEnd',
      });
      final count = c.moves.length;
      final cycle = await tester.startGesture(const Offset(40, 84));
      await cycle.moveTo(const Offset(40, 104));
      await tester.pump();
      await cycle.up();
      await tester.pump();
      expect(c.moves.length, count);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    },
  );
}
