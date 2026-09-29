import 'dart:async';
import 'dart:convert';

import 'package:flutter/foundation.dart';

import '../../session/editor_session.dart';
import 'browser_item.dart';

/// A registered folder, as the catalog owner reports it.
class CatalogSource {
  CatalogSource(Map<String, dynamic> j)
      : id = '${j['id']}',
        name = '${j['name']}',
        root = '${j['root']}',
        enabled = j['enabled'] == true,
        available = j['available'] == true,
        assets = (j['assets'] as num?)?.toInt() ?? 0;
  final String id, name, root;
  final bool enabled, available;
  final int assets;
}

/// One asset of a result set, as the owner built it. The view only draws it (a still's own file, a clip's frame, a
/// sound's waveform, a model itself) and never decides what is in the set.
class CatalogEntry {
  CatalogEntry(this.json) : id = '${json['id']}';
  final Map<String, dynamic> json;
  final String id;
  String get name => '${json['name']}';
  String get path => '${json['path']}';
  String get kind => '${json['kind']}';
  String get faceKey => '${json['faceKey']}';
  bool get missing => json['missing'] == true;
}

/// The Media Catalog as the Flutter skin sees it: which sources, types and words the person chose, the result set the
/// owner (native, `catalog/`) built for them, and the faces asked for what is in view. Nothing here scans a folder, keeps an
/// index or filters an asset list: a change of choice is a new query, and the answer is the owner's.
class CatalogSession extends ChangeNotifier implements ResultSource {
  CatalogSession(this.c);
  final EditorSession c;

  // what the owner reports
  List<CatalogSource> sources = const [];
  int revision = -1, total = 0, pending = 0;
  List<CatalogEntry> entries = const [];
  Object? failure;

  // what the person chose (a query's inputs)
  Set<String>? chosenSources; // null = every enabled source
  Set<String> kinds = {}; // empty = every type
  String text = '';
  ({String source, String prefix})? folder;
  String sort = 'name';
  bool descending = false;

  /// The sub-folders directly under the chosen folder (the owner's answer), for folder browsing.
  List<({String name, int assets})> subfolders = const [];

  /// Faces by the entry's `faceKey` (it changes when the file does): {thumbnail, peaks, facts}.
  final faces = <String, Map<String, dynamic>>{};
  var _asked = <String>{};
  var _sequence = 0;
  bool _disposed = false;

  Future<Map<String, dynamic>> _call(Map<String, dynamic> command) async => EditorSession.map(await c.native('catalog', {'command': jsonEncode(command)}));

  void _took(Map<String, dynamic> reply) {
    if (reply['error'] != null) throw StateError('${reply['error']}');
    if (reply['sources'] is List) sources = [for (final s in reply['sources'] as List) CatalogSource(EditorSession.map(s))];
    if (reply['pending'] is num) pending = (reply['pending'] as num).toInt();
  }

  Future<void> load() => _guard(() async {
        _took(await _call({'op': 'sources'}));
        await _query();
      });

  Future<void> addSource(String path, {String? name}) => _guard(() async {
        _took(await _call({'op': 'addSource', 'path': path, if (name != null) 'name': name}));
        _took(await _call({'op': 'refresh'}));
        await _query();
      });

  Future<void> removeSource(String id) => _guard(() async {
        chosenSources?.remove(id);
        _took(await _call({'op': 'removeSource', 'id': id}));
        await _query();
      });

  Future<void> enableSource(String id, bool enabled) => _guard(() async {
        _took(await _call({'op': 'enableSource', 'id': id, 'enabled': enabled}));
        await _query();
      });

  Future<void> refresh() => _guard(() async {
        _took(await _call({'op': 'refresh'}));
        await _query();
      });

  /// The slow things (fingerprints, image sizes) a batch at a time, off the scan's path.
  Future<void> enrichSome() => _guard(() async {
        _took(await _call({'op': 'enrich', 'limit': 200}));
      }, requery: false);

  Future<void> choose({Set<String>? Function()? sources, Set<String>? kinds, String? text, ({String source, String prefix})? Function()? folder, String? sort, bool? descending}) => _guard(() async {
        if (sort != null) this.sort = sort;
        if (descending != null) this.descending = descending;
        if (sources != null) chosenSources = sources();
        if (kinds != null) this.kinds = kinds;
        if (text != null) this.text = text;
        if (folder != null) this.folder = folder();
        await _query();
      });

  Future<void> _guard(Future<void> Function() body, {bool requery = true}) async {
    try {
      failure = null;
      await body();
    } catch (e) {
      failure = e;
    }
    if (!_disposed) notifyListeners();
  }

  Future<void> _query() async {
    final mine = ++_sequence;
    final reply = await _call({
      'op': 'query',
      if (chosenSources != null) 'sources': chosenSources!.toList(),
      if (folder != null) 'folder': {'source': folder!.source, 'prefix': folder!.prefix},
      if (kinds.isNotEmpty) 'kinds': kinds.toList(),
      if (text.trim().isNotEmpty) 'text': text.trim(),
      'sort': sort,
      'descending': descending,
      'limit': 2000,
    });
    if (mine != _sequence) return; // a newer choice was made while this one was being answered
    await _folders();
    if (reply['error'] != null) throw StateError('${reply['error']}');
    revision = (reply['revision'] as num).toInt();
    total = (reply['total'] as num).toInt();
    entries = [for (final e in reply['entries'] as List) CatalogEntry(EditorSession.map(e))];
    unawaited(_askFaces(mine));
  }

  /// Folder browsing needs one source in view: its sub-folders under the folder chosen (none when several are).
  Future<void> _folders() async {
    final only = folder?.source ?? ((chosenSources?.length ?? 0) == 1 ? chosenSources!.first : (sources.where((s) => s.enabled).length == 1 ? sources.firstWhere((s) => s.enabled).id : null));
    if (only == null) {
      subfolders = const [];
      return;
    }
    final reply = await _call({'op': 'folders', 'source': only, 'prefix': folder?.prefix ?? ''});
    subfolders = [for (final f in (reply['folders'] as List? ?? const [])) (name: '${EditorSession.map(f)['name']}', assets: (EditorSession.map(f)['assets'] as num).toInt())];
  }

  /// The source folder browsing is in (the chosen folder's, else the only one chosen), if any.
  String? get folderSource => folder?.source ?? ((chosenSources?.length ?? 0) == 1 ? chosenSources!.first : (sources.where((s) => s.enabled).length == 1 ? sources.firstWhere((s) => s.enabled).id : null));

  /// Faces for what is in the result set that has none yet, a few at a time (the owner makes them from the files with
  /// the routines the shelf already uses; each is cached under the file's own key).
  Future<void> _askFaces(int mine) async {
    final todo = [for (final e in entries) if (!e.missing && !faces.containsKey(e.faceKey) && _asked.add(e.faceKey)) e];
    for (var i = 0; i < todo.length; i += 24) {
      if (mine != _sequence || _disposed) return;
      final batch = todo.skip(i).take(24).toList();
      try {
        final reply = await _call({'op': 'faces', 'ids': [for (final e in batch) e.id]});
        final got = EditorSession.map(reply['faces']);
        for (final e in batch) {
          faces[e.faceKey] = EditorSession.map(got[e.id]);
        }
        if (!_disposed) notifyListeners();
      } catch (_) {
        for (final e in batch) {
          _asked.remove(e.faceKey);
        }
      }
    }
  }

  @override
  List<BrowserItem> get items => [for (final e in entries) itemFor(e)];

  BrowserItem itemFor(CatalogEntry e) {
    final face = faces[e.faceKey] ?? const {};
    final facts = EditorSession.map(face['facts']);
    final j = e.json;
    num? n(Object? v) => v is num ? v : null;
    return BrowserItem(
      id: e.id,
      name: e.name,
      path: e.path,
      kind: e.kind,
      mime: '${j['mime']}',
      source: '${j['sourceName']}',
      rel: '${j['rel']}',
      size: n(j['size'])?.toInt(),
      mtimeNs: n(j['mtimeNs'])?.toInt(),
      width: n(j['width'] ?? facts['width'])?.toInt(),
      height: n(j['height'] ?? facts['height'])?.toInt(),
      seconds: n(facts['seconds'])?.toDouble(),
      sampleRate: n(facts['sampleRate'])?.toInt(),
      channels: n(facts['channels'])?.toInt(),
      faceKey: e.faceKey,
      missing: e.missing,
      thumbnail: face['thumbnail'] as String?,
      peaks: face['peaks'],
    );
  }

  /// A clip's frame at a time (a scrub in the preview): a data URI, or null.
  Future<String?> frameAt(String id, double seconds, {int edge = 480}) async {
    final reply = await _call({'op': 'frame', 'id': id, 'at': seconds, 'edge': edge});
    return reply['frame'] as String?;
  }

  /// A still at a size the preview shows it at: a data URI, or null.
  Future<String?> pictureOf(String id, {int edge = 1024}) async {
    final reply = await _call({'op': 'picture', 'id': id, 'edge': edge});
    return reply['picture'] as String?;
  }

  /// The item the Media shelf's own faces draw (the same map its imported assets have).
  Map<String, dynamic> itemOf(CatalogEntry e) {
    final face = faces[e.faceKey] ?? const {};
    final facts = EditorSession.map(face['facts']);
    final j = e.json;
    return {
      'id': e.id,
      'name': e.name,
      'path': e.path,
      'mime': j['mime'],
      'family': const {'image': '2D', 'video': 'Video', 'audio': 'Audio', 'model': '3D', 'environment': 'HDR'}[e.kind] ?? '2D',
      'facts': {if (j['width'] != null) 'width': j['width'], if (j['height'] != null) 'height': j['height'], ...facts},
      if (face['thumbnail'] != null) 'thumbnail': face['thumbnail'],
      if (face['peaks'] != null) 'peaks': face['peaks'],
      'missing': e.missing,
      'used': false,
    };
  }

  @override
  void dispose() {
    _disposed = true;
    super.dispose();
  }
}
