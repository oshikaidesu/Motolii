
/// What a parameter is *for*, read off its declared name. One table, so a new Vism gets its glyph, unit and control
/// the moment it is declared. Shared by every face that reads a property row: Classic's Inspector derives a track
/// style and a tint from it, and the roll/rest actions ([lib/session/effect_actions.dart]) use it to know which
/// numbers are whole counts.
enum PropertyCharacter {
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

const _characterWords = <PropertyCharacter, List<String>>{
  PropertyCharacter.seed: ['seed'],
  PropertyCharacter.angle: ['angle', 'rotation', 'roll', 'tilt', 'spin'],
  PropertyCharacter.place: ['position', 'offset', 'center', 'centre', 'origin'],
  PropertyCharacter.size: ['size', 'radius', 'width', 'height', 'thickness', 'scale', 'length', 'distance'],
  PropertyCharacter.opacity: ['opacity', 'alpha', 'transparency'],
  PropertyCharacter.soft: ['softness', 'blur', 'feather', 'spread', 'smooth'],
  PropertyCharacter.time: ['evolution', 'time', 'phase', 'speed', 'rate', 'frequency'],
  PropertyCharacter.delay: ['delay', 'lag'],
  PropertyCharacter.count: ['count', 'columns', 'rows', 'copies', 'number', 'steps'],
  PropertyCharacter.detail: ['complexity', 'octaves', 'detail', 'iterations', 'quality'],
  PropertyCharacter.color: ['color', 'colour', 'tint', 'hue'],
  PropertyCharacter.level: ['threshold', 'level', 'gamma', 'contrast', 'brightness'],
  PropertyCharacter.direction: ['along', 'direction', 'axis', 'side'],
  PropertyCharacter.amount: ['amount', 'strength', 'intensity', 'mix', 'gain', 'weight', 'density', 'power'],
};

/// The declared or analysed subtype (Blender vocabulary) wins over words.
PropertyCharacter? declaredCharacter(Map<String, dynamic> row) => switch ('${row['subtype'] ?? ''}') {
      'SEED' => PropertyCharacter.seed,
      'ANGLE' => PropertyCharacter.angle,
      'OPACITY' => PropertyCharacter.opacity,
      'DISTANCE' || 'PIXEL' => PropertyCharacter.size,
      'TRANSLATION' => PropertyCharacter.place,
      'TIME' => PropertyCharacter.time,
      'LEVEL' => PropertyCharacter.level,
      'FACTOR' || 'PERCENTAGE' => PropertyCharacter.amount,
      'COUNT' => PropertyCharacter.count,
      _ => null,
    };

/// Read once per row object: the kind, tint, track and unit all ask.
final _characters = Expando<PropertyCharacter>();
PropertyCharacter characterOf(Map<String, dynamic> row) => _characters[row] ??= _deriveCharacter(row);

PropertyCharacter _deriveCharacter(Map<String, dynamic> row) {
  final declared = declaredCharacter(row);
  if (declared != null) return declared;
  final id = '${row['id']}'.split('.param.').last;
  final words = <String>{
    ...id.toLowerCase().split(RegExp('[^a-z]+')),
    ...'${row['label'] ?? ''}'.toLowerCase().split(RegExp('[^a-z]+')),
  }..remove('');
  for (final entry in _characterWords.entries) {
    if (entry.value.any(words.contains)) return entry.key;
  }
  if (row['choices'] is List) return PropertyCharacter.choice;
  return PropertyCharacter.none;
}

