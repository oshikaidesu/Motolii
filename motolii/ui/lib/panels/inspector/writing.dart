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

  /// One value into a property of every target: during a drag the change is
  /// relative (each layer keeps its own offset), a typed number is absolute.
  Future<void> _write(
    Map<String, dynamic> layer,
    Map<String, dynamic> row,
    dynamic next, {
    required bool preview,
  }) async {
    final id = '${row['id']}';
    // One edit; the host spreads it over the selection: a drag keeps each layer's own offset, a typed number is
    // put to every target as it is, and locked layers are left alone.
    await c.command('previewProperties', {
      'edits': [
        {'layer': layer['id'], 'property': id, 'value': next, 'spread': preview ? 'offset' : 'absolute'},
      ],
    });
    if (!preview) await c.command('commitPreview');
  }

  Future<void> _writeMany(
    Map<String, dynamic> layer,
    Map<String, double> values, {
    required bool preview,
  }) async {
    if (values.isEmpty) return;
    await c.command('previewProperties', {
      'edits': [
        for (final e in values.entries)
          {'layer': layer['id'], 'property': e.key, 'value': e.value, 'spread': preview ? 'offset' : 'absolute'},
      ],
    });
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

  /// Every bounded number of an effect nudged by chance, every number back to
  /// where it rests: the same action a New effect card offers, in
  /// [shared_effects.rollEffect] / [shared_effects.restEffect].
  Future<void> _roll(Map<String, dynamic> layer, Map<String, dynamic> effect) =>
      shared_effects.rollEffect(c, layer['id'] as int, effect);

  Future<void> _rest(Map<String, dynamic> layer, Map<String, dynamic> effect) =>
      shared_effects.restEffect(c, layer['id'] as int, effect);

  /// Choices into every target, absolute (a choice keeps no offset, unlike
  /// a number). The host decides who the targets are and which of them have the row.
  Future<void> _writeChoices(
    Map<String, dynamic> layer,
    Map<String, int> values, {
    required bool preview,
    bool always = false,
  }) async {
    if (values.isEmpty) return;
    await c.command('previewProperties', {
      'edits': [
        for (final e in values.entries) {'layer': layer['id'], 'property': e.key, 'value': e.value, 'spread': 'absolute'},
      ],
    });
    if (!preview) await c.command('commitPreview');
  }
}
