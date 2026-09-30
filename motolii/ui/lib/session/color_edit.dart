import 'package:flutter/widgets.dart';

import 'editor_session.dart';

/// The four numbers a stored colour carries, 0…1 (0…255 input is scaled).
List<double> rgbaOf(dynamic raw) {
  final values = (raw as List? ?? [1, 0, 0, 1])
      .map((v) => (v as num).toDouble())
      .toList();
  if (values.length < 4) values.add(1);
  final scale = values.any((v) => v > 1) ? 255.0 : 1.0;
  return values.take(4).map((v) => (v / scale).clamp(0.0, 1.0)).toList();
}

Color colorOf(List<double> rgba) =>
    Color.from(alpha: rgba[3], red: rgba[0], green: rgba[1], blue: rgba[2]);

/// What a colour editor edits: the host's colour target, else the active layer's first fill stop, else its first
/// colour property. Null: nothing selected has a colour.
Map<String, dynamic>? colorTarget(EditorSession c) {
  if (c.state['colorTarget'] is Map)
    return Map<String, dynamic>.from(c.state['colorTarget']);
  final layer = c.activeLayer;
  if (layer == null) return null;
  final fill = EditorSession.map(layer['fill']);
  final stops = EditorSession.maps(fill['stops']);
  if (stops.isNotEmpty) {
    return {
      'layer': layer['id'],
      'slot': stops.first['slot'] ?? fill['slot'],
      'rgba': stops.first['rgba'],
      'label': 'Fill',
    };
  }
  for (final row in EditorSession.maps(layer['properties'])) {
    if (row['kind'] == 'color' && row['slot'] != null)
      return {
        'layer': layer['id'],
        'slot': row['slot'],
        'rgba': row['value'],
        'label': row['label'] ?? row['id'],
      };
  }
  return null;
}

/// Everything a colour surface shows of the colour target: the target itself (its colour is a value the surface draws)
/// and the name of the layer it belongs to. A slice compares this instead of listening to every layer edit, so a
/// Position scrub leaves the colour surfaces still.
Object? colorReading(EditorSession c) {
  final target = colorTarget(c);
  if (target == null) return null;
  final layer = c.layers.where((l) => l['id'] == target['layer']).firstOrNull;
  return [target, layer?['name']];
}

/// What the wheel edits, in words: "Layer name · Fill" / "· Stroke" / "· Fill · stop", or the composition's ground.
String colorTargetTitle(EditorSession c, Map<String, dynamic> target) {
  if (target['slot'] == 'Background') return 'Composition · Background';
  final layer = c.layers.where((l) => l['id'] == target['layer']).firstOrNull;
  final slot = EditorSession.map(target['slot']);
  final what = slot.containsKey('ShapeStroke')
      ? 'Stroke'
      : slot.keys.any((k) => k.startsWith('ShapeGradient'))
      ? 'Fill · stop'
      : 'Fill';
  final name = '${layer?['name'] ?? ''}'.trim();
  return name.isEmpty ? what : '$name · $what';
}

/// One colour being edited: a gesture previews (`previewColor`) while it lasts and writes once when it ends
/// (`commitPreview`, or `setColor` when nothing was previewed); cancel returns the document to where it was. With no
/// target it only keeps the colour for [onPick]. Every colour editor (Classic's wheel, the hf instrument) drives this.
class ColorEdit extends ChangeNotifier {
  ColorEdit(this.c);
  final EditorSession c;

  Map<String, dynamic>? _target;
  Map<String, dynamic>? get target => _target;
  bool enabled = true;

  /// The colour while there is no target, and who keeps it.
  List<double> unbound = const [1, 0, 0, 1];
  ValueChanged<List<double>>? onPick;

  List<double>? _draft;
  double? _hue;
  bool _previewUsed = false, _ending = false, _cancelled = false;
  bool _disposed = false;

  List<double> get value =>
      _draft ?? (_target == null ? unbound : rgbaOf(_target!['rgba']));
  bool get dragging => _draft != null;

  /// The colour as hue/saturation/value. White, black and greys have no hue of their own: they keep the last one, so
  /// a drag through them does not jump to red.
  HSVColor get hsv {
    final h = HSVColor.fromColor(colorOf(value));
    if (h.saturation > 1 / 255 && _draft == null) _hue = h.hue;
    return h.withHue(_hue ?? h.hue);
  }

  bool get canPreview => c.supports('previewColor') && enabled;

  /// The target moved to another slot or layer: an unfinished gesture on the old one is cancelled.
  void retarget(Map<String, dynamic>? next) {
    if (!sameValue(_target?['slot'], next?['slot']) ||
        _target?['layer'] != next?['layer']) {
      cancel();
    }
    _target = next;
  }

  void begin() => _cancelled = false;

  void previewHsv(HSVColor next) {
    _hue = next.hue;
    final col = next.toColor();
    preview([col.r, col.g, col.b, value[3]]);
  }

  void previewAlpha(double a) {
    final v = value;
    preview([v[0], v[1], v[2], a.clamp(0.0, 1.0)]);
  }

  void preview(List<double> next) {
    if (_ending || _cancelled || !enabled) return;
    _draft = next;
    notifyListeners();
    if (_target == null) {
      onPick?.call(next);
    } else if (canPreview) {
      _previewUsed = true;
      c.commandDirect('previewColor', {
        if (_target!['layer'] != null) 'layer': _target!['layer'],
        'slot': _target!['slot'],
        'rgba': next,
      }, 'color');
    }
  }

  Future<void> commit() async {
    final target = _target, next = _draft, used = _previewUsed;
    _draft = null;
    _previewUsed = false;
    if (next == null) return;
    if (target == null || !enabled) {
      _changed();
      return;
    }
    _ending = true;
    try {
      await (used ? c.command('commitPreview') : _set(target, next));
    } finally {
      _ending = false;
      _changed();
    }
  }

  Future<void> cancel() async {
    if (_ending) return;
    _cancelled = true;
    final used = _previewUsed;
    _previewUsed = false;
    _draft = null;
    _changed();
    if (!used) return;
    _ending = true;
    try {
      await c.command('cancelPreview');
    } finally {
      _ending = false;
    }
  }

  /// A typed colour: written at once.
  Future<void> setNow(List<double> rgba) async {
    if (_target == null) {
      onPick?.call(rgba);
      return;
    }
    if (enabled) await _set(_target!, rgba);
  }

  Future<void> _set(Map<String, dynamic> target, List<double> rgba) =>
      c.command('setColor', {
        if (target['layer'] != null) 'layer': target['layer'],
        'slot': target['slot'],
        'rgba': rgba,
      });

  /// Pick the next Stage click's colour (the Stage reads it and hands it to the palette target); again to stop.
  void toggleEyedropper() => c.eyedropper.value = !c.eyedropper.value;

  void _changed() {
    if (!_disposed) notifyListeners();
  }

  @override
  void dispose() {
    _disposed = true;
    if (_previewUsed && !_ending) c.command('cancelPreview');
    super.dispose();
  }
}
