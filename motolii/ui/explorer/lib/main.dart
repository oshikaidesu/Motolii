// The Motolii component explorer: production panels (lib/ of motolii_stage5) over real documents in the real host,
// many states at once, with hot reload. Production does not know this exists; deleting explorer/ changes nothing.
import 'dart:io';

import 'package:flutter/material.dart' show ThemeMode;
import 'package:flutter/widgets.dart';
import 'package:motolii_stage5/hf/shell/place.dart' show H;
import 'package:widgetbook/widgetbook.dart';

import 'binding.dart';
import 'shots.dart';
import 'stories/density.dart';
import 'stories/panels.dart';
import 'stories/paper.dart';
import 'stories/timeline.dart';
import 'stories/workspace.dart';

/// Every story, in the order the explorer lists them.
final allStories = [...timelineStories, ...inspectorStories, ...browserStories, ...catalogStories, ...otherStories, ...paperStories, ...workspaceStories, ...densityStories];

void main() {
  // production reads some of its data relative to motolii/ui (where `flutter run` starts it); so does the explorer
  final ui = Platform.environment['MOTOLII_UI_DIR'];
  if (ui != null && ui.isNotEmpty) Directory.current = ui;
  ExplorerBinding.ensure();
  final shots = Platform.environment['MOTOLII_EXPLORER_SHOTS'];
  runApp(shots == null || shots.isEmpty
      ? const Explorer()
      // the same kind of root production's live main builds (overlay, text style), with the shots instead of the shell
      : WidgetsApp(
          color: H.window,
          debugShowCheckedModeBanner: false,
          textStyle: H.s(12),
          pageRouteBuilder: <T>(RouteSettings settings, WidgetBuilder builder) => PageRouteBuilder<T>(settings: settings, pageBuilder: (context, _, __) => builder(context)),
          home: ShotRunner(stories: allStories, dir: shots),
        ));
}

class Explorer extends StatelessWidget {
  const Explorer({super.key});
  @override
  Widget build(BuildContext context) => Widgetbook.material(
        themeMode: ThemeMode.dark,
        directories: [timelineComponent, ...panelComponents, paperComponent, workspaceComponent],
      );
}
