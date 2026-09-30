import 'package:docking/docking.dart';
import 'package:flutter/widgets.dart';

import '../effects/shelf.dart' show EffectScene;
import 'dock_theme.dart';

import '../theme/glyphs.dart' show HG;
import '../stage/stage.dart' show StagePanel;
import '../session/editor_session.dart';
import 'dock_workspace.dart';
import '../browser/browser.dart';
import '../app/console.dart';
import '../session/console_log.dart';
import '../desks/blend/desk.dart';
import '../desks/depth/desk.dart';
import '../desks/ease/desk.dart';
import '../desks/history/desk.dart';
import '../desks/notes/desk.dart';
import '../desks/relations/desk.dart';
import '../inspector/inspector_seat.dart';
import '../timeline/timeline.dart';
import '../desks/web/desk.dart';
import '../theme/surface.dart' show Surface;

/// The product workspace for the hf client. Faces/tools own their content;
/// this layer owns only placement, tabs, split/resize and reopening.
class LiveWorkspace {
  LiveWorkspace({required this.c, required this.scene, required this.console, this.onDetach}) {
    dock = DockWorkspace(
      {
        for (final entry in const [
          ('Create', 0, HG.plus),
          ('Effects', 1, HG.glow),
          ('Colors', 2, HG.color),
          ('Fonts', 3, HG.text),
          ('Media', 4, HG.image),
        ])
          entry.$1: PanelDef(
            entry.$1,
            entry.$1,
            () => LiveBrowser(c: c, scene: scene, fixedTab: entry.$2),
            glyph: entry.$3,
            minSize: 220,
          ),
        'Stage': PanelDef(
          'Stage',
          'Stage',
          () => StagePanel(controller: c, view: 'User'),
          glyph: HG.corners,
          minSize: 320,
        ),
        'Camera': PanelDef(
          'Camera',
          'Camera',
          () => StagePanel(controller: c, view: 'Camera'),
          glyph: HG.camera,
          minSize: 320,
        ),
        'Inspector': PanelDef(
          'Inspector',
          'Inspector',
          () => RightSeat(c: c),
          glyph: HG.list,
          minSize: 240,
        ),
        'Timeline': PanelDef(
          'Timeline',
          'Timeline',
          () => LiveTimeline(c: c),
          tools: () => LiveTimelineTools(c: c),
          glyph: HG.play,
          minSize: 180,
        ),
        // Graph and Console are Timeline-seat panels (Product Home); what they show is not built yet.
        'Graph': PanelDef('Graph', 'Graph', () => const ColoredBox(color: Surface.base), glyph: HG.alongPath, minSize: 180),
        'Console': PanelDef('Console', 'Console', () => LiveConsole(log: console), glyph: HG.type, minSize: 180),
        'Blend': PanelDef('Blend', 'Blend', () => NewBlend(controller: c), glyph: HG.composite, minSize: 220),
        'Depth': PanelDef('Depth', 'Depth', () => NewDepth(controller: c), glyph: HG.diamond, minSize: 220),
        'Ease': PanelDef('Ease', 'Ease', () => LiveEase(c: c), glyph: HG.arrow, minSize: 240),
        'History': PanelDef('History', 'History', () => NewHistory(controller: c), glyph: HG.power, minSize: 220),
        'Notes': PanelDef('Notes', 'Notes', () => LiveNotes(c: c), glyph: HG.star, minSize: 240),
        // Opened where they are asked for: Relations by a property's link in the Inspector, Web from the Dock menu.
        'Relations': PanelDef('Relations', 'Relations', () => RelationsPanel(controller: c), glyph: HG.attach, minSize: 240),
        'Web': PanelDef('Web', 'Web', () => NewWeb(controller: c), glyph: HG.search, minSize: 220),
      },
      // Browser, Stage and Inspector in a row over Timeline and the Desk (2026-09-29, the user's arrangement): the Browser no
      // longer runs the full height, and the desks (Ease, Depth, Blend, History, Notes) have a seat of their own.
      (item) => DockingColumn([
        DockingRow([
          DockingTabs([
            item('Create'),
            item('Effects'),
            item('Colors'),
            item('Fonts'),
            item('Media'),
          ], weight: .22),
          DockingTabs([
            item('Stage'),
            item('Camera'),
          ], weight: .50),
          item('Inspector', weight: .28),
        ], weight: .68),
        DockingRow([
          DockingTabs([item('Timeline'), item('Graph'), item('Console')], weight: .81),
          // the Desk is square-based: as wide as the row is tall at a MacBook's default window (0.32 of the height, 0.19 of the width)
          DockingTabs([item('Ease'), item('Depth'), item('Blend'), item('History'), item('Notes')], weight: .19),
        ], weight: .32),
      ]),
      onDetach: onDetach,
      allowClose: false,
      // the Home panels stay (Reset Layout brings them back anyway); a panel opened on demand closes (Classic WS-04/05)
      closable: (id) => !const {'Create', 'Effects', 'Colors', 'Fonts', 'Media', 'Stage', 'Camera', 'Inspector', 'Timeline', 'Graph', 'Console', 'Ease', 'Depth', 'Blend', 'History', 'Notes'}.contains(id),
      seats: true,
    );
  }

  final EditorSession c;
  final EffectScene scene;

  /// The session's messages, kept by the shell so closing the Console loses nothing.
  final ConsoleLog console;
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
