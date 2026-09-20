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
      final tick = EditorSession.map(
        jsonDecode(frames.request('{"op":"tick","quiet":true}')),
      );
      // The clock lands on whole frames, so most display frames ask for the
      // picture that is already on screen. Nothing to draw: give the thread back.
      if (tick['needsRender'] == false) return true;
    }
    final info = EditorSession.map(
      jsonDecode(frames.request('{"op":"renderInfo"}')),
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
      // One render costs 0.18 ms of CPU, so the GPU is waited for right here:
      // the picture, its window and the status all belong to this one frame.
      // The pair of surfaces per view still keeps the compositor off the one
      // being written.
      final surface = have.ids[_flip % have.ids.length];
      final code = frames.render(surface, name);
      if (code < 0) {
        final status = EditorSession.map(
          jsonDecode(frames.request('{"op":"status"}')),
        );
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
}
