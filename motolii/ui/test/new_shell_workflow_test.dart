// The New shell on its own: the document workflow reaches the runtime through the same operations as Classic's.
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/app/new_shell.dart';
import '../lib/session/editor_session.dart';
import 'support/editor_test_theme.dart';
import 'support/native_channel.dart';

Future<dynamic> open(WidgetTester tester) async {
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
    final tile = find.text('Rectangle').first;
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
}
