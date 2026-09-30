// A relation's source range, changed once, reaches every destination it drives (the real app and its host).
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'package:motolii_ui/desks/relations/model.dart';
import 'package:motolii_ui/desks/relations/session.dart';
import 'package:motolii_ui/timeline/face.dart';
import 'package:motolii_ui/app/main.dart' as app;

Future<void> frames(WidgetTester t, [int n = 20]) async {
  for (var i = 0; i < n; i++) {
    await t.pump(const Duration(milliseconds: 50));
  }
}

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('re-ranging a relation with two destinations keeps both', (t) async {
    app.main();
    await frames(t, 60);
    final c = t.widget<LiveTimeline>(find.byType(LiveTimeline)).c;
    final plain = [for (final l in c.layers) if (l['kind'] != 'Camera' && l['kind'] != 'Group') l['id'] as int];
    final src = plain[0], member = plain[1];
    Map<String, dynamic> relate(String property, double outMax, {double inMin = 0, double inMax = 100, bool preview = false}) => {
          'source': {'layer': src, 'property': 'position', 'component': 0},
          'inMin': inMin,
          'inMax': inMax,
          'members': [member],
          'property': property,
          'outMin': 0.0,
          'outMax': outMax,
          if (preview) 'preview': true,
        };
    await c.command('relate', relate('opacity', 1));
    await c.command('relate', relate('rotation', 90));
    await frames(t);
    expect(relationsOf(c).where((r) => r.source.layer == src).single.mappings, hasLength(2));

    // the panel's range scrub, let go: the RelationsSession writes every destination at once (a link has no preview)
    final rs = RelationsSession.of(c);
    rs.scrubRange('in', 10, 200);
    await rs.write(relationsOf(c).where((r) => r.source.layer == src).single);
    await frames(t);
    final after = relationsOf(c).where((r) => r.source.layer == src).toList();
    expect(after, hasLength(1), reason: 'both destinations carry the new source range');
    expect(after.single.inMin, 10);
    expect(after.single.mappings, hasLength(2));

    // the panel's delete: every destination comes off in one step, and one undo brings the whole relation back
    await rs.remove(after.single);
    await frames(t);
    expect(relationsOf(c).where((r) => r.source.layer == src), isEmpty);
    await c.command('undo');
    await frames(t);
    expect(relationsOf(c).where((r) => r.source.layer == src).single.mappings, hasLength(2), reason: 'one delete, one undo');
  });
}
