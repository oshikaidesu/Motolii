import 'package:docking/docking.dart';
import 'package:flutter/widgets.dart';

import '../hf/bp/effects.dart' show EffectScene;
import '../hf/dock/theme.dart';
import '../panels/stage.dart' show StagePanel;
import '../session/editor_session.dart';
import '../workspace/dock_workspace.dart';
import 'adapters/browser.dart';
import 'adapters/blend.dart';
import 'adapters/depth.dart';
import 'adapters/ease.dart';
import 'adapters/history.dart';
import 'adapters/notes.dart';
import 'adapters/right_seat.dart';
import 'adapters/timeline.dart';

/// The product workspace for the hf client. Faces/tools own their content;
/// this layer owns only placement, tabs, split/resize and reopening.
class LiveWorkspace {
  LiveWorkspace({required this.c, required this.scene, this.onDetach}) {
    dock = DockWorkspace(
      {
        for (final entry in const [
          ('Create', 0),
          ('Effects', 1),
          ('Colors', 2),
          ('Fonts', 3),
          ('Media', 4),
        ])
          entry.$1: PanelDef(
            entry.$1,
            entry.$1.toUpperCase(),
            () => LiveBrowser(c: c, scene: scene, fixedTab: entry.$2),
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
        'Blend': PanelDef('Blend', 'BLEND', () => NewBlend(controller: c), minSize: 220),
        'Depth': PanelDef('Depth', 'DEPTH', () => NewDepth(controller: c), minSize: 220),
        'Ease': PanelDef('Ease', 'EASE', () => LiveEase(c: c), minSize: 240),
        'History': PanelDef('History', 'HISTORY', () => NewHistory(controller: c), minSize: 220),
        'Notes': PanelDef('Notes', 'NOTES', () => LiveNotes(c: c), minSize: 240),
      },
      (item) => DockingRow([
        DockingTabs([
          item('Create'),
          item('Effects'),
          item('Colors'),
          item('Fonts'),
          item('Media'),
        ], weight: .21),
        DockingColumn([
          DockingRow([
            DockingTabs([
              item('Stage'),
              item('Camera'),
            ], weight: .76),
            item('Inspector', weight: .24),
          ], weight: .69),
          DockingRow([
            item('Timeline', weight: .78),
            DockingTabs([
              item('Blend'),
              item('Depth'),
              item('Ease'),
              item('History'),
              item('Notes'),
            ], weight: .22),
          ], weight: .31),
        ], weight: .79),
      ]),
      onDetach: onDetach,
    );
  }

  final EditorSession c;
  final EffectScene scene;
  final void Function(String id)? onDetach;
  late final DockWorkspace dock;

  Widget build() => TabbedViewTheme(
    data: hfDockTabs(),
    child: MultiSplitViewTheme(
      data: hfDockSplit(),
      child: dock.view(),
    ),
  );
}
