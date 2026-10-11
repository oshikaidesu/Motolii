// Reads an effect's WGSL uniform struct and guesses, field by field, what kind of value each one is,
// so a shader nobody has seen still gets a control that looks like what it moves. Pure Dart: no Flutter.
import 'dart:math' as math;

enum WType { f32, i32, u32, boolean, vec2, vec3, vec4 }

/// How a value is shown and handled. Each look is a different picture of the value, not a different number box.
enum ParamLook {
  /// No range known: a number you drag.
  scrub,

  /// A range: a number with a fill along its bottom.
  bar,

  /// A range that straddles zero: the fill grows out from the centre.
  bipolar,

  /// A share of a whole (0–100 %), drawn as a pie.
  percent,

  /// A direction or turn, drawn as a dial you turn.
  angle,

  /// A horizontal amount: dragged sideways.
  horizontal,

  /// A vertical amount: pictured as a level (the number still scrubs sideways like every other).
  vertical,

  /// A point (vec2): a pad you place it on, X dragged sideways, Y up and down.
  point,

  /// A size (vec2): width dragged sideways, height up and down, linkable.
  size,

  /// A small whole number: minus, the count, plus.
  stepper,

  /// One of a few named options.
  choice,

  /// On or off.
  toggle,

  /// A colour (vec3 / vec4).
  colour,

  /// A random seed: a number with a re-roll.
  seed,
}

/// One field of the struct as written, with the attributes found on it.
class Decl {
  const Decl(this.name, this.type, {this.attrs = const {}, this.advanced = false});
  final String name;
  final WType type;
  final Map<String, List<String>> attrs;
  final bool advanced;
}

/// What the Inspector needs to draw one parameter. Values are kept in display units (degrees, percent); [toShader] maps back.
class Param {
  const Param({
    required this.id,
    required this.label,
    required this.look,
    required this.type,
    this.min,
    this.max,
    this.def = 0,
    this.defY = 0,
    this.unit = '',
    this.dec = 0,
    this.options = const [],
    this.advanced = false,
    this.upIncreases = true,
    this.toShader = 1,
    this.why = '',
  });
  final String id, label, unit, why;
  final ParamLook look;
  final WType type;
  final double? min, max;
  final double def, defY, toShader;
  final int dec;
  final List<String> options;
  final bool advanced;

  /// For vertical looks: dragging up raises the value (false: dragging down does, e.g. a drop).
  final bool upIncreases;

  bool get bounded => min != null && max != null;
  double get step => dec == 0 ? 1 : math.pow(10, -dec).toDouble();
  double get perPx => bounded ? (max! - min!) / 200 : (dec == 0 ? 1 : step * 10);
}

final _structRe = RegExp(r'struct\s+\w+\s*\{([\s\S]*?)\}');
final _fieldRe = RegExp(r'^\s*(?:@\w+(?:\([^)]*\))?\s*)*(\w+)\s*:\s*([\w<>]+)\s*,?\s*(?://(.*))?$');
final _attrRe = RegExp(r'@(\w+)(?:\(([^)]*)\))?');

WType? _type(String t) => switch (t) {
  'f32' || 'f16' => WType.f32,
  'i32' => WType.i32,
  'u32' => WType.u32,
  'bool' => WType.boolean,
  'vec2f' || 'vec2<f32>' || 'vec2h' => WType.vec2,
  'vec3f' || 'vec3<f32>' || 'vec3h' => WType.vec3,
  'vec4f' || 'vec4<f32>' || 'vec4h' => WType.vec4,
  _ => null,
};

/// The first struct in [src], one [Decl] per field. Attributes come from `///` lines above a field and `//` after it,
/// e.g. `@range(0, 200) @unit(px) @enum(Low, Med, High) @default(12) @toggle`. A `// @advanced` line marks every field after it.
List<Decl> parseWgsl(String src) {
  final body = _structRe.firstMatch(src)?.group(1);
  if (body == null) return const [];
  final out = <Decl>[];
  var pending = <String, List<String>>{};
  var advanced = false;
  Map<String, List<String>> attrs(String s) => {
    for (final m in _attrRe.allMatches(s))
      m.group(1)!: (m.group(2) ?? '').split(',').map((e) => e.trim()).where((e) => e.isNotEmpty).toList(),
  };
  for (final raw in body.split('\n')) {
    final line = raw.trim();
    if (line.isEmpty) continue;
    if (line.startsWith('//')) {
      final a = attrs(line);
      if (a.containsKey('advanced') && a.length == 1 && !line.startsWith('///')) {
        advanced = true;
      } else {
        pending.addAll(a);
      }
      continue;
    }
    final m = _fieldRe.firstMatch(line);
    final t = m == null ? null : _type(m.group(2)!);
    if (m == null || t == null) continue;
    final all = {...pending, ...attrs(line), ...attrs(m.group(3) ?? '')};
    out.add(Decl(m.group(1)!, t, attrs: all, advanced: advanced || all.containsKey('advanced')));
    pending = {};
  }
  return out;
}

/// `blurRadius`, `blur_radius` → [blur, radius]; short forms spelled out.
List<String> paramWords(String name) {
  final spaced = name.replaceAllMapped(RegExp(r'([a-z0-9])([A-Z])'), (m) => '${m[1]}_${m[2]}');
  return [
    for (final w in spaced.toLowerCase().split(RegExp(r'[_\s]+')).where((w) => w.isNotEmpty)) ...(_long[w] ?? [w]),
  ];
}

const _long = {
  'thr': ['threshold'],
  'amt': ['amount'],
  'freq': ['frequency'],
  'oct': ['octaves'],
  'lac': ['lacunarity'],
  'exp': ['exposure'],
  'con': ['contrast'],
  'sat': ['saturation'],
  'temp': ['temperature'],
  'gam': ['gamma'],
  'rad': ['radius'],
  'str': ['strength'],
  'col': ['colour'],
  'color': ['colour'],
  'pos': ['position'],
  'rot': ['rotation'],
  'dx': ['offset', 'x'],
  'dy': ['offset', 'y'],
  'tintc': ['tint'],
  'clampc': ['clamp'],
  'rough': ['roughness'],
  'pin': ['pin', 'edges'],
  'add': ['additive'],
};

String paramLabel(List<String> w) {
  final s = w.map((e) => e.length == 1 ? e.toUpperCase() : e).join(' ');
  return s.isEmpty ? s : s[0].toUpperCase() + s.substring(1);
}

bool _any(List<String> w, Set<String> set) => w.any(set.contains);

const _angleW = {'angle', 'rotation', 'hue', 'phase', 'heading', 'direction', 'twist', 'spin', 'theta', 'turn'};
const _shareW = {
  'mix', 'amount', 'strength', 'opacity', 'intensity', 'blend', 'threshold', 'knee', 'falloff', 'softness', 'fade',
  'progress', 'coverage', 'density', 'wet', 'dry', 'roughness', 'tint', 'feather', 'smoothness',
};
const _upW = {'height', 'rise', 'lift', 'up', 'top', 'vertical'};
const _downW = {'drop', 'gravity', 'down', 'sink', 'fall', 'bottom'};
const _xW = {'width', 'horizontal', 'left', 'right'};
const _toggleW = {'is', 'use', 'enable', 'enabled', 'invert', 'flip', 'clamp', 'additive', 'pin', 'mirror', 'loop', 'repeat'};
const _colourW = {'colour', 'tint', 'rgb', 'rgba', 'fill', 'ink', 'shade', 'background', 'foreground'};
const _sizeW = {'size', 'scale', 'aspect', 'extent', 'stretch'};
const _pxW = {'radius', 'size', 'width', 'height', 'distance', 'length', 'offset', 'spread', 'blur'};

double? _num(String? s) => s == null ? null : double.tryParse(s);

/// The guess for one field. Explicit attributes win; then the type; then the name; then the shape of the range.
Param infer(Decl d) {
  final w = paramWords(d.name), a = d.attrs;
  final range = a['range'];
  var min = _num(range?.elementAtOrNull(0)), max = _num(range?.elementAtOrNull(1));
  var unit = a['unit']?.firstOrNull ?? '';
  final def = a['default'];
  var d0 = _num(def?.elementAtOrNull(0)) ?? 0, d1 = _num(def?.elementAtOrNull(1)) ?? d0;
  final lab = a['label']?.join(', ') ?? paramLabel(w);
  final ui = a['ui']?.firstOrNull;
  final whole = d.type == WType.i32 || d.type == WType.u32;
  int dec(double? lo, double? hi) {
    if (whole) return 0;
    if (a['step']?.firstOrNull case final s?) return s.contains('.') ? s.split('.').last.length : 0;
    final span = lo != null && hi != null ? hi - lo : 100;
    return span <= 4 ? 2 : (span <= 40 ? 1 : 0);
  }

  Param p(ParamLook look, String why, {double? lo, double? hi, double scale = 1, bool up = true, String? u, List<String> opts = const []}) {
    final l = lo ?? min, h = hi ?? max;
    return Param(
      id: d.name,
      label: lab,
      look: look,
      type: d.type,
      min: l,
      max: h,
      def: d0 / scale,
      defY: d1 / scale,
      unit: u ?? unit,
      dec: scale == 1 ? dec(l, h) : (look == ParamLook.percent || look == ParamLook.angle ? 0 : 1),
      options: opts,
      advanced: d.advanced,
      upIncreases: up,
      toShader: scale,
      why: why,
    );
  }

  if (a['enum'] case final o? when o.isNotEmpty) return p(ParamLook.choice, 'listed options', lo: 0, hi: o.length - 1.0, opts: o);
  if (ui != null) {
    final look = ParamLook.values.where((l) => l.name == ui).firstOrNull;
    if (look != null) return p(look, 'asked for @ui($ui)');
  }
  if (d.type == WType.boolean || a.containsKey('toggle') || (whole && _any(w, _toggleW) && (max == null || max <= 1))) {
    return p(ParamLook.toggle, 'on/off', lo: 0, hi: 1);
  }
  if (d.type == WType.vec3 || d.type == WType.vec4) return p(ParamLook.colour, _any(w, _colourW) ? 'colour name' : 'a 3–4 component value');
  if (d.type == WType.vec2) {
    if (_any(w, _sizeW)) return p(ParamLook.size, 'a 2D size', lo: min ?? 0, u: unit.isEmpty && _any(w, {'size'}) ? 'px' : null);
    final uv = _any(w, {'center', 'centre', 'origin', 'focus', 'uv', 'pivot'});
    return p(ParamLook.point, 'a 2D point', lo: min ?? (uv ? 0 : -1), hi: max ?? 1);
  }
  if (_any(w, {'seed'})) return p(ParamLook.seed, 'a random seed', lo: min ?? 0, hi: max ?? 9999);
  final rad = unit == 'rad' || (min != null && max != null && (max - math.pi).abs() < .01 && (min + math.pi).abs() < .01);
  if (unit == '°' || unit == 'deg' || rad || _any(w, _angleW)) {
    final s = rad ? math.pi / 180 : 1.0;
    return p(
      ParamLook.angle,
      rad ? 'radians, shown in degrees' : 'an angle',
      lo: min == null ? -180 : min / s,
      hi: max == null ? 180 : max / s,
      scale: s,
      u: '°',
    );
  }
  final unitShare = min == 0 && max == 1 && _any(w, _shareW);
  if (unit == '%' || unitShare) {
    return p(
      ParamLook.percent,
      unitShare ? '0–1 share, shown as %' : 'a percentage',
      lo: unitShare ? 0 : (min ?? 0),
      hi: unitShare ? 100 : (max ?? 100),
      scale: unitShare ? .01 : 1,
      u: '%',
    );
  }
  if (whole && min != null && max != null && max - min <= 12) return p(ParamLook.stepper, 'a small count');
  if (unit.isEmpty && _any(w, _pxW)) unit = 'px';
  final last = w.lastOrNull;
  if (last == 'y' || _any(w, _upW) || _any(w, _downW)) {
    return p(ParamLook.vertical, 'a vertical amount', up: !_any(w, _downW));
  }
  if (last == 'x' || _any(w, _xW)) return p(ParamLook.horizontal, 'a horizontal amount');
  if (min != null && max != null) {
    if (min < 0 && max > 0 && (min.abs() - max).abs() <= max * .5) return p(ParamLook.bipolar, 'a range around zero');
    return p(ParamLook.bar, 'a range');
  }
  return p(ParamLook.scrub, 'no range');
}

/// Parse and infer in one go.
List<Param> readParams(String wgsl) => [for (final d in parseWgsl(wgsl)) infer(d)];
