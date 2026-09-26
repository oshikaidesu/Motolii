import 'package:flutter/widgets.dart';

import '../../../hf/bp/common.dart' show DockedPanel;
import '../../../session/editor_session.dart';
import 'new_blend.dart';
import 'new_history.dart';

/// The Desk's panels drawn by the finished Desk instruments over the session's own state. A name not here keeps the
/// production panel, so a panel is promoted by adding one line.
Widget? newDeskFace(String name, EditorSession c, Key? key) => switch (name) {
      'History' => DockedPanel(child: NewHistory(key: key, controller: c)),
      'Blend' => DockedPanel(child: NewBlend(key: key, controller: c)),
      _ => null,
    };
