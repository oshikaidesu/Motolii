// The Layout Instrument on a real layer's layout rows, inside the production Inspector's Layout card.
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/app/new/inspector/session_layout.dart';
import '../lib/hf/insp/layout.dart';
import 'support/new_inspector_host.dart';
import 'support/window_fixture.dart';

Map<String, dynamic> row(String id, String label, Object v, {List<String>? choices}) => {
  ...windowNumber(id, label, v),
  if (choices != null) 'choices': choices,
};

Map<String, dynamic> groupLayer({int id = 1, bool locked = false, List<Map<String, dynamic>> extra = const []}) {
  final base = layerRow(id, 'Cards', locked: locked);
  return {
    ...base,
    'kind': 'Group',
    'properties': [
      ...(base['properties'] as List),
      row('layout.display', 'Grid', 2),
      row('layout.grid_columns', 'Columns', 3),
      row('layout.grid_rows', 'Rows', 0),
      row('layout.gap', 'Gap', 12.0),
      {...windowNumber('layout.padding', 'Padding', [16.0, 16.0]), 'kind': 'vec2'},
      row('layout.justify_content', 'Justify', 0),
      row('layout.align_items', 'Align', 1),
      row('layout.horizontal_sizing', 'Width', 2),
      row('layout.vertical_sizing', 'Height', 0),
      row('layout.width', 'Width', 360.0),
      row('layout.height', 'Height', 240.0),
      row('layout.transition_duration', 'Duration', .3),
      row('layout.transition_easing', 'Easing', 3),
      ...extra,
    ],
  };
}

Map<String, dynamic> childLayer() {
  final base = layerRow(2, 'Card 1');
  return {
    ...base,
    'parent': 1,
    'properties': [
      ...(base['properties'] as List),
      row('layout.position_type', 'Ignore layout', 0),
      row('layout.horizontal_sizing', 'Width', 1),
      row('layout.vertical_sizing', 'Height', 0),
      row('layout.width', 'Width', 120.0),
      row('layout.height', 'Height', 80.0),
      row('layout.column_start', 'Column start', 1),
      row('layout.row_start', 'Row start', 1),
      row('layout.column_span', 'Column span', 1),
      row('layout.row_span', 'Row span', 1),
    ],
  };
}

SessionLayoutStore storeOf(WidgetTester tester) => tester.widget<LayoutInstrument>(find.byType(LayoutInstrument)).store as SessionLayoutStore;

void main() {
  setUpAll(loadFonts);

  testWidgets('a Group shows the Instrument on its own layout rows, and a plain layer shows no Layout card', (tester) async {
    await mount(tester, state([groupLayer(), childLayer()], [1]), height: 1400);
    expect(find.byType(LayoutInstrument), findsOneWidget);
    final s = storeOf(tester);
    expect(s.child, isFalse);
    expect(s.gi('layout.grid_columns'), 3);
    expect(s.gd('layout.gap'), 12);
    expect(tester.takeException(), isNull);

    await mount(tester, state([layerRow(1, 'Plain')], [1]));
    expect(find.byType(LayoutInstrument), findsNothing);
  });

  testWidgets('a laid-out child shows its own lines, and the Ignore switch writes position_type', (tester) async {
    final c = await mount(tester, state([groupLayer(), childLayer()], [2]), height: 1400);
    final s = storeOf(tester);
    expect(s.child, isTrue);
    await tester.tap(key('ignore-switch'));
    await tester.pump();
    expect(c.ops, ['previewProperties', 'commitPreview']);
    expect((c.commands.first.$2['edits'] as List).single, {'layer': 2, 'property': 'layout.position_type', 'value': 1});
  });

  testWidgets('Grid on/off and a typed Gap are edits of the layer, each one preview and one commit', (tester) async {
    final c = await mount(tester, state([groupLayer()], [1]), height: 1400);
    await tester.tap(key('grid-switch'));
    await tester.pump();
    expect(c.ops, ['previewProperties', 'commitPreview']);
    expect((c.commands.first.$2['edits'] as List).single, {'layer': 1, 'property': 'layout.display', 'value': 0});
    c.commands.clear();
    final s = storeOf(tester);
    s.set('layout.gap', 24.0);
    expect((c.commands.first.$2['edits'] as List).single, {'layer': 1, 'property': 'layout.gap', 'value': 24.0});
    expect(c.commands.last.$1, 'commitPreview');
  });

  testWidgets('the alignment pad changes Justify and Align in one preview and one commit', (tester) async {
    final c = await mount(tester, state([groupLayer()], [1]), height: 1400);
    final s = storeOf(tester);
    s.applyMany({'layout.justify_content': 2, 'layout.align_items': 3});
    s.commit('layout.justify_content');
    expect(c.ops, ['previewProperties', 'commitPreview']);
    expect((c.commands.first.$2['edits'] as List).cast<Map>().map((e) => (e['property'], e['value'])), [('layout.justify_content', 2), ('layout.align_items', 3)]);
  });

  testWidgets('a locked Group refuses every edit', (tester) async {
    final c = await mount(tester, state([groupLayer(locked: true)], [1]), height: 1400);
    final s = storeOf(tester);
    expect(s.frozen, isTrue);
    s.set('layout.gap', 30.0);
    s.applyMany({'layout.justify_content': 2});
    s.setGrid(false);
    expect(c.commands, isEmpty);
  });

  testWidgets('every other layout row the document declares stays reachable behind Advanced', (tester) async {
    await mount(tester, state([groupLayer(extra: [row('layout.flex_direction', 'Direction', 1, choices: ['Row', 'Column'])])], [1]), height: 1400);
    final s = storeOf(tester);
    final extra = s.rows.firstWhere((r) => r['id'] == 'layout.flex_direction');
    expect(extra['advanced'], isTrue);
    expect(extra['value'], 1);
    expect(extra['choices'], ['Row', 'Column']);
  });

  testWidgets('undo or another edit shows in the Instrument; a running gesture keeps its own value', (tester) async {
    final c = await mount(tester, state([groupLayer()], [1]), height: 1400);
    final s = storeOf(tester);
    c.document.value = state([groupLayer()..['properties'] = [for (final p in groupLayer()['properties'] as List) (p as Map)['id'] == 'layout.gap' ? {...p, 'value': 40.0} : p]], [1]);
    await tester.pump();
    expect(s.gd('layout.gap'), 40);
    s.preview('layout.gap', 55.0);
    c.document.value = state([groupLayer()], [1]);
    await tester.pump();
    expect(s.gd('layout.gap'), 55, reason: 'the pointer is in charge until it lets go');
    s.commit('layout.gap');
    expect(tester.takeException(), isNull);
  });
}
