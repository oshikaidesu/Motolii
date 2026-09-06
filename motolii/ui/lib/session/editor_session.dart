import 'package:flutter/foundation.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';

import '../bridge/native_bridge.dart';
import '../bridge/protocol.dart';

class EditorSession {
  static const channel = NativeBridge.channel;
  final _bridge = NativeBridge();
  final document = ValueNotifier<Map<String, dynamic>>({});
  Map<String, dynamic> get state => document.value;
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

  final focusProperty = ValueNotifier<String?>(null);
  final keyedOnly = ValueNotifier<bool>(false);
  final viewCommand = ValueNotifier<String?>(null);
  Map<String, dynamic> windowInfo = {};
  final playing = ValueNotifier<bool>(false);
  final busy = ValueNotifier<bool>(false);
  final error = ValueNotifier<String?>(null);
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
    }
    if (next['playing'] is bool) {
      playing.value = next['playing'] as bool;
      if (!playing.value && _ticker != null) _cancelCadence();
    }
    if (notify) document.value = {...state, ...next};
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
  ]) => _bridge.request(operation, args);
  Future<void> _render({bool notify = true, bool playback = false}) async {
    final response = await native(
      'render',
      playback ? {'playing': true} : const {},
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
      _accept(await _request(operation, args));
      if (operation == DocumentOperation.select) {
        editingFocus.value = {'selection': true};
      }
      if (operation.requiresRender) await _render();
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
        } catch (_) {
          // Stop the authoritative transport as well as the request cadence.
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
    final picked = await native('pickImport');
    if (picked is List && picked.isNotEmpty)
      await command('import', {'paths': picked});
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
    document.dispose();
    textureId.dispose();
    frame.dispose();
    rendered.dispose();
    playing.dispose();
    busy.dispose();
    error.dispose();
    deskWork.dispose();
    deskDefault.dispose();
    deskDrawer.dispose();
    panePlaces.dispose();
    browserTab.dispose();
    focusProperty.dispose();
    editingFocus.dispose();
    keyedOnly.dispose();
    viewCommand.dispose();
  }
}
