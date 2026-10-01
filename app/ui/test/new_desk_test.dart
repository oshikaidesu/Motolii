// The Desk's panels drawn by the finished instruments over the session (promoted one at a time).
import 'support/ease_intervals.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'dart:convert';

import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';

import '../lib/live_hf/adapters/blend.dart';
import '../lib/live_hf/adapters/depth.dart';
import '../lib/hf/desk/depth.dart';
import '../lib/foundation/leaves.dart' show EditorTextField;
import '../lib/hf/desk/ease_skin.dart';
import '../lib/hf/desk/notes_skin.dart';
import '../lib/panels/ease_desk.dart';
import '../lib/panels/notes_desk.dart';
import '../lib/session/editor_session.dart';
import '../lib/live_hf/adapters/history.dart';
import 'support/editor_test_theme.dart';
import 'support/new_inspector_host.dart' show Recording;

Future<Recording> mountHistory(WidgetTester tester, Map<String, dynamic> history) async {
  tester.view.physicalSize = const Size(400, 700);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final c = Recording();
  c.document.value = {'layers': const [], 'selectedIds': const <int>[], 'capabilities': ['historyGoto'], 'history': history};
  await tester.pumpWidget(MaterialApp(theme: editorTestTheme, home: Scaffold(body: NewHistory(controller: c))));
  await tester.pumpAndSettle();
  return c;
}

void main() {
  final entries = [
    {'label': 'Open Project', 'kind': 'open', 'head': 0, 'at': 1000},
    {'label': 'Add Layer', 'kind': 'edit', 'head': 1, 'at': 1010},
    {'label': 'Move Layer', 'kind': 'edit', 'head': 2, 'at': 1020},
    {'label': 'Save', 'kind': 'save', 'head': 2, 'at': 1030},
  ];

  testWidgets('History: the document\'s entries, a click goes to that head, Undo and Redo step from the current one', (tester) async {
    final c = await mountHistory(tester, {'entries': entries, 'head': 1});
    expect(find.text('Add Layer'), findsOneWidget);
    expect(find.text('Save'), findsOneWidget);
    await tester.tap(find.text('Move Layer'));
    await tester.pumpAndSettle();
    expect(c.commands.last.$2['head'], 2);
    await tester.tap(find.byKey(const ValueKey('history-undo')));
    await tester.pumpAndSettle();
    expect(c.commands.last.$2['head'], 0, reason: 'one step back from the current point');
    await tester.tap(find.byKey(const ValueKey('history-redo')));
    await tester.pumpAndSettle();
    expect(c.commands.last.$2['head'], 2);
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('History: no entries says so, and the current row is not a jump', (tester) async {
    final c = await mountHistory(tester, {'entries': const [], 'head': 0});
    expect(find.text('No history'), findsOneWidget);
    expect(c.commands, isEmpty);
    await tester.pumpWidget(const SizedBox());
  });

  Future<Recording> mountBlend(WidgetTester tester, List<Map<String, dynamic>> layers, List<int> selected) async {
    tester.view.physicalSize = const Size(400, 900);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final c = Recording();
    c.document.value = {'layers': layers, 'selectedIds': selected, 'selectedId': selected.isEmpty ? null : selected.last, 'blendTargets': selected, 'capabilities': ['previewBlend', 'applyBlend'], 'path': 'p'};
    await tester.pumpWidget(MaterialApp(theme: editorTestTheme, home: Scaffold(body: NewBlend(controller: c))));
    await tester.pumpAndSettle();
    return c;
  }

  Map<String, dynamic> layer(int id, String mode, {String kind = 'Shape'}) => {'id': id, 'name': 'Layer $id', 'kind': kind, 'locked': false, 'blendMode': mode, 'blendPreviews': {'Multiply': [[0.1, 0.2, 0.3], [0.4, 0.5, 0.6]]}};

  testWidgets('Blend: the selected layers are the targets, a click applies one mode as one step', (tester) async {
    final c = await mountBlend(tester, [layer(1, 'Normal'), layer(2, 'Screen')], [1, 2]);
    expect(find.text('Layer 1'), findsOneWidget);
    expect(find.text('Targets differ'), findsOneWidget, reason: 'two layers wear two modes');
    await tester.tap(find.byKey(const ValueKey('blend-mark-2')));
    await tester.pumpAndSettle();
    expect(c.commands.where((e) => e.$1 == 'applyBlend').single.$2, {'mode': 'Multiply'}, reason: 'the host chooses the layers');
    expect(c.ops, isNot(contains('cancelPreview')), reason: 'the port folds the preview away, the desk sends no cancel of its own');
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('Blend: pointing at a mode previews it on the last target, leaving cancels, the specimen shows', (tester) async {
    final c = await mountBlend(tester, [layer(1, 'Normal')], [1]);
    final mouse = await tester.createGesture(kind: PointerDeviceKind.mouse);
    await mouse.addPointer(location: Offset.zero);
    addTearDown(mouse.removePointer);
    await mouse.moveTo(tester.getCenter(find.byKey(const ValueKey('blend-mark-2'))));
    await tester.pump(const Duration(milliseconds: 200));
    final preview = c.commands.where((e) => e.$1 == 'previewBlend').last.$2;
    expect(preview, {'mode': 'Multiply'});
    expect(find.byKey(const ValueKey('blend-beds')), findsOneWidget, reason: 'the runtime\'s own specimen of that mode');
    await mouse.moveTo(const Offset(390, 890));
    await tester.pump(const Duration(milliseconds: 200));
    expect(c.ops.last, 'cancelPreview');
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('Blend: no selection dims the desk and a click sends nothing', (tester) async {
    final c = await mountBlend(tester, [layer(1, 'Normal')], []);
    await tester.tap(find.byKey(const ValueKey('blend-mark-2')));
    await tester.pumpAndSettle();
    expect(c.ops, isNot(contains('applyBlend')));
    expect(find.text('No layer'), findsOneWidget);
    await tester.pumpWidget(const SizedBox());
  });

  Future<Recording> mountDepth(WidgetTester tester, {bool locked = false}) async {
    tester.view.physicalSize = const Size(400, 900);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final c = Recording();
    c.document.value = {
      'layers': [layer(1, 'Normal'), layer(9, 'Normal', kind: 'Camera')],
      'selectedIds': [1],
      'selectedId': 1,
      'capabilities': ['previewProperties', 'commitPreview', 'select'],
      'depthLayout': {
        'halfFov': .5,
        'items': [
          {'id': 1, 'name': 'Layer 1', 'point': [100.0, 50.0], 'locked': locked, 'inverseX': [1.0, 0.0, 0.0], 'inverseZ': [0.0, 0.0, 1.0], 'local': [100.0, 0.0, 50.0]},
        ],
        'camera': {'point': [0.0, -600.0], 'layer': 9, 'target': 1, 'orbit': [10.0, 0.0], 'baseDistance': 600.0},
      },
    };
    await tester.pumpWidget(MaterialApp(theme: editorTestTheme, home: Scaffold(body: NewDepth(controller: c))));
    await tester.pumpAndSettle();
    return c;
  }

  Offset planPoint(WidgetTester tester, {required bool camera}) {
    final box = find.byKey(const ValueKey('depth-diagram'));
    final g = DepthGeom(tester.getSize(box), DView.top, DCam(0, 0, -600), [DLayer(100, 0, 50, 0, 0)], pad: 26);
    return tester.getTopLeft(box) + (camera ? g.camPx() : g.layerPx(0));
  }

  testWidgets('Depth: the floor plan comes from the document, a drag on a layer previews its position and a release commits', (tester) async {
    final c = await mountDepth(tester);
    expect(find.text('Looking at Layer 1. Drag layers or the camera to move them. Camera settings live in Inspector.'), findsOneWidget);
    final from = planPoint(tester, camera: false);
    final gesture = await tester.startGesture(from);
    await tester.pump(const Duration(milliseconds: 150));
    expect(c.commands.first.$1, 'select');
    expect(c.commands.first.$2['ids'], [1]);
    await gesture.moveBy(const Offset(30, -20));
    await tester.pump();
    await tester.pump();
    final edit = c.commands.where((e) => e.$1 == 'previewProperties').last.$2['edits'] as List;
    final position = edit.firstWhere((e) => e['property'] == 'position')['value'] as List;
    expect(position[0], greaterThan(100), reason: 'dragged to the right on the plan');
    final z = edit.firstWhere((e) => e['property'] == 'position.z')['value'] as double;
    expect(z, greaterThan(50), reason: 'dragged up the plan is further away');
    await gesture.up();
    await tester.pumpAndSettle();
    expect(c.ops.last, 'commitPreview');
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('Depth: the camera drags round its target as an orbit and a distance, a locked layer only selects', (tester) async {
    final c = await mountDepth(tester);
    final gesture = await tester.startGesture(planPoint(tester, camera: true));
    await tester.pump(const Duration(milliseconds: 150));
    expect(c.commands.first.$2['ids'], [9], reason: 'the camera\'s own layer');
    await gesture.moveBy(const Offset(20, 0));
    await tester.pump();
    await tester.pump();
    final edit = c.commands.where((e) => e.$1 == 'previewProperties').last.$2['edits'] as List;
    expect(edit.map((e) => e['property']), ['camera.orbit', 'camera.distance']);
    await gesture.up();
    await tester.pumpAndSettle();
    expect(c.ops.last, 'commitPreview');
    await tester.pumpWidget(const SizedBox());

    final locked = await mountDepth(tester, locked: true);
    final g2 = await tester.startGesture(planPoint(tester, camera: false));
    await tester.pump(const Duration(milliseconds: 150));
    await g2.moveBy(const Offset(30, 0));
    await g2.up();
    await tester.pumpAndSettle();
    expect(locked.ops, isNot(contains('previewProperties')));
    await tester.pumpWidget(const SizedBox());
  });

  Map<String, dynamic> bounce(double dip) => {
        'kind': 'Bounce',
        'dip': dip,
        'samples': [[0.0, 0.0], [.27, dip], [1.0, 1.0]],
        'handles': [[.27, dip]],
        'overshoots': false,
      };

  Future<(EditorSession, List<Map<String, dynamic>>)> mountEase(WidgetTester tester, Map<String, dynamic> Function() snapshot) async {
    final c = EditorSession();
    final commands = <Map<String, dynamic>>[];
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(EditorSession.channel, (call) async {
      final args = Map<String, dynamic>.from(call.arguments as Map? ?? {});
      if (call.method == 'easeModel') {
        final source = Map<String, dynamic>.from(args['shape']);
        return bounce(args['point'] == null ? (source['dip'] as num).toDouble() : (args['point'][1] as num).toDouble());
      }
      if (call.method == 'request') {
        commands.add(Map<String, dynamic>.from(jsonDecode(args['command'])));
        return snapshot();
      }
      if (call.method == 'render') return snapshot();
      if (call.method == 'readSettings') return {};
      if (call.method == 'writeSettings') return true;
      return {};
    });
    c.document.value = snapshot();
    tester.view.physicalSize = const Size(340, 900);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    await tester.pumpWidget(MaterialApp(
      theme: editorTestTheme,
      home: Scaffold(body: EaseDesk(controller: c, skin: (context, view) => EaseSkin(view))),
    ));
    await tester.pumpAndSettle();
    return (c, commands);
  }

  Map<String, dynamic> keyedSnapshot(Map<String, dynamic> current) => withEaseIntervals({
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
              {'id': 'opacity', 'keys': [{'frame': 0, 'interp': current}, {'frame': 30, 'interp': {'kind': 'Linear'}}]},
            ],
          },
        ],
        'easeKinds': [bounce(.2), bounce(.4)],
      });

  testWidgets('Ease: the curve, its intervals and the shelf come from the desk; a handle drag is one commit, Esc puts it back', (tester) async {
    var current = bounce(.2);
    final (c, commands) = await mountEase(tester, () => keyedSnapshot(current));
    expect(find.text('Rectangle · opacity  0–30 f'), findsOneWidget);
    expect(find.byKey(const ValueKey('ease-seg-0')), findsOneWidget);
    expect(find.byKey(const ValueKey('ease-preset-0')), findsOneWidget);
    expect(find.byKey(const ValueKey('ease-param-dip')), findsOneWidget, reason: 'the runtime\'s own parameter of this curve');

    final plot = find.byKey(const ValueKey('ease-plot'));
    Offset handle() {
      final r = (Offset.zero & tester.getSize(plot)).deflate(18);
      return tester.getTopLeft(plot) + Offset(r.left + .27 * r.width, r.top + (1 - (current['dip'] as num).toDouble()) * r.height);
    }

    final drag = await tester.startGesture(handle(), pointer: 1);
    await drag.moveBy(const Offset(0, -30));
    await tester.pumpAndSettle();
    expect(commands, isEmpty, reason: 'a drag only previews');
    await drag.up();
    await tester.pumpAndSettle();
    expect(commands.single['op'], 'ease');
    expect(commands.single['dip'], greaterThan(.2));

    final cancel = await tester.startGesture(handle(), pointer: 2);
    await cancel.moveBy(const Offset(0, -20));
    await tester.pumpAndSettle();
    await tester.sendKeyEvent(LogicalKeyboardKey.escape);
    await cancel.up();
    await tester.pumpAndSettle();
    expect(commands.length, 1, reason: 'Esc lets go without committing');

    await tester.tap(find.byKey(const ValueKey('ease-preset-1')));
    await tester.pumpAndSettle();
    expect(commands.length, 2);
    expect(commands.last['dip'], .4);
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });

  testWidgets('Ease: keeping a curve and the new-key curve write the same desk rows Classic writes', (tester) async {
    final (c, _) = await mountEase(tester, () => keyedSnapshot(bounce(.2)));
    await tester.tap(find.byKey(const ValueKey('ease-copy')));
    await tester.pumpAndSettle();
    expect((c.deskWork.value['curveClip'] as Map)['kind'], 'Bounce');
    await tester.tap(find.byKey(const ValueKey('ease-save')));
    await tester.pumpAndSettle();
    expect((c.deskWork.value['easePresets'] as List).length, 1);
    expect(find.byKey(const ValueKey('ease-clear')), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('ease-clear')));
    await tester.pumpAndSettle();
    expect(c.deskWork.value['easePresets'], isEmpty);
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });

  testWidgets('Ease: two layers and no keys is a Sequence: ghost marks on the curve, one sequence command on Apply', (tester) async {
    Map<String, dynamic> snap() => {
          'selectedIds': [1, 2],
          'selectedKeys': [],
          'capabilities': ['ease', 'sequence', 'previewSequence'],
          'layers': [
            {'id': 1, 'name': 'A', 'ghostable': true, 'properties': []},
            {'id': 2, 'name': 'B', 'ghostable': true, 'properties': []},
          ],
          'easeKinds': [bounce(.2)],
        };
    final (c, commands) = await mountEase(tester, snap);
    expect(find.text('Sequence · 2 layers'), findsOneWidget);
    expect(find.text('ghosts'), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('ease-apply')));
    await tester.pumpAndSettle();
    final sequence = commands.where((e) => e['op'] == 'sequence').single;
    expect(sequence['layers'], [1, 2]);
    expect(sequence['shape']['kind'], 'Linear', reason: 'the desk sends the curve; the host decides the delays');
    expect(sequence.containsKey('ghosts'), isFalse);
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });

  testWidgets('Notes: the pages, cards and actions are the desk\'s own; a card is written, moved, resized and removed by the same notes commands', (tester) async {
    final c = EditorSession();
    final pages = <Map<String, dynamic>>[];
    final calls = <Map<String, dynamic>>[];
    Map<String, dynamic> snapshot() => {
          'notebook': {'pages': jsonDecode(jsonEncode(pages))},
          'selectedIds': [1],
          'selectedKeys': [
            {'layer': 1, 'property': 'scale', 'frame': 12},
            {'layer': 1, 'property': 'scale', 'frame': 30},
          ],
          'layers': [{'id': 1, 'name': 'Cube', 'kind': 'Mesh'}],
          'capabilities': ['notes'],
        };
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(EditorSession.channel, (call) async {
      if (call.method == 'request') {
        final command = Map<String, dynamic>.from(jsonDecode((call.arguments as Map)['command']));
        calls.add(command);
        if (command['op'] == 'notes') {
          if (command['action'] == 'addPage') {
            pages.add({'id': command['page'], 'title': command['title'], 'blocks': <Map<String, dynamic>>[]});
          } else {
            final page = pages.firstWhere((p) => p['id'] == command['page']);
            final blocks = page['blocks'] as List;
            switch (command['action']) {
              case 'putBlock':
                blocks.add(Map<String, dynamic>.from(command['block']));
              case 'patchBlock':
                (blocks.firstWhere((b) => b['id'] == command['id']) as Map).addAll(command['patch']);
              case 'renamePage':
                page['title'] = command['title'];
              case 'deleteBlock':
                blocks.removeWhere((b) => b['id'] == command['id']);
              case 'deletePage':
                pages.remove(page);
            }
          }
        }
      }
      return snapshot();
    });
    c.document.value = snapshot();
    tester.view.physicalSize = const Size(700, 900);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    await tester.pumpWidget(MaterialApp(
      theme: editorTestTheme,
      home: Scaffold(body: NotesPanel(controller: c, look: hfNoteLook, skin: (context, view) => NotesSkin(view))),
    ));
    await tester.pumpAndSettle();
    expect(find.text('Click anywhere to write'), findsOneWidget);
    await tester.tapAt(tester.getTopLeft(find.byType(InteractiveViewer)) + const Offset(40, 40));
    await tester.pumpAndSettle();
    expect(pages.length, 1);
    final input = find.byWidgetPredicate((w) => w is EditorTextField && w.hint == 'Write a note');
    await tester.enterText(input, 'Lighting and texture');
    await tester.tap(find.byKey(const ValueKey('notes-new-page')));
    await tester.pumpAndSettle();
    expect(pages.length, 2);
    expect(pages.first['blocks'][0]['text'], 'Lighting and texture');
    await tester.tap(find.byKey(const ValueKey('notes-page-0')));
    await tester.pumpAndSettle();
    expect(find.text('Lighting and texture'), findsOneWidget);

    final x0 = pages.first['blocks'][0]['x'] as num;
    await tester.drag(find.byKey(ValueKey('note-text:${pages.first['blocks'][0]['id']}')), const Offset(70, 90), warnIfMissed: false);
    await tester.pumpAndSettle();
    expect(pages.first['blocks'][0]['x'], closeTo(x0 + 70, 1), reason: 'any part of the card moves it');
    await tester.drag(find.byKey(const ValueKey('note-resize')), const Offset(40, 30));
    await tester.pumpAndSettle();
    expect(pages.first['blocks'][0]['width'], closeTo(260, 1));
    await tester.tap(find.byKey(const ValueKey('notes-link')));
    await tester.pumpAndSettle();
    final ref = pages.first['blocks'][1] as Map;
    expect(ref['kind'], 'reference');
    expect((ref['start'], ref['end']), (12, 30));
    expect(find.text('Cube · 12–30'), findsOneWidget);

    expect(find.byKey(const ValueKey('zoom-label')), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('zoom-in')));
    await tester.pumpAndSettle();
    expect(tester.widget<Text>(find.byKey(const ValueKey('zoom-label'))).data, '125%');
    await tester.tap(find.byKey(const ValueKey('zoom-fit')));
    await tester.pumpAndSettle();
    expect(tester.widget<Text>(find.byKey(const ValueKey('zoom-label'))).data, '100%');

    await tester.tap(find.byKey(const ValueKey('note-delete')));
    await tester.pumpAndSettle();
    expect((pages.first['blocks'] as List).length, 1, reason: 'the chosen card is removed');
    expect(calls.every((call) => call['op'] == 'notes'), isTrue);
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });
}
