import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';

final kEffects = [
  Fx('blur', 'Blur', tone: const Color(0xFF5BC0EB), mark: CardMark.blur, '''
struct Params {
  /// @range(0, 200) @default(12)
  radius: f32,
  /// @range(-180, 180)
  angle: f32,
  /// @range(0, 1) @default(1)
  mix: f32,
  /// @enum(Low, Med, High) @default(1)
  quality: u32,
  // @advanced
  /// @toggle @default(1) @label(Repeat edges)
  edge: u32,
  /// @range(0.1, 4) @default(1)
  gamma: f32,
  /// @range(-100, 100) @unit(px)
  offset: vec2f,
}
'''),
  // a third-party shader nobody wrote rows for: camelCase names, radians, a uv point, a size, a long enum
  Fx('kaleido', 'Kaleido', vendor: 'Lumen FX', tone: const Color(0xFFB8A1FF), mark: CardMark.shader, '''
struct KaleidoUniforms {
  segments: i32,      // @range(2, 12) @default(6)
  twist: f32,         // @range(-3.14159, 3.14159)
  center: vec2f,      // @default(0.5, 0.5)
  tileSize: vec2f,    // @range(8, 512) @default(64, 48)
  driftY: f32,        // @range(-1, 1)
  strength: f32,      // @range(0, 1) @default(0.8)
  // @advanced
  zoom: f32,          // @range(0.25, 4) @default(1)
  blendMode: u32,     // @enum(Normal, Add, Screen, Multiply, Overlay, Difference)
  edgeColor: vec4f,   // @default(0.1, 0.1, 0.12, 1)
  mirror: u32,
  seed: u32,
  time: f32,
}
'''),
  Fx('glow', 'Glow', tone: const Color(0xFFFFB86B), mark: CardMark.glow, '''
struct Params {
  /// @range(0, 1) @default(0.6)
  thr: f32,
  /// @range(0, 300) @default(40)
  size: f32,
  /// @range(0, 400) @unit(%) @default(100)
  gain: f32,
  /// @range(-50, 50) @unit(px)
  lift: f32,
  // @advanced
  /// @range(0, 1) @default(0.2)
  knee: f32,
  /// @default(1, 0.85, 0.6, 1)
  tint: vec4f,
  /// @range(0.1, 4) @default(1)
  aspect: f32,
  /// @range(0, 1) @default(0.5)
  falloff: f32,
  /// @default(1)
  additive: u32,
}
'''),
  Fx('color', 'Colour', tone: const Color(0xFFE974AB), mark: CardMark.colour, '''
struct Params {
  /// @range(-4, 4)
  exp: f32,
  /// @range(-100, 100)
  con: f32,
  /// @range(0, 200) @unit(%) @default(100)
  sat: f32,
  hue: f32,
  // @advanced
  /// @range(-100, 100)
  temp: f32,
  /// @range(-100, 100)
  tintc: f32,
  /// @range(0.1, 4) @default(1)
  gam: f32,
  clampc: u32,
}
'''),
  Fx('warp', 'Distort', tone: const Color(0xFF7DD5B1), mark: CardMark.distort, '''
struct Params {
  /// @range(-100, 100) @unit(%) @default(20)
  amt: f32,
  /// @range(0, 40) @default(6)
  freq: f32,
  /// @range(-10, 10) @default(1)
  speed: f32,
  // @advanced
  /// @range(0, 999)
  seed: u32,
  /// @range(1, 8) @default(2)
  oct: i32,
  /// @range(1, 4) @default(2)
  lac: f32,
  /// @range(0, 1) @default(0.5)
  rough: f32,
  pin: u32,
}
'''),
];
