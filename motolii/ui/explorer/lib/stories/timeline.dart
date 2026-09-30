// Timeline stories: the states the first visual round actually needed.
import 'package:motolii_ui/timeline/timeline.dart';
import 'package:motolii_ui/session/editor_session.dart';
import 'package:motolii_ui/timeline/semantics.dart';
import 'package:motolii_ui/timeline/session.dart';
import 'package:widgetbook/widgetbook.dart';

import '../story.dart';

LiveTimeline _timeline(EditorSession c) => LiveTimeline(c: c);

/// Chooses the [n]th layer (as the host lists them), through the product's own select.
Future<void> _choose(EditorSession c, [int n = 0]) async {
  final ids = [for (final l in c.layers) l['id']];
  if (ids.length > n) await c.command('select', {'ids': [ids[n]]});
}

final timelineStories = <Story>[
  const Story('Empty', Scene('empty.js'), _timeline),
  Story('Normal (4 layers)', Scene('night-sky.rrd', inputs: (s) => _choose(s, 2)), _timeline),
  Story('Dense (40 layers)', Scene('dense40.js', inputs: (s) => _choose(s, 3)), _timeline, height: 900),
  Story('Long names (English, Japanese)', Scene('long_names.js', inputs: (s) => _choose(s, 1)), _timeline),
  Story('Many keys', Scene('many_keys.js', inputs: (s) => _choose(s, 0)), _timeline),
  Story(
    'Properties open',
    Scene('many_keys.js', inputs: (s) async {
      await _choose(s, 0);
      final t = TimelineSession.of(s);
      for (final i in [2, 0]) {
        if (i < t.rows.length) t.toggleLanes(i);
      }
    }), _timeline, height: 520),
  const Story('Short bars', Scene('short_bars.js'), _timeline),
  Story('Narrow panel', Scene('dense40.js', inputs: (s) => _choose(s, 3)), _timeline, width: 420, height: 600),
  Story(
    'Playing',
    Scene('night-sky.rrd', inputs: (s) async {
      await s.command('seek', {'frame': 48});
      await s.command('play');
    }), _timeline),
  Story(
    'Bar held mid-drag',
    Scene('dense40.js', inputs: (s) async {
      await _choose(s, 3);
      final t = TimelineSession.of(s);
      final row = t.rows.indexWhere((r) => r.property == null && s.selectedIds.contains(r.id));
      if (row < 0) return;
      final start = (t.rows[row].layer['start'] as num).toDouble() + 4;
      t.press(TlBar(row, TlBarPart.body), frame: start, row: row + .5, mods: const TlMods());
      t.drag(frame: start + 24, row: row + .5);
    }), _timeline, height: 500),
];

final timelineComponent = WidgetbookComponent(name: 'Timeline', useCases: [for (final s in timelineStories) useCase(s)]);
