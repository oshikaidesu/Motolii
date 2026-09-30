// The other surfaces the first visual round needed, each the production panel the workspace builds.
import 'dart:io';

import 'package:flutter/widgets.dart';
import 'package:motolii_stage5/effects/shelf.dart' show EffectScene;
import 'package:motolii_stage5/hf/neutral.dart';
import 'package:motolii_stage5/browser/panel.dart';
import 'package:motolii_stage5/browser/media/catalog_controls.dart';
import 'package:motolii_stage5/browser/media/catalog_session.dart';
import 'package:motolii_stage5/browser/media/views.dart' show BrowserView;
import 'package:motolii_stage5/live_hf/adapters/depth.dart';
import 'package:motolii_stage5/live_hf/adapters/ease.dart';
import 'package:motolii_stage5/live_hf/adapters/notes.dart';
import 'package:motolii_stage5/live_hf/adapters/right_seat.dart';
import 'package:motolii_stage5/app/top/top_session.dart';
import 'package:motolii_stage5/stage/panel.dart' show StagePanel;
import 'package:motolii_stage5/session/editor_session.dart';
import 'package:widgetbook/widgetbook.dart';

import '../paper/explore_view.dart';
import '../paper/filmstrip.dart';
import 'package:motolii_stage5/browser/media/explore/graph.dart' show ExploreChoice;
import '../paper/live_tiles.dart';

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
  Story('Inspector 500x700 (surface oracle)', Scene('night-sky.rrd', inputs: (s) => _choose(s, 2)), _inspector, width: 500, height: 700),
  Story('Inspector effects 500x700 (surface oracle)', Scene('effects.js', inputs: (s) => _choose(s, 0)), _inspector, width: 500, height: 700),
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
Future<void> _registerFixtureSources(EditorSession c) => _reconcileSources(c, [(fixture('media'), 'Media'), (fixture('models/glb'), 'Models')]);

/// A work that already owns three assets (the catalog's sources are registered as well): the Browser's "This project".
Future<void> _projectHoldsSome(EditorSession c) async {
  await _registerFixtureSources(c);
  await c.importPaths([fixture('media/afterglow.jpg'), fixture('media/coast-drift.mp4'), fixture('media/drum-loop.wav')]);
}

/// The nested Models folder and Media, and a work holding three assets: a world with folders, Sources and a project.
Future<void> _graphWorld(EditorSession c) async {
  await _registerNestedSources(c);
  await c.importPaths([fixture('media/afterglow.jpg'), fixture('media/coast-drift.mp4'), fixture('models/glb/Camera_01.glb')]);
}

/// The folders as they nest on disk (Models holds glb/ and the raw glTF folders with their textures): folder browsing.
Future<void> _registerNestedSources(EditorSession c) => _reconcileSources(c, [(fixture('media'), 'Media'), (fixture('models'), 'Models')]);

/// Makes the catalog's sources exactly these (by name and root), then indexes: a story starts from a known catalog.
Future<void> _reconcileSources(EditorSession c, List<(String, String)> wanted) async {
  final s = CatalogSession(c);
  await s.load();
  for (final have in s.sources) {
    final want = wanted.where((w) => w.$2 == have.name);
    if (want.isEmpty || Directory(want.first.$1).resolveSymbolicLinksSync() != have.root) await s.removeSource(have.id);
  }
  for (final (path, name) in wanted) {
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
  const _CatalogHost(this.c, {this.view = BrowserView.thumbnail, this.startOn, this.startOpen = false, this.startSource, this.startColumn = 60, this.startProject = false, this.graphHops, this.graphOverlay = false});
  final bool graphOverlay;

  /// Explore as the product draws it (the sparse nearest-neighbour map): 0 = global, 1 or 2 = local hops. Null: the old
  /// colour-nearness prototype.
  final int? graphHops;
  final double startColumn;
  final bool startProject;
  final EditorSession c;
  final String? startSource;
  final BrowserView view;
  final String? startOn;
  final bool startOpen;
  @override
  State<_CatalogHost> createState() => _CatalogHostState();
}

class _CatalogHostState extends State<_CatalogHost> {
  late final CatalogSession session = CatalogSession(widget.c);
  late final ExploreChoice choice = ExploreChoice(hops: widget.graphHops ?? 0, overlay: widget.graphOverlay);
  @override
  void initState() {
    super.initState();
    session.load().then((_) {
      final named = session.sources.where((s) => s.name == widget.startSource);
      if (named.isNotEmpty) session.choose(sources: () => {named.first.id});
    });
  }

  @override
  void dispose() {
    session.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => widget.graphHops == null
      ? CatalogMedia(session: session, explore: exploreView, exploreLayout: exploreLayout, exploreRepaint: exploreChanged, exploreNote: 'Nearest by colour · prototype (no similarity source yet) · lines join the nearest five; sounds and models have no colour and stand outermost', initial: widget.view, startOn: widget.startOn, startOpen: widget.startOpen, startColumn: widget.startColumn, startProject: widget.startProject)
      : CatalogMedia(session: session, exploreChoice: choice, initial: widget.view, startOn: widget.startOn, startOpen: widget.startOpen, startColumn: widget.startColumn, startProject: widget.startProject);
}

final catalogStories = <Story>[
  Story('Browser catalog, real sources', Scene('night-sky.rrd', inputs: _registerFixtureSources), _catalogBrowser, width: 288, height: 900),
  Story('Browser catalog, this project', Scene('night-sky.rrd', inputs: _projectHoldsSome), (c) => _CatalogHost(c, startProject: true), width: 420, height: 700),
  Story('Explore 01 global wide', Scene('night-sky.rrd', inputs: _graphWorld), (c) => _CatalogHost(c, view: BrowserView.explore, graphHops: 0), width: 760, height: 640),
  Story('Explore 02 global narrow', Scene('night-sky.rrd', inputs: _graphWorld), (c) => _CatalogHost(c, view: BrowserView.explore, graphHops: 0), width: 320, height: 640),
  Story('Explore 03 global overlay', Scene('night-sky.rrd', inputs: _graphWorld), (c) => _CatalogHost(c, view: BrowserView.explore, graphHops: 0, graphOverlay: true), width: 760, height: 640),
  Story('Explore 04 global selected image', Scene('night-sky.rrd', inputs: _graphWorld), (c) => _CatalogHost(c, view: BrowserView.explore, startOn: 'afterglow.jpg', graphHops: 0), width: 760, height: 640),
  Story('Explore 05 global selected model', Scene('night-sky.rrd', inputs: _graphWorld), (c) => _CatalogHost(c, view: BrowserView.explore, startOn: 'ArmChair_01.glb', graphHops: 0), width: 760, height: 640),
  Story('Explore 06 local 1', Scene('night-sky.rrd', inputs: _graphWorld), (c) => _CatalogHost(c, view: BrowserView.explore, startOn: 'coast-drift.mp4', graphHops: 1), width: 760, height: 640),
  Story('Explore 07 local 2', Scene('night-sky.rrd', inputs: _graphWorld), (c) => _CatalogHost(c, view: BrowserView.explore, startOn: 'coast-drift.mp4', graphHops: 2), width: 760, height: 640),
  Story('Explore 08 local 2 overlay', Scene('night-sky.rrd', inputs: _graphWorld), (c) => _CatalogHost(c, view: BrowserView.explore, startOn: 'coast-drift.mp4', graphHops: 2, graphOverlay: true), width: 760, height: 640),
  Story('Browser catalog, big faces', Scene('night-sky.rrd', inputs: _registerFixtureSources), (c) => _CatalogHost(c, startColumn: 104, startOn: 'coast-drift.mp4'), width: 288, height: 900),
  Story('Browser catalog, real sources, wide', Scene('night-sky.rrd', inputs: _registerFixtureSources), _catalogBrowser, width: 560, height: 900),
  Story('Browser catalog, preview clip', Scene('night-sky.rrd', inputs: _registerFixtureSources), (c) => _CatalogHost(c, startOn: 'coast-drift.mp4', startOpen: true), width: 420, height: 800),
  Story('Browser catalog, preview model', Scene('night-sky.rrd', inputs: _registerFixtureSources), (c) => _CatalogHost(c, startOn: 'Camera_01.glb', startOpen: true), width: 420, height: 800),
  Story('Browser catalog, preview sound wide', Scene('night-sky.rrd', inputs: _registerFixtureSources), (c) => _CatalogHost(c, startOn: 'drum-loop.wav', startOpen: true), width: 640, height: 500),
  Story('Browser catalog, fluid filmstrip', Scene('night-sky.rrd', inputs: _registerFixtureSources), (c) => FluidFilmstrip(c), width: 1020, height: 860),
  Story('Browser catalog, live tiles', Scene('night-sky.rrd', inputs: _registerFixtureSources), (c) => LiveTiles(c), width: 360, height: 460),
  Story('Browser catalog, folders', Scene('night-sky.rrd', inputs: _registerNestedSources), (c) => _CatalogHost(c, view: BrowserView.list, startSource: 'Models'), width: 420, height: 700),
  Story('Browser catalog, list', Scene('night-sky.rrd', inputs: _registerFixtureSources), (c) => _CatalogHost(c, view: BrowserView.list, startOn: 'afterglow.jpg'), width: 420, height: 700),
  Story('Browser catalog, explore', Scene('night-sky.rrd', inputs: _registerFixtureSources), (c) => _CatalogHost(c, view: BrowserView.explore, startOn: 'afterglow.jpg'), width: 420, height: 700),
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
