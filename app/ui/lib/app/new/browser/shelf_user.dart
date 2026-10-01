import 'package:flutter/foundation.dart';

import '../../../hf/bp/things.dart';
import '../../../panels/browser/filter_library.dart';
import '../../../session/editor_session.dart';

/// The user's own Browser state for one shelf, kept where Classic keeps it: the desk settings. Favorites are Live's
/// first collection and the other six are the named collections, so both Browsers read and write the same rows.
/// Recent lists and saved searches are the two things Classic never had; they sit in the same desk settings under
/// their own keys. [views] is one live object the finished Browser's bodies read; a change of the settings reloads it.
class ShelfUser extends ChangeNotifier {
  ShelfUser(this.controller, this.shelf) : library = BrowserLibrary(controller) {
    _load();
    controller.deskWork.addListener(_changed);
  }

  final EditorSession controller;
  final String shelf;
  final BrowserLibrary library;
  final views = UserViews();
  static const recentKey = 'recent', searchesKey = 'searches', recentMax = 12;

  Map<String, dynamic> get _work => controller.deskWork.value;

  void _changed() {
    _load();
    notifyListeners();
  }

  void _load() {
    final held = EditorSession.map(_work['collections']);
    final byCollection = <int, Set<String>>{};
    for (final e in held.entries) {
      if (!e.key.startsWith('$shelf/')) continue;
      final n = (e.value as num?)?.toInt();
      if (n != null) (byCollection[n] ??= {}).add(e.key.substring(shelf.length + 1));
    }
    views.favorites
      ..clear()
      ..addAll(byCollection[1] ?? const {});
    views.collections.clear();
    for (var i = 2; i <= BrowserLibrary.collectionCount; i++) {
      final ids = byCollection[i];
      if (ids != null && ids.isNotEmpty) views.collections[library.collectionName(i)] = ids;
    }
    views.recent
      ..clear()
      ..addAll((EditorSession.map(_work[recentKey])[shelf] as List? ?? const []).whereType<String>());
    views.saved
      ..clear()
      ..addAll({for (final e in EditorSession.map(EditorSession.map(_work[searchesKey])[shelf]).entries) e.key: '${e.value}'});
  }

  bool isFavorite(String id) => library.collectionOf(shelf, id) == 1;
  int? collectionOf(String id) => library.collectionOf(shelf, id);

  /// Put rows in a collection (1 is Favorites); 0 takes them out.
  Future<void> collect(Iterable<String> ids, int which) => library.collect(shelf, ids, which);

  Future<void> used(String id) {
    final next = [id, ...views.recent.where((r) => r != id)].take(recentMax).toList();
    return controller.storeDesk(recentKey, {...EditorSession.map(_work[recentKey]), shelf: next});
  }

  Future<void> clearRecent() => controller.storeDesk(recentKey, {...EditorSession.map(_work[recentKey]), shelf: <String>[]});

  Future<void> saveSearch(String name, String query) => controller.storeDesk(searchesKey, {
        ...EditorSession.map(_work[searchesKey]),
        shelf: {...EditorSession.map(EditorSession.map(_work[searchesKey])[shelf]), name: query},
      });

  Future<void> dropSearch(String name) {
    final mine = {...EditorSession.map(EditorSession.map(_work[searchesKey])[shelf])}..remove(name);
    return controller.storeDesk(searchesKey, {...EditorSession.map(_work[searchesKey]), shelf: mine});
  }

  @override
  void dispose() {
    controller.deskWork.removeListener(_changed);
    super.dispose();
  }
}
