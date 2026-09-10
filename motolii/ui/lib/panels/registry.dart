import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../foundation/panel_catalog.dart';

import '../session/editor_session.dart';
import 'browser.dart';
import 'inspector.dart';
import 'font_browser.dart';
import 'desk.dart';
import 'timeline.dart';
import 'stage.dart';
import 'notes_desk.dart';
import 'ease_desk.dart';
import 'depth_desk.dart';
import 'blend_panel.dart';
import 'history_records.dart';
import 'web_panel.dart';

Widget buildPanel(String name, EditorSession c, Key? key, {Widget? leading}) {
  final spec = name == 'Desk' ? deskHostSpec : panelSpec(name);
  if (spec == null || name == 'Ease' || (name != 'Desk' && !spec.drawer))
    return _buildPanel(name, c, key, leading: leading);
  return LayoutBuilder(
    key: key,
    builder: (context, box) => SingleChildScrollView(
      primary: false,
      scrollDirection: Axis.horizontal,
      child: SingleChildScrollView(
        primary: false,
        child: SizedBox(
          width: math.max(spec.minWidth, box.maxWidth),
          height: math.max(spec.minHeight, box.maxHeight),
          child: _buildPanel(name, c, null, leading: leading),
        ),
      ),
    ),
  );
}

Widget _buildPanel(String name, EditorSession c, Key? key, {Widget? leading}) {
  if (['Create', 'Media', 'Effects', 'Colors', 'Files'].contains(name))
    return BrowserPanel(
      key: key,
      controller: c,
      fixedTab: name,
      showTabs: false,
    );
  return switch (name) {
    'Stage' => StagePanel(key: key, controller: c),
    'Fonts' => FontBrowser(key: key, controller: c),
    'Inspector' => InspectorPanel(key: key, controller: c),
    'Desk' => DeskPanel(
      key: key,
      controller: c,
      panelBuilder: (name, {leading}) =>
          buildPanel(name, c, ValueKey('drawer:$name'), leading: leading),
    ),
    'Notes' => NotesPanel(key: key, controller: c),
    'Ease' => EaseDesk(key: key, controller: c, leading: leading),
    'Depth' => DepthDesk(key: key, controller: c),
    'Blend' => BlendPanel(key: key, controller: c),
    'History' => HistoryRecords(key: key, controller: c),
    'Web' => WebPanel(key: key, controller: c),
    'Timeline' => TimelinePanel(key: key, controller: c),
    _ => const SizedBox.shrink(),
  };
}
