import 'package:docking/docking.dart';
import 'package:flutter/widgets.dart';

import '../hf/bp/effects.dart' show EffectScene;
import '../hf/dock/theme.dart';
import '../panels/stage.dart' show StagePanel;
import '../session/editor_session.dart';
import '../workspace/dock_workspace.dart';
import 'adapters/browser.dart';
import 'adapters/right_seat.dart';
import 'adapters/timeline.dart';

/// The product workspace for the hf client. Faces/tools own their content;
/// this layer owns only placement, tabs, split/resize and reopening.
class LiveWorkspace {
  LiveWorkspace({required this.c, required this.scene}) {
    dock = DockWorkspace(
      {
        'Browser': PanelDef(
          'Browser',
          'BROWSER',
          () => LiveBrowser(c: c, scene: scene),
          minSize: 220,
        ),
        'Stage': PanelDef(
          'Stage',
          'STAGE',
          () => StagePanel(controller: c, view: 'User'),
          minSize: 320,
        ),
        'Camera': PanelDef(
          'Camera',
          'CAMERA',
          () => StagePanel(controller: c, view: 'Camera'),
          minSize: 320,
        ),
        'Inspector': PanelDef(
          'Inspector',
          'INSPECTOR',
          () => RightSeat(c: c),
          minSize: 240,
        ),
        'Timeline': PanelDef(
          'Timeline',
          'TIMELINE',
          () => LiveTimeline(c: c),
          minSize: 180,
        ),
      },
      (item) => DockingRow([
        item('Browser', weight: .21),
        DockingColumn([
          DockingRow([
            DockingTabs([
              item('Stage'),
              item('Camera'),
            ], weight: .76),
            item('Inspector', weight: .24),
          ], weight: .69),
          item('Timeline', weight: .31),
        ], weight: .79),
      ]),
    );
  }

  final EditorSession c;
  final EffectScene scene;
  late final DockWorkspace dock;

  Widget build() => TabbedViewTheme(
    data: hfDockTabs(),
    child: MultiSplitViewTheme(
      data: hfDockSplit(),
      child: dock.view(),
    ),
  );
}
