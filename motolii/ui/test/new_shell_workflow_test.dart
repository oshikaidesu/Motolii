// The New shell on its own: the document workflow reaches the runtime through the same operations as Classic's.
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/app/new_shell.dart';
import '../lib/foundation/panel_controls.dart' show EditorScale;
import 'support/editor_test_theme.dart';
import 'support/dock_test_utils.dart';
import 'support/native_channel.dart';

Future<dynamic> open(WidgetTester tester) async {
  ignoreSqueezedTabChips();
  await tester.pumpWidget(MaterialApp(theme: editorTestTheme, home: const NewShell()));
  await tester.pumpAndSettle();
  return tester.state(find.byType(NewShell));
}

Future<void> pick(WidgetTester tester, String menu, String item) async {
  await tester.tap(find.text(menu.toUpperCase()));
  await tester.pumpAndSettle();
  await tester.tap(find.text(item).last);
  await tester.pumpAndSettle();
}

/// A Create key by its name: the board prints no caption; the name is the key's semantics label.
Finder mark(String name) => find.byWidgetPredicate((w) => w is Semantics && w.properties.label == name);

void main() {
  void size(WidgetTester tester) {
    tester.view.physicalSize = const Size(1600, 1000);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
  }

  testWidgets('create from the Browser, undo and redo from the Edit menu and the keyboard, save, reopen, export', (tester) async {
    size(tester);
    final native = Native();
    native.install();
    await open(tester);

    // Browser: pick a thing and it is made.
    final tile = mark('Rectangle').first;
    await tester.tap(tile);
    await tester.pump(const Duration(milliseconds: 40));
    await tester.tap(tile); // a shape is placed by double click
    await tester.pumpAndSettle();
    expect(native.ops, contains('create'));
    expect(native.operations.firstWhere((o) => o['op'] == 'create')['kind'], 'rectangle');

    // Edit menu.
    await pick(tester, 'Edit', 'Undo');
    await pick(tester, 'Edit', 'Redo');
    expect(native.ops.where((o) => o == 'undo').length, 1);
    expect(native.ops.where((o) => o == 'redo').length, 1);

    // Keyboard: Cmd+Z, and Cmd+Shift+Z.
    await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
    await tester.sendKeyEvent(LogicalKeyboardKey.keyZ);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
    await tester.pumpAndSettle();
    expect(native.ops.where((o) => o == 'undo').length, 2);

    // Save asks where (untitled), then saves there.
    await pick(tester, 'File', 'Save');
    expect(native.calls.map((c) => c.$1), contains('pickSave'));
    final save = native.operations.firstWhere((o) => o['op'] == 'save');
    expect(save['path'], '/tmp/motolii-test/work.rrd');

    // Reopen: pick a file, the runtime opens that path.
    await pick(tester, 'File', 'Open');
    expect(native.calls.map((c) => c.$1), contains('pickOpen'));
    expect(native.calls.where((c) => c.$1 == 'open').last.$2['path'], '/tmp/motolii-test/work.rrd');

    // Export from the Export sheet.
    await tester.tap(find.text('EXPORT').first);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Export…'));
    await tester.pump(const Duration(milliseconds: 50));
    final export = native.operations.firstWhere((o) => o['op'] == 'export');
    expect(export['path'], '/tmp/motolii-test/out.mp4');
    expect(export['end'], greaterThan(export['start']));
    // The status poll stops with the shell.
    await tester.pumpWidget(const SizedBox());
    await tester.pump(const Duration(seconds: 1));
    expect(tester.takeException(), isNull);
  });

  testWidgets('the UI scale is read at start, applied, changed from Settings and saved beside the other keys', (tester) async {
    size(tester);
    final native = Native()..settings = {'scale': 1.25, 'dock': {'classic': 'layout'}};
    native.install();
    ignoreSqueezedTabChips();
    final scale = ValueNotifier(1.0);
    await tester.pumpWidget(EditorScale(
      notifier: scale,
      child: MaterialApp(theme: editorTestTheme, home: const NewShell()),
    ));
    await tester.pumpAndSettle();
    expect(scale.value, 1.25, reason: 'the saved scale is applied');
    await tester.tap(find.text('SETTINGS'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('+').last); // the sheet is above the face
    await tester.pumpAndSettle();
    expect(scale.value, closeTo(1.26, 1e-9));
    expect(native.settings['scale'], closeTo(1.26, 1e-9));
    expect(native.settings['dock'], {'classic': 'layout'}, reason: "Classic's keys are kept");
    await tester.pumpWidget(const SizedBox());
  });

  testWidgets('the Browser answers the keyboard: arrows move the pick, Enter applies it, Cmd+F finds, Esc lets go', (tester) async {
    size(tester);
    final native = Native();
    native.install();
    await open(tester);

    // Pick the first shape by clicking it (a click on a thing that makes a layer only picks).
    await tester.tap(mark('Rectangle').first);
    await tester.pumpAndSettle();
    expect(native.ops, isNot(contains('create')), reason: 'a click only picks');

    // Enter applies the picked one.
    await tester.sendKeyEvent(LogicalKeyboardKey.enter);
    await tester.pumpAndSettle();
    expect(native.operations.where((o) => o['op'] == 'create').last['kind'], 'rectangle');

    // Arrow right moves the pick to the next thing; Enter makes that one.
    await tester.sendKeyEvent(LogicalKeyboardKey.arrowRight);
    await tester.sendKeyEvent(LogicalKeyboardKey.enter);
    await tester.pumpAndSettle();
    final made = native.operations.where((o) => o['op'] == 'create').map((o) => o['kind']).toList();
    expect(made.length, 2);
    expect(made.last, isNot('rectangle'), reason: 'the pick moved on');

    // Cmd+F puts the caret in the search field; typing narrows; Escape from the grid lets the pick go.
    await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
    await tester.sendKeyEvent(LogicalKeyboardKey.keyF);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(EditableText).first, 'ellip');
    await tester.pumpAndSettle();
    expect(mark('Ellipse'), findsWidgets);
    expect(mark('Rectangle'), findsNothing, reason: 'the search narrowed the list');
    await tester.enterText(find.byType(EditableText).first, '');
    await tester.pumpAndSettle();
    expect(mark('Rectangle'), findsWidgets);
    expect(tester.takeException(), isNull);
  });
}
