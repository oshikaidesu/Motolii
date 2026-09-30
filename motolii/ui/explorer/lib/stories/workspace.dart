// The whole product workspace (top bar, Create/Effects/…, Stage, Inspector, Timeline) as the live shell builds it, over a
// real document, at one window size and several UI scales, to compare densities side by side.
import 'package:flutter/widgets.dart';
import 'package:motolii_stage5/effects/shelf.dart' show EffectScene;
import 'package:motolii_stage5/hf/neutral.dart';
import 'package:motolii_stage5/app/top/top_session.dart';
import 'package:motolii_stage5/workspace/seats.dart';
import 'package:motolii_stage5/session/console_log.dart';
import 'package:motolii_stage5/session/editor_session.dart';
import 'package:motolii_stage5/session/status_notice.dart';
import 'package:widgetbook/widgetbook.dart';

import '../story.dart';

class WorkspaceFace extends StatefulWidget {
  const WorkspaceFace(this.c, {super.key});
  final EditorSession c;
  @override
  State<WorkspaceFace> createState() => _WorkspaceFaceState();
}

class _WorkspaceFaceState extends State<WorkspaceFace> {
  LiveWorkspace? workspace;
  ConsoleLog? console;

  @override
  void initState() {
    super.initState();
    final c = widget.c;
    final notice = c.slice('notice', const [], derived: () => freezeNotice(c.state) ?? effectsNotice(c.state));
    console = ConsoleLog(c, notice, (doc) => freezeNotice(doc) ?? effectsNotice(doc));
    EffectScene.build().then((scene) {
      if (mounted) setState(() => workspace = LiveWorkspace(c: c, scene: scene, console: console!));
    });
  }

  @override
  void dispose() {
    console?.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => workspace == null
      ? const ColoredBox(color: N.g10)
      : Column(children: [SessionTop(c: widget.c), Expanded(child: workspace!.build())]);
}

Future<void> _pick(EditorSession c) async {
  final ids = [for (final l in c.layers) l['id']];
  if (ids.length > 3) await c.command('select', {'ids': [ids[3]]});
}

const workspaceScales = [.90, 1.0, 1.15];
final workspaceStories = <Story>[
  for (final s in workspaceScales)
    Story('Workspace ${(s * 100).round()}%', Scene('night-sky.rrd', inputs: _pick), (c) => WorkspaceFace(c), width: 1440, height: 900, scale: s),
];

final workspaceComponent = WidgetbookComponent(name: 'Workspace density', useCases: [for (final s in workspaceStories) useCase(s)]);
