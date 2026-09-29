// The Motolii component explorer: production panels (lib/ of motolii_stage5) over real documents in the real host,
// many states at once, with hot reload. Production does not know this exists; deleting explorer/ changes nothing.
import 'dart:io';

import 'package:flutter/material.dart' show ThemeMode;
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import 'binding.dart';
import 'shots.dart';
import 'stories/timeline.dart';

/// Every story, in the order the explorer lists them.
final allStories = [...timelineStories];

void main() {
  ExplorerBinding.ensure();
  final shots = Platform.environment['MOTOLII_EXPLORER_SHOTS'];
  runApp(shots == null || shots.isEmpty ? const Explorer() : Directionality(textDirection: TextDirection.ltr, child: ShotRunner(stories: allStories, dir: shots)));
}

class Explorer extends StatelessWidget {
  const Explorer({super.key});
  @override
  Widget build(BuildContext context) => Widgetbook.material(
        themeMode: ThemeMode.dark,
        directories: [timelineComponent],
      );
}
