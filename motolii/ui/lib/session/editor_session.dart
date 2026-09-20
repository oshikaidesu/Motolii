import 'dart:async';
import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';

import '../bridge/native_bridge.dart';
import '../bridge/native_frames.dart';
import '../bridge/protocol.dart';

String effectsNotice(Map<String, dynamic> status) {
  final errors = (status['catalogErrors'] as List? ?? const [])
      .map((e) => '$e')
      .where((e) => e.isNotEmpty)
      .toList();
  return errors.isEmpty ? '' : 'Effects: ${errors.join('; ')}';
}

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
  int? _attachmentId;
  final _runtimeEpoch = ValueNotifier<int>(0);
  ValueListenable<int> get runtimeEpoch => _runtimeEpoch;
  final document = ValueNotifier<Map<String, dynamic>>({});
  Map<String, dynamic> get state => document.value;
  bool get animating => state['animate'] == true;

  /// On by default: the frame Animate was turned on at becomes the first key
  /// of anything touched later at another frame. Settings can turn it off.
  bool get animateFrom => deskWork.value['animateFrom'] != false;

  /// Settings: the projection new material is born with. 2.5D faces the
  /// camera wherever it sits; 3D stands in the world and turns with it.
  String get flatProjection =>
      deskWork.value['flatProjection'] == '3D' ? '3D' : '2.5D';
  String? _sentFlatProjection = '2.5D';
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

  /// The shape a newborn key gets; Easy Ease until the Ease desk says otherwise.
  static const easyEase = {
    'kind': 'Bezier',
    'x1': 0.42,
    'y1': 0.0,
    'x2': 0.58,
    'y2': 1.0,
  };
  Map<String, dynamic> get newKeyShape {
    final chosen = map(deskWork.value['newKeyShape']);
    return chosen.isEmpty ? Map.of(easyEase) : chosen;
  }

  Future<void> setAnimate(bool on) => command('animate', {
    'enabled': on,
    'from': animateFrom,
    'shape': newKeyShape,
  });

  /// Point the Fonts shelf at a text layer: the Inspector's font value is the
  /// name, choosing is the shelf's job (the colour swatch and wheel, likewise).
  Future<void> focusFont(Map<String, dynamic> layer) async {
    final held = textStyleTarget.value;
    textStyleTarget.value = {
      if (held != null && held['layer'] == layer['id']) ...held,
      'layer': layer['id'],
      'scope': held != null && held['layer'] == layer['id']
          ? held['scope'] ?? 'all'
          : 'all',
    };
    browserTab.value = 'Fonts';
    await placePanel('Fonts', 'show');
  }

  Future<void> focusColor(Map<String, dynamic> args) async {
    await command('focusColor', args);
    browserTab.value = 'Colors';
    await placePanel('Colors', 'show');
  }

  /// The key selection before the current one, with the layers it sits on,
  /// so a shortcut can bring a motion back without a trip to the Timeline.
  Map<String, dynamic>? previousKeys;
  Future<void> reselectKeys() async {
    final back = previousKeys;
    if (back == null) return;
    await command('select', back);
  }

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
  static void take(
    ValueNotifier<Map<String, dynamic>> held,
    Map<String, dynamic> next,
  ) {
    final was = held.value;
    Map<String, dynamic>? merged;
    for (final entry in next.entries) {
      if (was.containsKey(entry.key) && sameValue(was[entry.key], entry.value))
        continue;
      (merged ??= {...was})[entry.key] = entry.value;
    }
    if (merged != null) held.value = merged;
  }

  void absorb(Map<String, dynamic> next) {
    final keys = maps(state['selectedKeys']);
    if (keys.isNotEmpty &&
        next['selectedKeys'] is List &&
        !sameValue(state['selectedKeys'], next['selectedKeys'])) {
      previousKeys = {'ids': selectedIds, 'keys': keys};
    }
    take(document, next);
  }

  /// The output picture (Camera view). Playback gates on it.
  final textureId = ValueNotifier<int?>(null);

  /// One Flutter texture per live view: `Camera` (the output) and `User` (the Stage).
  final textureIds = ValueNotifier<Map<String, int>>({});
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

  /// Files are being carried over this window; the shelves can say "drop here".
  final dragging = ValueNotifier<bool>(false);

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
        (r['contentRevision'] == null ||
            state['contentRevision'] == null ||
            r['contentRevision'] == state['contentRevision']) &&
        (r['documentRevision'] == null ||
            r['documentRevision'] == state['documentRevision']);
  }

  /// Layers as of the last rendered frame. A playback frame carries only the
  /// values that move (liveLayers); they are laid over the document layers by id.
  List<Map<String, dynamic>> liveLayers() {
    if (!renderedIsFresh) return layers;
    final r = rendered.value, s = state;
    if (identical(r, _liveFromRendered) && identical(s, _liveFromState))
      return _liveCache!;
    _liveFromRendered = r;
    _liveFromState = s;
    if (r['layers'] is List) return _liveCache = maps(r['layers']);
    final live = {for (final l in maps(r['liveLayers'])) l['id']: l};
    return _liveCache = [
      for (final l in layers)
        if (live[l['id']] case final o?) overlayLayer(l, o) else l,
    ];
  }

  /// One overlay per (document, frame): Stage and Inspector read the same list.
  List<Map<String, dynamic>>? _liveCache;
  Map<String, dynamic>? _liveFromRendered, _liveFromState;

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

  /// A reply, with every map and every list of maps typed, once. After this
  /// [map] and [maps] hand back the object they are given. "Typed" is the
  /// exact runtime type this makes: a narrower literal (a test's
  /// `List<Map<String, Object>>`) is copied, so a reader's `orElse` fits.
  static Object? typed(Object? v) {
    if (v is Map) {
      if (_typedMap(v)) return v;
      return <String, dynamic>{
        for (final e in v.entries) '${e.key}': typed(e.value),
      };
    }
    if (v is List) {
      if (v.isNotEmpty && v.every((e) => e is Map)) {
        if (v.runtimeType == _mapsType && v.every(_typedMap)) return v;
        return <Map<String, dynamic>>[
          for (final e in v) typed(e) as Map<String, dynamic>,
        ];
      }
      if (v.runtimeType == _listType && v.every(_typedLeaf)) return v;
      return <dynamic>[for (final e in v) typed(e)];
    }
    return v;
  }

  static final _mapType = <String, dynamic>{}.runtimeType;
  static final _mapsType = <Map<String, dynamic>>[].runtimeType;
  static final _listType = <dynamic>[].runtimeType;
  static bool _typedMap(Object? v) =>
      v is Map && v.runtimeType == _mapType && v.values.every(_typedLeaf);
  static bool _typedLeaf(Object? v) {
    if (v is Map) return _typedMap(v);
    if (v is List) {
      if (v.runtimeType == _mapsType) return v.every(_typedMap);
      return v.runtimeType == _listType && v.every(_typedLeaf);
    }
    return true;
  }

  /// Views, not copies: a typed map or list comes back as is. Callers that
  /// change what they get copy first (`.toList()`, `{...}`).
  static Map<String, dynamic> map(dynamic v) {
    if (v is Map<String, dynamic> && v.runtimeType == _mapType) return v;
    return v is Map ? Map<String, dynamic>.from(v) : {};
  }

  static List<Map<String, dynamic>> maps(dynamic v) {
    if (v is List<Map<String, dynamic>> && v.runtimeType == _mapsType) return v;
    if (v is! List) return const [];
    return [for (final e in v) map(e)];
  }

  Future<dynamic> native(
    String method, [
    Map<String, dynamic> args = const {},
  ]) async {
    final connects = method == 'attach' || method == 'open';
    final reply = await _bridge.invoke(method, {
      ...args,
      if (connects && _attachmentId != null) 'attachmentId': _attachmentId,
    });
    if (connects) {
      final attachment = map(reply)['attachmentId'];
      if (attachment is num) {
        if (_disposed) {
          try {
            await _bridge.invoke('detach', {
              'attachmentId': attachment.toInt(),
            });
          } catch (_) {}
        } else {
          _attachmentId = attachment.toInt();
        }
      }
    }
    if (method == 'openPanelWindow') _panelWindows++;
    return reply;
  }

  /// The same-frame path, once the host has handed over its runtime. Null in
  /// tests and while no document is open: everything then goes by channel.
  FfiFrames? _frames;
  bool get sameFrame => _frames != null;

  /// Panel windows the host shows besides this one; they read the document
  /// through the host's broadcast, which this window feeds after each reply.
  int _panelWindows = 0;

  /// The IOSurfaces each view is drawn into (two, taken in turn, so the
  /// raster of one frame never reads the surface the next is drawn into)
  /// and the size they were made for.
  final _surfaces = <String, ({int width, int height, List<int> ids})>{};
  int _flip = 0;

  void _bindFrames(Map<String, dynamic> envelope) {
    final library = envelope['library'], context = envelope['context'];
    if (library is! String || context is! int || windowInfo['main'] == false)
      return;
    try {
      final held = _frames;
      if (held == null) {
        _frames = FfiFrames(library, context);
      } else if (held.context != context) {
        held.context = context;
        _surfaces.clear();
      }
    } catch (e) {
      debugPrint('PROBE room=bridge verdict=channel-fallback reason=$e');
      _frames = null;
    }
  }

  /// Replies that arrive while a frame is being built wait until it is done:
  /// a build may not mark widgets outside its own subtree.
  List<void Function()>? _deferred;
  void _flushDeferred() {
    final held = _deferred;
    _deferred = null;
    if (held == null) return;
    for (final apply in held) {
      try {
        apply();
      } catch (e) {
        if (!_disposed) error.value = '$e';
      }
    }
  }

  void _accept(dynamic reply, {bool notify = true}) {
    if (_disposed) return;
    if (_deferred case final held?) {
      held.add(() => _accept(reply, notify: notify));
      return;
    }
    final envelope = map(typed(reply));
    final epoch = envelope['runtimeEpoch'];
    if (epoch is num && epoch.toInt() != _runtimeEpoch.value) {
      _surfaces.clear();
      rendered.value = {};
      _runtimeEpoch.value = epoch.toInt();
      if (windowInfo['main'] != false) {
        _sentFlatProjection = null;
        _syncPreferences();
      }
    }
    _bindFrames(envelope);
    final inner = map(envelope['status']);
    final next = inner.isEmpty ? envelope : inner;
    if (next['error'] != null && '${next['error']}'.isNotEmpty) {
      throw StateError('${next['error']}');
    }
    if (envelope['textureId'] is num)
      textureId.value = (envelope['textureId'] as num).toInt();
    if (envelope['textureIds'] is Map) {
      final ids = {
        for (final e in (envelope['textureIds'] as Map).entries)
          if (e.value is num) '${e.key}': (e.value as num).toInt(),
      };
      if (!sameValue(ids, textureIds.value)) textureIds.value = ids;
    }
    if (envelope['frameReady'] == true) {
      if (next['frame'] is num) frame.value = (next['frame'] as num).toInt();
      rendered.value = next;
    } else if (next['needsRender'] == false &&
        next['contentRevision'] != null &&
        next['contentRevision'] == rendered.value['contentRevision'] &&
        next['frame'] == rendered.value['frame']) {
      take(rendered, next);
    }
    if (next['playing'] is bool) {
      playing.value = next['playing'] as bool;
      if (!playing.value && _ticker != null) _cancelCadence();
      if (playing.value &&
          _ticker == null &&
          textureId.value != null &&
          windowInfo['main'] != false) {
        _playRequested = true;
        _beginCadence(++_generation);
      }
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
    deskWork.addListener(_syncPreferences);
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
        if (state.containsKey('theme') &&
            !sameValue(deskWork.value['theme'], state['theme'])) {
          deskWork.value = {...deskWork.value, 'theme': typed(state['theme'])};
        }
      }
      if (call.method == 'documentChanged') {
        final envelope = map(call.arguments);
        try {
          _accept(envelope, notify: envelope['frameOnly'] != true);
        } catch (e) {
          if (!_disposed) error.value = '$e';
        }
        // vism/ の file が変わった。絵は main の窓が 1 回だけ描き直す(他の窓は絵を受け取る側)。
        if (envelope['effectsReloaded'] == true &&
            map(envelope['status'])['needsRender'] == true &&
            windowInfo['main'] != false) {
          refreshPreview();
        }
      }
      // vism/ の file が変わった(host の見張り)。棚を読み直すのは main の窓の仕事。
      if (call.method == 'effectsChanged' && windowInfo['main'] != false) {
        command('reloadEffects');
      }
      if (call.method == 'documentClosed') {
        _cancelCadence();
        playing.value = false;
        textureId.value = null;
        rendered.value = {};
        document.value = {};
        _surfaces.clear();
        _frames?.dispose();
        _frames = null;
      }
      if (call.method == 'windowClosed') {
        if (_panelWindows > 0) _panelWindows--;
        windowClosed?.call(map(call.arguments));
      }
      if (call.method == 'dragHover')
        dragging.value = map(call.arguments)['active'] == true;
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
    _panelWindows = (info['panelWindows'] as num?)?.toInt() ?? 0;
    panePlaces.value = map(map(info['paneState'])['places']);
    deskDrawer.value = map(info['paneState'])['drawer'] as String?;
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

  Map<String, dynamic> _snapshotContext(DocumentOperation operation) => {
    'knownSnapshotId': state['snapshotId'],
    'knownReferenceId': state['referenceId'],
    'deferSnapshot':
        operation != DocumentOperation.play &&
        operation != DocumentOperation.pause,
  };

  Future<dynamic> _request(
    DocumentOperation operation, [
    Map<String, dynamic> args = const {},
  ]) {
    if (_frames case final frames?) {
      return Future.value(_requestNow(frames, operation, args));
    }
    return _bridge.request(operation, args, _snapshotContext(operation));
  }

  /// A specimen or a measurement answers this window alone; anything else
  /// may have moved the document and is worth telling the other windows.
  static const _askedAlone = {'visualSample', 'renderInfo', 'easeModel'};

  Map<String, dynamic> _requestNow(
    FfiFrames frames,
    DocumentOperation operation,
    Map<String, dynamic> args,
  ) {
    final raw = frames.request(
      operation.encode({...args, ..._snapshotContext(operation)}),
    );
    final reply = map(typed(jsonDecode(raw)));
    if (!_askedAlone.contains(operation.wireName) && reply['error'] == null) {
      _broadcast(raw);
    }
    return reply;
  }

  void _broadcast(
    String status, {
    bool frameReady = false,
    bool frameOnly = false,
  }) {
    if (_panelWindows <= 0 || _disposed) return;
    _bridge
        .invoke('broadcast', {
          'status': status,
          'frameReady': frameReady,
          'frameOnly': frameOnly,
        })
        .catchError((_) => null);
  }

  Future<void> _render({bool notify = true, bool playback = false}) async {
    if (_frames case final frames?) {
      _renderNow(frames, notify: notify, playback: playback);
      return;
    }
    final response = await native('render', {
      'playing': playback,
      'knownSnapshotId': state['snapshotId'],
      'knownReferenceId': state['referenceId'],
    });
    _accept(response, notify: notify);
  }

  /// Draw every view the runtime lists into its surface and take the status,
  /// all before returning. A view whose surface is missing or the wrong size
  /// is skipped this time: the host makes one (a channel round trip, once per
  /// size) and the render repeats. Returns whether every view was drawn.
  bool _renderNow(
    FfiFrames frames, {
    bool notify = true,
    bool playback = false,
  }) {
    if (playback) {
      final tick = map(
        jsonDecode(frames.request('{"op":"tick","quiet":true}')),
      );
      // The clock lands on whole frames, so most display frames ask for the
      // picture that is already on screen. Nothing to draw: give the thread back.
      if (tick['needsRender'] == false) return true;
    }
    final info = map(jsonDecode(frames.request('{"op":"renderInfo"}')));
    if (info['error'] != null) throw StateError('${info['error']}');
    final listed = maps(info['views']);
    if (listed.isEmpty) throw StateError('Document dimensions missing');
    final views = _shownViews.isEmpty
        ? listed
        : [
            for (final v in listed)
              if (_shownViews.contains('${v['view']}')) v,
          ];
    if (views.isEmpty) return true;
    var drawn = false, missing = false;
    _flip ^= 1;
    for (final view in views) {
      final name = '${view['view']}';
      final width = (view['width'] as num).toInt(),
          height = (view['height'] as num).toInt();
      final have = _surfaces[name];
      if (have == null || have.width != width || have.height != height) {
        missing = true;
        continue;
      }
      // One render costs 0.18 ms of CPU, so the GPU is waited for right here:
      // the picture, its window and the status all belong to this one frame.
      // The pair of surfaces per view still keeps the compositor off the one
      // being written.
      final surface = have.ids[_flip % have.ids.length];
      final code = frames.render(surface, name);
      if (code < 0) {
        final status = map(jsonDecode(frames.request('{"op":"status"}')));
        throw StateError('Rust render failed for $name: ${status['error']}');
      }
      drawn = true;
    }
    final raw = frames.request(
      jsonEncode({
        'op': 'status',
        'knownSnapshotId': state['snapshotId'],
        'knownReferenceId': state['referenceId'],
      }),
    );
    _accept({
      'status': jsonDecode(raw),
      'frameReady': drawn,
      'frameOnly': playback,
    }, notify: notify);
    _broadcast(raw, frameReady: drawn, frameOnly: playback);
    if (missing) {
      _serial(() async {
        await _ensureSurfaces(views);
        if (_frames case final frames?) _renderNow(frames, notify: notify);
      }, displayBusy: false);
    }
    return !missing;
  }

  /// The host makes (or keeps) the surfaces and textures for these views and
  /// drops the ones no longer listed.
  Future<void> _ensureSurfaces(List<Map<String, dynamic>> views) async {
    final reply = map(await native('ensureSurfaces', {'views': views}));
    if (_disposed) return;
    _surfaces.clear();
    for (final entry in map(reply['surfaces']).entries) {
      final surface = map(entry.value);
      _surfaces[entry.key] = (
        width: (surface['width'] as num).toInt(),
        height: (surface['height'] as num).toInt(),
        ids: [for (final id in surface['ids'] as List) (id as num).toInt()],
      );
    }
    _accept(reply);
  }

  /// The same-frame path for a view change: the request, its render and the
  /// status all complete before this frame is built, so the picture the
  /// raster shows is the one asked for. The status reaches the notifiers
  /// after the frame (a build may not mark widgets outside its own subtree).
  /// Returns true when every view was drawn with the request applied; false
  /// means the request went the ordinary way and the picture follows later.
  bool commandNow(String op, [Map<String, dynamic> args = const {}]) {
    final frames = _frames;
    if (frames == null || _disposed || _pendingWork > 0 || _deferred != null) {
      command(op, args);
      return false;
    }
    final DocumentOperation operation;
    try {
      operation = DocumentOperation.parse(op);
      DocumentOperation.validateArguments(args);
    } catch (e) {
      error.value = '$e';
      return false;
    }
    _deferred = [];
    scheduleMicrotask(_flushDeferred);
    try {
      final response = _requestNow(frames, operation, args);
      final needsRender =
          response['needsRender'] as bool? ?? operation.requiresRender;
      final stateless =
          response['ok'] == true ||
          (response.length == 1 && response['needsRender'] == true);
      if (!stateless) _accept(response);
      if (!needsRender) return false;
      return _renderNow(frames);
    } catch (e) {
      _deferred?.add(() => throw e);
      return false;
    }
  }

  Future<void> refreshPreview() => _serial(() => _render(), displayBusy: false);

  /// The views a tab is looking at. A hidden tab's picture is not drawn: the
  /// Stage tab withdraws its window, and every other view (Camera) is skipped
  /// here. Empty means nobody has said yet — draw them all, as before.
  final _shownViews = <String>{};

  /// A Stage tab that comes into view asks for its own texture.
  Future<void> attachView(String view) {
    _shownViews.add(view);
    return _serial(() async => _accept(await native('attach', {'view': view})));
  }

  /// A tab that leaves view (or is disposed) stops asking for its picture.
  void detachView(String view) => _shownViews.remove(view);

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
      final needsRender =
          response['needsRender'] as bool? ?? operation.requiresRender;
      // 状態を持たない返信は 2 つだけ — 繰り延べた {"needsRender":true} と
      // quiet な seek/tick の {"ok":true}。それ以外は必ず取り込む。
      final stateless =
          response['ok'] == true ||
          (response.length == 1 && response['needsRender'] == true);
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
    });
  }

  void _beginCadence(int generation) {
    if (_disposed || windowInfo['main'] == false) return;
    _ticker?.dispose();
    _ticker = Ticker((_) {
      if (_disposed || generation != _generation || _pendingWork > 0) return;
      // Same frame: the tick's picture and status are in before this frame builds.
      if (_frames case final frames?) {
        try {
          _renderNow(frames, notify: false, playback: true);
        } catch (e) {
          debugPrint('PROBE room=playback verdict=cadence-stopped reason=$e');
          _schedulePause(renderFinal: false);
          error.value = '$e';
        }
        return;
      }
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

  void dispose() {
    if (_disposed) return;
    final attachmentId = _attachmentId;
    _cancelCadence();
    _disposed = true;
    _bridge.listen(null);
    // Retire this attachment only; the application owns the document and clock.
    _tail.then((_) async {
      try {
        if (attachmentId != null) {
          await _bridge.invoke('detach', {'attachmentId': attachmentId});
        }
      } catch (_) {}
      _frames?.dispose();
      _frames = null;
    });
    for (final slice in _slices.values) {
      slice.dispose();
    }
    _slices.clear();
    document.removeListener(_spreadDocument);
    document.dispose();
    _runtimeEpoch.dispose();
    textureId.dispose();
    textureIds.dispose();
    frame.dispose();
    rendered.dispose();
    playing.dispose();
    busy.dispose();
    dragging.dispose();
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
