part of '../browser.dart';

/// What the frame does with tags: the groups a shelf offers filled in with
/// the values its rows carry, what passes the filter, and the band of the
/// picked rows' tags above the zoom bar.
extension BrowserFrameFilters on _BrowserPanelState {
  /// The groups with their tags filled in: declared as given, values from
  /// what the items carry (sorted, numbers first), ranges from the user (or
  /// the shelf's seeds until the user keeps their own), then the user's tags.
  List<FilterGroup> get groups {
    final items = shelf.items(this);
    return [
      for (final g in shelf.groups(this))
        switch (g.kind) {
          FilterKind.declared => g,
          FilterKind.actual => FilterGroup(
            g.name,
            _sortedValues({
              for (final i in items) ?shelf.valueOf(this, i, g.name),
            }),
            kind: g.kind,
            unit: g.unit,
          ),
          FilterKind.range => FilterGroup(
            g.name,
            library.rangesOn(tab, g.name) ?? g.tags,
            kind: g.kind,
            unit: g.unit,
          ),
        },
      if (library.tagsOn(tab).isNotEmpty)
        FilterGroup(userTagGroup, library.tagsOn(tab)),
    ];
  }

  bool _matches(FilterGroup g, Map<String, dynamic> item, Set<String> chosen) =>
      switch (g.kind) {
        FilterKind.declared => chosen.any(tagsOf(item).contains),
        FilterKind.actual => chosen.contains(shelf.valueOf(this, item, g.name)),
        FilterKind.range => switch (shelf.numberOf(this, item, g.name)) {
          null => false,
          final v => chosen.any((r) => inRange(r, v)),
        },
      };
  Set<String> tagsOf(Map<String, dynamic> item) => {
    ...shelf.tagsOf(this, item),
    ...library.tagsOf(tab, id(item)),
  };

  /// Groups combine with AND, tags within one with OR; a collection is one
  /// more AND. [except] leaves one group out (to count what it would keep).
  bool _passesFilter(Map<String, dynamic> item, {String? except}) {
    final f = filter;
    if (f.collection != null &&
        library.collectionOf(tab, id(item)) != f.collection)
      return false;
    final kinds = {for (final g in shelf.groups(this)) g.name: g};
    for (final e in f.groups.entries) {
      if (e.value.isEmpty || e.key == except) continue;
      final g = kinds[e.key] ?? FilterGroup(e.key, const []);
      if (!_matches(g, item, e.value)) return false;
    }
    return true;
  }

  void _toggleTag(String group, String tag, bool add) {
    final set = filter.groups.putIfAbsent(group, () => <String>{});
    if (add) {
      set.contains(tag) ? set.remove(tag) : set.add(tag);
    } else if (set.length == 1 && set.contains(tag)) {
      set.clear();
    } else {
      set
        ..clear()
        ..add(tag);
    }
    relist();
  }

  void _saveLabel() {
    final name = filter.describe();
    if (name.isEmpty) return;
    library.saveLabel(tab, {
      'name': name,
      'filter': filter.toJson(),
      'rail': _chosen,
      'query': search.text,
    });
  }

  void _restoreLabel(Map<String, dynamic> label) {
    refresh(() {
      filter.restore(EditorSession.map(label['filter']));
      classifications[tab] = '${label['rail'] ?? 'All'}';
      search.text = '${label['query'] ?? ''}';
      filtersShown[tab] = true;
      _derive();
    });
  }

  /// The picked rows' tags: theirs (quiet), the user's (removable), Add….
  Widget _quickTags() => ValueListenableBuilder<Set<String>>(
    valueListenable: picked,
    builder: (context, chosen, _) => chosen.isEmpty
        ? const SizedBox()
        : _quickTagsFor(visible.where((i) => chosen.contains(id(i))).toList()),
  );

  Widget _quickTagsFor(List<Map<String, dynamic>> rows) {
    if (rows.isEmpty) return const SizedBox();
    Set<String>? builtin;
    final own = <String>{};
    for (final row in rows) {
      final theirs = shelf.tagsOf(this, row);
      builtin = builtin == null ? {...theirs} : builtin.intersection(theirs);
      own.addAll(library.tagsOf(tab, id(row)));
    }
    final title = rows.length == 1
        ? '${rows.single['name'] ?? id(rows.single)}'
        : '${rows.length} rows';
    return QuickTags(
      title: title,
      builtin: (builtin ?? const {}).toList()..sort(),
      own: own.toList(),
      addFocus: quickAddFocus,
      addController: quickAdd,
      onAdd: (tag) => library.tag(tab, rows.map(id), tag),
      onRemove: (tag) => library.untag(tab, rows.map(id), tag),
    );
  }
}

List<String> _sortedValues(Set<String> values) {
  final list = values.toList();
  list.sort((a, b) {
    final x = double.tryParse(a.split('×').first),
        y = double.tryParse(b.split('×').first);
    if (x != null && y != null && x != y) return x.compareTo(y);
    return a.compareTo(b);
  });
  return list;
}
