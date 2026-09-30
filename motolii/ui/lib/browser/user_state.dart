import 'package:flutter/foundation.dart';

import 'things.dart';
import '../session/editor_session.dart';

/// Live view state uses the existing deskWork keys shared with the other Browser surfaces.
class LiveBrowserUser extends ChangeNotifier {
  LiveBrowserUser(this.controller, this.shelf) {
    _read();
    controller.deskWork.addListener(_changed);
  }

  final EditorSession controller;
  final String shelf;
  static const recentLimit = 12;
  static const _collectionNames = [
    'Favorites',
    'Orange',
    'Yellow',
    'Green',
    'Blue',
    'Purple',
    'Gray',
  ];
  final views = UserViews();
  Map<String, dynamic> get _work => controller.deskWork.value;

  static String _key(String shelf, String id) => '$shelf/$id';

  void _read() {
    final held = EditorSession.map(_work['collections']);
    final grouped = <int, Set<String>>{};
    for (final entry in held.entries) {
      if (!entry.key.startsWith('$shelf/')) continue;
      final collection = (entry.value as num?)?.toInt();
      if (collection != null) {
        (grouped[collection] ??= {}).add(entry.key.substring(shelf.length + 1));
      }
    }
    views.favorites
      ..clear()
      ..addAll(grouped[1] ?? const {});
    views.collections.clear();
    final names = EditorSession.map(_work['collectionNames']);
    for (var i = 2; i <= 7; i++) {
      final ids = grouped[i];
      if (ids != null && ids.isNotEmpty) {
        views.collections['${names['$i'] ?? _collectionNames[i - 1]}'] = ids;
      }
    }
    views.recent
      ..clear()
      ..addAll(
        (EditorSession.map(_work['recent'])[shelf] as List? ?? const [])
            .whereType<String>(),
      );
    views.saved
      ..clear()
      ..addAll({
        for (final entry in EditorSession.map(
          EditorSession.map(_work['searches'])[shelf],
        ).entries)
          entry.key: '${entry.value}',
      });
  }

  void _changed() {
    _read();
    notifyListeners();
  }

  String collectionName(int n) => '${EditorSession.map(_work['collectionNames'])['$n'] ?? _collectionNames[n - 1]}';

  /// Classic's collection lines for a tile menu: add the pick to any of the seven, remove the clicked one from its own.
  List<(String, String)> collectionLines(String clicked) => [
        for (var n = 1; n <= 7; n++) ('collect:$n', 'Add to ${collectionName(n)}'),
        if (collectionOf(clicked) case final n?) ('collect:0', 'Remove from ${collectionName(n)}'),
      ];

  int? collectionOf(String id) =>
      (EditorSession.map(_work['collections'])[_key(shelf, id)] as num?)
          ?.toInt();

  Future<void> collect(Iterable<String> ids, int collection) {
    final next = {...EditorSession.map(_work['collections'])};
    for (final id in ids) {
      if (collection == 0) {
        next.remove(_key(shelf, id));
      } else {
        next[_key(shelf, id)] = collection;
      }
    }
    return controller.storeDesk('collections', next);
  }

  Future<void> used(String id) {
    final next = [
      id,
      ...views.recent.where((value) => value != id),
    ].take(recentLimit).toList();
    return controller.storeDesk('recent', {
      ...EditorSession.map(_work['recent']),
      shelf: next,
    });
  }

  @override
  void dispose() {
    controller.deskWork.removeListener(_changed);
    super.dispose();
  }
}
