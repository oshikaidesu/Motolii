part of 'editor_session.dart';

/// 描かせる／再生の拍: the surfaces each view is drawn into, the render that
/// answers one frame, and the transport that asks for the next one.
mixin SessionRender on SessionCore {
  /// The IOSurfaces each view is drawn into (two, taken in turn, so the
  /// raster of one frame never reads the surface the next is drawn into)
  /// and the size they were made for.
  final _surfaces = <String, ({int width, int height, List<int> ids})>{};
  int _flip = 0;
  Ticker? _ticker;

  void _clearSurfaces() => _surfaces.clear();
  bool get _cadenceRunning => _ticker != null;

  /// 計器(`kDebugMode` の裏、再生の道だけ)。名前を付けた区間を Dart の Timeline に
  /// 載せる: Flutter 自身の `Frame` / `Animate` / `BUILD` / `LAYOUT` / `PAINT` と
  /// 同じ流れなので、VM service の `getVMTimeline` が 1 回でまとめて渡す。
  /// ここを自前で測るのは、FFI の先(Rust と GPU)が Flutter のどの計器にも映らないため。
  /// 段ごとの取り分。Timeline は DevTools を開かないと読めないので、同じ計測を
  /// 足し込んで、再生が止まった時に 1 行で出す(毎コマは出さない)。
  static final Map<String, int> _spanUs = {};

  static T _span<T>(String name, T Function() body) {
    if (!kDebugMode) return body();
    final at = Stopwatch()..start();
    try {
      return Timeline.timeSync(name, body);
    } finally {
      _spanUs[name] = (_spanUs[name] ?? 0) + at.elapsedMicroseconds;
    }
  }

  Future<void> _render({bool notify = true, bool playback = false}) async {
    if (_frames case final frames? when !playing.value) {
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
      final tick = EditorSession.map(
        _span(
          'motolii.tick',
          () => jsonDecode(frames.request('{"op":"tick","quiet":true}')),
        ),
      );
      // The clock lands on whole frames, so most display frames ask for the
      // picture that is already on screen. Nothing to draw: give the thread back.
      if (tick['needsRender'] == false) return true;
    }
    final info = EditorSession.map(
      _span(
        'motolii.renderInfo',
        () => jsonDecode(frames.request('{"op":"renderInfo"}')),
      ),
    );
    if (info['error'] != null) throw StateError('${info['error']}');
    final listed = EditorSession.maps(info['views']);
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
      // The call returns once the work is submitted: the window, the roi and the
      // status all belong to this one frame, and the GPU finishing it is somebody
      // else's thread. The pair of surfaces per view keeps the compositor off the
      // one being written, and native refuses (1) a surface still being drawn.
      final surface = have.ids[_flip % have.ids.length];
      final code = _span(
        'motolii.render.$name',
        () => frames.render(surface, name),
      );
      if (code < 0) {
        final status = EditorSession.map(
          jsonDecode(frames.request('{"op":"status"}')),
        );
        throw StateError('Rust render failed for $name: ${status['error']}');
      }
      if (code > 0) continue;
      drawn = true;
    }
    final raw = _span(
      'motolii.status',
      () => frames.request(
        jsonEncode({
          'op': 'status',
          'knownSnapshotId': state['snapshotId'],
          'knownReferenceId': state['referenceId'],
        }),
      ),
    );
    final decoded = _span('motolii.decode', () => jsonDecode(raw));
    _span(
      'motolii.accept',
      () => _accept({
        'status': decoded,
        'frameReady': drawn,
        'frameOnly': playback,
      }, notify: notify),
    );
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
    final reply = EditorSession.map(
      await native('ensureSurfaces', {'views': views}),
    );
    if (_disposed) return;
    _surfaces.clear();
    for (final entry in EditorSession.map(reply['surfaces']).entries) {
      final surface = EditorSession.map(entry.value);
      _surfaces[entry.key] = (
        width: (surface['width'] as num).toInt(),
        height: (surface['height'] as num).toInt(),
        ids: [for (final id in surface['ids'] as List) (id as num).toInt()],
      );
    }
    _accept(reply);
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

  /// 再生 1 回ぶんの合否(kDebugMode)。分布は出さない — 予算 1 コマ 16.7 ms に対する
  /// 可否だけ: UI thread が拍のうち何 % を占めたか、予算超えと倍超えの本数、飛ばした数。
  int _budgetTicks = 0, _budgetOver16 = 0, _budgetOver33 = 0, _budgetUs = 0;
  Stopwatch? _budgetSpan;

  void _noteBudget(int micros) {
    _budgetTicks++;
    _budgetUs += micros;
    if (micros > 16700) _budgetOver16++;
    if (micros > 33300) _budgetOver33++;
  }

  /// 再生が止まった時に 1 回だけ。毎コマは出さない。
  void _reportBudget() {
    final span = _budgetSpan?.elapsedMicroseconds ?? 0;
    if (!kDebugMode || _budgetTicks == 0 || span == 0) return;
    debugPrint(
      'PROBE room=playback-budget ticks=$_budgetTicks '
      'ui=${(_budgetUs * 100 / span).round()}% '
      'over16.7=$_budgetOver16 over33=$_budgetOver33 '
      'skipped=${state['framesSkipped'] ?? 0} seconds=${(span / 1e6).toStringAsFixed(1)}',
    );
    // 93% の中身。段ごとに、1 拍あたり何 ms で、拍の予算 16.7 ms の何 % か。
    final stages = _spanUs.entries.toList()
      ..sort((a, b) => b.value.compareTo(a.value));
    if (stages.isNotEmpty) {
      debugPrint(
        'PROBE room=playback-stages ${stages.map((e) {
          final perTick = e.value / _budgetTicks / 1000;
          return '${e.key.replaceFirst('motolii.', '')}='
              '${perTick.toStringAsFixed(1)}ms'
              '(${(perTick * 100 / 16.7).round()}%)';
        }).join(' ')}',
      );
    }
    _budgetSpan = null;
  }

  void _beginCadence(int generation) {
    if (_disposed || windowInfo['main'] == false) return;
    _budgetTicks = _budgetOver16 = _budgetOver33 = _budgetUs = 0;
    _spanUs.clear();
    _budgetSpan = kDebugMode ? (Stopwatch()..start()) : null;
    _ticker?.dispose();
    _ticker = Ticker((_) {
      if (_disposed || generation != _generation || _pendingWork > 0) return;
      // Same frame: the tick's picture and status are in before this frame builds.
      if (_frames case final frames?) {
        final spent = kDebugMode ? (Stopwatch()..start()) : null;
        try {
          _span(
            'motolii.renderNow',
            () => _renderNow(frames, notify: false, playback: true),
          );
        } catch (e) {
          debugPrint('PROBE room=playback verdict=cadence-stopped reason=$e');
          _schedulePause(renderFinal: false);
          error.value = '$e';
        }
        if (spent != null) _noteBudget(spent.elapsedMicroseconds);
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
    if (_ticker != null) _reportBudget();
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
}
