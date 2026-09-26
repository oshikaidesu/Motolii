// The Desk's panels drawn by the finished instruments over the session (promoted one at a time).
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:flutter/gestures.dart';

import '../lib/app/new/desk/new_blend.dart';
import '../lib/app/new/desk/new_history.dart';
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
    c.document.value = {'layers': layers, 'selectedIds': selected, 'selectedId': selected.isEmpty ? null : selected.last, 'capabilities': ['previewBlend', 'setAttrs'], 'path': 'p'};
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
    final set = c.commands.where((e) => e.$1 == 'setAttrs').single.$2;
    expect(set['layers'], [1, 2]);
    expect((set['patch'] as Map)['blendMode'], 'Multiply');
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
    expect(preview, {'layer': 1, 'mode': 'Multiply'});
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
    expect(c.ops, isNot(contains('setAttrs')));
    expect(find.text('No layer'), findsOneWidget);
    await tester.pumpWidget(const SizedBox());
  });
}
