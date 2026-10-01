import 'dart:math' as math;

/// What the host says for `easeIntervals` (Rust: selected_keys_mean_intervals_the_host_names), for tests that hand the
/// desk a hand-made status: the intervals the selected keys mean, from the same rows the status carries.
Map<String, dynamic> withEaseIntervals(Map<String, dynamic> status) {
  final selected = (status['selectedKeys'] as List? ?? const []).whereType<Map>().toList();
  final out = <Map<String, dynamic>>[];
  for (final layer in (status['layers'] as List? ?? const []).whereType<Map>()) {
    for (final row in [
      ...(layer['properties'] as List? ?? const []).whereType<Map>(),
      for (final fx in (layer['effects'] as List? ?? const []).whereType<Map>()) ...(fx['params'] as List? ?? const []).whereType<Map>(),
    ]) {
      final chosen = selected.where((k) => k['layer'] == layer['id'] && (k['property'] == null || k['property'] == row['id'])).toList();
      if (chosen.isEmpty) continue;
      final frames = chosen.map((k) => (k['frame'] as num).toInt()).toSet();
      final last = frames.reduce(math.max);
      final keys = (row['keys'] as List? ?? const []).whereType<Map>().toList()..sort((a, b) => (a['frame'] as num).compareTo(b['frame'] as num));
      for (var i = 0; i + 1 < keys.length; i++) {
        final frame = (keys[i]['frame'] as num).toInt();
        if (!frames.contains(frame) || (frames.length > 1 && frame == last)) continue;
        out.add({'layer': layer['id'], 'name': layer['name'], 'property': row['id'], 'frame': frame, 'end': keys[i + 1]['frame'], 'shape': keys[i]['interp'], 'locked': layer['locked'] == true});
      }
    }
  }
  return {...status, 'easeIntervals': out};
}
