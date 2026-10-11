// Draws the caustic as triangles. Each light-view cell (two triangles) is a patch of incoming light;
// its refracted corners (newUv) form a new triangle on the floor. Brightness = oldArea / newArea
// (Wallace-style: light is conserved, so a squeezed patch is brighter), one pass per wavelength band.
struct Hit {
  newUv: vec2<f32>,
  oldUv: vec2<f32>,
  valid: f32,
  transmission: f32,
  padding: vec2<f32>,
};
struct Uniforms {
  inverseLightViewProjection: mat4x4<f32>,
  lightViewProjection: mat4x4<f32>,
  receiverOriginWidth: vec4<f32>,
  receiverTangentHeight: vec4<f32>,
  receiverBitangentIntensity: vec4<f32>,
  receiverNormalRefractiveIndex: vec4<f32>,
  absorptionDispersion: vec4<f32>,
  targetSizes: vec4<f32>,
};
@group(0) @binding(0) var<storage, read> hits: array<Hit>;
@group(0) @binding(1) var<uniform> u: Uniforms;

// CIE 1931 2-degree CMFs integrated against D65 in six bands, normalized to Y = 1 (from luma.gl).
const WAVELENGTH_XYZ = array<vec3<f32>, 6>(
  vec3<f32>(0.03671527, 0.01357357, 0.00000000),
  vec3<f32>(0.30218842, 0.14160332, 0.00008598),
  vec3<f32>(0.36062779, 0.40239216, 0.00124922),
  vec3<f32>(0.07092146, 0.36745709, 0.04500246),
  vec3<f32>(0.08806719, 0.06954351, 0.58510595),
  vec3<f32>(0.09195043, 0.00543034, 0.45738513)
);

struct VOut {
  @builtin(position) position: vec4<f32>,
  @location(0) @interpolate(flat) color: vec4<f32>,
};

fn triArea(a: vec2<f32>, b: vec2<f32>, c: vec2<f32>) -> f32 {
  let e1 = b - a;
  let e2 = c - a;
  return 0.5 * abs(e1.x * e2.y - e1.y * e2.x);
}

fn build(instance: u32, vertexIndex: u32, footprint: bool) -> VOut {
  var out: VOut;
  out.position = vec4<f32>(2.0, 2.0, 2.0, 1.0);
  out.color = vec4<f32>(0.0);
  let n = u32(u.targetSizes.x);
  let cells = n - 1u;
  let band = instance % 6u;
  if (footprint && band != 0u) {
    return out;
  }
  let cell = instance / 6u;
  let cx = cell % cells;
  let cy = cell / cells;
  let tri = vertexIndex / 3u;
  let local = vertexIndex % 3u;
  var ox = array<u32, 3>(0u, 1u, 0u);
  var oy = array<u32, 3>(0u, 0u, 1u);
  if (tri == 1u) {
    ox = array<u32, 3>(1u, 1u, 0u);
    oy = array<u32, 3>(0u, 1u, 1u);
  }
  var h: array<Hit, 3>;
  for (var k = 0u; k < 3u; k++) {
    h[k] = hits[((cy + oy[k]) * n + (cx + ox[k])) * 6u + band];
  }
  // A triangle is drawn only if all three rays found their exit.
  let ok = footprint || (h[0].valid > 0.5 && h[1].valid > 0.5 && h[2].valid > 0.5);
  // The footprint (where this glass shades the floor) needs only the entry side to exist.
  let hasEntry = h[0].padding.x > 0.5 && h[1].padding.x > 0.5 && h[2].padding.x > 0.5;
  if (!ok || !hasEntry) {
    return out;
  }
  var uv = h[local].newUv;
  if (footprint) {
    uv = h[local].oldUv;
  }
  out.position = vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, 0.0, 1.0);
  if (footprint) {
    out.color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    return out;
  }
  let areaNew = triArea(h[0].newUv, h[1].newUv, h[2].newUv);
  let areaOld = triArea(h[0].oldUv, h[1].oldUv, h[2].oldUv);
  let ratio = min(areaOld / max(areaNew, 1e-10), 64.0);
  let transmission = (h[0].transmission + h[1].transmission + h[2].transmission) / 3.0;
  let intensity = u.receiverBitangentIntensity.w;
  out.color = vec4<f32>(WAVELENGTH_XYZ[band] * intensity * transmission * ratio, 0.0);
  return out;
}

@vertex fn vsCaustic(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VOut {
  return build(ii, vi, false);
}
@vertex fn vsFootprint(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VOut {
  return build(ii * 6u, vi, true);
}
@fragment fn fs(input: VOut) -> @location(0) vec4<f32> {
  return input.color;
}
