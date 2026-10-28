/*{
    "DESCRIPTION": "Borders round what is seen, each round the one before (Alight Motion's Multi-Border): pixi-filters' OutlineFilter (MIT, Copyright (c) 2013-2025 Mathew Groves, Chad Engler; pixi-filters 6.1.5, lib/outline/outline.mjs), its fragment shader's lines as they are, run once a pass on the pass before; its uniforms ISF's inputs: the thickness in pixels made a share of the picture as its apply() makes it (thickness / the input's size), the angle step from quality as its getAngleStep (2 pi / max(quality x 100, 1)). A thickness of 0 leaves that border out.",
    "CREDIT": "pixi-filters OutlineFilter (MIT)",
    "CATEGORIES": [
        "Stylize"
    ],
    "ISFVSN": "2",
    "INPUTS": [
        { "NAME": "inputImage", "TYPE": "image" },
        { "NAME": "thickness1", "LABEL": "Border 1", "TYPE": "float", "DEFAULT": 8, "MIN": 0, "MAX": 60 },
        { "NAME": "color1", "LABEL": "Colour 1", "TYPE": "color", "DEFAULT": [ 1, 1, 1, 1 ] },
        { "NAME": "thickness2", "LABEL": "Border 2", "TYPE": "float", "DEFAULT": 8, "MIN": 0, "MAX": 60 },
        { "NAME": "color2", "LABEL": "Colour 2", "TYPE": "color", "DEFAULT": [ 1, 0.2, 0.4, 1 ] },
        { "NAME": "thickness3", "LABEL": "Border 3", "TYPE": "float", "DEFAULT": 0, "MIN": 0, "MAX": 60 },
        { "NAME": "color3", "LABEL": "Colour 3", "TYPE": "color", "DEFAULT": [ 0.2, 0.4, 1, 1 ] },
        { "NAME": "alpha", "TYPE": "float", "DEFAULT": 1, "MIN": 0, "MAX": 1 },
        { "NAME": "quality", "TYPE": "float", "DEFAULT": 0.1, "MIN": 0.01, "MAX": 1 }
    ],
    "PASSES": [
        { "TARGET": "one" },
        { "TARGET": "two" },
        { }
    ]
}*/

const float DOUBLE_PI = 2. * 3.14159265358979323846264;

// (the pass's own: which picture it reads, and its border)
vec4 uTextureAt(vec2 p) { return PASSINDEX == 0 ? IMG_NORM_PIXEL(inputImage, p) : PASSINDEX == 1 ? IMG_NORM_PIXEL(one, p) : IMG_NORM_PIXEL(two, p); }
vec2 uThickness() { return vec2(PASSINDEX == 0 ? thickness1 : PASSINDEX == 1 ? thickness2 : thickness3) / RENDERSIZE; }
vec3 uColor() { return PASSINDEX == 0 ? color1.rgb : PASSINDEX == 1 ? color2.rgb : color3.rgb; }

float outlineMaxAlphaAtPos(vec2 pos) {
    if (uThickness().x == 0. || uThickness().y == 0.) {
        return 0.;
    }

    vec4 displacedColor;
    vec2 displacedPos;
    float maxAlpha = 0.;
    float ANGLE_STEP = DOUBLE_PI / max(quality * 100., 1.);

    for (float angle = 0.; angle <= DOUBLE_PI; angle += ANGLE_STEP) {
        displacedPos.x = pos.x + uThickness().x * cos(angle);
        displacedPos.y = pos.y + uThickness().y * sin(angle);
        displacedColor = uTextureAt(clamp(displacedPos, vec2(0.), vec2(1.)));
        maxAlpha = max(maxAlpha, displacedColor.a);
    }

    return maxAlpha;
}

void main(void) {
    vec4 sourceColor = uTextureAt(isf_FragNormCoord);
    vec4 contentColor = sourceColor;
    float outlineAlpha = alpha * outlineMaxAlphaAtPos(isf_FragNormCoord.xy) * (1.-sourceColor.a);
    vec4 outlineColor = vec4(uColor() * outlineAlpha, outlineAlpha);
    gl_FragColor = contentColor + outlineColor;
}
