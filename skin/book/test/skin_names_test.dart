// skins/ROLES.md names every role the window reads, and nothing it does not: a maker (or a model) reads that list alone.
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('ROLES.md is the roles the window reads', () {
    final said = RegExp(r"""(?:ColorTheme\.(?:of|number|word)\(|(?:roles|words)\.value\[)'(\w+)'|_p\('(\w+)', N\.""");
    final read = {
      for (final f in Directory('lib').listSync(recursive: true).whereType<File>().where((f) => f.path.endsWith('.dart')))
        for (final m in said.allMatches(f.readAsStringSync())) m.group(1) ?? m.group(2)!,
    }..addAll(['FontSansFile', 'FontMonoFile']); // (read as any Font...File: theme.dart's _fonts)
    final listed = {for (final m in RegExp(r'`(\w+)`').allMatches(File('skins/ROLES.md').readAsStringSync())) m.group(1)!}..removeAll(['light', 'dark']);
    expect(listed.difference(read), isEmpty, reason: 'listed but not read');
    expect(read.difference(listed), isEmpty, reason: 'read but not listed');
  });
}
