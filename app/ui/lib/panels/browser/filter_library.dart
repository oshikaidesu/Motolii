import 'package:flutter/widgets.dart';

import '../../foundation/theme.dart';
import '../../session/editor_session.dart';

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
  static List<Color> get collectionColors => EditorInk.dark.collectionColors;

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

  /// The user's ranges on one range group (null: keep the shelf's seeds).
  List<String>? rangesOn(String shelf, String group) {
    final held = EditorSession.map(
      controller.deskWork.value['ranges'],
    )[key(shelf, group)];
    return held is List ? held.whereType<String>().toList() : null;
  }

  Future<void> setRanges(String shelf, String group, List<String> ranges) =>
      controller.storeDesk('ranges', {
        ...EditorSession.map(controller.deskWork.value['ranges']),
        key(shelf, group): ranges,
      });

  /// The collections' names: Live's colours until the user renames one.
  String collectionName(int which) {
    final held = EditorSession.map(
      controller.deskWork.value['collectionNames'],
    )['$which'];
    return held is String && held.trim().isNotEmpty
        ? held
        : collectionNames[which - 1];
  }

  Future<void> renameCollection(int which, String name) =>
      controller.storeDesk('collectionNames', {
        ...EditorSession.map(controller.deskWork.value['collectionNames']),
        '$which': name.trim(),
      });

  /// Which filter groups the user folded on a shelf.
  Set<String> foldsOn(String shelf) =>
      (EditorSession.map(controller.deskWork.value['folds'])[shelf] as List? ??
              const [])
          .whereType<String>()
          .toSet();

  Future<void> setFolds(String shelf, Set<String> folds) =>
      controller.storeDesk('folds', {
        ...EditorSession.map(controller.deskWork.value['folds']),
        shelf: folds.toList(),
      });

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

/// A range tag is `min-max`; either end may be empty. Matching is
/// min ≤ value < max.
(double?, double?) parseRange(String tag) {
  final dash = tag.indexOf('-', 1);
  if (dash < 0) return (double.tryParse(tag), null);
  return (
    double.tryParse(tag.substring(0, dash).trim()),
    double.tryParse(tag.substring(dash + 1).trim()),
  );
}

bool inRange(String tag, double value) {
  final (lo, hi) = parseRange(tag);
  return (lo == null || value >= lo) && (hi == null || value < hi);
}

String rangeLabel(String tag, String unit) {
  final (lo, hi) = parseRange(tag);
  String n(double v) => v == v.roundToDouble() ? '${v.toInt()}' : '$v';
  final u = unit.isEmpty ? '' : ' $unit';
  if (lo == null && hi == null) return 'any';
  if (lo == null) return '< ${n(hi!)}$u';
  if (hi == null) return '${n(lo)}$u +';
  return '${n(lo)}–${n(hi)}$u';
}

/// The user's own tags live in one more group beside the item's own.
const userTagGroup = 'Tags';
