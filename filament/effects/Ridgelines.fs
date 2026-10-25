/*{
    "CATEGORIES": [
        "Generator", "Noise"
    ],
    "CREDIT": "by VIDVOX (simplex by Ashima Arts / Stefan Gustavson)",
    "DESCRIPTION": "Ridged multifractal noise — turns simplex noise's smooth zero-crossings into sharp ridgelines.",
    "ISFVSN": "2",
    "INPUTS": [
        {
            "DEFAULT": 4.0,
            "LABEL": "Scale",
            "MAX": 50.0,
            "MIN": 1.0,
            "NAME": "scale",
            "TYPE": "float"
        },
        {
            "DEFAULT": 4,
            "LABEL": "Octaves",
            "LABELS": [
                "1",
                "2",
                "3",
                "4",
                "5",
                "6"
            ],
            "NAME": "octaves",
            "TYPE": "long",
            "VALUES": [
                1,
                2,
                3,
                4,
                5,
                6
            ]
        },
        {
            "DEFAULT": 0.5,
            "LABEL": "Persistence",
            "MAX": 1.0,
            "MIN": 0.0,
            "NAME": "persistence",
            "TYPE": "float"
        },
        {
            "DEFAULT": 2.0,
            "LABEL": "Lacunarity",
            "MAX": 4.0,
            "MIN": 1.0,
            "NAME": "lacunarity",
            "TYPE": "float"
        },
        {
            "DEFAULT": 2.0,
            "LABEL": "Sharpness",
            "MAX": 4.0,
            "MIN": 1.0,
            "NAME": "sharpness",
            "TYPE": "float"
        },
        {
            "DEFAULT": 2.0,
            "LABEL": "Cascade Gain",
            "MAX": 4.0,
            "MIN": 0.0,
            "NAME": "gain",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.0,
            "LABEL": "Speed",
            "MAX": 2.0,
            "MIN": 0.0,
            "NAME": "speed",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1.5,
            "LABEL": "Contrast",
            "MAX": 3.0,
            "MIN": 0.0,
            "NAME": "contrast",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.0,
            "LABEL": "Seed",
            "MAX": 100.0,
            "MIN": 0.0,
            "NAME": "seed",
            "TYPE": "float"
        },
        {
            "DEFAULT": [
                0.0,
                0.0,
                0.0,
                1.0
            ],
            "LABEL": "Low Color",
            "NAME": "colorLow",
            "TYPE": "color"
        },
        {
            "DEFAULT": [
                1.0,
                1.0,
                1.0,
                1.0
            ],
            "LABEL": "High Color",
            "NAME": "colorHigh",
            "TYPE": "color"
        }
    ]
}*/


//	multifractal cascade after Musgrave 1998



//	2D simplex noise. Output range approximately [-1, 1].
//	Standard Ashima Arts / Stefan Gustavson implementation (MIT).
vec3 permute(vec3 x)	{
    return mod(((x * 34.0) + 1.0) * x, 289.0);
}

float snoise(vec2 v)	{
    const vec4 C = vec4( 0.211324865405187,
                         0.366025403784439,
                        -0.577350269189626,
                         0.024390243902439);

    vec2 i  = floor(v + dot(v, C.yy));
    vec2 x0 = v - i + dot(i, C.xx);

    vec2 i1  = (x0.x > x0.y) ? vec2(1.0, 0.0) : vec2(0.0, 1.0);
    vec4 x12 = x0.xyxy + C.xxzz;
    x12.xy  -= i1;

    i = mod(i, 289.0);
    vec3 p = permute(permute(i.y + vec3(0.0, i1.y, 1.0))
                   + i.x + vec3(0.0, i1.x, 1.0));

    vec3 m = max(0.5 - vec3(dot(x0,    x0),
                            dot(x12.xy, x12.xy),
                            dot(x12.zw, x12.zw)), 0.0);
    m = m * m;
    m = m * m;

    vec3 x  = 2.0 * fract(p * C.www) - 1.0;
    vec3 h  = abs(x) - 0.5;
    vec3 ox = floor(x + 0.5);
    vec3 a0 = x - ox;
    m *= 1.79284291400159 - 0.85373472095314 * (a0*a0 + h*h);

    vec3 g;
    g.x  = a0.x  * x0.x  + h.x  * x0.y;
    g.yz = a0.yz * x12.xz + h.yz * x12.yw;
    return 130.0 * dot(m, g);
}


//	Per-octave drift direction (golden-ratio rotation so headings never align).
vec2 octaveDir(int oct)	{
	float a = float(oct) * 1.61803398875 + seed * 0.1;
	return vec2(cos(a), sin(a));
}


void main()	{
	vec2 uv = isf_FragNormCoord;
	
	//	Aspect-correct so noise features render visually round on any frame.
	float aspect = RENDERSIZE.x / RENDERSIZE.y;
	vec2  pos    = vec2(uv.x * aspect, uv.y);
	
	//	Stable per-shader offset from the seed so different seeds give
	//	completely different patterns rather than translated versions of one.
	vec2 seedOffset = vec2(seed * 3.14159, seed * 2.71828);
	
	//	Musgrave-style multifractal accumulation
	//
	//	Each octave: signal = pow(1 - |simplex|, sharpness)
	//	                        → sharp ridges where simplex crosses zero
	//	Cascade:    signal *= weight from previous octave
	//	                        → higher-frequency detail only appears WHERE
	//	                          lower-frequency ridges already exist,
	//	                          which is what creates the natural "detail
	//	                          along the ridges, smooth in the valleys"
	//	                          terrain look.
	float total     = 0.0;
	//	first octave gets full weight
	float weight    = 1.0;
	float amplitude = 1.0;
	float frequency = scale;
	float maxAmp    = 0.0;
	
	//	GLSL wants a constant loop bound; iterate the max and break.
	for (int i = 0; i < 6; ++i)	{
		if (i >= octaves) break;
	
		vec2 samplePos = pos * frequency
					   + seedOffset
					   + octaveDir(i) * TIME * speed;
	
		//	Ridge function: |n| has valleys at the noise zero-crossings.
		//	1-|n| inverts those to ridges; pow() sharpens them.
		float signal = pow(1.0 - abs(snoise(samplePos)), sharpness);
	
		//	Multifractal cascade: this octave's contribution is gated by the
		//	previous octave's ridge strength. The gate STRENGTH (how much
		//	the previous octave matters) is driven by `gain`:
		//	  gain == 0  →  cascadeFactor = 0  →  mix picks 1.0  →  no gate,
		//	                plain ridged fBM with persistence falloff.
		//	  gain >= 4  →  cascadeFactor = 1  →  mix picks weight  →  full
		//	                Musgrave cascade (detail only on lower-freq ridges).
		//	Without this mix, `signal *= weight` would zero out every octave
		//	after the first when gain=0 (weight = clamp(signal*0,0,1) = 0),
		//	collapsing the output to a single octave.
		float cascadeFactor = clamp(gain * 0.25, 0.0, 1.0);
		signal *= mix(1.0, weight, cascadeFactor);
	
		total  += signal * amplitude;
		maxAmp += amplitude;
	
		//	Update weight for next iteration from the current signal.
		weight = clamp(signal * gain, 0.0, 1.0);
	
		amplitude *= persistence;
		frequency *= lacunarity;
	}
	
	//	Normalize to roughly [0, 1] regardless of octave count / persistence.
	float t = total / max(maxAmp, 0.000001);
	t = clamp(t, 0.0, 1.0);
	
	//	Contrast around mid value.
	t = (t - 0.5) * contrast + 0.5;
	t = clamp(t, 0.0, 1.0);
	
	vec3 rgb = mix(colorLow.rgb, colorHigh.rgb, t);
	float a  = mix(colorLow.a,   colorHigh.a,   t);
	
	gl_FragColor = vec4(rgb, a);
}
