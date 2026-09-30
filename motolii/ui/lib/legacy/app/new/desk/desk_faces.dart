import 'package:flutter/widgets.dart';

import '../../../../browser/parts.dart' show DockedPanel;
import '../../../../session/editor_session.dart';
import '../../../hf/desk/ease_skin.dart';
import '../../../hf/desk/notes_skin.dart';
import '../../../panels/notes_desk.dart';
import '../../../panels/ease_desk.dart';
import '../../../../stage/panel.dart';
import '../../../panels/timeline.dart';
import '../stage/new_stage_chrome.dart';
import '../timeline/new_timeline_bar.dart';
import '../../../../desks/blend/desk.dart';
import '../../../../desks/depth/desk.dart';
import '../../../../desks/history/desk.dart';
import '../../../../desks/web/desk.dart';

/// The Desk's panels drawn by the finished Desk instruments over the session's own state. A name not here keeps the
/// production panel, so a panel is promoted by adding one line.
Widget? newDeskFace(String name, EditorSession c, Key? key, {Widget? leading}) => switch (name) {
      'History' => DockedPanel(child: NewHistory(key: key, controller: c)),
      'Blend' => DockedPanel(child: NewBlend(key: key, controller: c)),
      'Depth' => DockedPanel(child: NewDepth(key: key, controller: c)),
      'Ease' => DockedPanel(child: EaseDesk(key: key, controller: c, leading: leading, skin: (context, view) => EaseSkin(view))),
      'Notes' => DockedPanel(child: NotesPanel(key: key, controller: c, look: hfNoteLook, skin: (context, view) => NotesSkin(view))),
      'Web' => DockedPanel(child: NewWeb(key: key, controller: c)),
      'Stage' => StagePanel(key: key, controller: c, view: 'User', topBar: newStageTopBar, bottomBar: newStageBottomBar),
      'Camera' => StagePanel(key: key, controller: c, view: 'Camera', topBar: newStageTopBar, bottomBar: newStageBottomBar),
      'Timeline' => TimelinePanel(key: key, controller: c, topBarButtons: newTimelineBarButtons),
      _ => null,
    };
