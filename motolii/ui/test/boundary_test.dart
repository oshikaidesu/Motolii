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

/// A document command whose arguments flip a value read from the document (`!x`, `x != true`, `x == false`): the
/// UI computing the next state from its copy, so two presses before the reply do not undo each other. The owner
/// flips what it holds instead.
List<(int, String)> flipsInACommand(String source) {
  final found = <(int, String)>[];
  for (final m in RegExp(r"command\('(\w+)',\s*\{").allMatches(source)) {
    var depth = 0, end = m.end - 1;
    for (; end < source.length; end++) {
      if (source[end] == '{') depth++;
      if (source[end] == '}' && --depth == 0) break;
    }
    if (RegExp(r':\s*!(?!=)|!=\s*true\b|==\s*false\b').hasMatch(source.substring(m.end, end))) {
      found.add(('\n'.allMatches(source.substring(0, m.start)).length + 1, m.group(1)!));
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

  test('the flip finder reads a whole argument map', () {
    expect(flipsInACommand("c.command('setAttrs', {'layers': [r.id], 'patch': {field: r.layer[field] != true}});"), hasLength(1));
    expect(flipsInACommand("c.command('enableEffect', {\n  'id': e['id'],\n  'enabled': e['enabled'] == false,\n});"), hasLength(1));
    expect(flipsInACommand("c.command('animate', {'enabled': !c.animating});"), hasLength(1));
    expect(flipsInACommand("c.command('toggle', {'layer': id, 'flag': f});"), isEmpty);
    expect(flipsInACommand("c.command('select', {'ids': ids, 'keys': a != b ? [] : k});"), isEmpty);
  });

  test('a document command never flips what the UI read', () {
    final found = [
      for (final file in Directory('lib').listSync(recursive: true).whereType<File>().where((f) => f.path.endsWith('.dart')))
        for (final (line, op) in flipsInACommand(file.readAsStringSync())) '${file.path}:$line flips a read value into `$op`',
    ];
    expect(found, isEmpty, reason: 'send the press; the owner flips what it holds');
  });

  test('one UI action is one document write', () {
    final found = [
      for (final file in Directory('lib').listSync(recursive: true).whereType<File>().where((f) => f.path.endsWith('.dart')))
        for (final (line, op) in writesInALoop(file.readAsStringSync())) '${file.path}:$line sends `$op` once per item',
    ];
    expect(found, isEmpty, reason: 'give the owner the list: one operation, one undo step');
  });
}
