import 'package:flutter/widgets.dart';

import '../../../theme/editor_metrics.dart';
import '../../../theme/editor_theme.dart';
import '../../timeline_core/menu.dart';
import '../../timeline_core/frame.dart';

export '../../timeline_core/menu.dart';

/// Classic's presentation of the shared Timeline menu (timeline_core): the editor menu with dividers and shortcuts.
mixin TimelineEditorMenu<T extends StatefulWidget>
    on State<T>, TimelineFrame<T>, TimelineMenu<T> {
  Future<void> menu(TapDownDetails details) => menuAt(
    details.localPosition,
    (items) => showEditorMenu<String>(context, details.globalPosition, [
      for (final item in items) ...[
        EditorMenuItem<String>(
          value: item.value,
          enabled: item.enabled,
          child: item.shortcut.isEmpty
              ? Text(item.label)
              : Row(
                  children: [
                    Expanded(child: Text(item.label)),
                    const SizedBox(width: EditorMetrics.s16),
                    Text(item.shortcut),
                  ],
                ),
        ),
        if (item.groupEnd) const EditorMenuDivider(),
      ],
    ]),
  );
}
