import 'package:flutter/widgets.dart';

import '../../../hf/bp/common.dart' show DockedPanel;
import '../../../session/editor_session.dart';
import '../../../hf/desk/ease_skin.dart';
import '../../../hf/desk/notes_skin.dart';
import '../../../panels/notes_desk.dart';
import '../../../panels/ease_desk.dart';
import 'new_blend.dart';
import 'new_depth.dart';
import 'new_history.dart';
import 'new_web.dart';

/// The Desk's panels drawn by the finished Desk instruments over the session's own state. A name not here keeps the
/// production panel, so a panel is promoted by adding one line.
Widget? newDeskFace(String name, EditorSession c, Key? key, {Widget? leading}) => switch (name) {
      'History' => DockedPanel(child: NewHistory(key: key, controller: c)),
      'Blend' => DockedPanel(child: NewBlend(key: key, controller: c)),
      'Depth' => DockedPanel(child: NewDepth(key: key, controller: c)),
      'Ease' => DockedPanel(child: EaseDesk(key: key, controller: c, leading: leading, skin: (context, view) => EaseSkin(view))),
      'Notes' => DockedPanel(child: NotesPanel(key: key, controller: c, look: hfNoteLook, skin: (context, view) => NotesSkin(view))),
      'Web' => DockedPanel(child: NewWeb(key: key, controller: c)),
      _ => null,
    };
