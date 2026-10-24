import 'dart:math' as math;

import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/sets/inspector/param_infer.dart';

Map<String, Param> read(String body) => {for (final p in readParams('struct P {\n$body\n}')) p.id: p};

void main() {
  test('reads fields, both comment styles, and the advanced marker', () {
    final d = parseWgsl('''
struct Params {
  /// @range(0, 200) @unit(px)
  radius: f32,
  mix: f32, // @range(0, 1) @default(0.5)
  // @advanced
  @align(16) center: vec2<f32>,
  junk line
}''');
    expect(d.map((e) => e.name), ['radius', 'mix', 'center']);
    expect(d[0].attrs['range'], ['0', '200']);
    expect(d[1].attrs['default'], ['0.5']);
    expect(d[2].type, WType.vec2);
    expect([for (final e in d) e.advanced], [false, false, true]);
  });

  test('every look is reachable from names, types and ranges alone', () {
    final p = read('''
  hueShift: f32,
  twist: f32,          // @range(-3.14159, 3.14159)
  mix: f32,            // @range(0, 1) @default(0.25)
  gain: f32,           // @unit(%) @range(0, 400)
  offset_y: f32,       // @range(-50, 50)
  dropDistance: f32,   // @range(0, 80)
  width: f32,          // @range(0, 100)
  contrast: f32,       // @range(-100, 100)
  zoom: f32,           // @range(0.25, 4)
  time: f32,
  segments: i32,       // @range(2, 12)
  seed: u32,
  center: vec2f,
  tileSize: vec2f,
  edgeColor: vec4f,
  invert: u32,
  mode: u32,           // @enum(A, B, C)
''');
    expect({for (final e in p.entries) e.key: e.value.look}, {
      'hueShift': ParamLook.angle,
      'twist': ParamLook.angle,
      'mix': ParamLook.percent,
      'gain': ParamLook.percent,
      'offset_y': ParamLook.vertical,
      'dropDistance': ParamLook.vertical,
      'width': ParamLook.horizontal,
      'contrast': ParamLook.bipolar,
      'zoom': ParamLook.bar,
      'time': ParamLook.scrub,
      'segments': ParamLook.stepper,
      'seed': ParamLook.seed,
      'center': ParamLook.point,
      'tileSize': ParamLook.size,
      'edgeColor': ParamLook.colour,
      'invert': ParamLook.toggle,
      'mode': ParamLook.choice,
    });
  });

  test('display units: a 0–1 share reads as %, radians read as degrees, and both map back for the shader', () {
    final p = read('''
  mix: f32,     // @range(0, 1) @default(0.25)
  twist: f32,   // @range(-3.14159, 3.14159) @default(1.5708)
''');
    expect(p['mix']!.def, closeTo(25, 1e-9));
    expect(p['mix']!.max, 100);
    expect(p['mix']!.def * p['mix']!.toShader, closeTo(.25, 1e-9));
    expect(p['twist']!.def, closeTo(90, .01));
    expect(p['twist']!.max, closeTo(180, .01));
    expect(p['twist']!.unit, '°');
    expect(p['twist']!.def * p['twist']!.toShader, closeTo(math.pi / 2, 1e-4));
  });

  test('a vertical value knows which way is up', () {
    final p = read('''
  rise: f32,
  gravity: f32,
''');
    expect(p['rise']!.upIncreases, isTrue);
    expect(p['gravity']!.upIncreases, isFalse);
  });

  test('labels are spelled out and explicit attributes win', () {
    final p = read('''
  thr: f32,          // @range(0, 1)
  blurRadius: f32,
  angle: f32,        // @ui(bar) @range(0, 10)
  edge: u32,         // @toggle @label(Repeat edges)
''');
    expect(p['thr']!.label, 'Threshold');
    expect(p['blurRadius']!.label, 'Blur radius');
    expect(p['blurRadius']!.unit, 'px');
    expect(p['angle']!.look, ParamLook.bar);
    expect(p['edge']!.look, ParamLook.toggle);
    expect(p['edge']!.label, 'Repeat edges');
  });
}
