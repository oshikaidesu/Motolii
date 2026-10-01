/*{
  "ID": "motolii.overkill",
  "LABEL": "Overkill",
  "STAGE": "pass",
  "DESCRIPTION": "Too much on purpose, the pixel side: barrel curve, chromatic split that grows with radius, scanlines, a ripple ring that walks out from the centre, grain, and a vignette — one pass, all dials open. Pair with Overload (the block side)",
  "CATEGORIES": ["Stylize"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "curve", "LABEL": "Curve", "TYPE": "float", "DEFAULT": 0.18, "MIN": -1.0, "MAX": 1.0 },
    { "NAME": "split", "LABEL": "Split", "TYPE": "float", "DEFAULT": 0.012, "MIN": 0.0, "MAX": 0.1 },
    { "NAME": "scan", "LABEL": "Scanlines", "TYPE": "float", "DEFAULT": 0.35, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "ripple", "LABEL": "Ripple", "TYPE": "float", "DEFAULT": 0.02, "MIN": 0.0, "MAX": 0.2 },
    { "NAME": "grain", "LABEL": "Grain", "TYPE": "float", "DEFAULT": 0.12, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "vignette", "LABEL": "Vignette", "TYPE": "float", "DEFAULT": 0.7, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "speed", "LABEL": "Speed", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 10.0 }
  ]
}*/

float hash(vec2 p) { return fract(sin(dot(p, vec2(12.9898, 78.233))) * 43758.5453); }

void main() {
	vec2 uv = isf_FragNormCoord;
	vec2 c = uv - 0.5;
	float r2 = dot(c, c);
	// barrel / pincushion
	vec2 warped = 0.5 + c * (1.0 + curve * r2 * 2.0);
	// ripple ring walking out from the centre
	float ring = sin(length(c) * 40.0 - TIME * speed * 6.0);
	warped += normalize(c + 1e-6) * ring * ripple * smoothstep(0.0, 0.5, length(c));
	// chromatic split grows with radius
	vec2 dir = normalize(c + 1e-6) * split * (0.3 + r2 * 4.0);
	float rr = IMG_NORM_PIXEL(inputImage, warped + dir).r;
	float gg = IMG_NORM_PIXEL(inputImage, warped).g;
	float bb = IMG_NORM_PIXEL(inputImage, warped - dir).b;
	float aa = IMG_NORM_PIXEL(inputImage, warped).a;
	vec3 col = vec3(rr, gg, bb);
	// scanlines
	float line = 0.5 + 0.5 * sin(uv.y * RENDERSIZE.y * 1.5 + TIME * speed * 2.0);
	col *= 1.0 - scan * 0.5 * line;
	// grain
	col += (hash(uv * RENDERSIZE + fract(TIME * speed) * 91.0) - 0.5) * grain;
	// vignette
	col *= 1.0 - vignette * r2 * 1.6;
	gl_FragColor = vec4(clamp(col, 0.0, 1.0), aa);
}
