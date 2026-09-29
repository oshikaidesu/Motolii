// The other surfaces the first visual round needed, each the production panel the workspace builds.
import 'package:flutter/widgets.dart';
import 'package:motolii_stage5/hf/bp/effects.dart' show EffectScene;
import 'package:motolii_stage5/hf/neutral.dart';
import 'package:motolii_stage5/live_hf/adapters/browser.dart';
import 'package:motolii_stage5/live_hf/adapters/depth.dart';
import 'package:motolii_stage5/live_hf/adapters/ease.dart';
import 'package:motolii_stage5/live_hf/adapters/notes.dart';
import 'package:motolii_stage5/live_hf/adapters/right_seat.dart';
import 'package:motolii_stage5/live_hf/adapters/top.dart';
import 'package:motolii_stage5/panels/stage.dart' show StagePanel;
import 'package:motolii_stage5/session/editor_session.dart';
import 'package:widgetbook/widgetbook.dart';

import '../story.dart';

Future<void> _choose(EditorSession c, [int n = 0]) async {
  final ids = [for (final l in c.layers) l['id']];
  if (ids.length > n) await c.command('select', {'ids': [ids[n]]});
}

/// The effect tiles' sample picture, built once as the shell builds it.
final Future<EffectScene> _scene = EffectScene.build();
Widget _browser(EditorSession c, int tab) => FutureBuilder<EffectScene>(
      future: _scene,
      builder: (_, s) => s.data == null ? const ColoredBox(color: N.g10) : LiveBrowser(c: c, scene: s.data!, fixedTab: tab),
    );

Widget _inspector(EditorSession c) => RightSeat(c: c);

final inspectorStories = <Story>[
  Story('Transform (shape)', Scene('night-sky.rrd', inputs: (s) => _choose(s, 2)), _inspector, width: 387, height: 640),
  Story('Transform (text)', Scene('night-sky.rrd', inputs: (s) => _choose(s, 1)), _inspector, width: 387, height: 640),
  Story('Effects, long parameter list', Scene('effects.js', inputs: (s) => _choose(s, 0)), _inspector, width: 387, height: 1100),
  Story('Inspector narrow', Scene('night-sky.rrd', inputs: (s) => _choose(s, 2)), _inspector, width: 260, height: 640),
  Story('Nothing chosen', Scene('night-sky.rrd', inputs: (s) => s.command('select', {'ids': []})), _inspector, width: 387, height: 400),
];

final browserStories = <Story>[
  for (final (i, name) in const ['Create', 'Effects', 'Colors', 'Fonts', 'Media'].indexed) ...[
    Story('Browser $name', Scene('night-sky.rrd', inputs: (s) => _choose(s, 2)), (c) => _browser(c, i), width: 324, height: 800),
    Story('Browser $name narrow', Scene('night-sky.rrd', inputs: (s) => _choose(s, 2)), (c) => _browser(c, i), width: 240, height: 600),
  ],
];

final otherStories = <Story>[
  Story('Stage chrome', Scene('night-sky.rrd', inputs: (s) => _choose(s, 2)), (c) => StagePanel(controller: c, view: 'User'), width: 779, height: 520),
  Story('Top bar', const Scene('night-sky.rrd'), (c) => SessionTop(c: c), width: 1536, height: 64),
  Story('Depth desk', Scene('dense40.js', inputs: (s) => _choose(s, 3)), (c) => NewDepth(controller: c), width: 420, height: 420),
  Story('Ease desk', Scene('many_keys.js', inputs: (s) => _choose(s, 0)), (c) => LiveEase(c: c), width: 420, height: 420),
  Story('Notes desk', const Scene('night-sky.rrd'), (c) => LiveNotes(c: c), width: 420, height: 420),
];

final panelComponents = [
  WidgetbookComponent(name: 'Inspector', useCases: [for (final s in inspectorStories) useCase(s)]),
  WidgetbookComponent(name: 'Browser', useCases: [for (final s in browserStories) useCase(s)]),
  WidgetbookComponent(name: 'Stage, top bar, desks', useCases: [for (final s in otherStories) useCase(s)]),
];
