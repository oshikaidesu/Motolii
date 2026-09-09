import 'package:flutter/foundation.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';

import '../bridge/native_bridge.dart';
import '../bridge/protocol.dart';

/// A slice of the status. Panels listen to the keys they read, so a drag that
/// moves `layers` leaves Fonts, History and the Browser alone. `derived` names
/// what a panel reads through a getter (the active layer, say) rather than a
/// key; it is compared by value.
class DocumentSlice extends ChangeNotifier
    implements ValueListenable<Map<String, dynamic>> {
  DocumentSlice._(this._session, this._keys, this._derived) {
    _last = _derived?.call();
  }
  final EditorSession _session;
  final Set<String> _keys;
  final Object? Function()? _derived;
  Object? _last;
  @override
  Map<String, dynamic> get value => _session.state;
  void _consider(Set<String> changed) {
    final now = _derived?.call();
    final moved = !sameValue(_last, now);
    _last = now;
    if (moved || _keys.any(changed.contains)) notifyListeners();
  }
}

/// Value equality over decoded JSON.
bool sameValue(Object? a, Object? b) {
  if (identical(a, b)) return true;
  if (a is List) {
    if (b is! List || a.length != b.length) return false;
    for (var i = 0; i < a.length; i++) {
      if (!sameValue(a[i], b[i])) return false;
    }
    return true;
  }
  if (a is Map) {
    if (b is! Map || a.length != b.length) return false;
    for (final entry in a.entries) {
      if (!b.containsKey(entry.key) || !sameValue(entry.value, b[entry.key]))
        return false;
    }
    return true;
  }
  return a == b;
}

class EditorSession {
  static const channel = NativeBridge.channel;
  final _bridge = NativeBridge();
  final document = ValueNotifier<Map<String, dynamic>>({});
  Map<String, dynamic> get state => document.value;
  final _slices = <String, DocumentSlice>{};
  Map<String, dynamic> _spread = const {};

  /// The status a panel reads, as its own listenable. Named so the panels that
  /// share a reading share one slice.
  DocumentSlice slice(
    String name,
    List<String> keys, {
    Object? Function()? derived,
  }) => _slices[name] ??= DocumentSlice._(this, keys.toSet(), derived);

  void _spreadDocument() {
    final was = _spread, now = state;
    _spread = now;
    if (_slices.isEmpty) return;
    final changed = <String>{};
    for (final key in was.keys) {
      if (!now.containsKey(key) || !sameValue(was[key], now[key]))
        changed.add(key);
    }
    for (final key in now.keys) {
      if (!was.containsKey(key)) changed.add(key);
    }
    for (final slice in _slices.values.toList()) {
      slice._consider(changed);
    }
  }

  /// Take in what actually moved. Keys that arrive unchanged keep the object
  /// they had, so a reply that says nothing new notifies nobody.
  void absorb(Map<String, dynamic> next) {
    final was = state;
    Map<String, dynamic>? merged;
    for (final entry in next.entries) {
      if (was.containsKey(entry.key) && sameValue(was[entry.key], entry.value))
        continue;
      (merged ??= {...was})[entry.key] = entry.value;
    }
    if (merged != null) document.value = merged;
  }
  final textureId = ValueNotifier<int?>(null);
  final frame = ValueNotifier<int>(0);
  final rendered = ValueNotifier<Map<String, dynamic>>({});
  Future<bool> Function()? confirmClose;
  void Function(Map<String, dynamic>)? windowClosed;
  void Function(List<String>)? filesDropped;
  bool Function(Map<String, dynamic>)? fileDropTarget;
  final pendingEditors = <Future<void> Function()>{};
  Future<void> flushEditors() async {
    for (final flush in pendingEditors.toList()) {
      await flush();
    }
  }

  final editingFocus = ValueNotifier<Map<String, dynamic>>({});
  void focusEditing(int layer, String property) {
    editingFocus.value = {'layer': layer, 'property': property};
  }

  final textStyleTarget = ValueNotifier<Map<String, dynamic>?>(null);

  final focusProperty = ValueNotifier<String?>(null);

  /// An anchor the pointer hovers in the Inspector, as a fraction of the
  /// layer's bounds; the Stage marks where the pivot would land.
  final anchorPreview = ValueNotifier<List<double>?>(null);
  final keyedOnly = ValueNotifier<bool>(false);
  final viewCommand = ValueNotifier<String?>(null);
  Map<String, dynamic> windowInfo = {};
  final playing = ValueNotifier<bool>(false);
  final busy = ValueNotifier<bool>(false);
  final error = ValueNotifier<String?>(null);

  /// 直前の取り込みで棚に入った asset の id。Browser が Media を開いて選ぶ。
  final importedAssets = ValueNotifier<List<String>>([]);

  /// While on, the next click on the Stage reads a colour instead of editing.
  final eyedropper = ValueNotifier<bool>(false);

  /// Timeline に見えているコマ数。尺の無い物を置く時の既定の長さの元(Rust が割合を決める)。
  final visibleFrames = ValueNotifier<int?>(null);
  final deskWork = ValueNotifier<Map<String, dynamic>>({});
  Future<void> _deskSave = Future.value();
  Future<void> storeDesk(String key, dynamic value) {
    deskWork.value = {...deskWork.value, key: value};
    return _deskSave = _deskSave.catchError((_) {}).then((_) async {
      final settings = map(await native('readSettings'));
      await native('writeSettings', {...settings, 'deskWork': deskWork.value});
    });
  }

  final panePlaces = ValueNotifier<Map<String, dynamic>>({});
  Future<void> Function(String, String)? panelPlacementRequested;
  Future<void> placePanel(String name, String placement) async {
    await native('placePanel', {'name': name, 'placement': placement});
  }

  final deskDefault = ValueNotifier<String>('Tools');
  final deskDrawer = ValueNotifier<String?>(null);
  final browserTab = ValueNotifier<String>('Create');
  Future<void> _tail = Future.value();
  Ticker? _ticker;
  bool _disposed = false, _playRequested = false, _pauseQueued = false;
  int _pendingWork = 0;
  int _generation = 0;
  String _path = const String.fromEnvironment('MOTOLII_DOCUMENT');
  List<Map<String, dynamic>> get layers => maps(state['layers']);

  /// Whether the last rendered frame describes the current frame of this document.
  bool get renderedIsFresh {
    final r = rendered.value;
    return r.isNotEmpty &&
        r['frame'] == frame.value &&
        (r['contentRevision'] == null || state['contentRevision'] == null || r['contentRevision'] == state['contentRevision']) &&
        (r['documentRevision'] == null ||
            r['documentRevision'] == state['documentRevision']);
  }

  /// Layers as of the last rendered frame. A playback frame carries only the
  /// values that move (liveLayers); they are laid over the document layers by id.
  List<Map<String, dynamic>> liveLayers() {
    if (!renderedIsFresh) return layers;
    final r = rendered.value;
    if (r['layers'] is List) return maps(r['layers']);
    final live = {for (final l in maps(r['liveLayers'])) l['id']: l};
    return [
      for (final l in layers)
        if (live[l['id']] case final o?) overlayLayer(l, o) else l,
    ];
  }

  static Map<String, dynamic> overlayLayer(
    Map<String, dynamic> base,
    Map<String, dynamic> live,
  ) {
    final out = {...base, ...live};
    out['properties'] = _overlayRows(
      maps(base['properties']),
      maps(live['properties']),
    );
    final liveEffects = {for (final e in maps(live['effects'])) e['id']: e};
    out['effects'] = [
      for (final e in maps(base['effects']))
        if (liveEffects[e['id']] case final o?)
          {...e, 'params': _overlayRows(maps(e['params']), maps(o['params']))}
        else
          e,
    ];
    return out;
  }

  static List<Map<String, dynamic>> _overlayRows(
    List<Map<String, dynamic>> base,
    List<Map<String, dynamic>> live,
  ) {
    final byId = {for (final r in live) r['id']: r};
    return [
      for (final r in base)
        if (byId[r['id']] case final o?) {...r, ...o} else r,
    ];
  }

  List<int> get selectedIds =>
      (state['selectedIds'] as List? ?? [state['selectedId']])
          .whereType<num>()
          .map((n) => n.toInt())
          .toList();
  Map<String, dynamic>? get activeLayer {
    if (selectedIds.isEmpty) return null;
    for (final l in layers) {
      if (l['id'] == selectedIds.last) return l;
    }
    return null;
  }

  bool supports(String op) =>
      (state['capabilities'] as List? ?? const []).contains(op);
  static Map<String, dynamic> map(dynamic v) =>
      v is Map ? Map<String, dynamic>.from(v) : {};
  static List<Map<String, dynamic>> maps(dynamic v) =>
      (v as List? ?? const []).map(map).toList();
  Future<dynamic> native(
    String method, [
    Map<String, dynamic> args = const {},
  ]) => _bridge.invoke(method, args);
  void _accept(dynamic reply, {bool notify = true}) {
    if (_disposed) return;
    final envelope = map(reply);
    final inner = map(envelope['status']);
    final next = inner.isEmpty ? envelope : inner;
    if (next['error'] != null && '${next['error']}'.isNotEmpty) {
      throw StateError('${next['error']}');
    }
    if (envelope['textureId'] is num)
      textureId.value = (envelope['textureId'] as num).toInt();
    if (envelope['frameReady'] == true) {
      if (next['frame'] is num) frame.value = (next['frame'] as num).toInt();
      rendered.value = next;
    } else if (next['needsRender'] == false &&
        next['contentRevision'] != null &&
        next['contentRevision'] == rendered.value['contentRevision'] &&
        next['frame'] == rendered.value['frame']) {
      rendered.value = {...rendered.value, ...next};
    }
    if (next['playing'] is bool) {
      playing.value = next['playing'] as bool;
      if (!playing.value && _ticker != null) _cancelCadence();
    }
    if (notify || next['layers'] is List) absorb(next);
  }

  Future<void> _serial(
    Future<void> Function() action, {
    bool displayBusy = true,
  }) {
    if (_disposed) return Future<void>.value();
    _pendingWork++;
    final work = _tail.then((_) async {
      try {
        if (_disposed) return;
        if (displayBusy) {
          busy.value = true;
          error.value = null;
        }
        await action();
      } catch (e) {
        if (!_disposed) error.value = '$e';
      } finally {
        _pendingWork--;
        if (!_disposed && displayBusy) busy.value = false;
      }
    });
    _tail = work;
    return work;
  }

  EditorSession() {
    document.addListener(_spreadDocument);
    _bridge.listen((call) async {
      if (_disposed) return call.method == 'confirmClose' ? true : null;
      if (call.method == 'confirmClose') {
        await flushEditors();
        return await confirmClose?.call() ?? true;
      }
      if (call.method == 'flushEditors') {
        await flushEditors();
        return true;
      }
      if (call.method == 'placePanel') {
        final args = map(call.arguments);
        await panelPlacementRequested?.call(
          '${args['name']}',
          '${args['placement']}',
        );
        return true;
      }
      if (call.method == 'paneState') {
        final state = map(call.arguments);
        panePlaces.value = map(state['places']);
        deskDrawer.value = state['drawer'] as String?;
      }
      if (call.method == 'documentChanged') {
        final envelope = map(call.arguments);
        try {
          _accept(envelope, notify: envelope['frameOnly'] != true);
        } catch (e) {
          if (!_disposed) error.value = '$e';
        }
      }
      if (call.method == 'documentClosed') {
        _cancelCadence();
        playing.value = false;
        textureId.value = null;
        rendered.value = {};
        document.value = {};
      }
      if (call.method == 'windowClosed')
        windowClosed?.call(map(call.arguments));
      if (call.method == 'filesDropped' &&
          fileDropTarget?.call(map(call.arguments)) != true)
        filesDropped?.call(
          (map(call.arguments)['paths'] as List? ?? [])
              .whereType<String>()
              .toList(),
        );
      return null;
    });
  }
  Future<void> initialize() async {
    final info = map(await native('windowInfo'));
    if (_disposed) return;
    windowInfo = info;
    panePlaces.value = map(map(info['paneState'])['places']);
    deskDrawer.value = map(info['paneState'])['drawer'] as String?;
    if (windowInfo['main'] == false) {
      _accept(await native('attach'));
    } else {
      try {
        _accept(await native('attach'));
      } on PlatformException catch (e) {
        if (e.message?.contains('Open a document first') != true) rethrow;
        await open();
        return;
      }
      // `motolii-ui.sh dev <document>`: the launch names a document and nothing is open yet.
      if (_path.isNotEmpty && '${state['path'] ?? ''}'.isEmpty) {
        await open();
        return;
      }
      await _render();
      if (playing.value) {
        _playRequested = true;
        _beginCadence(++_generation);
      }
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

  Future<dynamic> _request(
    DocumentOperation operation, [
    Map<String, dynamic> args = const {},
  ]) => _bridge.request(operation, args, {
    'knownSnapshotId': state['snapshotId'],
    'knownReferenceId': state['referenceId'],
    'deferSnapshot': operation != DocumentOperation.play && operation != DocumentOperation.pause,
  });
  Future<void> _render({bool notify = true, bool playback = false}) async {
    final response = await native(
      'render',
      {'playing': playback, 'knownSnapshotId': state['snapshotId'], 'knownReferenceId': state['referenceId']},
    );
    _accept(response, notify: notify);
  }

  Future<void> refreshPreview() => _serial(() => _render(), displayBusy: false);

  Future<void> command(String op, [Map<String, dynamic> args = const {}]) {
    if (_disposed) return Future<void>.value();
    if (op == 'save') {
      return native('flushEditors').then((_) => _command(op, args));
    }
    return _command(op, args);
  }

  Future<void> _command(String op, Map<String, dynamic> args) {
    if ((op == 'create' || op == 'placeAsset') &&
        !args.containsKey('visibleFrames') &&
        visibleFrames.value != null)
      args = {...args, 'visibleFrames': visibleFrames.value};
    final DocumentOperation operation;
    try {
      operation = DocumentOperation.parse(op);
      DocumentOperation.validateArguments(args);
    } catch (e) {
      error.value = '$e';
      return Future<void>.value();
    }
    if (operation == DocumentOperation.play) {
      _startPlayback();
      return _tail;
    }
    if (operation == DocumentOperation.pause) {
      stopPlayback();
      return _tail;
    }
    final requiresPause = operation.requiresPause;
    if (requiresPause) _schedulePause(renderFinal: false);
    return _serial(() async {
      if (requiresPause && playing.value)
        throw StateError('Playback did not stop before $op');
      final response = map(await _request(operation, args));
      final needsRender = response['needsRender'] as bool? ??
          operation.requiresRender;
      // 状態を持たない返信は 2 つだけ — 繰り延べた {"needsRender":true} と
      // quiet な seek/tick の {"ok":true}。それ以外は必ず取り込む。
      final stateless = response.length == 1 &&
          (response['needsRender'] == true || response['ok'] == true);
      if (!stateless) _accept(response);
      if (operation == DocumentOperation.select) {
        editingFocus.value = {'selection': true};
      }
      if (needsRender) await _render();
    });
  }

  void seek(int value) {
    command('seek', {'frame': value});
  }

  void cancelPreview() {
    command('cancelPreview');
  }

  void togglePlayback() {
    if (_disposed) return;
    if (_pauseQueued && !_playRequested) {
      _startPlayback();
    } else if (playing.value || _playRequested) {
      stopPlayback();
    } else {
      _startPlayback();
    }
  }

  void _startPlayback() {
    if (_disposed || !supports('play') || textureId.value == null) return;
    if ((state['durationFrames'] as num? ?? 0) < 2) return;
    final generation = ++_generation;
    _playRequested = true;
    _serial(() async {
      if (generation != _generation) return;
      try {
        _accept(await _request(DocumentOperation.play));
      } catch (_) {
        if (generation == _generation) _cancelCadence();
        rethrow;
      }
      if (_disposed || generation != _generation) return;
      if (!playing.value) {
        _playRequested = false;
        return;
      }
      _beginCadence(generation);
    });
  }

  void _beginCadence(int generation) {
    _ticker?.dispose();
    _ticker = Ticker((_) {
      if (_disposed || generation != _generation || _pendingWork > 0) return;
      _serial(() async {
        if (_disposed || generation != _generation) return;
        try {
          await _render(notify: false, playback: true);
        } catch (e) {
          // Stop the authoritative transport as well as the request cadence.
          debugPrint('PROBE room=playback verdict=cadence-stopped reason=$e');
          _schedulePause(renderFinal: false);
          rethrow;
        }
      }, displayBusy: false);
    })..start();
  }

  void _cancelCadence() {
    _generation++;
    _ticker?.dispose();
    _ticker = null;
    _playRequested = false;
  }

  void _schedulePause({required bool renderFinal}) {
    if (_disposed) return;
    final needed = _playRequested || playing.value || _ticker != null;
    _cancelCadence();
    if (!needed || _pauseQueued || !supports('pause')) return;
    _pauseQueued = true;
    _serial(() async {
      try {
        _accept(await _request(DocumentOperation.pause));
        if (renderFinal && !_disposed) await _render(notify: false);
      } finally {
        _pauseQueued = false;
      }
    }, displayBusy: false);
  }

  void stopPlayback() {
    _schedulePause(renderFinal: true);
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

  void dispose() {
    if (_disposed) return;
    final ownedPlayback = _playRequested || _ticker != null;
    _cancelCadence();
    _disposed = true;
    _bridge.listen(null);
    // Let an already-running native call retire before releasing its document.
    _tail.then((_) async {
      if (ownedPlayback) {
        try {
          await _request(DocumentOperation.pause);
        } catch (_) {}
      }
      try {
        await native('close');
      } catch (_) {}
    });
    for (final slice in _slices.values) {
      slice.dispose();
    }
    _slices.clear();
    document.removeListener(_spreadDocument);
    document.dispose();
    textureId.dispose();
    frame.dispose();
    rendered.dispose();
    playing.dispose();
    busy.dispose();
    error.dispose();
    importedAssets.dispose();
    visibleFrames.dispose();
    deskWork.dispose();
    eyedropper.dispose();
    deskDefault.dispose();
    deskDrawer.dispose();
    panePlaces.dispose();
    anchorPreview.dispose();
    browserTab.dispose();
    textStyleTarget.dispose();
    focusProperty.dispose();
    editingFocus.dispose();
    keyedOnly.dispose();
    viewCommand.dispose();
  }
}
