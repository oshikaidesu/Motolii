// session_playhead.dart: the playhead: stepped, sought, played and stopped; the key under it set, taken off, deleted.
part of 'session.dart';

extension SessionPlayhead on Session {
  /// Tab / Shift-Tab / arrows: the next layer, wrapping; with nothing selected the first (or the last going back).
  void step(int by) {
    final i = primary == null ? (by > 0 ? -1 : 0) : indexOf(primary!.id), n = layers.length;
    select(layers[(i + by) % n].id, key: true);
  }

  // ---- W3: playhead, play, keys ---------------------------------------------------------------------------------------------------

  void seek(int f) {
    frame = f.clamp(0, Session.lastFrame);
    _applyAll();
    _changed();
  }

  void toggle() {
    playing = !playing;
    _timer?.cancel();
    if (playing) {
      _from = frame;
      _clock
        ..reset()
        ..start();
      _timer = Timer.periodic(const Duration(milliseconds: 16), (_) {
        frame = (_from + _clock.elapsedMilliseconds * Session.fps ~/ 1000) % (Session.lastFrame + 1); // loops
        _applyAll();
        _changed();
      });
    } else {
      _clock.stop();
    }
    _changed();
  }

  bool get keyAtHead => primary?.pk.containsKey(frame) ?? false;

  /// The key mark: no key here adds one at the playhead (the layer's current position); a key here removes it. One undo step.
  void toggleKey() {
    final l = primary;
    if (l == null) return;
    _edit(l, () {
      if (l.pk.containsKey(frame)) {
        l.pk.remove(frame);
      } else {
        l.pk[frame] = Offset(l.doc['pos.x'], l.doc['pos.y']);
      }
    });
  }

  /// Delete: every picked key, or else the key under the playhead. One undo step.
  void deleteKey() {
    final doomed = <LayerM, List<int>>{};
    for (final id in keySel) {
      final i = id.indexOf(':');
      doomed.putIfAbsent(byId(id.substring(0, i)), () => []).add(int.parse(id.substring(i + 1)));
    }
    final p = primary;
    if (doomed.isEmpty && p != null && keyAtHead) doomed[p] = [frame];
    if (doomed.isEmpty) return;
    _world(() {
      for (final e in doomed.entries) {
        e.value.forEach(e.key.pk.remove);
      }
      keySel.clear();
    });
  }
}
