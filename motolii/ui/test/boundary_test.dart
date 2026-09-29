// The presentation chooses what the user intends; the owner decides what it means (CONTRIBUTING.md). Only what can be
// found without false alarms is checked here, from the source.
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

/// Document commands awaited one by one in a `for` body: one UI action expanded into several writes, so several undo
/// steps. The owner takes the list in one operation instead. Draining the latest preview (`while`) is one transaction.
List<(int, String)> writesInALoop(String source) {
  final found = <(int, String)>[];
  for (final m in RegExp(r'\bfor \(').allMatches(source)) {
    final close = source.indexOf(')', m.end);
    final start = source.indexOf('{', close);
    if (start < 0 || start - close > 3) continue;
    var depth = 0, end = start;
    for (; end < source.length; end++) {
      if (source[end] == '{') depth++;
      if (source[end] == '}' && --depth == 0) break;
    }
    final line = '\n'.allMatches(source.substring(0, m.start)).length + 1;
    for (final op in RegExp(r"await [\w.!]*command\('(\w+)'").allMatches(source.substring(start, end))) {
      found.add((line, op.group(1)!));
    }
  }
  return found;
}

void main() {
  test('the write-loop finder tells a loop of writes from a drain or a list', () {
    expect(writesInALoop("for (final m in r.mappings) { await c.command('unrelate', {}); }"), hasLength(1));
    expect(writesInALoop("for (var i = 0; i < n; i++) {\n  if (x) { await controller.command('notes', {}); }\n}"), hasLength(1));
    expect(writesInALoop("while (_pending != null) { await c.command('previewProperties', {}); }"), isEmpty);
    expect(writesInALoop("await c.command('notes', {'images': [for (final p in paths) {'path': p}]});"), isEmpty);
    expect(writesInALoop("for (final m in r.mappings) GestureDetector(onTap: () => c.command('unrelate', {}))"), isEmpty);
  });

  test('one UI action is one document write', () {
    final found = [
      for (final file in Directory('lib').listSync(recursive: true).whereType<File>().where((f) => f.path.endsWith('.dart')))
        for (final (line, op) in writesInALoop(file.readAsStringSync())) '${file.path}:$line sends `$op` once per item',
    ];
    expect(found, isEmpty, reason: 'give the owner the list: one operation, one undo step');
  });
}
