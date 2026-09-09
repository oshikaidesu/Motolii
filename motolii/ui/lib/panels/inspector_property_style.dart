part of 'inspector.dart';

/// What a control is, decided from the declaration, never from the label.
enum _Kind { bounded, scalar, angle, vec2, scale, color, choice, text }

_Kind _kindOf(Map<String, dynamic> row) {
  final id = '${row['id']}';
  final kind = '${row['kind']}';
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

/// What a parameter is *for*, read off its declared name. One table, so a
/// new Vism gets its glyph, unit and control the moment it is declared.
enum _Character {
  amount,
  size,
  ratio,
  angle,
  place,
  seed,
  time,
  count,
  color,
  level,
  soft,
  opacity,
  direction,
  choice,
  delay,
  detail,
  none,
}

const _characterWords = <_Character, List<String>>{
  _Character.seed: ['seed'],
  _Character.angle: ['angle', 'rotation', 'roll', 'tilt', 'spin'],
  _Character.place: ['position', 'offset', 'center', 'centre', 'origin'],
  _Character.size: [
    'size',
    'radius',
    'width',
    'height',
    'thickness',
    'scale',
    'length',
    'distance',
  ],
  _Character.opacity: ['opacity', 'alpha', 'transparency'],
  _Character.soft: ['softness', 'blur', 'feather', 'spread', 'smooth'],
  _Character.time: ['evolution', 'time', 'phase', 'speed', 'rate', 'frequency'],
  _Character.delay: ['delay', 'lag'],
  _Character.count: ['count', 'columns', 'rows', 'copies', 'number', 'steps'],
  _Character.detail: [
    'complexity',
    'octaves',
    'detail',
    'iterations',
    'quality',
  ],
  _Character.color: ['color', 'colour', 'tint', 'hue'],
  _Character.level: ['threshold', 'level', 'gamma', 'contrast', 'brightness'],
  _Character.direction: ['along', 'direction', 'axis', 'side'],
  _Character.amount: [
    'amount',
    'strength',
    'intensity',
    'mix',
    'gain',
    'weight',
    'density',
    'power',
  ],
};

const _characterGlyphs = <_Character, IconData>{
  _Character.amount: Icons.tune,
  _Character.size: Icons.straighten,
  _Character.ratio: Icons.aspect_ratio,
  _Character.angle: Icons.rotate_right,
  _Character.place: Icons.open_with,
  _Character.seed: Icons.casino_outlined,
  _Character.time: Icons.timelapse,
  _Character.count: Icons.grid_on,
  _Character.color: Icons.palette_outlined,
  _Character.level: Icons.linear_scale,
  _Character.soft: Icons.blur_on,
  _Character.opacity: Icons.opacity,
  _Character.direction: Icons.alt_route,
  _Character.choice: Icons.category_outlined,
  _Character.delay: Icons.history_toggle_off,
  _Character.detail: Icons.grain,
};

/// The declared or analysed subtype (Blender vocabulary) wins over words.
_Character? _declaredCharacter(Map<String, dynamic> row) =>
    switch ('${row['subtype'] ?? ''}') {
      'SEED' => _Character.seed,
      'ANGLE' => _Character.angle,
      'OPACITY' => _Character.opacity,
      'DISTANCE' || 'PIXEL' => _Character.size,
      'TRANSLATION' => _Character.place,
      'TIME' => _Character.time,
      'LEVEL' => _Character.level,
      'FACTOR' || 'PERCENTAGE' => _Character.amount,
      'COUNT' => _Character.count,
      _ => null,
    };

_Character _characterOf(Map<String, dynamic> row) {
  final declared = _declaredCharacter(row);
  if (declared != null) return declared;
  final id = '${row['id']}'.split('.param.').last;
  final words = <String>{
    ...id.toLowerCase().split(RegExp('[^a-z]+')),
    ...'${row['label'] ?? ''}'.toLowerCase().split(RegExp('[^a-z]+')),
  }..remove('');
  for (final entry in _characterWords.entries) {
    if (entry.value.any(words.contains)) return entry.key;
  }
  if (row['choices'] is List) return _Character.choice;
  return _Character.none;
}

IconData? _glyphOf(Map<String, dynamic> row) =>
    _characterGlyphs[_characterOf(row)];

/// One hue per family, so a glance sorts the numbers before a word is read.
Color? _tintOf(Map<String, dynamic> row) => switch (_characterOf(row)) {
  _Character.place ||
  _Character.size ||
  _Character.ratio ||
  _Character.soft => EditorTheme.spatial,
  _Character.amount ||
  _Character.opacity ||
  _Character.level => EditorTheme.amount,
  _Character.time || _Character.delay => EditorTheme.time,
  _Character.count || _Character.detail => EditorTheme.count,
  _Character.seed => EditorTheme.seed,
  _Character.angle => EditorTheme.angle,
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

double _span(Map<String, dynamic> row) =>
    ((row['max'] as num?)?.toDouble() ?? 0) -
    ((row['min'] as num?)?.toDouble() ?? 0);

/// A range that is a real reach (opacity 0..1, octaves 1..8), not a guard
/// (amount 0..100000): only a real reach earns a track.
bool _tight(Map<String, dynamic> row) {
  final v = (row['value'] as num?)?.toDouble() ?? 0;
  final d = (row['default'] as num?)?.toDouble() ?? v;
  return _span(row) <= 20 * math.max(d.abs(), 1);
}

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
