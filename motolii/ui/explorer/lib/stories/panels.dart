// The other surfaces the first visual round needed, each the production panel the workspace builds.
import 'dart:io';

import 'package:flutter/widgets.dart';
import 'package:motolii_stage5/hf/bp/effects.dart' show EffectScene;
import 'package:motolii_stage5/hf/neutral.dart';
import 'package:motolii_stage5/live_hf/adapters/browser.dart';
import 'package:motolii_stage5/live_hf/adapters/catalog_media.dart';
import 'package:motolii_stage5/live_hf/adapters/catalog_session.dart';
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

/// Pictures of different shapes in the library, through the product's own import.
Future<void> _importMedia(EditorSession c) async {
  // every kind the shelf shows: stills, clips, sounds, environments (media/), and models (models/glb/*.glb, the format the importer takes)
  const exts = ['.jpg', '.png', '.mp4', '.mov', '.wav', '.mp3', '.hdr', '.exr'];
  final dir = Directory(fixture('media'));
  final paths = [for (final f in dir.listSync()..sort((a, b) => a.path.compareTo(b.path))) if (exts.any(f.path.endsWith)) f.path];
  final models = Directory(fixture('models/glb'));
  if (models.existsSync()) paths.addAll([for (final f in models.listSync()) if (f.path.endsWith('.glb')) f.path]);
  // one by one: what the host cannot take (says why on the console) does not stop the rest
  for (final p in paths) {
    try {
      await c.command('import', {'paths': [p]});
    } catch (e) {
      stdout.writeln('EXPLORER_IMPORT_SKIPPED $p: $e');
    }
  }
}

final browserStories = <Story>[
  Story('Browser Media, a real library', Scene('night-sky.rrd', inputs: _importMedia), (c) => _browser(c, 4), width: 288, height: 900),
  Story('Browser Media, a real library, wide', Scene('night-sky.rrd', inputs: _importMedia), (c) => _browser(c, 4), width: 560, height: 900),
  Story('Browser Colors, a real palette', Scene('palette.js', inputs: (s) => _choose(s, 1)), (c) => _browser(c, 2), width: 288, height: 800),
  for (final (i, name) in const ['Create', 'Effects', 'Colors', 'Fonts', 'Media'].indexed) ...[
    Story('Browser $name', Scene('night-sky.rrd', inputs: (s) => _choose(s, 2)), (c) => _browser(c, i), width: 288, height: 800),
    Story('Browser $name narrow', Scene('night-sky.rrd', inputs: (s) => _choose(s, 2)), (c) => _browser(c, i), width: 240, height: 600),
  ],
];

/// The real Catalog over the explorer's own fixtures: two registered folders (media/ and the GLB models), indexed by the
/// host, queried by the host; the Browser only asks and draws. State is the explorer's own (run.sh sets MOTOLII_STATE_DIR).
Future<void> _registerFixtureSources(EditorSession c) async {
  final s = CatalogSession(c);
  await s.load();
  for (final (path, name) in [(fixture('media'), 'Media'), (fixture('models/glb'), 'Models')]) {
    if (!s.sources.any((x) => x.name == name)) await s.addSource(path, name: name);
  }
  await s.refresh();
  while (s.pending > 0) {
    await s.enrichSome();
  }
  s.dispose();
}

Widget _catalogBrowser(EditorSession c) => _CatalogHost(c);

class _CatalogHost extends StatefulWidget {
  const _CatalogHost(this.c);
  final EditorSession c;
  @override
  State<_CatalogHost> createState() => _CatalogHostState();
}

class _CatalogHostState extends State<_CatalogHost> {
  late final CatalogSession session = CatalogSession(widget.c)..load();
  @override
  void dispose() {
    session.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => CatalogMedia(session: session);
}

final catalogStories = <Story>[
  Story('Browser catalog, real sources', Scene('night-sky.rrd', inputs: _registerFixtureSources), _catalogBrowser, width: 288, height: 900),
  Story('Browser catalog, real sources, wide', Scene('night-sky.rrd', inputs: _registerFixtureSources), _catalogBrowser, width: 560, height: 900),
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
  WidgetbookComponent(name: 'Browser', useCases: [for (final s in [...browserStories, ...catalogStories]) useCase(s)]),
  WidgetbookComponent(name: 'Stage, top bar, desks', useCases: [for (final s in otherStories) useCase(s)]),
];
