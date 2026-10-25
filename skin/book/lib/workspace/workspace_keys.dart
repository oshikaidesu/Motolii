// workspace_keys.dart: the key marks beside a value (none, keyed, a key here), read and put.
part of 'workspace.dart';

/// Joined: a property's key mark as the document has it at the playhead (the most keyed of its values).
/// Joined: the key marks the inspector shows (for the checks: the same the inspector is given).
Map<String, KeyS> wsKeyMarks(Ws ws, String? id) => _keyMarks(ws, id);

Map<String, KeyS> _keyMarks(Ws ws, String? id) {
  final m = ws.inside, t = id == null ? 0 : int.tryParse(id);
  if (m == null || t == null) return const {};
  final at = _drawn(ws);
  final marks = {for (final MapEntry(:key, :value) in _keyable(ws, t).entries) key: KeyS.values[m.keyed(t, value, at)]};
  final props = <String, KeyS>{};   // (a property's mark is the most of its values': a key on any, keyed)
  for (final MapEntry(:key, :value) in marks.entries) {
    final p = propOf(key);
    if (p != key && (props[p] == null || value.index > props[p]!.index)) props[p] = value;
  }
  return {...marks, ...props};
}

/// Joined: the fields of the cards whose values take keys, by the inspector's id and the document's name.
Map<String, String> _keyable(Ws ws, int t) {
  final m = ws.inside!, moving = m.moving(t).toSet();
  return {
    for (final (_, names) in m.cards(t))
      for (final n in names)
        if (moving.contains(n)) 'v_$n': n,
  };
}

/// Joined: a key mark changed in the inspector: a key at the playhead, that key taken away, or no keys at all.
void _mark(Ws ws, String? id, String prop, KeyS mark) {
  final m = ws.inside!, t = id == null ? 0 : int.tryParse(id);
  if (t == null) return;
  final at = ws.frame / Ws.fps, has = m.values(t).toSet();
  // (only the values it has: a flat thing has no scale_z, and a key asked of it was refused and took the whole step,
  //  the number let go of with it, back: with Animate on, a flat thing's first scale was lost)
  m.later('together', number: 1);   // (one step, without waiting)
  for (final n in [for (final MapEntry(:key, :value) in _keyable(ws, t).entries) if (key == prop || propOf(key) == prop) value].where(has.contains)) {
    m.later(switch (mark) { KeyS.at => 'key_here', KeyS.anim => 'unkey', KeyS.off => 'stay' }, thing: t, name: n, number: at);
  }
  ws.after(m.later('together'));
}
