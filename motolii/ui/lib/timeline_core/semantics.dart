import 'dart:math' as math;

import 'layout.dart';

/// What a skin found under its pointer, in the Timeline's own terms (rows by index in [TimelineSession.rows], time
/// in frames). A skin decides this with its own geometry; the session never sees a pixel.
sealed class TlTarget {
  const TlTarget();
}

/// A row's name (a layer: pick it, carry it to reorder; a lane: pick its layer).
class TlRowName extends TlTarget {
  const TlRowName(this.row);
  final int row;
}

/// A layer's bar: its body (move) or one of its ends (trim).
class TlBar extends TlTarget {
  const TlBar(this.row, this.part);
  final int row;
  final TlBarPart part;
}

enum TlBarPart { body, start, end }

/// A folded layer's summary mark: every key of the layer at [frame].
class TlLayerKey extends TlTarget {
  const TlLayerKey(this.row, this.frame);
  final int row, frame;
}

/// A key on a property lane.
class TlLaneKey extends TlTarget {
  const TlLaneKey(this.row, this.frame);
  final int row, frame;
}

/// The span between two consecutive keys of a lane: picks both ends.
class TlSpan extends TlTarget {
  const TlSpan(this.row, this.from, this.to);
  final int row, from, to;
}

/// Nothing: a press here starts a marquee.
class TlEmpty extends TlTarget {
  const TlEmpty();
}

enum TlGesture { rowPending, rowDrag, keys, move, trimIn, trimOut, slip, marquee }

/// The modifiers held at a press, read by the skin.
class TlMods {
  const TlMods({this.primary = false, this.shift = false, this.alt = false});

  /// Cmd (Ctrl elsewhere).
  final bool primary, shift, alt;
}

/// Where carried rows (or an asset) land: before, after or inside [target], or at the root's end. [depth] and [at]
/// (a row position) say where a skin draws the guide.
class TlDrop {
  const TlDrop(this.layers, this.target, this.placement, {required this.depth, required this.at, this.inside = false});
  final List<int> layers;
  final int? target;
  final String placement;
  final int depth;
  final double at;
  final bool inside;
  Map<String, dynamic> get command => {'layers': layers, 'target': target, 'placement': placement};
}

Map<String, dynamic> tlKeyOf(TrackRow row, Map<String, dynamic> key) => {'layer': row.id, 'property': row.property?['id'], 'frame': key['frame']};

bool tlSameKey(Map<String, dynamic> a, Map<String, dynamic> b) => a['layer'] == b['layer'] && a['property'] == b['property'] && a['frame'] == b['frame'];

/// Keys picked by a press on [found]: added to or taken from [current] when [additive], else [found] alone (a press on
/// keys already all picked keeps the selection, so they can be carried together).
List<Map<String, dynamic>> tlPick(List<Map<String, dynamic>> current, List<Map<String, dynamic>> found, {required bool additive}) {
  var chosen = current.toList();
  final already = found.every((f) => chosen.any((k) => tlSameKey(k, f)));
  if (additive) {
    already ? chosen.removeWhere((k) => found.any((f) => tlSameKey(k, f))) : chosen.addAll(found.where((f) => !chosen.any((k) => tlSameKey(k, f))));
  } else if (!already) {
    chosen = found;
  }
  return chosen;
}

/// What a marquee over [frames] × [rows] (row positions) picks: keys inside it (a lane's keys, a folded layer's
/// summary), and layers whose bar it touches; [keys] / [ids] already picked are kept (an additive marquee).
({List<Map<String, dynamic>> keys, List<int> ids}) tlMarquee(List<TrackRow> rows,
    {required (double, double) frames, required (double, double) rows_, List<Map<String, dynamic>> keys = const [], List<int> ids = const []}) {
  final outKeys = [...keys];
  final outIds = {...ids};
  bool inFrames(num f) => f >= frames.$1 && f <= frames.$2;
  for (var n = 0; n < rows.length; n++) {
    final centre = n + .5;
    if (centre < rows_.$1 || centre > rows_.$2) continue;
    final row = rows[n];
    if (row.property != null) {
      for (final k in row.keys) {
        if (!inFrames(k['frame'] as num)) continue;
        final key = tlKeyOf(row, k);
        if (!outKeys.any((s) => tlSameKey(s, key))) outKeys.add(key);
        outIds.add(row.id);
      }
    } else {
      if (!row.lanesOpen) {
        for (final k in row.allKeys) {
          if (inFrames(k['frame'] as num) && !outKeys.any((s) => tlSameKey(s, k))) {
            outKeys.add(k);
            outIds.add(row.id);
          }
        }
      }
      final start = (row.layer['start'] as num? ?? 0).toDouble(), end = start + (row.layer['duration'] as num? ?? 0).toDouble();
      if (start <= frames.$2 && end >= frames.$1) outIds.add(row.id);
    }
  }
  return (keys: outKeys, ids: outIds.toList());
}

/// Where a gripped bar is drawn under the pointer before the document answers: only a picture. What the timing
/// becomes is the host's (`timing_delta`, from the mode and the frames sent).
Map<String, dynamic> tlDrawnTiming(int id, Map<String, dynamic> layer, TlGesture g, int delta) {
  final s = (layer['start'] as num? ?? 0).toInt(), d = (layer['duration'] as num? ?? 1).toInt(), i = (layer['sourceIn'] as num? ?? 0).toInt();
  return switch (g) {
    TlGesture.trimIn => () {
        final t = delta.clamp(-math.min(s, i), d - 1);
        return {'layer': id, 'start': s + t, 'duration': d - t, 'sourceIn': i + t};
      }(),
    TlGesture.trimOut => {'layer': id, 'start': s, 'duration': math.max(1, d + delta), 'sourceIn': i},
    TlGesture.slip => {'layer': id, 'start': s, 'duration': d, 'sourceIn': math.max(0, i - delta)},
    _ => {'layer': id, 'start': math.max(0, s + delta), 'duration': d, 'sourceIn': i},
  };
}
