import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../foundation/metrics.dart';
import '../../foundation/panel_controls.dart';
import '../../foundation/theme.dart';
import '../../session/editor_session.dart';
import 'parts.dart';
import 'shelf.dart';

/// Tags and collections the way Live 12's browser keeps them (manual 4.4,
/// 4.5): the item itself carries default tags in filter groups; the user adds
/// tags of their own, colours items into up to seven collections, and saves a
/// filter as a label. All of it lives with the user, not in the document
/// (`deskWork`), keyed by the shelf and the item's id.

/// The user's tags, collections and labels for every shelf, read from and
/// written to the desk settings.
class BrowserLibrary {
  BrowserLibrary(this.controller);
  final EditorSession controller;

  static const collectionCount = 7;
  static const collectionNames = [
    'Favorites',
    'Orange',
    'Yellow',
    'Green',
    'Blue',
    'Purple',
    'Gray',
  ];
  static const collectionColors = [
    Color(0xffe05252),
    Color(0xffe0a052),
    Color(0xffe0d452),
    Color(0xff6fd06f),
    Color(0xff52b9e0),
    Color(0xff8f7ae0),
    Color(0xff8a8a8a),
  ];

  Map<String, dynamic> get _tags =>
      EditorSession.map(controller.deskWork.value['tags']);
  Map<String, dynamic> get _collections =>
      EditorSession.map(controller.deskWork.value['collections']);
  Map<String, dynamic> get _labels =>
      EditorSession.map(controller.deskWork.value['labels']);

  static String key(String shelf, String id) => '$shelf/$id';

  /// The user's own tags on one item, in the order they were added.
  List<String> tagsOf(String shelf, String id) =>
      (_tags[key(shelf, id)] as List? ?? const []).whereType<String>().toList();

  /// Every user tag on this shelf, in first-use order.
  List<String> tagsOn(String shelf) {
    final seen = <String>{};
    for (final entry in _tags.entries) {
      if (!entry.key.startsWith('$shelf/')) continue;
      for (final t in (entry.value as List? ?? const []).whereType<String>())
        seen.add(t);
    }
    return seen.toList();
  }

  Future<void> tag(String shelf, Iterable<String> ids, String tag) {
    final next = {..._tags};
    for (final id in ids) {
      final have = tagsOf(shelf, id);
      if (!have.contains(tag)) next[key(shelf, id)] = [...have, tag];
    }
    return controller.storeDesk('tags', next);
  }

  Future<void> untag(String shelf, Iterable<String> ids, String tag) {
    final next = {..._tags};
    for (final id in ids) {
      final have = tagsOf(shelf, id).where((t) => t != tag).toList();
      if (have.isEmpty) {
        next.remove(key(shelf, id));
      } else {
        next[key(shelf, id)] = have;
      }
    }
    return controller.storeDesk('tags', next);
  }

  /// The collection an item sits in (1..7), or null.
  int? collectionOf(String shelf, String id) =>
      (_collections[key(shelf, id)] as num?)?.toInt();

  /// Put items into a collection; 0 takes them out of any.
  Future<void> collect(String shelf, Iterable<String> ids, int which) {
    final next = {..._collections};
    for (final id in ids) {
      if (which == 0) {
        next.remove(key(shelf, id));
      } else {
        next[key(shelf, id)] = which;
      }
    }
    return controller.storeDesk('collections', next);
  }

  /// Saved filters on one shelf: name and the filter it restores.
  List<Map<String, dynamic>> labelsOn(String shelf) =>
      EditorSession.maps(_labels[shelf]);

  Future<void> saveLabel(String shelf, Map<String, dynamic> label) =>
      controller.storeDesk('labels', {
        ..._labels,
        shelf: [
          ...labelsOn(shelf).where((l) => l['name'] != label['name']),
          label,
        ],
      });

  Future<void> dropLabel(String shelf, String name) =>
      controller.storeDesk('labels', {
        ..._labels,
        shelf: labelsOn(shelf).where((l) => l['name'] != name).toList(),
      });
}

/// The filter in force on one shelf: which tags in which group, and which
/// collection. Groups combine with AND, tags within a group with OR.
class ShelfFilter {
  final Map<String, Set<String>> groups = {};
  int? collection;
  bool get isEmpty =>
      collection == null && groups.values.every((s) => s.isEmpty);
  void clear() {
    groups.clear();
    collection = null;
  }

  Map<String, dynamic> toJson() => {
    'groups': {
      for (final e in groups.entries)
        if (e.value.isNotEmpty) e.key: e.value.toList(),
    },
    if (collection != null) 'collection': collection,
  };

  void restore(Map<String, dynamic> json) {
    clear();
    for (final e in EditorSession.map(json['groups']).entries) {
      groups[e.key] = (e.value as List? ?? const [])
          .whereType<String>()
          .toSet();
    }
    collection = (json['collection'] as num?)?.toInt();
  }

  /// The label a saved filter takes when none is given: its tags.
  String describe() => [
    if (collection != null) BrowserLibrary.collectionNames[collection! - 1],
    for (final s in groups.values) ...s,
  ].join(' · ');
}

/// The user's own tags live in one more group beside the item's own.
const userTagGroup = 'Tags';

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
  });
  final List<FilterGroup> groups;
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
                  decoration: const BoxDecoration(
                    border: Border(bottom: BorderSide(color: EditorTheme.line)),
                  ),
                  padding: const EdgeInsets.fromLTRB(
                    EditorMetrics.s8,
                    EditorMetrics.s3,
                    EditorMetrics.s8,
                    EditorMetrics.s4,
                  ),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      InkWell(
                        key: ValueKey('browser:filter-group:${group.name}'),
                        onTap: () => onFold(group.name),
                        child: SizedBox(
                          height: EditorMetrics.row,
                          child: Row(
                            children: [
                              Text(
                                group.name,
                                style: const TextStyle(
                                  fontSize: EditorMetrics.font,
                                  color: EditorTheme.ink,
                                ),
                              ),
                              const SizedBox(width: EditorMetrics.s4),
                              Icon(
                                folded.contains(group.name)
                                    ? Icons.arrow_right
                                    : Icons.arrow_drop_down,
                                size: EditorMetrics.s14,
                                color: EditorTheme.muted,
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
                                  style: const TextStyle(
                                    fontSize: EditorMetrics.dense,
                                    color: EditorTheme.accent,
                                  ),
                                ),
                            ],
                          ),
                        ),
                      ),
                      if (!folded.contains(group.name))
                        Wrap(
                          spacing: EditorMetrics.s4,
                          runSpacing: EditorMetrics.s3,
                          children: [
                            for (final tag in group.tags)
                              _TagChip(
                                key: ValueKey(
                                  'browser:filter:${group.name}:$tag',
                                ),
                                tag: tag,
                                chosen:
                                    filter.groups[group.name]?.contains(tag) ??
                                    false,
                                onTap: (add) => onToggle(group.name, tag, add),
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
        height: EditorMetrics.control,
        padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s8),
        color: filter.isEmpty ? Colors.transparent : EditorTheme.raised,
        child: Row(
          children: [
            Text(
              'Results',
              style: const TextStyle(
                fontSize: EditorMetrics.font,
                color: EditorTheme.ink,
              ),
            ),
            const SizedBox(width: EditorMetrics.s8),
            Text(
              active == 0
                  ? '$results'
                  : '$results · $active ${active == 1 ? 'filter' : 'filters'}',
              style: TextStyle(
                fontSize: EditorMetrics.dense,
                color: active == 0 ? EditorTheme.muted : EditorTheme.accent,
              ),
            ),
            const Spacer(),
            shelfAction('Clear', filter.isEmpty ? null : onClear),
            const SizedBox(width: EditorMetrics.s4),
            EditorTooltip(
              message: 'Add label · keep this filter in the rail',
              child: InkWell(
                key: const ValueKey('browser:label:add'),
                onTap: filter.isEmpty ? null : onSaveLabel,
                child: Icon(
                  Icons.playlist_add,
                  size: EditorMetrics.s14,
                  color: filter.isEmpty
                      ? EditorTheme.disabledInk
                      : EditorTheme.ink,
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
  });
  final String tag;
  final bool chosen;
  final ValueChanged<bool> onTap;
  @override
  Widget build(BuildContext context) => InkWell(
    onTap: () => onTap(
      HardwareKeyboard.instance.isMetaPressed ||
          HardwareKeyboard.instance.isControlPressed,
    ),
    // No alignment on the box: with one it would take the whole line.
    child: Container(
      height: EditorMetrics.row,
      padding: const EdgeInsets.symmetric(
        horizontal: EditorMetrics.s6,
        vertical: EditorMetrics.s3,
      ),
      decoration: BoxDecoration(
        color: chosen ? EditorTheme.accent : EditorTheme.raised,
        borderRadius: BorderRadius.circular(EditorMetrics.s2),
      ),
      child: Text(
        tag,
        style: TextStyle(
          fontSize: EditorMetrics.dense,
          color: chosen ? EditorTheme.app : EditorTheme.ink,
        ),
      ),
    ),
  );
}

/// The rail's lower half: the seven collections, then the saved labels.
class RailCollections extends StatelessWidget {
  const RailCollections({
    super.key,
    required this.chosen,
    required this.labels,
    required this.onCollection,
    required this.onLabel,
    required this.onDropLabel,
  });
  final int? chosen;
  final List<Map<String, dynamic>> labels;
  final ValueChanged<int> onCollection;
  final ValueChanged<Map<String, dynamic>> onLabel;
  final ValueChanged<String> onDropLabel;
  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      const _RailTitle('Collections'),
      for (var i = 1; i <= BrowserLibrary.collectionCount; i++)
        EditorTooltip(
          message:
              '${BrowserLibrary.collectionNames[i - 1]} · press $i on selected rows to add them',
          child: InkWell(
            key: ValueKey('browser:collection:$i'),
            onTap: () => onCollection(i),
            child: Container(
              height: EditorMetrics.control,
              padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s8),
              color: chosen == i ? EditorTheme.raised : Colors.transparent,
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
                      BrowserLibrary.collectionNames[i - 1],
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: TextStyle(
                        fontSize: EditorMetrics.font,
                        color: chosen == i
                            ? EditorTheme.accent
                            : EditorTheme.ink,
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      if (labels.isNotEmpty) const _RailTitle('Labels'),
      for (final label in labels)
        Row(
          children: [
            Expanded(
              child: shelfButton('${label['name']}', () => onLabel(label)),
            ),
            EditorTooltip(
              message: 'Forget this label',
              child: InkWell(
                key: ValueKey('browser:label:drop:${label['name']}'),
                onTap: () => onDropLabel('${label['name']}'),
                child: const Padding(
                  padding: EdgeInsets.all(EditorMetrics.s4),
                  child: Icon(
                    Icons.close,
                    size: EditorMetrics.s12,
                    color: EditorTheme.muted,
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
      EditorMetrics.s8,
      EditorMetrics.s8,
      EditorMetrics.s4,
      EditorMetrics.s4,
    ),
    child: Text(
      text.toUpperCase(),
      maxLines: 1,
      overflow: TextOverflow.clip,
      style: const TextStyle(
        fontSize: EditorMetrics.micro,
        letterSpacing: 1,
        color: EditorTheme.muted,
      ),
    ),
  );
}

/// The band above the zoom bar while rows are picked: the picked items' own
/// tags (quiet, not removable), the user's tags with an ×, and an Add… field.
class QuickTags extends StatelessWidget {
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
  Widget build(BuildContext context) => Container(
    key: const ValueKey('browser:quicktags'),
    padding: const EdgeInsets.symmetric(
      horizontal: EditorMetrics.s8,
      vertical: EditorMetrics.s4,
    ),
    decoration: const BoxDecoration(
      border: Border(top: BorderSide(color: EditorTheme.line)),
    ),
    child: Row(
      children: [
        Text(
          title,
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
          style: const TextStyle(
            fontSize: EditorMetrics.dense,
            color: EditorTheme.muted,
          ),
        ),
        const SizedBox(width: EditorMetrics.s8),
        Expanded(
          child: SingleChildScrollView(
            scrollDirection: Axis.horizontal,
            child: Row(
              children: [
                for (final t in builtin) _Chip(t, muted: true),
                for (final t in own)
                  _Chip(
                    t,
                    onRemove: () => onRemove(t),
                    key: ValueKey('browser:quicktag:$t'),
                  ),
              ],
            ),
          ),
        ),
        SizedBox(
          width: EditorMetrics.s96,
          child: EditorFieldFrame(
            focus: addFocus,
            padding: EdgeInsets.zero,
            child: TextField(
              key: const ValueKey('browser:quicktags:add'),
              controller: addController,
              focusNode: addFocus,
              style: const TextStyle(
                fontSize: EditorMetrics.font,
                color: EditorTheme.ink,
              ),
              decoration: const InputDecoration(
                isDense: true,
                hintText: 'Add…',
                contentPadding: EdgeInsets.symmetric(
                  horizontal: EditorMetrics.s6,
                  vertical: EditorMetrics.s4,
                ),
              ),
              onSubmitted: (value) {
                final tag = value.trim();
                if (tag.isNotEmpty) onAdd(tag);
                addController.clear();
              },
            ),
          ),
        ),
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
      border: Border.all(color: EditorTheme.line),
      borderRadius: BorderRadius.circular(EditorMetrics.s2),
    ),
    child: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Text(
          label,
          style: TextStyle(
            fontSize: EditorMetrics.dense,
            color: muted ? EditorTheme.muted : EditorTheme.ink,
          ),
        ),
        if (onRemove != null)
          InkWell(
            onTap: onRemove,
            child: const Padding(
              padding: EdgeInsets.all(EditorMetrics.s3),
              child: Icon(
                Icons.close,
                size: EditorMetrics.s12,
                color: EditorTheme.muted,
              ),
            ),
          )
        else
          const SizedBox(width: EditorMetrics.s6),
      ],
    ),
  );
}
