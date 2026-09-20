import 'package:flutter/widgets.dart';

import '../../foundation/glyphs.dart';
import '../../foundation/leaves.dart';
import '../../foundation/metrics.dart';
import '../../foundation/theme.dart';
import 'filter_library.dart';
import 'parts.dart';
import 'tile.dart';

/// The rail's lower half: the seven collections, then the saved labels.
/// A double-click on a collection turns its row into a field (Live renames
/// them in place too); Enter keeps the name, Escape or leaving drops it.
class RailCollections extends StatefulWidget {
  const RailCollections({
    super.key,
    required this.chosen,
    required this.labels,
    required this.onCollection,
    required this.onLabel,
    required this.onDropLabel,
    required this.onDrop,
    required this.names,
    required this.onRename,
  });
  final int? chosen;
  final List<Map<String, dynamic>> labels;
  final ValueChanged<int> onCollection;
  final ValueChanged<Map<String, dynamic>> onLabel;
  final ValueChanged<String> onDropLabel;

  /// Rows dropped on a collection: (which, ids). A Media tile carries its
  /// asset id; any other tile carries the picked ids.
  final void Function(int which, Set<String> ids) onDrop;

  /// The collections' names, and a rename (double-click on the row).
  final List<String> names;
  final void Function(int which, String name) onRename;
  @override
  State<RailCollections> createState() => _RailCollectionsState();
}

class _RailCollectionsState extends State<RailCollections> {
  int? editing;
  final field = TextEditingController();
  final focus = FocusNode();
  @override
  void initState() {
    super.initState();
    focus.addListener(() {
      if (!focus.hasFocus && editing != null) setState(() => editing = null);
    });
  }

  @override
  void dispose() {
    field.dispose();
    focus.dispose();
    super.dispose();
  }

  void _keep() {
    final which = editing;
    if (which != null && field.text.trim().isNotEmpty)
      widget.onRename(which, field.text);
    setState(() => editing = null);
  }

  Widget _row(int i) {
    final chosen = widget.chosen;
    if (editing == i) {
      return SizedBox(
        height: EditorMetrics.control,
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s6),
          child: EditorTextField(
            key: const ValueKey('browser:collection:rename'),
            controller: field,
            focusNode: focus,
            autofocus: true,
            style: TextStyle(
              fontSize: EditorMetrics.font,
              color: EditorTheme.of(context).ink,
            ),
            padding: const EdgeInsets.symmetric(
              horizontal: EditorMetrics.s4,
              vertical: EditorMetrics.s3,
            ),
            onSubmitted: (_) => _keep(),
          ),
        ),
      );
    }
    return DragTarget<Object>(
      onWillAcceptWithDetails: (d) =>
          d.data is BrowserDrag ||
          (d.data is Map && (d.data as Map)['asset'] != null),
      onAcceptWithDetails: (d) => widget.onDrop(i, switch (d.data) {
        BrowserDrag(:final ids) => ids,
        final Map m => {'${m['asset']}'},
        _ => const <String>{},
      }),
      builder: (context, hovering, _) => EditorTooltip(
        message:
            '${widget.names[i - 1]} · drop rows here, or press $i on picked rows · double-click to rename',
        child: EditorPress(
          key: ValueKey('browser:collection:$i'),
          onTap: () => widget.onCollection(i),
          onDoubleTap: () => setState(() {
            editing = i;
            field.text = widget.names[i - 1];
            field.selection = TextSelection(
              baseOffset: 0,
              extentOffset: field.text.length,
            );
          }),
          child: Container(
            height: EditorMetrics.control,
            padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s6),
            color: hovering.isNotEmpty
                ? EditorTheme.of(context).spatial.withValues(alpha: .3)
                : chosen == i
                ? EditorTheme.of(context).raised
                : EditorTheme.clear,
            child: Row(
              children: [
                Container(
                  width: EditorMetrics.s8,
                  height: EditorMetrics.s8,
                  decoration: BoxDecoration(
                    color: BrowserLibrary.collectionColors[i - 1],
                    shape: BoxShape.circle,
                  ),
                ),
                const SizedBox(width: EditorMetrics.s6),
                Expanded(
                  child: Text(
                    widget.names[i - 1],
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(
                      fontSize: EditorMetrics.font,
                      color: chosen == i
                          ? EditorTheme.of(context).accent
                          : EditorTheme.of(context).ink,
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      const _RailTitle('Collections'),
      for (var i = 1; i <= BrowserLibrary.collectionCount; i++) _row(i),
      if (widget.labels.isNotEmpty) const _RailTitle('Labels'),
      for (final label in widget.labels)
        Row(
          children: [
            Expanded(
              child: shelfButton(
                '${label['name']}',
                () => widget.onLabel(label),
              ),
            ),
            EditorTooltip(
              message: 'Forget this label',
              child: EditorPress(
                key: ValueKey('browser:label:drop:${label['name']}'),
                onTap: () => widget.onDropLabel('${label['name']}'),
                child: Padding(
                  padding: EdgeInsets.all(EditorMetrics.s4),
                  child: Icon(
                    Glyph.close,
                    size: EditorMetrics.s12,
                    color: EditorTheme.of(context).muted,
                  ),
                ),
              ),
            ),
          ],
        ),
    ],
  );
}

class _RailTitle extends StatelessWidget {
  const _RailTitle(this.text);
  final String text;
  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.fromLTRB(
      EditorMetrics.s6,
      EditorMetrics.s8,
      EditorMetrics.s4,
      EditorMetrics.s4,
    ),
    child: Text(
      text.toUpperCase(),
      maxLines: 1,
      overflow: TextOverflow.ellipsis,
      style: TextStyle(
        fontSize: EditorMetrics.micro,
        letterSpacing: 1,
        color: EditorTheme.of(context).muted,
      ),
    ),
  );
}
