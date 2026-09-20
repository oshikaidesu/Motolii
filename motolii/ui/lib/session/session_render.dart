part of 'editor_session.dart';

/// 描かせる／再生の拍: the surfaces each view is drawn into, the render that
/// answers one frame, and the transport that asks for the next one.
mixin SessionRender on SessionCore {
  void _clearSurfaces() {}

  Future<void> _render({bool notify = true, bool playback = false}) async {
    final response = await native('render', {
      'playing': playback,
      'knownSnapshotId': state['snapshotId'],
      'knownReferenceId': state['referenceId'],
    });
    _accept(response, notify: notify);
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

  void _cancelCadence() {
    _generation++;
    _playRequested = false;
  }

  void _schedulePause({required bool renderFinal}) {
    if (_disposed) return;
    final needed = _playRequested || playing.value;
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
