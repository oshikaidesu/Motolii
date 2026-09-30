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
    final text = formatBaseline({'lib/z.dart': 2, 'lib/a.dart': 1, 'lib/q.dart': 0}, {'lib/m.dart': (5, 2), 'lib/n.dart': (0, 0)});
    final back = parseBaseline(text);
    expect(back.raw, {'lib/a.dart': 1, 'lib/z.dart': 2});
    expect(back.px, {'lib/m.dart': (5, 2)});
  });

  group('Surface.px exceptions', () {
    final base = parseBaseline('px lib/a.dart 4 3\npx lib/b.dart 2 0\n');

    test('holding, or going down, passes', () {
      final r = ratchet({}, base, px: {'lib/a.dart': (4, 3), 'lib/b.dart': (1, 0)});
      expect(r.ok, isTrue);
      expect(r.pxBetter.keys, ['lib/b.dart']);
    });

    test('one more in a known file fails', () {
      final r = ratchet({}, base, px: {'lib/a.dart': (5, 3)});
      expect(r.ok, isFalse);
      expect(r.pxOver.keys, ['lib/a.dart']);
    });

    test('a new exception without its reason fails even when the total fell', () {
      final r = ratchet({}, base, px: {'lib/a.dart': (3, 4)});
      expect(r.pxOver.keys, ['lib/a.dart']);
    });

    test('a file the baseline does not know may hold none', () {
      final r = ratchet({}, base, px: {'lib/new.dart': (1, 0)});
      expect(r.ok, isFalse);
      expect(r.pxFresh.keys, ['lib/new.dart']);
    });
  });

  test('a regenerated baseline may not raise any count', () {
    final old = parseBaseline('lib/a.dart 3\npx lib/b.dart 2 1\n');
    expect(risers({'lib/a.dart': 3}, {'lib/b.dart': (2, 1)}, old, hadPx: true), isEmpty);
    expect(risers({'lib/a.dart': 4}, {'lib/b.dart': (2, 1)}, old, hadPx: true), hasLength(1));
    expect(risers({'lib/a.dart': 3}, {'lib/b.dart': (3, 1)}, old, hadPx: true), hasLength(1));
    expect(risers({'lib/a.dart': 3}, {'lib/c.dart': (1, 0)}, old, hadPx: true), hasLength(1));
    // the first time the exception section exists it is taken as it is
    expect(risers({'lib/a.dart': 3}, {'lib/c.dart': (9, 9)}, parseBaseline('lib/a.dart 3\n'), hadPx: false), isEmpty);
  });
}
