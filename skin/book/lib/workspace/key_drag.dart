// Joined: keys slid in time on the timeline (After Effects' drag of keyframes): every value of the thing with a key
// at that moment keeps what it holds and how its stretch goes, at the new moment; all of them one step. Told without
// waiting; the timeline shows where the hand has them by the playhead until it lets go.
import 'ws.dart';

/// The keys [keys] (a row and a frame each) put [by] frames later (earlier below 0).
void wsSlideKeys(Ws ws, Iterable<WsKey> keys, int by) {
  final m = ws.inside;
  if (m == null || by == 0 || keys.isEmpty) return;
  final moves = <(int, String, double, double, double, String)>[];   // (thing, value, from, to, what it holds, its way)
  for (final k in keys) {
    final t = int.tryParse(k.layer);
    if (t == null) continue;
    final at = k.frame / Ws.fps, to = (k.frame + by).clamp(0, Ws.duration) / Ws.fps;
    for (final n in m.moving(t)) {
      if (!m.keyMoments(t, n).any((s) => (s - at).abs() < .5 / Ws.fps)) continue;
      moves.add((t, n, at, to, m.value(t, n, at) ?? 0, m.wayOf(t, n, at)));
    }
  }
  if (moves.isEmpty) return;
  m.later('together', number: 1);
  for (final (t, n, at, _, _, _) in moves) {
    m.later('unkey', thing: t, name: n, number: at);   // (all taken up first: one slid onto another's old moment is not lost)
  }
  for (final (t, n, _, to, v, way) in moves) {
    m.later('key', thing: t, name: n, word: '$to', number: v);
    if (way.isNotEmpty) m.later('way', thing: t, name: n, number: to, word: way);
  }
  ws.after(m.later('together'), (_, _) => ws.reread());
}

/// The picked keys taken away (every value of the thing that has a key at that moment), one step.
void wsRemoveKeys(Ws ws, Iterable<WsKey> keys) {
  final m = ws.inside;
  if (m == null || keys.isEmpty) return;
  m.later('together', number: 1);
  for (final k in keys) {
    final t = int.tryParse(k.layer);
    if (t == null) continue;
    final at = k.frame / Ws.fps;
    for (final n in m.moving(t)) {
      if (m.keyMoments(t, n).any((s) => (s - at).abs() < .5 / Ws.fps)) m.later('unkey', thing: t, name: n, number: at);
    }
  }
  ws.clearKeys();
  ws.after(m.later('together'), (_, _) => ws.reread());
}

/// The keys Cmd+C kept: the value, frames after the first of them, what it holds, its way. None: Cmd+C kept a thing.
List<(String, int, double, String)> wsKeysKept = const [];

/// The picked keys kept (After Effects' Copy and paste keyframes).
void wsCopyKeys(Ws ws, Iterable<WsKey> keys) {
  final m = ws.inside;
  if (m == null || keys.isEmpty) return;
  final first = keys.map((k) => k.frame).reduce((a, b) => a < b ? a : b);
  wsKeysKept = [
    for (final k in keys)
      if (int.tryParse(k.layer) case final t?)
        for (final n in m.moving(t))
          if (m.keyMoments(t, n).any((s) => (s - k.frame / Ws.fps).abs() < .5 / Ws.fps))
            (n, k.frame - first, m.value(t, n, k.frame / Ws.fps) ?? 0, m.wayOf(t, n, k.frame / Ws.fps)),
  ];
}

/// Those keys on the picked thing, the first at the playhead, one step, picked.
void wsPasteKeys(Ws ws) {
  final m = ws.inside, t = int.tryParse(ws.selected ?? '');
  if (m == null || t == null || wsKeysKept.isEmpty) return;
  m.later('together', number: 1);
  for (final (n, after, v, way) in wsKeysKept) {
    final at = (ws.frame + after) / Ws.fps;
    m.later('key', thing: t, name: n, word: '$at', number: v);
    if (way.isNotEmpty) m.later('way', thing: t, name: n, number: at, word: way);
  }
  final pasted = [for (final f in {for (final k in wsKeysKept) ws.frame + k.$2}) (layer: '$t', frame: f)];
  ws.after(m.later('together'), (_, _) {
    ws
      ..reread()
      ..pickKeys(pasted);
  });
}
