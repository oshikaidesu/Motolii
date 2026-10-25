// The row menu's Keys from MIDI (om_midi's panel): a file chosen, its BPM shown, a kind keyed onto the document.
import 'dart:io';

import 'package:file_selector_platform_interface/file_selector_platform_interface.dart';
import 'package:flutter/gestures.dart' show kSecondaryMouseButton;
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/sets/inspector/inspector_rows_prop_row.dart' show PropRow;
import 'package:plugin_platform_interface/plugin_platform_interface.dart';

import 'door.dart';

/// The system's open dialog, answered (file_selector's own way of trying it: its platform instance replaced).
class _Chosen extends FileSelectorPlatform with MockPlatformInterfaceMixin {
  _Chosen(this.path);
  final String path;
  @override
  Future<XFile?> openFile({List<XTypeGroup>? acceptedTypeGroups, String? initialDirectory, String? confirmButtonText}) async => XFile(path);
}

/// Four quarter notes, C D E F, one track, no tempo (so 120: half a second each), 96 ticks a quarter.
File _fourNotes() {
  final notes = <int>[];
  for (final (i, p) in [60, 62, 64, 65].indexed) {
    notes.addAll([0, 0x90, p, 100, 0x60, 0x80, p, 0]);
  }
  final track = [...notes, 0, 0xff, 0x2f, 0];
  final bytes = [...'MThd'.codeUnits, 0, 0, 0, 6, 0, 0, 0, 1, 0, 96, ...'MTrk'.codeUnits, 0, 0, track.length >> 8, track.length & 0xff, ...track];
  return File('${Directory.systemTemp.createTempSync('midi').path}/four.mid')..writeAsBytesSync(bytes);
}

void main() {
  testWidgets('Keys from MIDI: the file chosen says its BPM, a kind keyed at another BPM is the document\'s, one step', (t) async {
    final file = _fourNotes();
    FileSelectorPlatform.instance = _Chosen(file.path);
    final ws = await openWorkspace(t), m = ws.inside!;
    final a = m.add('shape', 'rect');
    ws.reread();
    ws.select('$a');
    await t.pumpAndSettle();
    final row = find.byWidgetPredicate((w) => w is PropRow && w.id == 'v_rotate');
    expect(row, findsOneWidget, reason: [for (final e in find.byType(PropRow).evaluate()) (e.widget as PropRow).id].join(' '));
    final g = await t.startGesture(t.getCenter(row), buttons: kSecondaryMouseButton);
    await g.up();
    await t.pumpAndSettle();
    await t.tap(find.text('Keys from MIDI…'));
    await t.pumpAndSettle();
    await t.tap(find.text('Choose…'));
    await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 100)));
    await t.pumpAndSettle();
    expect(find.text('four.mid'), findsOneWidget);
    expect(find.text('120'), findsOneWidget, reason: 'a file that says no tempo is at 120, as om_midi\'s BPM box is');
    await t.enterText(find.descendant(of: find.byType(PropRow), matching: find.byType(EditableText)).last, '60');
    await t.ensureVisible(find.text('Apply'));   // (the panel opens under the row: the card scrolled to it, as a hand would)
    await t.pumpAndSettle();
    await t.tap(find.text('Count'));
    await t.pumpAndSettle();
    await t.tap(find.text('Apply'));
    await t.pumpAndSettle();
    expect(find.text('Apply'), findsNothing, reason: 'the panel goes once the keys are made');
    await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 100)));   // (the document's next picture)
    await t.pumpAndSettle();
    expect(m.keyMoments(a, 'rotate'), [0, 1, 2, 3], reason: 'at 60 a note a second (120 in the file): ${m.trouble}');
    expect([for (final at in const [0.0, 1.0, 2.0, 3.0]) m.value(a, 'rotate', at)], [1, 2, 3, 4], reason: 'counted, as om_midi counts');
    expect(m.undo() && m.keyMoments(a, 'rotate').isEmpty, isTrue, reason: 'one step takes all of them back');
    expect(t.takeException(), isNull);
  }, skip: !insideBuilt);
}
