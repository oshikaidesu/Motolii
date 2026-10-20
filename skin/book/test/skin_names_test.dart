// skins/ROLES.md names every role the window reads, and skins/PARTS.md every picture it draws, and nothing else: a maker (or
// a model) reads those lists alone.
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('ROLES.md is the roles the window reads', () {
    final said = RegExp(r"""(?:ColorTheme\.(?:of|number|word)\(|(?:roles|words)\.value\[)'(\w+)'|_p\('(\w+)', N\.|'(Handle[A-Z]\w*)'""");
    final read = {
      for (final f in Directory('lib').listSync(recursive: true).whereType<File>().where((f) => f.path.endsWith('.dart')))
        for (final m in said.allMatches(f.readAsStringSync())) m.group(1) ?? m.group(2) ?? m.group(3)!,
    }..addAll(['FontSansFile', 'FontMonoFile']); // (read as any Font...File: theme.dart's _fonts; the handles' roles are TransformZone's)
    final listed = {for (final m in RegExp(r'`(\w+)`').allMatches(File('skins/ROLES.md').readAsStringSync())) m.group(1)!}..removeAll(['light', 'dark']);
    expect(listed.difference(read), isEmpty, reason: 'listed but not read');
    expect(read.difference(listed), isEmpty, reason: 'read but not listed');
  });

  test('PARTS.md is the pictures the window draws', () {
    // (the window's files: what workspace.dart reaches by import and export)
    final live = <String>{}, todo = ['lib/workspace/workspace.dart'];
    while (todo.isNotEmpty) {
      final f = File(todo.removeLast()).absolute.uri.normalizePath().toFilePath();
      if (!live.add(f) || !File(f).existsSync()) continue;
      for (final m in RegExp(r"^(?:import|export|part)\s+'([^']+)'", multiLine: true).allMatches(File(f).readAsStringSync())) {
        final u = m.group(1)!;
        if (u.startsWith('package:lab_book/')) todo.add('lib/${u.substring(17)}');
        if (!u.contains(':')) todo.add('${File(f).parent.path}/$u');
      }
    }
    final code = [for (final f in live) if (File(f).existsSync()) File(f).readAsStringSync()].join('\n');
    // (each prefix's pictures are the names of one kind of glyph; drawn = named somewhere in the window's files)
    const kinds = {'tp': 'TpGlyph', 'glyph': 'G', 'dock': 'DockG', 'stage': '_Icon'};
    final drawn = {
      for (final e in kinds.entries)
        for (final m in RegExp('\\b${e.value}\\.(\\w+)\\b').allMatches(code.replaceAll(RegExp('case ${e.value}\\.\\w+'), ''))) '${e.key}.${m.group(1)}',
    };
    final listed = {for (final m in RegExp(r'`((?:tp|glyph|dock|stage)\.\w+)`').allMatches(File('skins/PARTS.md').readAsStringSync())) m.group(1)!};
    // (frames: the parts that wear one, each named in SkinFrame('...'); a listed frame may add .hover or .on)
    final framed = {for (final m in RegExp(r"SkinFrame\(\s*'(\w+)'").allMatches(code)) 'frame.${m.group(1)}'};
    final framesListed = {for (final m in RegExp(r'`(frame\.\w+)(?:\.(?:hover|on))?`').allMatches(File('skins/PARTS.md').readAsStringSync())) m.group(1)!};
    expect(framesListed, framed, reason: 'PARTS.md frames are the parts that wear one');
    expect(listed.difference(drawn), isEmpty, reason: 'listed but never drawn');
    expect(drawn.difference(listed).where((n) => !n.endsWith('.values') && !n.endsWith('.name')), isEmpty, reason: 'drawn but not listed');
  });
}
