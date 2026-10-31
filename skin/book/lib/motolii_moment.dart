// motolii_moment.dart: the moment's copy read (skin/BRIDGE.md M): where things are and what they show at the picture's moment. Made by the inside with each picture.
part of 'motolii.dart';

extension MotoliiMoment on Motolii {
  /// Whether [at] is the moment of the picture shown (what the moment's copy holds).
  bool isNow(double at) => _isNow(at);

  Map<String, dynamic>? _momentOf(int thing) => (_moment['thing'] as Map<String, dynamic>)['$thing'] as Map<String, dynamic>?;

  /// Whether [at] is the moment the moment's picture is of (to a frame).
  bool _isNow(double at) => (((_moment['now'] as num?)?.toDouble() ?? -1) - at).abs() < .5 / rate;

  /// The moment of the picture the stage last drew: where a playhead is while it plays.
  double get now => (_moment['now'] as num?)?.toDouble() ?? 0;

  double get drawnAt => now;

  /// A value at a moment, its own (not what a relation makes of it: [shows]): one that stays, the document's; one that
  /// moves, the picked thing's at the moment's picture's moment.
  double? value(int thing, String name, double at) {
    if (!moving(thing).contains(name) || at == 0) return ((_of(thing)['value'] as Map<String, dynamic>?)?[name] as num?)?.toDouble() ?? _usual(thing, name);   // (the document's picture holds each value at its first moment)
    if (_isNow(at)) {   // (the picked thing's, and those the window watches: [watch])
      if ((_momentOf(thing)?['value'] as Map<String, dynamic>?)?[name] case final num v) return v.toDouble();
    }
    final keys = keyMoments(thing, name), i = keys.indexWhere((k) => (k - at).abs() < .5 / rate);
    if (i >= 0) return (((_of(thing)['keyValues'] as Map<String, dynamic>?)?[name] as List<dynamic>?)?[i] as num?)?.toDouble();   // (at one of its keys: what the key holds)
    return _unasked('value', thing, name, at);
  }

  /// A read the pictures do not hold (another moment of a thing not watched): not asked of the inside here, the window
  /// would wait (skin/BRIDGE.md rule 1); ask it as a question ([shownAt]), or watch the thing ([subscribe]). Said once.
  double? _unasked(String what, int thing, String name, double at) {
    if (_watched.contains(thing)) return null;   // (watched just now: in the next picture)
    if (_unaskedSaid.add('$what $name')) debugPrint('BRIDGE: $what($thing, $name, $at) is not in the pictures: ask it ([shownAt]) or watch the thing');
    return null;
  }

  /// A value as the stage shows it (what a relation or behaviour makes of it), and whether one drives it; null: none.
  (double, bool)? shows(int thing, String name, double at) {
    if (!_isNow(at)) return switch (_unasked('shows', thing, name, at)) { _ => null };
    final v = (_momentOf(thing)?['shows'] as Map<String, dynamic>?)?[name] as List<dynamic>?;
    return v == null ? null : ((v[0] as num).toDouble(), v[1] as bool);
  }

  /// A camera as the working eye sees it, the inside's words (doc_gizmo); '': by the camera's own eye.
  String gizmo(int thing) => _momentOf(thing)?['gizmo'] as String? ?? '';

  /// A thing's frame on the picture at the stage's moment: its middle (x, y), how wide, how high (points of the picture,
  /// from the top left), how far it is turned (degrees, clockwise as seen), the point it turns about (x, y), and how deep
  /// it is (0: flat). null: not shown.
  List<double>? frame(int thing) => switch (_momentOf(thing)?['frame']) { final List<dynamic> f => Motolii._numbers(f), _ => null };

  /// What of a thing is seen: the upright box of its outline on the picture (points from the top left). null: not shown.
  Rect? seen(int thing) => switch (_momentOf(thing)?['seen']) { final List<dynamic> b => Rect.fromLTRB((b[0] as num).toDouble(), (b[1] as num).toDouble(), (b[2] as num).toDouble(), (b[3] as num).toDouble()), _ => null };

  /// A path shape's points where the picture shows them (the picked thing's; its handles not); empty: none.
  List<Offset> points(int thing) {
    final p = Motolii._numbers(_momentOf(thing)?['points']);
    return [for (var i = 0; i + 1 < p.length; i += 2) Offset(p[i], p[i + 1])];
  }

  /// Its curves' handles the same way, each with the point it is the handle of.
  List<(Offset, Offset)> handles(int thing) {
    final h = Motolii._numbers(_momentOf(thing)?['handles']), of = Motolii._numbers(_momentOf(thing)?['of']);
    return [for (var i = 0; i + 1 < h.length; i += 2) (Offset(h[i], h[i + 1]), i + 1 < of.length ? Offset(of[i], of[i + 1]) : Offset(h[i], h[i + 1]))];
  }
}
