import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/foundation/panel_controls.dart';
import '../lib/panels/inspector.dart';
import '../lib/session/editor_session.dart';
import 'support/editor_test_theme.dart';

/// The Layout card is six lines for a Group and two for a laid-out child,
/// on the same rows and the same ops as before; every other layer has no
/// Layout card at all. The rows mirror layout.rs (GROUP_ROWS / ITEM_ROWS /
/// SPACE_ROWS) as native hands them over for each Display.
Map<String, dynamic> _row(
  String id,
  String label,
  Object value, {
  num? min,
  num? max,
  List<String>? choices,
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

const _sizing = ['Hug', 'Fill', 'Fixed'];

List<Map<String, dynamic>> _spaceRows() => [
  _row('layout.margin', 'Margin', 0.0, min: 0, max: 100000),
  _row('layout.hardness', 'Hardness', 0.5, min: 0, max: 1),
  _row('layout.heaviness', 'Heaviness', 1.0, min: 0, max: 100),
  _row('layout.flex_shrink', 'Flex Shrink', 1.0, min: 0, max: 1000),
  _row(
    'layout.transition_duration',
    'Transition Duration',
    0.0,
    min: 0,
    max: 60,
  ),
  _row(
    'layout.transition_easing',
    'Transition Easing',
    0,
    choices: ['Ease', 'Linear', 'Ease In', 'Ease Out', 'Ease In Out'],
  ),
  _row('layout.transition_delay', 'Transition Delay', 0.0, min: 0, max: 60),
  _row('layout.loop_duration', 'Loop Duration', 0.0, min: 0, max: 3600),
];

List<Map<String, dynamic>> _groupRows(int display, {int direction = 0}) => [
  _row('layout.display', 'Display', display, choices: ['None', 'Flex', 'Grid']),
  if (display == 1) ...[
    _row(
      'layout.flex_direction',
      'Flex Direction',
      direction,
      choices: ['Row', 'Column', 'Row Reverse', 'Column Reverse', 'Depth'],
    ),
    _row(
      'layout.flex_wrap',
      'Flex Wrap',
      0,
      choices: ['No Wrap', 'Wrap', 'Wrap Reverse'],
    ),
    _row(
      'layout.justify_content',
      'Justify Content',
      0,
      choices: [
        'Start',
        'End',
        'Center',
        'Space Between',
        'Space Around',
        'Space Evenly',
      ],
    ),
    _row(
      'layout.align_items',
      'Align Items',
      0,
      choices: ['Stretch', 'Start', 'End', 'Center'],
    ),
  ],
  if (display != 0) ...[
    _row(
      'layout.depth_alignment',
      'Depth Alignment',
      0,
      choices: ['Back', 'Center', 'Front'],
    ),
    if (display == 2) ...[
      _row('layout.grid_columns', 'Grid Columns', 3.0, min: 1, max: 64),
      _row('layout.grid_rows', 'Grid Rows', 0.0, min: 0, max: 64),
    ],
    _row('layout.gap', 'Gap', 12.0, min: 0, max: 100000),
    _row('layout.padding', 'Padding', [16.0, 16.0], kind: 'vec2'),
    _row('layout.horizontal_sizing', 'Horizontal Sizing', 0, choices: _sizing),
    _row('layout.vertical_sizing', 'Vertical Sizing', 2, choices: _sizing),
    _row('layout.width', 'Width', 400.0, min: 0, max: 100000),
    _row('layout.height', 'Height', 300.0, min: 0, max: 100000),
    _row('layout.border_radius', 'Border Radius', 0.0, min: 0, max: 100000),
    _row(
      'layout.overflow',
      'Overflow',
      0,
      choices: ['Visible', 'Clip', 'Bounce'],
    ),
    _row('layout.stagger', 'Stagger', 0.0, min: 0, max: 60),
    if (display == 2) ...[
      _row('layout.column.1', 'Column 1', 1.0, min: 0, max: 1000),
      _row('layout.column.2', 'Column 2', 1.0, min: 0, max: 1000),
      _row('layout.column.3', 'Column 3', 1.0, min: 0, max: 1000),
    ],
  ],
  ..._spaceRows(),
];

List<Map<String, dynamic>> _itemRows({bool grid = false}) => [
  _row(
    'layout.position_type',
    'Position Type',
    0,
    choices: ['Relative', 'Absolute'],
  ),
  _row('layout.snap_to_grid', 'Snap to Grid', 0.0, min: 0, max: 1),
  _row(
    'layout.horizontal_constraint',
    'Horizontal Constraint',
    0,
    choices: ['Left', 'Right', 'Left & Right', 'Center', 'Scale'],
  ),
  _row('layout.horizontal_sizing', 'Horizontal Sizing', 1, choices: _sizing),
  _row('layout.vertical_sizing', 'Vertical Sizing', 0, choices: _sizing),
  _row('layout.width', 'Width', 100.0, min: 0, max: 100000),
  _row('layout.height', 'Height', 100.0, min: 0, max: 100000),
  _row(
    'layout.align_self',
    'Align Self',
    0,
    choices: ['Auto', 'Stretch', 'Start', 'End', 'Center'],
  ),
  if (grid) ...[
    _row('layout.column_start', 'Column Start', 2.0, min: 0, max: 64),
    _row('layout.row_start', 'Row Start', 1.0, min: 0, max: 64),
    _row('layout.column_span', 'Column Span', 2.0, min: 1, max: 64),
    _row('layout.row_span', 'Row Span', 1.0, min: 1, max: 64),
  ],
  _row(
    'layout.object_fit',
    'Object Fit',
    0,
    choices: ['Fill', 'Contain', 'Cover', 'None'],
  ),
  ..._spaceRows(),
];

Map<String, dynamic> _layer(
  int id,
  String kind,
  List<Map<String, dynamic>> layout, {
  int? parent,
}) => {
  'id': id,
  'name': '$kind $id',
  'kind': kind,
  'locked': false,
  'parent': parent,
  'colors': const [],
  'blendMode': 'Normal',
  'projection': '2D',
  'properties': [
    _row('position', 'Position', [0.0, 0.0], kind: 'vec2'),
    _row('scale', 'Scale', [1.0, 1.0], kind: 'vec2'),
    _row('rotation', 'Rotation', 0.0),
    _row('opacity', 'Opacity', 1.0),
    ...layout,
  ],
  'effects': const [],
};

Map<String, dynamic> _document(List<Map<String, dynamic>> layers, int shown) =>
    {
      'layers': layers,
      'selectedId': shown,
      'selectedIds': [shown],
      'selectedKeys': const [],
      'capabilities': const [
        'previewProperties',
        'commitPreview',
        'cancelPreview',
        'toggleKey',
      ],
      'durationFrames': 60,
      'fps': 30.0,
      'contentRevision': '$shown',
    };

const _parentLines = ['grid', 'gap', 'align', 'width', 'height', 'transition'];
const _childLines = ['ignore', 'width', 'height', 'cell'];

Finder _line(String name) => find.byKey(ValueKey('layout:$name'));

Future<(EditorSession, List<Map<String, dynamic>>)> _mount(
  WidgetTester tester,
  Map<String, dynamic> document,
) async {
  tester.view.physicalSize = const Size(420, 1600);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final sent = <Map<String, dynamic>>[];
  TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
      .setMockMethodCallHandler(EditorSession.channel, (call) async {
        final args = call.arguments;
        if (args is Map && args['command'] is String) {
          sent.add(
            Map<String, dynamic>.from(jsonDecode(args['command'] as String)),
          );
        }
        return <String, dynamic>{};
      });
  final c = EditorSession();
  addTearDown(c.dispose);
  c.document.value = document;
  await tester.pumpWidget(
    MaterialApp(
      theme: editorTestTheme,
      home: Scaffold(body: InspectorPanel(controller: c)),
    ),
  );
  await tester.pumpAndSettle();
  return (c, sent);
}

void main() {
  testWidgets('a shape on its own has no Layout card', (tester) async {
    await _mount(tester, _document([_layer(1, 'Shape', _spaceRows())], 1));
    expect(find.text('LAYOUT'), findsNothing);
    for (final name in [..._parentLines, ..._childLines]) {
      expect(_line(name), findsNothing, reason: name);
    }
    // The space rows are not dumped anywhere either.
    expect(find.text('Margin'), findsNothing);
  });

  testWidgets('a group with Display None shows the grid line alone, off', (
    tester,
  ) async {
    final (_, sent) = await _mount(
      tester,
      _document([_layer(1, 'Group', _groupRows(0))], 1),
    );
    expect(find.text('LAYOUT'), findsOneWidget);
    expect(_line('grid'), findsOneWidget);
    for (final name in _parentLines.skip(1)) {
      expect(_line(name), findsNothing, reason: name);
    }
    expect(find.byType(EditorPad), findsNothing);
    final on = tester.widget<EditorSwitch>(
      find.descendant(of: _line('grid'), matching: find.byType(EditorSwitch)),
    );
    expect(on.on, isFalse);
    on.onChanged!(true);
    await tester.pump();
    final preview = sent.where((m) => m['op'] == 'previewProperties').single;
    expect((preview['edits'] as List).single['property'], 'layout.display');
    expect((preview['edits'] as List).single['value'], 2);
  });

  testWidgets('a Grid group shows the six parent lines, no CSS words', (
    tester,
  ) async {
    await _mount(tester, _document([_layer(1, 'Group', _groupRows(2))], 1));
    for (final name in _parentLines) {
      expect(_line(name), findsOneWidget, reason: name);
    }
    for (final name in ['ignore', 'cell', 'direction', 'display']) {
      expect(_line(name), findsNothing, reason: name);
    }
    // The wells are the same rows as before, by id.
    expect(
      tester
          .widget<EditorNumericField>(
            find.byKey(const ValueKey('inspector:layout.grid_columns:0')),
          )
          .value,
      3.0,
    );
    expect(
      find.byKey(const ValueKey('inspector:layout.gap:0')),
      findsOneWidget,
    );
    expect(
      find.byKey(const ValueKey('inspector:layout.padding:1')),
      findsOneWidget,
    );
    expect(
      find.byKey(const ValueKey('inspector:layout.transition_duration:0')),
      findsOneWidget,
    );
    // No CSS word on the card: none of the row names is drawn as text.
    for (final word in [
      'Display',
      'Grid Columns',
      'Gap',
      'Padding',
      'Justify Content',
      'Align Items',
      'Horizontal Sizing',
      'Width',
      'Transition Duration',
    ]) {
      expect(find.text(word), findsNothing, reason: word);
    }
    // The rest of the rows sit behind Advanced, not on the card; the track
    // sizes among them.
    expect(find.text('Advanced'), findsOneWidget);
    expect(find.text('Stagger'), findsNothing);
    expect(find.text('Column 1'), findsNothing);
    await tester.tap(find.text('Advanced'));
    await tester.pumpAndSettle();
    expect(find.text('Stagger'), findsOneWidget);
    expect(find.text('Overflow'), findsOneWidget);
    expect(find.text('Margin'), findsOneWidget);
    expect(find.text('Column 1'), findsOneWidget);
  });

  testWidgets('a Flex document keeps its rows readable behind Advanced', (
    tester,
  ) async {
    await _mount(tester, _document([_layer(1, 'Group', _groupRows(1))], 1));
    expect(_line('grid'), findsOneWidget);
    expect(_line('align'), findsOneWidget);
    expect(find.text('Flex Direction'), findsNothing);
    await tester.tap(find.text('Advanced'));
    await tester.pumpAndSettle();
    expect(find.text('Flex Direction'), findsOneWidget);
    expect(find.text('Flex Wrap'), findsOneWidget);
    expect(find.text('Justify Content'), findsNothing);
  });

  testWidgets('the pad snaps to the nine places and writes both choices', (
    tester,
  ) async {
    final (_, sent) = await _mount(
      tester,
      _document([_layer(1, 'Group', _groupRows(2))], 1),
    );
    final pad = tester.widget<EditorPad>(find.byType(EditorPad));
    expect(pad.unit, isTrue);
    expect(pad.snaps, hasLength(9));
    expect(pad.snapped(const Offset(.9, .1)), const Offset(1, 0));
    // x is across (Justify Content), y is down (Align Items). The grid
    // snapshot leaves these rows off the panel; the pad writes them anyway.
    pad.onBegin();
    await pad.onPreview(1.0, 0.0);
    await pad.onFinish();
    await tester.pump();
    final preview = sent.where((m) => m['op'] == 'previewProperties').single;
    final edits = {
      for (final e in (preview['edits'] as List).cast<Map>())
        e['property']: e['value'],
    };
    expect(edits, {'layout.justify_content': 1, 'layout.align_items': 1});
    expect(sent.last['op'], 'commitPreview');
  });

  testWidgets('sizing reads Hug / Fill / Fixed; the number counts under Fixed', (
    tester,
  ) async {
    final (_, sent) = await _mount(
      tester,
      _document([_layer(1, 'Group', _groupRows(2))], 1),
    );
    // Width is Hug: its number is greyed and inert.
    final width = find.descendant(
      of: _line('width'),
      matching: find.byType(Opacity),
    );
    expect(tester.widget<Opacity>(width).opacity, lessThan(1));
    // Height is Fixed: full ink.
    final height = find.descendant(
      of: _line('height'),
      matching: find.byType(Opacity),
    );
    expect(tester.widget<Opacity>(height).opacity, 1);
    // The mode reads as a plain word, not a glyph: Hug on width, Fixed on
    // height, and picking Fixed writes the same row as before.
    expect(
      find.descendant(of: _line('width'), matching: find.text('Hug')),
      findsOneWidget,
    );
    expect(
      find.descendant(of: _line('height'), matching: find.text('Fixed')),
      findsOneWidget,
    );
    final mode = tester.widget<EditorChoice<dynamic>>(
      find.descendant(
        of: _line('width'),
        matching: find.byType(EditorChoice<dynamic>),
      ),
    );
    mode.onChanged!(2);
    await tester.pump();
    final preview = sent.where((m) => m['op'] == 'previewProperties').single;
    expect(
      (preview['edits'] as List).single['property'],
      'layout.horizontal_sizing',
    );
    expect((preview['edits'] as List).single['value'], 2);
  });

  testWidgets('a child of a laid-out group shows the child lines only', (
    tester,
  ) async {
    await _mount(
      tester,
      _document([
        _layer(1, 'Group', _groupRows(2)),
        _layer(2, 'Shape', _itemRows(), parent: 1),
      ], 2),
    );
    expect(find.text('LAYOUT'), findsOneWidget);
    for (final name in ['ignore', 'width', 'height']) {
      expect(_line(name), findsOneWidget, reason: name);
    }
    for (final name in ['grid', 'align', 'gap', 'transition', 'cell']) {
      expect(_line(name), findsNothing, reason: name);
    }
    expect(find.byType(EditorPad), findsNothing);
    expect(find.text('Position Type'), findsNothing);
    expect(find.text('Align Self'), findsNothing);
  });

  testWidgets('a child of a Grid group shows its cell on one line', (
    tester,
  ) async {
    await _mount(
      tester,
      _document([
        _layer(1, 'Group', _groupRows(2)),
        _layer(2, 'Shape', _itemRows(grid: true), parent: 1),
      ], 2),
    );
    expect(_line('cell'), findsOneWidget);
    for (final id in [
      'layout.column_start',
      'layout.row_start',
      'layout.column_span',
      'layout.row_span',
    ]) {
      expect(find.byKey(ValueKey('inspector:$id:0')), findsOneWidget);
    }
    expect(
      tester
          .widget<EditorNumericField>(
            find.byKey(const ValueKey('inspector:layout.column_span:0')),
          )
          .value,
      2.0,
    );
  });

  testWidgets('counts read whole, zero rows reads auto, units ride inside', (
    tester,
  ) async {
    await _mount(tester, _document([_layer(1, 'Group', _groupRows(2))], 1));
    // Columns is a count: 3, never 3.00, and the field names itself.
    final grid = _line('grid');
    expect(find.descendant(of: grid, matching: find.text('3')), findsOneWidget);
    expect(find.descendant(of: grid, matching: find.text('3.00')), findsNothing);
    expect(
      find.descendant(of: grid, matching: find.text('cols')),
      findsOneWidget,
    );
    // Rows at 0 means as many as the children need: the word, not a zero.
    expect(
      find.descendant(of: grid, matching: find.text('auto')),
      findsOneWidget,
    );
    expect(find.descendant(of: grid, matching: find.text('0')), findsNothing);
    expect(
      find.descendant(of: grid, matching: find.text('rows')),
      findsOneWidget,
    );
    // The number is still the number underneath: scrubbing it off zero
    // brings the digits back.
    final rows = tester.widget<EditorNumericField>(
      find.byKey(const ValueKey('inspector:layout.grid_rows:0')),
    );
    expect(rows.value, 0.0);
    expect(rows.zeroWord, 'auto');
    expect(rows.decimals, 0);
    // Gap and padding carry px; the transition carries seconds.
    final gap = _line('gap');
    expect(find.descendant(of: gap, matching: find.text('12')), findsOneWidget);
    expect(find.descendant(of: gap, matching: find.text('px')), findsNWidgets(3));
    expect(
      tester
          .widget<EditorNumericField>(
            find.byKey(const ValueKey('inspector:layout.transition_duration:0')),
          )
          .unit,
      's',
    );
    // W and H lead the two size rows, so which is which is readable.
    expect(
      find.descendant(of: _line('width'), matching: find.text('W')),
      findsOneWidget,
    );
    expect(
      find.descendant(of: _line('height'), matching: find.text('H')),
      findsOneWidget,
    );
  });
}
