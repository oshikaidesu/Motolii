part of '../inspector.dart';

/// What a control is, decided from the declaration, never from the label.
enum _Kind { bounded, scalar, angle, vec2, scale, color, choice, text, layer }

typedef _Character = PropertyCharacter;
final _characterOf = characterOf;
final _span = spanOf;
final _tight = tightRange;

_Kind _kindOf(Map<String, dynamic> row) {
  final id = '${row['id']}';
  final kind = '${row['kind']}';
  // A field that points at a layer: the same picker as the camera's target.
  if (row['layer'] == true) return _Kind.layer;
  if (row['choices'] is List || kind == 'enum') return _Kind.choice;
  if (kind == 'color') return _Kind.color;
  if (kind == 'text') return _Kind.text;
  if (id == 'scale') return _Kind.scale;
  if (id.startsWith('rotation') || id == 'camera.roll') return _Kind.angle;
  if (id.contains('.param.') &&
      row['value'] is num &&
      _characterOf(row) == _Character.angle) {
    return _Kind.angle;
  }
  if (kind == 'vec2' || row['value'] is List) return _Kind.vec2;
  if (row['min'] != null && row['max'] != null) return _Kind.bounded;
  return _Kind.scalar;
}


/// One hue per family, so a glance sorts the numbers before a word is read.
Color? _tintOf(Map<String, dynamic> row, EditorTheme colors) =>
    switch (_characterOf(row)) {
      _Character.place ||
      _Character.size ||
      _Character.ratio ||
      _Character.soft => colors.spatial,
      _Character.amount ||
      _Character.opacity ||
      _Character.level => colors.amount,
      _Character.time || _Character.delay => colors.time,
      _Character.count || _Character.detail => colors.count,
      _Character.seed => colors.seed,
      _Character.angle => colors.angle,
      _ => null,
    };

/// The track's grammar by family: a threshold lights its far side, a count
/// shows whole steps, a length sits on a ruler, an amount fills.
TrackStyle _trackOf(Map<String, dynamic> row) => switch (_characterOf(row)) {
  _Character.level => TrackStyle.level,
  _Character.count ||
  _Character.detail when _span(row) <= 12 => TrackStyle.steps,
  _Character.size || _Character.soft => TrackStyle.ruler,
  _ => TrackStyle.fill,
};


String? _unitOf(Map<String, dynamic> row) {
  final id = '${row['id']}';
  if (_kindOf(row) == _Kind.angle) return '°';
  if (id == 'opacity') return '%';
  if (id.startsWith('position') || id == 'anchor' || id == 'camera.center') {
    return 'px';
  }
  if (row['unit'] is String) return row['unit'] as String;
  if (id.contains('.param.')) {
    final max = row['max'] as num?;
    return switch (_characterOf(row)) {
      _Character.angle => '°',
      _Character.place || _Character.size || _Character.soft => 'px',
      _Character.opacity ||
      _Character.amount when max == 1 || max == 100 => '%',
      _ => null,
    };
  }
  return null;
}
