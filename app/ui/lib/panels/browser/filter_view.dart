import 'package:flutter/widgets.dart';
import 'package:flutter/services.dart';

import '../../foundation/glyphs.dart';
import '../../foundation/leaves.dart';
import '../../foundation/metrics.dart';
import '../../foundation/theme.dart';
import 'filter_library.dart';
import 'shelf.dart';

/// The band under the search field, the way Live 12 draws it: one row per
/// group — its name with a fold arrow, then its tags as chips flowing across —
/// and a results bar under them saying how many filters are on, with Clear
/// and Add label.
class FilterView extends StatelessWidget {
  const FilterView({
    super.key,
    required this.groups,
    required this.filter,
    required this.folded,
    required this.results,
    required this.onFold,
    required this.onToggle,
    required this.onClear,
    required this.onSaveLabel,
    required this.onAddRange,
    required this.onDropRange,
  });
  final List<FilterGroup> groups;

  /// A range group's + pressed with a new `min-max`; a range chip's × pressed.
  final void Function(String group, String range) onAddRange;
  final void Function(String group, String range) onDropRange;
  final ShelfFilter filter;
  final Set<String> folded;
  final int results;
  final ValueChanged<String> onFold;

  /// A tag pressed: `add` when Cmd/Ctrl is held (extend within the group).
  final void Function(String group, String tag, bool add) onToggle;
  final VoidCallback onClear;
  final VoidCallback? onSaveLabel;

  int get active =>
      filter.groups.values.where((s) => s.isNotEmpty).length +
      (filter.collection == null ? 0 : 1);

  @override
  Widget build(BuildContext context) => Column(
    key: const ValueKey('browser:filters'),
    mainAxisSize: MainAxisSize.min,
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      Flexible(
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              for (final group in groups)
                Container(
                  decoration: BoxDecoration(
                    border: Border(
                      bottom: BorderSide(color: EditorTheme.of(context).line),
                    ),
                  ),
                  padding: const EdgeInsets.fromLTRB(
                    EditorMetrics.s6,
                    EditorMetrics.s2,
                    EditorMetrics.s6,
                    EditorMetrics.s3,
                  ),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      EditorPress(
                        key: ValueKey('browser:filter-group:${group.name}'),
                        onTap: () => onFold(group.name),
                        child: SizedBox(
                          height: EditorMetrics.s16,
                          child: Row(
                            children: [
                              Text(
                                group.name,
                                style: TextStyle(
                                  fontSize: EditorMetrics.dense,
                                  color: EditorTheme.of(context).ink,
                                ),
                              ),
                              const SizedBox(width: EditorMetrics.s4),
                              Icon(
                                folded.contains(group.name)
                                    ? Glyph.arrow_right
                                    : Glyph.arrow_drop_down,
                                size: EditorMetrics.s12,
                                color: EditorTheme.of(context).muted,
                              ),
                              if (folded.contains(group.name) &&
                                  (filter.groups[group.name]?.isNotEmpty ??
                                      false))
                                Text(
                                  (filter.groups[group.name] ?? const {}).join(
                                    ', ',
                                  ),
                                  maxLines: 1,
                                  overflow: TextOverflow.ellipsis,
                                  style: TextStyle(
                                    fontSize: EditorMetrics.micro,
                                    color: EditorTheme.of(context).accent,
                                  ),
                                ),
                            ],
                          ),
                        ),
                      ),
                      if (!folded.contains(group.name))
                        Wrap(
                          spacing: EditorMetrics.s3,
                          runSpacing: EditorMetrics.s2,
                          children: [
                            for (final tag in group.tags)
                              _TagChip(
                                key: ValueKey(
                                  'browser:filter:${group.name}:$tag',
                                ),
                                tag: group.kind == FilterKind.range
                                    ? rangeLabel(tag, group.unit)
                                    : tag,
                                chosen:
                                    filter.groups[group.name]?.contains(tag) ??
                                    false,
                                onTap: (add) => onToggle(group.name, tag, add),
                                onRemove: group.kind == FilterKind.range
                                    ? () => onDropRange(group.name, tag)
                                    : null,
                              ),
                            if (group.kind == FilterKind.range)
                              _RangeAdder(
                                key: ValueKey('browser:range:${group.name}'),
                                unit: group.unit,
                                onAdd: (range) => onAddRange(group.name, range),
                              ),
                          ],
                        ),
                    ],
                  ),
                ),
            ],
          ),
        ),
      ),
      Container(
        height: EditorMetrics.row,
        padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s6),
        color: filter.isEmpty
            ? EditorTheme.clear
            : EditorTheme.of(context).raised,
        child: Row(
          children: [
            Text(
              'Results',
              style: TextStyle(
                fontSize: EditorMetrics.dense,
                color: EditorTheme.of(context).ink,
              ),
            ),
            const SizedBox(width: EditorMetrics.s8),
            Text(
              active == 0
                  ? '$results'
                  : '$results · $active ${active == 1 ? 'filter' : 'filters'}',
              style: TextStyle(
                fontSize: EditorMetrics.micro,
                color: active == 0
                    ? EditorTheme.of(context).muted
                    : EditorTheme.of(context).accent,
              ),
            ),
            const Spacer(),
            EditorPress(
              onTap: filter.isEmpty ? null : onClear,
              child: Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: EditorMetrics.s4,
                ),
                child: Text(
                  'Clear',
                  style: TextStyle(
                    fontSize: EditorMetrics.dense,
                    color: filter.isEmpty
                        ? EditorTheme.of(context).disabledInk
                        : EditorTheme.of(context).ink,
                  ),
                ),
              ),
            ),
            EditorTooltip(
              message: 'Add label · keep this filter in the rail',
              child: EditorPress(
                key: const ValueKey('browser:label:add'),
                onTap: filter.isEmpty ? null : onSaveLabel,
                child: Icon(
                  Glyph.playlist_add,
                  size: EditorMetrics.s12,
                  color: filter.isEmpty
                      ? EditorTheme.of(context).disabledInk
                      : EditorTheme.of(context).ink,
                ),
              ),
            ),
          ],
        ),
      ),
    ],
  );
}

class _TagChip extends StatelessWidget {
  const _TagChip({
    super.key,
    required this.tag,
    required this.chosen,
    required this.onTap,
    this.onRemove,
  });
  final String tag;
  final bool chosen;
  final ValueChanged<bool> onTap;
  final VoidCallback? onRemove;
  @override
  Widget build(BuildContext context) => EditorPress(
    onTap: () => onTap(
      HardwareKeyboard.instance.isMetaPressed ||
          HardwareKeyboard.instance.isControlPressed,
    ),
    // No alignment on the box: with one it would take the whole line.
    child: Container(
      height: EditorMetrics.s16,
      padding: const EdgeInsets.symmetric(
        horizontal: EditorMetrics.s5,
        vertical: EditorMetrics.s2,
      ),
      decoration: BoxDecoration(
        color: chosen
            ? EditorTheme.of(context).accent
            : EditorTheme.of(context).raised,
        borderRadius: BorderRadius.circular(EditorMetrics.s2),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            tag,
            style: TextStyle(
              fontSize: EditorMetrics.micro,
              color: chosen
                  ? EditorTheme.of(context).app
                  : EditorTheme.of(context).ink,
            ),
          ),
          if (onRemove != null)
            GestureDetector(
              onTap: onRemove,
              child: Padding(
                padding: const EdgeInsets.only(left: EditorMetrics.s3),
                child: Icon(
                  Glyph.close,
                  size: EditorMetrics.s10,
                  color: chosen
                      ? EditorTheme.of(context).app
                      : EditorTheme.of(context).muted,
                ),
              ),
            ),
        ],
      ),
    ),
  );
}

/// The + at the end of a range group: two small fields, Enter makes the
/// range a chip. Either end may stay empty.
class _RangeAdder extends StatefulWidget {
  const _RangeAdder({super.key, required this.unit, required this.onAdd});
  final String unit;
  final ValueChanged<String> onAdd;
  @override
  State<_RangeAdder> createState() => _RangeAdderState();
}

class _RangeAdderState extends State<_RangeAdder> {
  bool open = false;
  final lo = TextEditingController(), hi = TextEditingController();
  @override
  void dispose() {
    lo.dispose();
    hi.dispose();
    super.dispose();
  }

  void _submit() {
    final range = '${lo.text.trim()}-${hi.text.trim()}';
    if (range != '-') widget.onAdd(range);
    lo.clear();
    hi.clear();
    setState(() => open = false);
  }

  Widget _field(TextEditingController c, String hint, Key key) => SizedBox(
    width: EditorMetrics.s36,
    height: EditorMetrics.s16,
    child: EditorTextField(
      key: key,
      controller: c,
      autofocus: c == lo,
      style: TextStyle(
        fontSize: EditorMetrics.micro,
        color: EditorTheme.of(context).ink,
      ),
      hint: hint,
      padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s3),
      onSubmitted: (_) => _submit(),
    ),
  );

  @override
  Widget build(BuildContext context) => open
      ? Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            _field(lo, 'min', const ValueKey('browser:range:min')),
            const Text('–', style: TextStyle(fontSize: EditorMetrics.micro)),
            _field(hi, 'max', const ValueKey('browser:range:max')),
            if (widget.unit.isNotEmpty)
              Text(
                ' ${widget.unit}',
                style: TextStyle(
                  fontSize: EditorMetrics.micro,
                  color: EditorTheme.of(context).muted,
                ),
              ),
          ],
        )
      : EditorPress(
          onTap: () => setState(() => open = true),
          child: Container(
            height: EditorMetrics.s16,
            padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s5),
            decoration: BoxDecoration(
              border: Border.all(color: EditorTheme.of(context).line),
              borderRadius: BorderRadius.circular(EditorMetrics.s2),
            ),
            child: Icon(
              Glyph.add,
              size: EditorMetrics.s10,
              color: EditorTheme.of(context).muted,
            ),
          ),
        );
}
