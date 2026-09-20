part of 'editor_session.dart';

/// file の仕事: opening, saving, importing and the script, plus the desk
/// settings this window keeps next to the document.
mixin SessionFiles on SessionCore {
  String _path = const String.fromEnvironment('MOTOLII_DOCUMENT');
  Future<void> _deskSave = Future.value();

  Future<void> initialize() async {
    final info = EditorSession.map(await native('windowInfo'));
    if (_disposed) return;
    windowInfo = info;
    _panelWindows = (info['panelWindows'] as num?)?.toInt() ?? 0;
    panePlaces.value = EditorSession.map(
      EditorSession.map(info['paneState'])['places'],
    );
    deskDrawer.value =
        EditorSession.map(info['paneState'])['drawer'] as String?;
    if (windowInfo['main'] == false) {
      _accept(await native('attach'));
    } else {
      try {
        _accept(await native('attach'));
      } on PlatformException catch (e) {
        if (_disposed) return;
        if (e.message?.contains('Open a document first') != true) rethrow;
        await open();
        return;
      }
      if (_disposed) return;
      // `motolii-ui.sh dev <document>`: the launch names a document and nothing is open yet.
      if (_path.isNotEmpty && '${state['path'] ?? ''}'.isEmpty) {
        await open();
        return;
      }
      await _render();
    }
  }

  Future<void> open([String? path]) {
    _schedulePause(renderFinal: false);
    if (path != null) _path = path;
    return _serial(() async {
      _accept(await native('open', {'path': _path}));
      await _render();
    });
  }

  Future<void> chooseOpen() async {
    final picked = await native('pickOpen');
    if (picked is String && picked.isNotEmpty) await open(picked);
  }

  Future<void> save({bool as = false}) async {
    String? path = as ? null : state['path'] as String?;
    path ??= await native('pickSave', {'name': 'Untitled.rrd'}) as String?;
    if (path != null) await command('save', {'path': path});
  }

  /// A script lays out window operations; its names are the window's names.
  Future<void> runScript() async {
    final picked = await native('pickImport', {
      'extensions': ['js'],
    });
    if (picked is List && picked.isNotEmpty) {
      await command('runScript', {'path': '${picked.first}'});
    }
  }

  Future<void> rerunScript() => command('rerunScript');

  Future<void> importFiles() async {
    final picked = await native('pickImport', {
      'extensions': state['importExtensions'] ?? const [],
    });
    if (picked is List) await importPaths(picked.whereType<String>().toList());
  }

  /// 落とした物も選んだ物もここへ来る。門(拡張子)は Rust の一覧 1 本を読み、
  /// 通らない物は名前を出して断る。フォルダは Rust 側が読める物だけ拾う。
  Future<void> importPaths(List<String> paths) async {
    final allowed = (state['importExtensions'] as List? ?? const [])
        .map((e) => '$e'.toLowerCase())
        .toSet();
    bool admissible(String path) {
      if (allowed.isEmpty) return true;
      final name = path.split('/').last;
      final dot = name.lastIndexOf('.');
      if (dot <= 0) return true;
      return allowed.contains(name.substring(dot + 1).toLowerCase());
    }

    Set<String> assetIds() => (state['assets'] as List? ?? const [])
        .whereType<Map>()
        .map((a) => '${a['id']}')
        .toSet();
    final accepted = paths.where(admissible).toList();
    final skipped = paths.where((p) => !admissible(p)).toList();
    final before = assetIds();
    if (accepted.isNotEmpty) await command('import', {'paths': accepted});
    final fresh = assetIds().difference(before).toList();
    if (fresh.isNotEmpty) importedAssets.value = fresh;
    if (skipped.isNotEmpty) {
      final names = skipped.map((p) => p.split('/').last).join(', ');
      error.value = 'Not supported: $names';
    }
  }

  void restoreDeskWork(Map<String, dynamic> settings) {
    _sentFlatProjection = null;
    deskWork.value = settings;
    _syncPreferences();
  }

  void _syncPreferences() {
    if (_sentFlatProjection == flatProjection) return;
    _sentFlatProjection = flatProjection;
    command('preferences', {'flatProjection': flatProjection});
  }

  Future<void> storeDesk(String key, dynamic value) {
    deskWork.value = {...deskWork.value, key: value};
    return _deskSave = _deskSave.catchError((_) {}).then((_) async {
      final settings = EditorSession.map(await native('readSettings'));
      await native('writeSettings', {...settings, 'deskWork': deskWork.value});
    });
  }

  Future<void> placePanel(String name, String placement) async {
    await native('placePanel', {'name': name, 'placement': placement});
  }
}
