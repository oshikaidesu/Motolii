import 'package:motolii_lints/src/ratchet.dart';
import 'package:test/test.dart';

void main() {
  test('a file may not exceed its baseline, an unknown file must be clean', () {
    final base = parseBaseline('# c\nlib/a.dart 3\nlib/b.dart 2\n');
    final r = ratchet({'lib/a.dart': 4, 'lib/b.dart': 1, 'lib/c.dart': 1, 'lib/d.dart': 0}, base);
    expect(r.ok, isFalse);
    expect(r.over.keys, ['lib/a.dart']);
    expect(r.fresh.keys, ['lib/c.dart']);
    expect(r.better.keys, ['lib/b.dart']);
  });

  test('holding or lowering passes; a fixed file asks for its line to come down', () {
    final base = parseBaseline('lib/a.dart 3\nlib/b.dart 2\n');
    final r = ratchet({'lib/a.dart': 3}, base);
    expect(r.ok, isTrue);
    expect(r.better.keys, ['lib/b.dart']);
  });

  test('the baseline round-trips and drops clean files', () {
    final text = formatBaseline({'lib/z.dart': 2, 'lib/a.dart': 1, 'lib/q.dart': 0});
    expect(parseBaseline(text), {'lib/a.dart': 1, 'lib/z.dart': 2});
  });
}
