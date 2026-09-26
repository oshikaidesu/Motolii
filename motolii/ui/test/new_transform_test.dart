// The Transform Instrument on the real session, inside the production Inspector's Transform card.
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/app/new/inspector/new_transform.dart';
import '../lib/app/new/inspector/session_transform.dart';
import '../lib/hf/insp/transform.dart';
import '../lib/panels/inspector.dart';
import '../lib/session/editor_session.dart';
import 'support/editor_test_theme.dart';
import 'support/window_fixture.dart';

class Recording extends EditorSession {
  final commands = <(String, Map<String, dynamic>)>[];
  List<String> get ops => [for (final c in commands) c.$1];
  @override
  Future<void> refreshPreview() async {}
  @override
  Future<void> command(String op, [Map<String, dynamic> args = const {}]) async {
    commands.add((op, args));
  }
}

const capabilities = ['previewProperties', 'commitPreview', 'toggleKey', 'anchor', 'setAttrs', 'animate'];

Map<String, dynamic> layerRow(int id, String name, {List<double>? position, bool locked = false, String projection = '2D', int? parent, List<Map<String, dynamic>>? extra, bool keyed = false}) {
  final l = windowLayer(id, 'Shape', 0);
  return {
    ...l,
    'name': name,
    'locked': locked,
    'projection': projection,
    'parent': parent,
    'anchorFraction': [.5, .5],
    'effects': [],
    'properties': [
      for (final p in l['properties'] as List)
        if ((p as Map)['id'] == 'position')
          <String, dynamic>{...p, 'value': position ?? [320.0, 180.0], if (keyed) 'keys': [{'frame': 0}, {'frame': 30}], if (keyed) 'keyedNow': true}
        else
          p,
      ...?extra,
    ],
  };
}

Map<String, dynamic> state(List<Map<String, dynamic>> layers, List<int> selected) => {
  ...windowStatus(1, 0),
  'layers': layers,
  'selectedId': selected.first,
  'selectedIds': selected,
  'capabilities': capabilities,
  'animate': false,
};

Future<Recording> mount(WidgetTester tester, Map<String, dynamic> doc) async {
  tester.view.physicalSize = const Size(700, 1000);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final c = Recording()..document.value = doc;
  await tester.pumpWidget(MaterialApp(
    theme: editorTestTheme,
    home: Align(
      alignment: Alignment.topLeft,
      child: SizedBox(
        width: 340,
        height: 960,
        child: InspectorPanel(
          controller: c,
          instruments: InspectorInstruments(transform: (context, controller) => NewTransform(controller: controller)),
        ),
      ),
    ),
  ));
  await tester.pump();
  return c;
}

SessionTransformStore storeOf(WidgetTester tester) => tester.widget<TransformInstrument>(find.byType(TransformInstrument)).store as SessionTransformStore;
Finder key(String k) => find.byKey(ValueKey(k));

void main() {
  setUpAll(() async {
    final inter = FontLoader('Inter')
      ..addFont(rootBundle.load('assets/fonts/Inter-Regular.ttf'))
      ..addFont(rootBundle.load('assets/fonts/Inter-Medium.ttf'))
      ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.ttf'));
    await inter.load();
  });

  testWidgets('the Transform card is the Instrument, read from the layer rows, with one name on the panel', (tester) async {
    await mount(tester, state([layerRow(1, 'Logo', position: [320, 180])], [1]));
    expect(find.byType(TransformInstrument), findsOneWidget);
    expect(key('tf-title'), findsNothing, reason: 'the Inspector header already names the layer');
    final s = storeOf(tester);
    expect(s.active.name, 'Logo');
    expect(s.active.position.take(2), [320, 180]);
    expect(tester.takeException(), isNull);
  });

  testWidgets('a typed position is absolute and reaches the document as one preview and one commit', (tester) async {
    final c = await mount(tester, state([layerRow(1, 'Logo')], [1]));
    await tester.tap(key('toy-position-0'));
    await tester.pump();
    await tester.enterText(find.descendant(of: key('toy-position-0'), matching: find.byType(EditableText)), '250');
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pump(const Duration(seconds: 1));
    final preview = c.commands.where((v) => v.$1 == 'previewProperties').toList();
    expect(preview, isNotEmpty);
    expect((preview.last.$2['edits'] as List).single, {'layer': 1, 'property': 'position', 'value': [250.0, 180.0]});
    expect(c.ops.last, 'commitPreview');
    expect(c.ops.where((o) => o == 'commitPreview').length, 1);
  });

  testWidgets('a drag is relative across the selection, a locked layer is left alone, one commit ends it', (tester) async {
    final c = await mount(tester, state([
      layerRow(1, 'A', position: [100, 100]),
      layerRow(2, 'B', position: [300, 50]),
      layerRow(3, 'Locked', position: [0, 0], locked: true),
    ], [1, 3, 2])); // the shown layer is the last selected: B
    final s = storeOf(tester);
    expect(s.activeId, 2);
    s.preview('position', [330.0, 50.0]); // B moves +30 in x
    final edits = (c.commands.last.$2['edits'] as List).cast<Map>();
    expect(edits.map((e) => e['layer']), [1, 2], reason: 'the locked layer is not a target');
    expect(edits.firstWhere((e) => e['layer'] == 1)['value'], [130.0, 100.0], reason: 'A keeps its own offset');
    s.commit('position');
    expect(c.ops, ['previewProperties', 'commitPreview']);
  });

  testWidgets('a locked layer refuses every edit', (tester) async {
    final c = await mount(tester, state([layerRow(1, 'Locked', locked: true)], [1]));
    final s = storeOf(tester);
    expect(s.frozen, isTrue);
    s.set('position', [9.0, 9.0]);
    s.toggleKey('position');
    s.setAnchor(0, 0);
    s.setProjection('3D');
    s.setParent(2);
    expect(c.commands, isEmpty);
  });

  testWidgets('what the document says later (undo, another edit, a new selection) is what the Instrument shows', (tester) async {
    final c = await mount(tester, state([layerRow(1, 'A', position: [10, 20]), layerRow(2, 'B', position: [70, 80])], [1]));
    final s = storeOf(tester);
    expect(s.active.position.take(2), [10, 20]);
    c.document.value = state([layerRow(1, 'A', position: [15, 25]), layerRow(2, 'B', position: [70, 80])], [1]);
    await tester.pump();
    expect(s.active.position.take(2), [15, 25], reason: 'undo or another edit');
    c.document.value = state([layerRow(1, 'A', position: [15, 25]), layerRow(2, 'B', position: [70, 80])], [2]);
    await tester.pump();
    expect(s.activeId, 2, reason: 'selection followed');
    expect(s.active.position.take(2), [70, 80]);
    // No selection at all: the card is empty, not broken.
    c.document.value = {...state([layerRow(1, 'A')], [1]), 'selectedId': null, 'selectedIds': <int>[]};
    await tester.pump();
    expect(tester.takeException(), isNull);
  });

  testWidgets('while a gesture runs the Instrument keeps its own values; the commit reads the document again', (tester) async {
    final c = await mount(tester, state([layerRow(1, 'A', position: [10, 20])], [1]));
    final s = storeOf(tester);
    s.preview('position', [50.0, 60.0]);
    // The runtime answers with the live values while the drag is still going.
    c.document.value = state([layerRow(1, 'A', position: [49, 59])], [1]);
    await tester.pump();
    expect(s.active.position.take(2), [50, 60], reason: 'the pointer is in charge until it lets go');
    s.commit('position');
    c.document.value = state([layerRow(1, 'A', position: [50, 60])], [1]);
    await tester.pump();
    expect(s.active.position.take(2), [50, 60]);
  });

  testWidgets('keys, anchor, space and parent go through the operations Classic uses', (tester) async {
    final c = await mount(tester, state([
      layerRow(1, 'A', extra: [
        {...windowNumber('depth', 'Depth', 0.0)},
      ]),
      layerRow(2, 'B'),
    ], [1]));
    final s = storeOf(tester);
    s.toggleKey('rotation');
    s.setAnchor(1, 0);
    s.setProjection('3D');
    s.setParent(2);
    expect(c.ops, ['toggleKey', 'anchor', 'setAttrs', 'setAttrs']);
    expect(c.commands[0].$2, {'layer': 1, 'property': 'rotation'});
    expect(c.commands[1].$2, {'layer': 1, 'xFraction': 1.0, 'yFraction': 0.0});
    expect(c.commands[2].$2, {'layers': [1], 'patch': {'projection': '3D'}});
    expect(c.commands[3].$2, {'layers': [1], 'patch': {'parent': 2}});
    s.setAnimate(true);
  });

  testWidgets('the Stage hears where the pivot would land while the pointer is over an Anchor cell', (tester) async {
    final c = await mount(tester, state([layerRow(1, 'A')], [1]));
    final s = storeOf(tester);
    expect(c.anchorPreview.value, isNull);
    s.anchorPreview.value = [0.0, 1.0];
    expect(c.anchorPreview.value, [0.0, 1.0]);
    s.anchorPreview.value = null;
    expect(c.anchorPreview.value, isNull);
  });

  testWidgets('a key on the row shows as keyed; the Instrument reads it from the same fields Classic reads', (tester) async {
    await mount(tester, state([layerRow(1, 'A', keyed: true)], [1]));
    final a = storeOf(tester).active;
    expect(a.animated, contains('position'));
    expect(a.keyedNow, contains('position'));
    expect(a.animated, isNot(contains('rotation')));
  });
}
