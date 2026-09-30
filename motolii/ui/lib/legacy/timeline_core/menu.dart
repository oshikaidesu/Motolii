import 'package:flutter/widgets.dart';

import 'frame.dart';

/// One line of the Timeline's right-click menu; [groupEnd] ends a group (a presentation may draw a divider after it).
typedef TimelineMenuItem = ({
  String value,
  String label,
  bool enabled,
  String shortcut,
  bool groupEnd,
});

/// The Timeline's right-click menu: picks the row under the press first (a selection it is part of is kept), offers
/// what can be done to it, and does the chosen one — to the document, or to how the row's lanes open. Each
/// presentation shows the items its own way through [show].
mixin TimelineMenu<T extends StatefulWidget> on State<T>, TimelineFrame<T> {
  /// [at] is in the tracks' own coordinates (the ones `layout.rowAt` reads).
  Future<void> menuAt(
    Offset at,
    Future<String?> Function(List<TimelineMenuItem> items) show,
  ) async {
    final row = layout.rowAt(at.dy);
    final target = row >= 0 && row < tracks.length ? tracks[row].id : null;
    if (target != null && !timelineSession.selectedIds.contains(target)) {
      await timelineSession.command('select', {
        'ids': [target],
        'keys': [],
      });
    }
    if (!mounted) return;
    final frozen = target != null && tracks[row].layer['frozen'] == true;
    final chosen = await show([
      if (target != null) ...[
        // DAW の Freeze Track / Unfreeze(Ableton の右クリック)。中を固めて軽くし、配置は生きたまま。
        (
          value: frozen ? 'freeze:off' : 'freeze:on',
          label: frozen ? 'Unfreeze' : 'Freeze',
          enabled: has('freeze'),
          shortcut: '',
          groupEnd: true,
        ),
        (
          value: 'lanes:keyed',
          label: 'Show animated properties',
          enabled: true,
          shortcut: '',
          groupEnd: false,
        ),
        (
          value: 'lanes:all',
          label: 'Show all properties',
          enabled: true,
          shortcut: '',
          groupEnd: false,
        ),
        (
          value: 'lanes:hide',
          label: 'Hide properties',
          enabled: true,
          shortcut: '',
          groupEnd: true,
        ),
      ],
      for (final (value, label, shortcut) in const [
        ('copy', 'Copy', '⌘C'),
        ('cut', 'Cut', '⌘X'),
        ('paste', 'Paste', '⌘V'),
        ('duplicate', 'Duplicate', '⌘D'),
        ('delete', 'Delete', '⌫'),
        ('group', 'Group', '⌘G'),
        ('ungroup', 'Ungroup', '⇧⌘G'),
        ('split', 'Split', '⌘K'),
      ])
        (
          value: value,
          label: label,
          enabled: has(value),
          shortcut: shortcut,
          groupEnd: false,
        ),
    ]);
    if (!mounted || chosen == null) return;
    if (chosen.startsWith('lanes:') && target != null) {
      setState(() {
        allProperties.remove(target);
        if (chosen == 'lanes:hide')
          expanded.remove(target);
        else {
          expanded.add(target);
          if (chosen == 'lanes:all') allProperties.add(target);
        }
        relane();
      });
    } else if (chosen.startsWith('freeze:') && target != null) {
      timelineSession.command('freeze', {
        'layer': target,
        'enabled': chosen == 'freeze:on',
      });
    } else if (!chosen.contains(':')) {
      timelineSession.command(chosen);
    }
  }
}
