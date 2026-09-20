import 'package:flutter/foundation.dart' show listEquals;
import 'package:flutter/widgets.dart';

import '../../foundation/glyphs.dart';
import '../../foundation/leaves.dart';
import '../../foundation/metrics.dart';
import '../../foundation/panel_controls.dart';
import '../../foundation/theme.dart';

/// The band above the zoom bar while rows are picked: the picked items' own
/// tags (quiet, not removable), the user's tags with an ×, and an Add… field.
class QuickTags extends StatefulWidget {
  const QuickTags({
    super.key,
    required this.title,
    required this.builtin,
    required this.own,
    required this.addFocus,
    required this.addController,
    required this.onAdd,
    required this.onRemove,
  });
  final String title;
  final List<String> builtin, own;
  final FocusNode addFocus;
  final TextEditingController addController;
  final ValueChanged<String> onAdd;
  final ValueChanged<String> onRemove;
  @override
  State<QuickTags> createState() => _QuickTagsState();
}

class _QuickTagsState extends State<QuickTags> {
  late Widget input = _input();
  late Widget tags = _tags();

  @override
  void didUpdateWidget(QuickTags old) {
    super.didUpdateWidget(old);
    if (!listEquals(old.builtin, widget.builtin) ||
        !listEquals(old.own, widget.own)) {
      tags = _tags();
    }
    if (old.addFocus != widget.addFocus ||
        old.addController != widget.addController) {
      input = _input();
    }
  }

  Widget _tags() => Expanded(
    child: SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: Row(
        children: [
          for (final t in widget.builtin) _Chip(t, muted: true),
          for (final t in widget.own)
            _Chip(
              t,
              onRemove: () => widget.onRemove(t),
              key: ValueKey('browser:quicktag:$t'),
            ),
        ],
      ),
    ),
  );

  Widget _input() => SizedBox(
    width: EditorMetrics.s96,
    child: EditorFieldFrame(
      focus: widget.addFocus,
      padding: EdgeInsets.zero,
      child: EditorTextField(
        key: const ValueKey('browser:quicktags:add'),
        controller: widget.addController,
        focusNode: widget.addFocus,
        style: TextStyle(
          fontSize: EditorMetrics.font,
          color: EditorTheme.of(context).ink,
        ),
        hint: 'Add…',
        padding: const EdgeInsets.symmetric(
          horizontal: EditorMetrics.s6,
          vertical: EditorMetrics.s4,
        ),
        onSubmitted: (value) {
          final tag = value.trim();
          if (tag.isNotEmpty) widget.onAdd(tag);
          widget.addController.clear();
        },
      ),
    ),
  );

  @override
  Widget build(BuildContext context) => Container(
    key: const ValueKey('browser:quicktags'),
    padding: const EdgeInsets.symmetric(
      horizontal: EditorMetrics.s6,
      vertical: EditorMetrics.s4,
    ),
    decoration: BoxDecoration(
      border: Border(top: BorderSide(color: EditorTheme.of(context).line)),
    ),
    child: Row(
      children: [
        Expanded(
          child: SizedBox(
            height: EditorMetrics.row,
            child: Align(
              alignment: Alignment.centerLeft,
              child: Text(
                widget.title,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: TextStyle(
                  fontSize: EditorMetrics.dense,
                  color: EditorTheme.of(context).muted,
                ),
              ),
            ),
          ),
        ),
        const SizedBox(width: EditorMetrics.s8),
        tags,
        input,
      ],
    ),
  );
}

class _Chip extends StatelessWidget {
  const _Chip(this.label, {super.key, this.muted = false, this.onRemove});
  final String label;
  final bool muted;
  final VoidCallback? onRemove;
  @override
  Widget build(BuildContext context) => Container(
    margin: const EdgeInsets.only(right: EditorMetrics.s4),
    padding: const EdgeInsets.only(left: EditorMetrics.s6),
    decoration: BoxDecoration(
      border: Border.all(color: EditorTheme.of(context).line),
      borderRadius: BorderRadius.circular(EditorMetrics.s2),
    ),
    child: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Text(
          label,
          style: TextStyle(
            fontSize: EditorMetrics.dense,
            color: muted
                ? EditorTheme.of(context).muted
                : EditorTheme.of(context).ink,
          ),
        ),
        if (onRemove != null)
          EditorPress(
            onTap: onRemove,
            child: Padding(
              padding: EdgeInsets.all(EditorMetrics.s3),
              child: Icon(
                Glyph.close,
                size: EditorMetrics.s12,
                color: EditorTheme.of(context).muted,
              ),
            ),
          )
        else
          const SizedBox(width: EditorMetrics.s6),
      ],
    ),
  );
}
