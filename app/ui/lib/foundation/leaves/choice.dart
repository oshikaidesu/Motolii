import 'package:flutter/widgets.dart';

import '../glyphs.dart';
import '../metrics.dart';
import '../theme.dart';
import 'floating.dart';

/// The leaf that picks one value out of a short list.

/// The one value out of a short list, drawn as before: the current label in
/// a bordered box with a drop glyph, the menu below it.
class EditorChoice<T> extends StatelessWidget {
  const EditorChoice({
    super.key,
    required this.value,
    required this.choices,
    required this.onChanged,
  });
  final T? value;
  final List<MapEntry<T, String>> choices;
  final ValueChanged<T>? onChanged;
  @override
  Widget build(BuildContext context) {
    final enabled = onChanged != null;
    final label = choices
        .where((e) => e.key == value)
        .map((e) => e.value)
        .firstOrNull;
    return EditorMenuAnchor(
      items: [
        for (final e in choices)
          EditorMenuChoiceRow(
            onPressed: enabled ? () => onChanged!(e.key) : null,
            child: Text(e.value, maxLines: 1, overflow: TextOverflow.ellipsis),
          ),
      ],
      builder: (context, menu) => GestureDetector(
        onTap: enabled ? (menu.isOpen ? menu.close : menu.open) : null,
        child: Container(
          height: EditorMetrics.row,
          padding: const EdgeInsets.only(left: EditorMetrics.s4),
          decoration: BoxDecoration(
            color: EditorTheme.of(context).app,
            border: Border.all(
              color: enabled
                  ? EditorTheme.of(context).border
                  : EditorTheme.of(context).line,
            ),
          ),
          child: Row(
            children: [
              Expanded(
                child: Text(
                  label ?? '',
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: TextStyle(
                    fontSize: EditorMetrics.font,
                    color: enabled
                        ? EditorTheme.of(context).ink
                        : EditorTheme.of(context).muted,
                  ),
                ),
              ),
              Icon(
                Glyph.arrow_drop_down,
                size: EditorMetrics.s16,
                color: EditorTheme.of(context).muted,
              ),
            ],
          ),
        ),
      ),
    );
  }
}
