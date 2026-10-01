import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

/// The hf GUI is its own client of Motolii Live: everything it reaches is the hf faces, the session, the native bridge
/// or itself. Shared TimelineCore and viewport motion are explicit dependencies; imports into legacy panels, app
/// shells, foundation UI or shortcuts remain forbidden.
void main() {
  test('live_hf reaches only hf, session, bridge and itself', () {
    final allowed = RegExp(
      r'^lib/(hf|session|bridge|live_hf|timeline_core|input)/',
    );
    final seen = <String>{},
        stack = ['lib/live_hf/main.dart'],
        outside = <String>[];
    final directive = RegExp(
      r"^(?:import|export|part) '([^']+)'",
      multiLine: true,
    );
    while (stack.isNotEmpty) {
      final f = stack.removeLast();
      if (!seen.add(f)) continue;
      for (final m in directive.allMatches(File(f).readAsStringSync())) {
        final to = m.group(1)!;
        if (to.startsWith('package:') || to.startsWith('dart:')) continue;
        final p = Uri.file(f).resolve(to).toFilePath();
        final rel = p.substring(p.indexOf('lib/'));
        if (!allowed.hasMatch(rel)) outside.add('$f -> $rel');
        stack.add(rel);
      }
    }
    expect(outside, isEmpty);
    expect(seen.length, greaterThan(10));
  });
}
