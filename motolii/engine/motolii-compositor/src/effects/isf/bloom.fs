/*{
	"DESCRIPTION": "Bloom/glow: extracts the parts of the image above a luminance threshold, blurs them with a 5x5 Gaussian-weighted kernel, and adds the result back additively. Single-pass approximation of the classic bright-pass -> separable-blur -> additive-composite bloom pipeline (the 5x5 kernel here is the outer product of the same [0.0625, 0.25, 0.375, 0.25, 0.0625] 1D weights a two-pass horizontal+vertical Gaussian would use, so the blur itself is not a lower-quality shortcut -- only the boundary/edge-clamping behavior differs slightly from a true two-pass separable blur, see the Rust host module doc for the exact difference).",
	"CREDIT": "Motolii -- hand-authored against the ISF 2.0 spec (isf.video). No network access was available in the sandbox this file was written in to pull a live community .fs file, so this is not literally copy-pasted from an existing repository -- see effects/isf/mod.rs module doc for what that means for the evidence this file is meant to produce. The threshold/blur/additive-composite algorithm and its default values are a direct port of this host's own pre-existing hand-written bloom (`effects/glow.rs`, portedfrom `spikes/m5-known-implementation/M5-R0/src/glow.rs`) -- reused here as a numerical reference/sanity-check for what 'a proper bloom' should look like, not as reused Rust code or pipeline shape.",
	"CATEGORIES": [
		"Stylize",
		"Blur"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		},
		{
			"NAME": "threshold",
			"TYPE": "float",
			"DEFAULT": 1.0,
			"MIN": 0.0,
			"MAX": 4.0
		},
		{
			"NAME": "intensity",
			"TYPE": "float",
			"DEFAULT": 0.75,
			"MIN": 0.0,
			"MAX": 4.0
		},
		{
			"NAME": "radius",
			"TYPE": "float",
			"DEFAULT": 1.0,
			"MIN": 1.0,
			"MAX": 8.0
		}
	]
}*/

// 5x5 kernel = outer product of the 5-tap 1D weights [0.0625, 0.25, 0.375, 0.25, 0.0625]
// (sums to 1.0; same weights this host's own `effects/glow.rs` uses for its two
// separable 1D passes -- see JSON CREDIT above).
const float WEIGHTS[5] = float[5](0.0625, 0.25, 0.375, 0.25, 0.0625);

vec4 brightPass(vec4 value) {
	float luminance = dot(value.rgb, vec3(0.2126, 0.7152, 0.0722));
	float contribution = max(luminance - threshold, 0.0) / max(luminance, 0.000001);
	return value * contribution;
}

void main() {
	vec2 texel = 1.0 / RENDERSIZE;
	float step_px = max(1.0, floor(radius + 0.5));

	vec4 source = IMG_THIS_PIXEL(inputImage);
	vec4 bloom = vec4(0.0);
	for (int j = -2; j <= 2; j++) {
		for (int i = -2; i <= 2; i++) {
			vec2 offset = vec2(float(i), float(j)) * step_px * texel;
			vec4 tap = IMG_NORM_PIXEL(inputImage, isf_FragNormCoord + offset);
			bloom += brightPass(tap) * (WEIGHTS[i + 2] * WEIGHTS[j + 2]);
		}
	}
	bloom *= intensity;

	// Premultiplied-safe additive composite (same formula as this host's
	// `effects/glow.rs` composite_fs -- additive blending is linear in
	// premultiplied space, so no un/re-premultiply step is needed here, unlike
	// a non-linear op like contrast would need).
	vec3 outRgb = source.rgb + bloom.rgb;
	float outA = source.a + bloom.a * (1.0 - source.a);
	gl_FragColor = clamp(vec4(outRgb, outA), vec4(0.0), vec4(1.0));
}
