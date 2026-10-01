import 'package:flutter/widgets.dart';

import '../../foundation/metrics.dart';
import '../../foundation/theme.dart';
import '../timeline.dart';
import 'frame.dart';

/// Timeline の献立(右クリック) — 行を選び直してから品書きを開き、
/// 選ばれた一品を書類か行の開き方へ渡す。
mixin TimelineMenu on State<TimelinePanel>, TimelineFrame {
  Future<void> menu(TapDownDetails details) async {
    final row = layout.rowAt(details.localPosition.dy);
    final target = row >= 0 && row < tracks.length ? tracks[row].id : null;
    if (row >= 0 &&
        row < tracks.length &&
        !widget.controller.selectedIds.contains(tracks[row].id)) {
      await widget.controller.command('select', {
        'ids': [tracks[row].id],
        'keys': [],
      });
    }
    if (!mounted) return;
    final actions = <String, String>{
      'copy': 'Copy',
      'cut': 'Cut',
      'paste': 'Paste',
      'duplicate': 'Duplicate',
      'delete': 'Delete',
      'group': 'Group',
      'ungroup': 'Ungroup',
      'split': 'Split',
    };
    final frozen =
        row >= 0 && row < tracks.length && tracks[row].layer['frozen'] == true;
    final chosen = await showEditorMenu<String>(
      context,
      details.globalPosition,
      [
        if (target != null) ...[
          // DAW の Freeze Track / Unfreeze(Ableton の右クリック)。中を固めて軽くし、配置は生きたまま。
          EditorMenuItem<String>(
            value: frozen ? 'freeze:off' : 'freeze:on',
            enabled: has('freeze'),
            child: Text(frozen ? 'Unfreeze' : 'Freeze'),
          ),
          const EditorMenuDivider(),
          const EditorMenuItem<String>(
            value: 'lanes:keyed',
            child: Text('Show animated properties'),
          ),
          const EditorMenuItem<String>(
            value: 'lanes:all',
            child: Text('Show all properties'),
          ),
          const EditorMenuItem<String>(
            value: 'lanes:hide',
            child: Text('Hide properties'),
          ),
          const EditorMenuDivider(),
        ],
        for (final entry in actions.entries)
          EditorMenuItem<String>(
            value: entry.key,
            enabled: has(entry.key),
            child: Row(
              children: [
                Expanded(child: Text(entry.value)),
                const SizedBox(width: EditorMetrics.s16),
                Text(
                  const {
                        'copy': '⌘C',
                        'cut': '⌘X',
                        'paste': '⌘V',
                        'duplicate': '⌘D',
                        'delete': '⌫',
                        'group': '⌘G',
                        'ungroup': '⇧⌘G',
                        'split': '⌘K',
                      }[entry.key] ??
                      '',
                ),
              ],
            ),
          ),
      ],
    );
    if (chosen != null && chosen.startsWith('lanes:') && target != null) {
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
    } else if (chosen != null &&
        chosen.startsWith('freeze:') &&
        target != null) {
      widget.controller.command('freeze', {
        'layer': target,
        'enabled': chosen == 'freeze:on',
      });
    } else if (chosen != null)
      widget.controller.command(chosen);
  }
}
