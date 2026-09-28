import 'package:docking/docking.dart';
import 'package:flutter/widgets.dart';

import '../hf/bp/effects.dart' show EffectScene;
import '../hf/dock/theme.dart';
import '../hf/bp/common.dart' show kGround;
import '../hf/glyphs.dart' show HG;
import '../panels/stage.dart' show StagePanel;
import '../session/editor_session.dart';
import '../workspace/dock_workspace.dart';
import 'adapters/browser.dart';
import 'adapters/console.dart';
import '../session/console_log.dart';
import 'adapters/blend.dart';
import 'adapters/depth.dart';
import 'adapters/ease.dart';
import 'adapters/history.dart';
import 'adapters/notes.dart';
import 'adapters/relations.dart';
import 'adapters/right_seat.dart';
import 'adapters/timeline.dart';
import '../hf/shell/sheet.dart' show HfAction;
import 'adapters/inspector_alt.dart';
import 'adapters/web.dart';

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
          () => ValueListenableBuilder(valueListenable: altInspectorSkin, builder: (_, alt, __) => alt ? AltInspector(c: c) : RightSeat(c: c)),
          // TEMPORARY: the Inspector's Skin Swap Proof switch
          tools: () => HfAction('A/B', onTap: () => altInspectorSkin.value = !altInspectorSkin.value),
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
        'Graph': PanelDef('Graph', 'Graph', () => const ColoredBox(color: kGround), glyph: HG.alongPath, minSize: 180),
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
      // Weights are the REFERENCE frame's own rectangles (`proto_hf/shell_face.dart`'s 1536×1024 geometry),
      // not tuned by eye: Browser 324, the Stage/Inspector row 779+387, that row 632 tall against
      // Timeline's 291 — Dock adds move/resize/split/detach/persist over this shape, it does not redraw
      // it. The desks (Blend/Depth/Ease/History/Notes) have no seat of their own in the reference: they
      // are not part of the default topology, only PanelDefs `activate()` can still open on demand.
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
            ], weight: .668),
            item('Inspector', weight: .332),
          ], weight: .685),
          DockingTabs([item('Timeline'), item('Graph'), item('Console')], weight: .315),
        ], weight: .79),
      ]),
      onDetach: onDetach,
      allowClose: false,
      // the Home panels stay (Reset Layout brings them back anyway); a panel opened on demand closes (Classic WS-04/05)
      closable: (id) => !const {'Create', 'Effects', 'Colors', 'Fonts', 'Media', 'Stage', 'Camera', 'Inspector', 'Timeline', 'Graph', 'Console'}.contains(id),
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
