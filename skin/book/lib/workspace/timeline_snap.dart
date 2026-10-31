// timeline_snap.dart: what a bar's end, a key or the hand comes to rest on: Alight Motion's snapFrame and moveTarget (its
// Timeline.kt), the nearest within 8 points, a bar by whichever end is nearer, a line where it lands. Live's grid beside
// them while there is a tempo, and held with Cmd none (its manual, 6.7 and 6.10).
part of 'timeline.dart';

extension _WsTimelineSnap on _WsTimelineState {
  int? _restOn(int f) {
    if (HardwareKeyboard.instance.isMetaPressed) return null;
    int? best;
    for (final s in _stops.followedBy([if (_ws!.beats case final b?) b.nearest(f)])) {
      if ((s - f).abs() * _view.ppf <= Shape.z(8) && (best == null || (s - f).abs() < (best - f).abs())) best = s;
    }
    return best;
  }

  /// How far the hand's [by] goes, the nearest of [ends] resting where it can.
  int _rested(int by, List<int> ends) {
    var went = by;
    _restAt = null;
    for (final e in ends) {
      final s = _restOn(e + by);
      if (s != null && (_restAt == null || (s - e - by).abs() < (went - by).abs())) {
        went = s - e;
        _restAt = s;
      }
    }
    return went;
  }
}
