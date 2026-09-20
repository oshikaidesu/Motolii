part of '../inspector.dart';

/// Writing: preview while a gesture runs, commit when it ends. Every edit
/// goes to every target, relative during a drag and absolute when typed.
mixin _InspectorWriting on _InspectorReading {
  bool _scaleLocked = true;

  List<Map<String, dynamic>>? _gestureLayers;

  void _begin() => _gestureLayers = c.liveLayers().toList();

  dynamic _axisValue(Map<String, dynamic> row, int axis, double value) {
    row = _gestureLayers == null
        ? row
        : _property(
                _gestureLayers!.firstWhere(
                  (v) => v['id'] == _active?['id'],
                  orElse: () => <String, dynamic>{},
                ),
                '${row['id']}',
              ) ??
              row;
    final current = row['value'];
    if (row['id'] == 'scale' && _scaleLocked && current is List) {
      final base = (current[axis] as num).toDouble();
      return [
        for (final n in current) base == 0 ? value : (n as num) * value / base,
      ];
    }
    return _withAxis(current, axis, value);
  }

  /// One absolute value into one axis of a property: preview during a drag,
  /// preview + commit for a typed number.
  /// Every selected, unlocked layer; the shown layer alone when it is not
  /// part of the selection. A drag moves them all by the same amount, a
  /// typed number sets them all.
  /// Once per (gesture, document, frame): every well asks, and the live
  /// layers are a copy of the whole document.
  Object? _targetsKey;

  List<Map<String, dynamic>>? _targetsCache;

  List<Map<String, dynamic>> _targets(Map<String, dynamic> layer) {
    final key = (_gestureLayers, c.state, c.rendered.value, layer['id']);
    if (key != _targetsKey) {
      _targetsKey = key;
      final live = (_gestureLayers ?? c.liveLayers())
          .where((v) => c.selectedIds.contains(v['id']) && v['locked'] != true)
          .toList();
      _targetsCache = live.any((v) => v['id'] == layer['id']) ? live : [layer];
    }
    return _targetsCache!;
  }

  /// One value into a property of every target: during a drag the change is
  /// relative (each layer keeps its own offset), a typed number is absolute.
  Future<void> _write(
    Map<String, dynamic> layer,
    Map<String, dynamic> row,
    dynamic next, {
    required bool preview,
  }) async {
    final id = '${row['id']}';
    if (preview && _gestureLayers != null) {
      final base = _gestureLayers!
          .where((v) => v['id'] == layer['id'])
          .firstOrNull;
      if (base != null) row = _property(base, id) ?? row;
    }
    final edits = <Map<String, dynamic>>[];
    for (final target in _targets(layer)) {
      final own = _property(target, id);
      if (own == null) continue;
      dynamic value = next;
      if (preview && target['id'] != layer['id']) {
        final base = row['value'], mine = own['value'];
        if (next is num && base is num && mine is num) {
          value = mine + (next - base);
        } else if (next is List && base is List && mine is List) {
          value = [
            for (var i = 0; i < next.length; i++)
              i < base.length && i < mine.length && next[i] is num
                  ? (mine[i] as num) + (next[i] as num) - (base[i] as num)
                  : next[i],
          ];
        }
      }
      edits.add({'layer': target['id'], 'property': id, 'value': value});
    }
    if (edits.isEmpty) return;
    await c.command('previewProperties', {'edits': edits});
    if (!preview) await c.command('commitPreview');
  }

  Future<void> _writeMany(
    Map<String, dynamic> layer,
    Map<String, double> values, {
    required bool preview,
  }) async {
    final edits = [
      for (final target in _targets(layer))
        for (final e in values.entries)
          if (_property(target, e.key) != null)
            {
              'layer': target['id'],
              'property': e.key,
              'value': preview && target['id'] != layer['id']
                  ? (_property(target, e.key)!['value'] as num) +
                        e.value -
                        (_property(
                              (_gestureLayers ?? [layer]).firstWhere(
                                (v) => v['id'] == layer['id'],
                              ),
                              e.key,
                            )!['value']
                            as num)
                  : e.value,
            },
    ];
    if (edits.isEmpty) return;
    await c.command('previewProperties', {'edits': edits});
    if (!preview) await c.command('commitPreview');
  }

  dynamic _withAxis(dynamic current, int axis, double v) {
    if (current is List) {
      final out = List<dynamic>.from(current);
      if (axis < out.length) out[axis] = v;
      return out;
    }
    return v;
  }

  Future<void> _finish(bool cancel) async {
    try {
      await c.command(cancel ? 'cancelPreview' : 'commitPreview');
    } finally {
      _gestureLayers = null;
    }
  }

  /// Every bounded number of an effect nudged by chance — within a fifth of
  /// its reach around where it is, so a throw stays playable — seeds fully,
  /// counts whole, as one edit (the Ableton dice).
  Future<void> _roll(
    Map<String, dynamic> layer,
    Map<String, dynamic> effect,
  ) async {
    final rnd = math.Random();
    final values = <String, double>{};
    for (final row in panelRows(effect['params'])) {
      final id = '${row['id']}';
      if (row['value'] is! num || row['choices'] is List) continue;
      final min = (row['min'] as num?)?.toDouble(),
          max = (row['max'] as num?)?.toDouble();
      if (id.endsWith('.seed')) {
        values[id] = rnd.nextInt(10000).toDouble();
      } else if (min != null && max != null) {
        final reach = (max - min) * .2;
        var next =
            ((row['value'] as num).toDouble() +
                    (rnd.nextDouble() * 2 - 1) * reach)
                .clamp(min, max)
                .toDouble();
        if (_characterOf(row) == _Character.count) next = next.roundToDouble();
        values[id] = next;
      }
    }
    if (values.isNotEmpty) await _writeMany(layer, values, preview: false);
  }

  /// Every number of an effect back to where it rests, as one edit.
  Future<void> _rest(
    Map<String, dynamic> layer,
    Map<String, dynamic> effect,
  ) async {
    final values = <String, double>{};
    for (final row in panelRows(effect['params'])) {
      final d = row['default'];
      if (row['value'] is num && d is num && row['choices'] is! List) {
        values['${row['id']}'] = d.toDouble();
      }
    }
    if (values.isNotEmpty) await _writeMany(layer, values, preview: false);
  }

  /// Choices into every target, absolute (a choice keeps no offset, unlike
  /// a number). [always] writes a row the snapshot left out of the panel —
  /// the alignment of a grid, which the document reads all the same.
  Future<void> _writeChoices(
    Map<String, dynamic> layer,
    Map<String, int> values, {
    required bool preview,
    bool always = false,
  }) async {
    final edits = [
      for (final target in _targets(layer))
        for (final e in values.entries)
          if (always || _property(target, e.key) != null)
            {'layer': target['id'], 'property': e.key, 'value': e.value},
    ];
    if (edits.isEmpty) return;
    await c.command('previewProperties', {'edits': edits});
    if (!preview) await c.command('commitPreview');
  }
}
