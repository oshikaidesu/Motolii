/*{
 "DESCRIPTION": "Sashimono Edit's 模様で塗る (brush_fill, 色; MIT, src/sashimono/effects/builtin.py), made an ISF effect by filament/effects/from_sashimono.py",
 "CREDIT": "Sashimono Edit (MIT)",
 "CATEGORIES": [
  "Color"
 ],
 "ISFVSN": "2",
 "INPUTS": [
  {
   "NAME": "inputImage",
   "TYPE": "image"
  },
  {
   "NAME": "pattern",
   "LABEL": "模様",
   "TYPE": "long",
   "VALUES": [
    0,
    1,
    2,
    3,
    4,
    5,
    6,
    7
   ],
   "LABELS": [
    "単色",
    "線形グラデーション",
    "円形グラデーション",
    "山形グラデーション",
    "ストライプ",
    "水玉",
    "格子",
    "ノイズ"
   ],
   "DEFAULT": 1
  },
  {
   "NAME": "blend",
   "LABEL": "合成",
   "TYPE": "long",
   "VALUES": [
    0,
    1,
    2,
    3,
    4,
    5,
    6,
    7,
    8,
    9,
    10,
    11,
    12,
    13,
    14,
    15,
    16,
    17,
    18,
    19,
    20,
    21,
    22,
    23,
    24,
    25
   ],
   "LABELS": [
    "通常",
    "加算",
    "減算",
    "乗算",
    "スクリーン",
    "オーバーレイ",
    "ソフトライト",
    "ハードライト",
    "覆い焼きカラー",
    "焼き込みカラー",
    "比較(明)",
    "比較(暗)",
    "カラー比較(明)",
    "カラー比較(暗)",
    "差の絶対値",
    "除外",
    "焼き込みリニア",
    "リニアライト",
    "ビビッドライト",
    "ピンライト",
    "ハードミックス",
    "除算",
    "色相",
    "彩度",
    "カラー",
    "輝度"
   ],
   "DEFAULT": 0
  },
  {
   "NAME": "opacity",
   "LABEL": "濃さ",
   "TYPE": "float",
   "DEFAULT": 100,
   "MIN": 0,
   "MAX": 100
  },
  {
   "NAME": "pattern_only",
   "LABEL": "模様だけで塗る",
   "TYPE": "bool",
   "DEFAULT": false
  },
  {
   "NAME": "key_only",
   "LABEL": "目印の色の所だけ塗る",
   "TYPE": "bool",
   "DEFAULT": false
  },
  {
   "NAME": "key_color",
   "LABEL": "目印の色",
   "TYPE": "color",
   "DEFAULT": [
    1.0,
    0.0,
    1.0,
    1.0
   ]
  },
  {
   "NAME": "keep_color",
   "LABEL": "目印と一緒に描いた線の色",
   "TYPE": "color",
   "DEFAULT": [
    0.0,
    0.0,
    0.0,
    0.0
   ]
  },
  {
   "NAME": "noise_mask",
   "LABEL": "ノイズで薄める",
   "TYPE": "long",
   "VALUES": [
    0,
    1,
    2
   ],
   "LABELS": [
    "使わない",
    "不透明度",
    "色"
   ],
   "DEFAULT": 0
  },
  {
   "NAME": "stops",
   "LABEL": "色の数",
   "TYPE": "long",
   "DEFAULT": 2,
   "MIN": 1,
   "MAX": 16
  },
  {
   "NAME": "color0",
   "LABEL": "色 1",
   "TYPE": "color",
   "DEFAULT": [
    1.0,
    1.0,
    1.0,
    1.0
   ]
  },
  {
   "NAME": "offset0",
   "LABEL": "位置 1",
   "TYPE": "float",
   "DEFAULT": 0,
   "MIN": 0,
   "MAX": 1
  },
  {
   "NAME": "color1",
   "LABEL": "色 2",
   "TYPE": "color",
   "DEFAULT": [
    0.0,
    0.0,
    0.0,
    1.0
   ]
  },
  {
   "NAME": "offset1",
   "LABEL": "位置 2",
   "TYPE": "float",
   "DEFAULT": 1,
   "MIN": 0,
   "MAX": 1
  },
  {
   "NAME": "color2",
   "LABEL": "色 3",
   "TYPE": "color",
   "DEFAULT": [
    0.0,
    0.0,
    0.0,
    1.0
   ]
  },
  {
   "NAME": "offset2",
   "LABEL": "位置 3",
   "TYPE": "float",
   "DEFAULT": 1,
   "MIN": 0,
   "MAX": 1
  },
  {
   "NAME": "color3",
   "LABEL": "色 4",
   "TYPE": "color",
   "DEFAULT": [
    0.0,
    0.0,
    0.0,
    1.0
   ]
  },
  {
   "NAME": "offset3",
   "LABEL": "位置 4",
   "TYPE": "float",
   "DEFAULT": 1,
   "MIN": 0,
   "MAX": 1
  },
  {
   "NAME": "color4",
   "LABEL": "色 5",
   "TYPE": "color",
   "DEFAULT": [
    0.0,
    0.0,
    0.0,
    1.0
   ]
  },
  {
   "NAME": "offset4",
   "LABEL": "位置 5",
   "TYPE": "float",
   "DEFAULT": 1,
   "MIN": 0,
   "MAX": 1
  },
  {
   "NAME": "color5",
   "LABEL": "色 6",
   "TYPE": "color",
   "DEFAULT": [
    0.0,
    0.0,
    0.0,
    1.0
   ]
  },
  {
   "NAME": "offset5",
   "LABEL": "位置 6",
   "TYPE": "float",
   "DEFAULT": 1,
   "MIN": 0,
   "MAX": 1
  },
  {
   "NAME": "color6",
   "LABEL": "色 7",
   "TYPE": "color",
   "DEFAULT": [
    0.0,
    0.0,
    0.0,
    1.0
   ]
  },
  {
   "NAME": "offset6",
   "LABEL": "位置 7",
   "TYPE": "float",
   "DEFAULT": 1,
   "MIN": 0,
   "MAX": 1
  },
  {
   "NAME": "color7",
   "LABEL": "色 8",
   "TYPE": "color",
   "DEFAULT": [
    0.0,
    0.0,
    0.0,
    1.0
   ]
  },
  {
   "NAME": "offset7",
   "LABEL": "位置 8",
   "TYPE": "float",
   "DEFAULT": 1,
   "MIN": 0,
   "MAX": 1
  },
  {
   "NAME": "color8",
   "LABEL": "色 9",
   "TYPE": "color",
   "DEFAULT": [
    0.0,
    0.0,
    0.0,
    1.0
   ]
  },
  {
   "NAME": "offset8",
   "LABEL": "位置 9",
   "TYPE": "float",
   "DEFAULT": 1,
   "MIN": 0,
   "MAX": 1
  },
  {
   "NAME": "color9",
   "LABEL": "色 10",
   "TYPE": "color",
   "DEFAULT": [
    0.0,
    0.0,
    0.0,
    1.0
   ]
  },
  {
   "NAME": "offset9",
   "LABEL": "位置 10",
   "TYPE": "float",
   "DEFAULT": 1,
   "MIN": 0,
   "MAX": 1
  },
  {
   "NAME": "color10",
   "LABEL": "色 11",
   "TYPE": "color",
   "DEFAULT": [
    0.0,
    0.0,
    0.0,
    1.0
   ]
  },
  {
   "NAME": "offset10",
   "LABEL": "位置 11",
   "TYPE": "float",
   "DEFAULT": 1,
   "MIN": 0,
   "MAX": 1
  },
  {
   "NAME": "color11",
   "LABEL": "色 12",
   "TYPE": "color",
   "DEFAULT": [
    0.0,
    0.0,
    0.0,
    1.0
   ]
  },
  {
   "NAME": "offset11",
   "LABEL": "位置 12",
   "TYPE": "float",
   "DEFAULT": 1,
   "MIN": 0,
   "MAX": 1
  },
  {
   "NAME": "color12",
   "LABEL": "色 13",
   "TYPE": "color",
   "DEFAULT": [
    0.0,
    0.0,
    0.0,
    1.0
   ]
  },
  {
   "NAME": "offset12",
   "LABEL": "位置 13",
   "TYPE": "float",
   "DEFAULT": 1,
   "MIN": 0,
   "MAX": 1
  },
  {
   "NAME": "color13",
   "LABEL": "色 14",
   "TYPE": "color",
   "DEFAULT": [
    0.0,
    0.0,
    0.0,
    1.0
   ]
  },
  {
   "NAME": "offset13",
   "LABEL": "位置 14",
   "TYPE": "float",
   "DEFAULT": 1,
   "MIN": 0,
   "MAX": 1
  },
  {
   "NAME": "color14",
   "LABEL": "色 15",
   "TYPE": "color",
   "DEFAULT": [
    0.0,
    0.0,
    0.0,
    1.0
   ]
  },
  {
   "NAME": "offset14",
   "LABEL": "位置 15",
   "TYPE": "float",
   "DEFAULT": 1,
   "MIN": 0,
   "MAX": 1
  },
  {
   "NAME": "color15",
   "LABEL": "色 16",
   "TYPE": "color",
   "DEFAULT": [
    0.0,
    0.0,
    0.0,
    1.0
   ]
  },
  {
   "NAME": "offset15",
   "LABEL": "位置 16",
   "TYPE": "float",
   "DEFAULT": 1,
   "MIN": 0,
   "MAX": 1
  },
  {
   "NAME": "extend",
   "LABEL": "端の扱い",
   "TYPE": "long",
   "VALUES": [
    0,
    1,
    2
   ],
   "LABELS": [
    "端の色",
    "繰り返し",
    "折り返し"
   ],
   "DEFAULT": 0
  },
  {
   "NAME": "center_x",
   "LABEL": "中心 X",
   "TYPE": "float",
   "DEFAULT": 0,
   "MIN": -8000,
   "MAX": 8000
  },
  {
   "NAME": "center_y",
   "LABEL": "中心 Y（下が正）",
   "TYPE": "float",
   "DEFAULT": 0,
   "MIN": -8000,
   "MAX": 8000
  },
  {
   "NAME": "angle",
   "LABEL": "角度",
   "TYPE": "float",
   "DEFAULT": 0,
   "MIN": -3600,
   "MAX": 3600
  },
  {
   "NAME": "size",
   "LABEL": "長さ",
   "TYPE": "float",
   "DEFAULT": 400,
   "MIN": 1,
   "MAX": 20000
  },
  {
   "NAME": "offset",
   "LABEL": "ずらし",
   "TYPE": "float",
   "DEFAULT": 0,
   "MIN": -20000,
   "MAX": 20000
  },
  {
   "NAME": "radius_x",
   "LABEL": "半径 X",
   "TYPE": "float",
   "DEFAULT": 300,
   "MIN": 1,
   "MAX": 20000
  },
  {
   "NAME": "radius_y",
   "LABEL": "半径 Y",
   "TYPE": "float",
   "DEFAULT": 300,
   "MIN": 1,
   "MAX": 20000
  },
  {
   "NAME": "width_a",
   "LABEL": "帯の幅 1",
   "TYPE": "float",
   "DEFAULT": 40,
   "MIN": 0,
   "MAX": 4000
  },
  {
   "NAME": "width_b",
   "LABEL": "帯の幅 2",
   "TYPE": "float",
   "DEFAULT": 40,
   "MIN": 0,
   "MAX": 4000
  },
  {
   "NAME": "dot_radius",
   "LABEL": "水玉の半径",
   "TYPE": "float",
   "DEFAULT": 10,
   "MIN": 0,
   "MAX": 2000
  },
  {
   "NAME": "span",
   "LABEL": "水玉の間隔",
   "TYPE": "float",
   "DEFAULT": 40,
   "MIN": 1,
   "MAX": 4000
  },
  {
   "NAME": "aspect",
   "LABEL": "縦横比",
   "TYPE": "float",
   "DEFAULT": 0,
   "MIN": -100,
   "MAX": 100
  },
  {
   "NAME": "thickness",
   "LABEL": "線の太さ",
   "TYPE": "float",
   "DEFAULT": 2,
   "MIN": 0,
   "MAX": 2000
  },
  {
   "NAME": "cell_width",
   "LABEL": "格子の幅",
   "TYPE": "float",
   "DEFAULT": 40,
   "MIN": 1,
   "MAX": 4000
  },
  {
   "NAME": "cell_height",
   "LABEL": "格子の高さ",
   "TYPE": "float",
   "DEFAULT": 40,
   "MIN": 1,
   "MAX": 4000
  },
  {
   "NAME": "zoom",
   "LABEL": "拡大率",
   "TYPE": "float",
   "DEFAULT": 100,
   "MIN": 1,
   "MAX": 10000
  },
  {
   "NAME": "inverted",
   "LABEL": "反転",
   "TYPE": "bool",
   "DEFAULT": false
  },
  {
   "NAME": "relative",
   "LABEL": "長さを絵の幅の割合で決める",
   "TYPE": "bool",
   "DEFAULT": false
  },
  {
   "NAME": "noise_kind",
   "LABEL": "ノイズの種類",
   "TYPE": "long",
   "VALUES": [
    0,
    1,
    2,
    3,
    4,
    5,
    6
   ],
   "LABELS": [
    "なめらか",
    "砂嵐",
    "粗いなめらか",
    "うねり",
    "ボロノイ（面ごとの色）",
    "セル（点からの距離）",
    "ブロック"
   ],
   "DEFAULT": 0
  },
  {
   "NAME": "noise_strength",
   "LABEL": "ノイズの強さ",
   "TYPE": "float",
   "DEFAULT": 100,
   "MIN": 0,
   "MAX": 1000
  },
  {
   "NAME": "noise_threshold",
   "LABEL": "ノイズのしきい値",
   "TYPE": "float",
   "DEFAULT": 0,
   "MIN": 0,
   "MAX": 100
  },
  {
   "NAME": "noise_levels",
   "LABEL": "ノイズの段階",
   "TYPE": "float",
   "DEFAULT": 256,
   "MIN": 2,
   "MAX": 256
  },
  {
   "NAME": "noise_octaves",
   "LABEL": "ノイズの重ね数",
   "TYPE": "long",
   "DEFAULT": 5,
   "MIN": 1,
   "MAX": 8
  },
  {
   "NAME": "turbulence",
   "LABEL": "乱流",
   "TYPE": "bool",
   "DEFAULT": false
  },
  {
   "NAME": "colorful",
   "LABEL": "ノイズを色ごとに変える",
   "TYPE": "bool",
   "DEFAULT": false
  },
  {
   "NAME": "noise_scale_x",
   "LABEL": "ノイズの横の大きさ",
   "TYPE": "float",
   "DEFAULT": 100,
   "MIN": 1,
   "MAX": 100000
  },
  {
   "NAME": "noise_scale_y",
   "LABEL": "ノイズの縦の大きさ",
   "TYPE": "float",
   "DEFAULT": 100,
   "MIN": 1,
   "MAX": 100000
  },
  {
   "NAME": "noise_x",
   "LABEL": "ノイズの位置 X",
   "TYPE": "float",
   "DEFAULT": 0,
   "MIN": -100000,
   "MAX": 100000
  },
  {
   "NAME": "noise_y",
   "LABEL": "ノイズの位置 Y",
   "TYPE": "float",
   "DEFAULT": 0,
   "MIN": -100000,
   "MAX": 100000
  },
  {
   "NAME": "noise_z",
   "LABEL": "ノイズの奥行き",
   "TYPE": "float",
   "DEFAULT": 0,
   "MIN": -100000,
   "MAX": 100000
  },
  {
   "NAME": "speed_x",
   "LABEL": "ノイズの速さ X",
   "TYPE": "float",
   "DEFAULT": 0,
   "MIN": -100000,
   "MAX": 100000
  },
  {
   "NAME": "speed_y",
   "LABEL": "ノイズの速さ Y",
   "TYPE": "float",
   "DEFAULT": 0,
   "MIN": -100000,
   "MAX": 100000
  },
  {
   "NAME": "speed_z",
   "LABEL": "奥行きの速さ",
   "TYPE": "float",
   "DEFAULT": 0,
   "MIN": -1000,
   "MAX": 1000
  }
 ]
}*/
#define u_texture inputImage
#define u_source inputImage
#define v_uv isf_FragNormCoord
#define frag_color gl_FragColor
#define u_size RENDERSIZE
#define u_pass PASSINDEX
#define u_time clip_time
#define u_frame (clip_time * 60.0)
#define u_fps 60.0
#define u_duration clip_length
#define u_object vec4(0.0, 0.0, RENDERSIZE)
#define u_origin (RENDERSIZE * 0.5)
#define u_content vec4(0.0, 0.0, RENDERSIZE)
#define u_pixel_scale 1.0
#define texelFetch(t, p, l) texelFetch(t, ivec2((p).x, textureSize(t, 0).y - 1 - (p).y), l)
#define gradient_count 0
const vec3 LUMA = vec3(0.2126, 0.7152, 0.0722);  // Rec.709
const float PI = 3.14159265358979;
// 画面からカメラまでの距離 sashimono.engine.gpu.projection.CAMERA_DISTANCE と同じ値
// 画面の画素で数えた距離なので、合成の画素へ縮める 縮めないと、画質を落としたプレビューで
// 奥行きの付き方（傾けた板の遠近・奥へ置いた絵の縮み方）が書き出しと変わる
#define CAMERA (1024.0 * u_pixel_scale)

// 1 画素を事前乗算で読む 外は端の画素（CLAMP_TO_EDGE と同じ） 外を透明にすると、
// 画面いっぱいの素材を拡大して置いたときに端の 1 列が薄れる
vec4 texel_premul(ivec2 p, bool premultiplied) {
    ivec2 size = textureSize(u_texture, 0);
    vec4 c = texelFetch(u_texture, clamp(p, ivec2(0), size - 1), 0);
    return premultiplied ? c : vec4(c.rgb * c.a, c.a);
}

// 画素の間を事前乗算のまま直線で補う pixel は入力の画素（左下が原点）
vec4 bilinear_premul(vec2 pixel, bool premultiplied) {
    vec2 p = pixel - 0.5;
    ivec2 i = ivec2(floor(p));
    // 端数は GL の補間と同じく 1/256 に丸める 画素の真ん中を読んだつもりが計算の誤差で
    // 1e-7 だけずれると、隣の画素がわずかに混ざって等倍の写しが 1 だけ変わる
    vec2 f = floor((p - floor(p)) * 256.0 + 0.5) / 256.0;
    vec4 low = mix(
        texel_premul(i, premultiplied), texel_premul(i + ivec2(1, 0), premultiplied), f.x
    );
    vec4 high = mix(
        texel_premul(i + ivec2(0, 1), premultiplied),
        texel_premul(i + ivec2(1, 1), premultiplied),
        f.x
    );
    return mix(low, high, f.y);
}

// uv の 1 画素が入力で覆う範囲の平均（事前乗算） du と dv は出力の 1 画素ぶん動いたときの
// uv の動き（dFdx と dFdy） 分かれ道の中では測れないので、呼ぶ側が先に測って渡す
// 読む数は覆う画素の数に合わせ、極端に縮めたときも 8x8 で止めて重くしすぎない
vec4 area_premul(vec2 uv, vec2 du, vec2 dv, bool premultiplied) {
    vec2 size = vec2(textureSize(u_texture, 0));
    vec2 pixel = uv * size;
    vec2 dx = du * size;
    vec2 dy = dv * size;
    float reach = max(length(dx), length(dy));
    // 傾けた板の裏へ回った所の隣は、とても遠い点になる そこまでを範囲にすると、
    // 縁の画素が遠くを平均して薄れる
    if (reach > 256.0) return bilinear_premul(pixel, premultiplied);
    int n = int(clamp(ceil(reach - 0.001), 1.0, 8.0));
    vec4 sum = vec4(0.0);
    for (int j = 0; j < n; ++j) {
        for (int i = 0; i < n; ++i) {
            vec2 at = (vec2(float(i), float(j)) + 0.5) / float(n) - 0.5;
            sum += bilinear_premul(pixel + dx * at.x + dy * at.y, premultiplied);
        }
    }
    return sum / float(n * n);
}

vec2 object_center() { return (u_object.xy + u_object.zw) * 0.5; }
vec2 object_size() { return max(abs(u_object.zw - u_object.xy), vec2(1.0)); }
// 絵の原点 YMM4 が位置の設定を足し込む点 素材や図形では範囲の中央と同じだが、
// 場面切り替えの場面の絵は画面の中央が原点で、範囲（中身の描かれた所）とずれる
vec2 object_origin() { return u_origin; }

// 画素の位置で読む 外は透明 端を引き伸ばして読むと、動かした絵の外側に
// 縁の色が帯になって伸びる
// 画素の間は事前乗算で補う GL の補間のままだと、ずらしたり回したりした縁へ透明な所に
// 残った色がにじむ（:mod:`sashimono.effects.sampling` #179）
vec4 sample_pixel(vec2 pixel) {
    vec2 uv = pixel / u_size;
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) return vec4(0.0);
    vec4 c = bilinear_premul(pixel, false);
    return c.a > 0.0001 ? vec4(c.rgb / c.a, c.a) : vec4(0.0);
}

// ストレートアルファどうしの重ね（上が手前）
vec4 over(vec4 above, vec4 below) {
    float alpha = above.a + below.a * (1.0 - above.a);
    vec3 rgb = above.rgb * above.a + below.rgb * below.a * (1.0 - above.a);
    return alpha > 0.0001 ? vec4(rgb / alpha, alpha) : vec4(0.0);
}

// イージング 種類は 0 直線 1 Sine 2 Quad 3 Cubic 4 Quart 5 Quint 6 Expo 7 Circ
// 8 Back 9 Elastic 10 Bounce 11 Jump、向きは 0 In 1 Out 2 InOut
float bounce_out(float t) {
    if (t < 1.0 / 2.75) return 7.5625 * t * t;
    if (t < 2.0 / 2.75) { t -= 1.5 / 2.75; return 7.5625 * t * t + 0.75; }
    if (t < 2.5 / 2.75) { t -= 2.25 / 2.75; return 7.5625 * t * t + 0.9375; }
    t -= 2.625 / 2.75;
    return 7.5625 * t * t + 0.984375;
}

float ease_in(float t, int kind) {
    if (kind == 1) return 1.0 - cos(t * PI * 0.5);
    if (kind == 2) return t * t;
    if (kind == 3) return t * t * t;
    if (kind == 4) return t * t * t * t;
    if (kind == 5) return t * t * t * t * t;
    if (kind == 6) return t <= 0.0 ? 0.0 : pow(2.0, 10.0 * (t - 1.0));
    if (kind == 7) return 1.0 - sqrt(max(1.0 - t * t, 0.0));
    if (kind == 8) return t * t * (2.70158 * t - 1.70158);
    if (kind == 9) {
        if (t <= 0.0 || t >= 1.0) return t;
        return -pow(2.0, 10.0 * (t - 1.0)) * sin((t - 1.075) * 2.0 * PI / 0.3);
    }
    if (kind == 10) return 1.0 - bounce_out(1.0 - t);
    if (kind == 11) return t >= 1.0 ? 1.0 : 0.0;
    return t;
}

float ease(float t, int kind, int mode) {
    t = clamp(t, 0.0, 1.0);
    if (mode == 0) return ease_in(t, kind);
    if (mode == 1) return 1.0 - ease_in(1.0 - t, kind);
    return t < 0.5 ? ease_in(t * 2.0, kind) * 0.5 : 1.0 - ease_in(2.0 - t * 2.0, kind) * 0.5;
}

// 行で書いた 3x3 行列 GLSL の mat3 は列で並べるので、転置して渡す
mat3 rows3(vec3 a, vec3 b, vec3 c) { return transpose(mat3(a, b, c)); }

// X → Y → Z の順に回す行列 角度は度 向きは sashimono.engine.gpu.projection.rotate と同じ
// （画面の Y 下向き、奥が正で考える）
mat3 rotation3(vec3 degrees) {
    vec3 r = radians(degrees);
    mat3 rx = rows3(vec3(1.0, 0.0, 0.0), vec3(0.0, cos(r.x), sin(r.x)),
                    vec3(0.0, -sin(r.x), cos(r.x)));
    mat3 ry = rows3(vec3(cos(r.y), 0.0, -sin(r.y)), vec3(0.0, 1.0, 0.0),
                    vec3(sin(r.y), 0.0, cos(r.y)));
    mat3 rz = rows3(vec3(cos(r.z), -sin(r.z), 0.0), vec3(sin(r.z), cos(r.z), 0.0),
                    vec3(0.0, 0.0, 1.0));
    return rz * ry * rx;
}

// 傾けた板の逆算 画面の点（支点からの画素、Y 上向き）が、回す前の板のどこに
// あたるかを返す カメラから画面の点へ伸ばした線と、回した板の面との交点を
// 回す前へ戻す 板が真横を向いて交わらなければ、遠くの点（透明）を返す
// camera はカメラの真正面の点（支点からの画素、Y 上向き） 支点と離れていると、
// 離れた側ほど遠近が付く
vec2 untilt_seen_from(vec2 point, vec3 degrees, vec2 camera) {
    mat3 rotation = rotation3(degrees);
    vec3 normal = rotation * vec3(0.0, 0.0, 1.0);
    vec3 eye = vec3(camera.x, -camera.y, -CAMERA);
    vec3 direction = vec3(point.x - camera.x, -(point.y - camera.y), CAMERA);
    float facing = dot(normal, direction);
    if (abs(facing) < 1e-5) return vec2(1e6);
    float distance = -dot(normal, eye) / facing;
    if (distance <= 0.0) return vec2(1e6);
    vec3 hit = eye + direction * distance;
    vec3 local = transpose(rotation) * hit;
    return vec2(local.x, -local.y);
}

vec2 untilt(vec2 point, vec3 degrees) {
    return untilt_seen_from(point, degrees, vec2(0.0));
}

// 奥行き z（画素、奥が正）へ置いたときの拡大率
float depth_scale(float z) { return CAMERA / max(CAMERA + z, 1.0); }

vec4 premul(vec4 c) { return vec4(c.rgb * c.a, c.a); }
vec4 unpremul(vec4 c) { return c.a > 0.0001 ? vec4(c.rgb / c.a, c.a) : vec4(0.0); }

// 事前乗算アルファでぼかす ストレートのまま畳むと、透明な画素の色が
// 混ざって縁が黒ずむ
vec4 blur1d(vec2 uv, vec2 direction, float radius) {
    if (radius < 0.5) return texture(u_texture, uv);
    float sigma = max(radius * 0.5, 0.5);
    vec2 step = direction / u_size;
    vec4 sum = premul(texture(u_texture, uv));
    float total = 1.0;
    int taps = int(min(radius, 96.0));
    for (int i = 1; i <= taps; ++i) {
        float offset = float(i);
        float w = exp(-0.5 * offset * offset / (sigma * sigma));
        sum += premul(texture(u_texture, uv + step * offset)) * w;
        sum += premul(texture(u_texture, uv - step * offset)) * w;
        total += 2.0 * w;
    }
    return unpremul(sum / total);
}


float hash(vec2 p) {
    return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453123);
}

// エフェクトが読む画像（FileSpec の texture）の 1 点 p は画像の左上を原点とする
// 画素（Y は下が正） 画像は上の行から積んであるので、そのまま割れば UV になる
// loop が偽なら画像の外は透明 真なら敷き詰める（AviUtl2 の ループ画像）
vec4 image_pixel(vec2 size, vec2 p, bool loop) {
    // 大きさは合成の画素（画質を落とすと画像の画素より小さい） 1 で止めると、画質を
    // 落としたプレビューで小さな模様だけ縮まずに大きく出る 0 で割らないためだけに止める
    vec2 uv = p / max(size, vec2(1e-4));
    if (loop) {
        uv = fract(uv);
    } else if (uv.x < 0.0 || uv.y < 0.0 || uv.x >= 1.0 || uv.y >= 1.0) {
        return vec4(0.0);
    }
    return texture(u_texture, uv);
}



































const int MAX_STOPS = 16;










































vec3 to_srgb(vec3 c) { return clamp(c, 0.0, 1.0); }   // (what it is given is sRGB already: from_sashimono.py)
vec3 to_linear(vec3 c) { return c; }

vec4 stop_color(int i) {
    if (i == 0) return color0;
    if (i == 1) return color1;
    if (i == 2) return color2;
    if (i == 3) return color3;
    if (i == 4) return color4;
    if (i == 5) return color5;
    if (i == 6) return color6;
    if (i == 7) return color7;
    if (i == 8) return color8;
    if (i == 9) return color9;
    if (i == 10) return color10;
    if (i == 11) return color11;
    if (i == 12) return color12;
    if (i == 13) return color13;
    if (i == 14) return color14;
    return color15;
}
float stop_offset(int i) {
    if (i == 0) return offset0;
    if (i == 1) return offset1;
    if (i == 2) return offset2;
    if (i == 3) return offset3;
    if (i == 4) return offset4;
    if (i == 5) return offset5;
    if (i == 6) return offset6;
    if (i == 7) return offset7;
    if (i == 8) return offset8;
    if (i == 9) return offset9;
    if (i == 10) return offset10;
    if (i == 11) return offset11;
    if (i == 12) return offset12;
    if (i == 13) return offset13;
    if (i == 14) return offset14;
    return offset15;
}

// 色のバーで並べた止まり（#364） 数が 2 より少なければ前からの「色 1」「位置 1」… で塗る
// 色は sRGB のまま届く（混ぜるのも sRGB なので、リニアへ往復させない）

vec4 gradient_colors[MAX_STOPS];
float gradient_offsets[MAX_STOPS];

// 単色の模様はバーを使わず「色 1」で塗る（設定パネルも単色ではバーを出さず色の欄を出す）
bool from_bar() { return gradient_count >= 2 && pattern != 0; }
int stop_count() {
    return from_bar() ? min(gradient_count, MAX_STOPS) : clamp(stops, 1, MAX_STOPS);
}
float stop_at(int i) { return from_bar() ? gradient_offsets[i] : stop_offset(i); }
vec4 stop_srgb(int i) {
    if (from_bar()) return gradient_colors[i];
    vec4 c = stop_color(i);
    return vec4(to_srgb(c.rgb), c.a);
}

// 色は sRGB で補間する（YMM4 と同じ） 前からの項目の色はリニアで届くので先に戻す
vec4 ramp(float t) {
    int count = stop_count();
    if (count == 1 || t <= stop_at(0)) return stop_srgb(0);
    for (int i = 1; i < count; ++i) {
        float right = stop_at(i);
        if (t <= right) {
            float left = stop_at(i - 1);
            float k = right > left ? (t - left) / (right - left) : 1.0;
            return mix(stop_srgb(i - 1), stop_srgb(i), clamp(k, 0.0, 1.0));
        }
    }
    return stop_srgb(count - 1);
}

float extended(float t) {
    if (extend == 1) return fract(t);
    if (extend == 2) return 1.0 - abs(fract(t * 0.5) * 2.0 - 1.0);
    return clamp(t, 0.0, 1.0);
}

vec2 rotated(vec2 p, float degrees) {
    float r = radians(degrees);
    return mat2(cos(r), sin(r), -sin(r), cos(r)) * p;
}


// sin を使う乱数は大きな座標で桁が落ち、格子の値が横に揃って筋が出る 小数部だけで混ぜる
float lattice_value(vec2 cell, float slice) {
    vec3 p = fract(vec3(cell.xyx + vec3(slice * 17.13, slice * 5.71, slice * 11.3)) * 0.1031);
    p += dot(p, p.yzx + 33.33);
    return fract((p.x + p.y) * p.z);
}

float smooth_value(vec2 q, float slice) {
    vec2 cell = floor(q);
    vec2 f = fract(q);
    vec2 u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
    if (noise_kind == 2) {
        // 粗いなめらか（Fractal）は格子の値をつなぐ 四角い粒が残る YMM4 の絵もそうだった
        float a = lattice_value(cell, slice);
        float b = lattice_value(cell + vec2(1.0, 0.0), slice);
        float c = lattice_value(cell + vec2(0.0, 1.0), slice);
        float d = lattice_value(cell + vec2(1.0, 1.0), slice);
        return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
    }
    // なめらかな種類は勾配ノイズ 格子の値をつなぐと四角い粒が見える
    float corners[4];
    for (int k = 0; k < 4; ++k) {
        vec2 offset = vec2(float(k % 2), float(k / 2));
        float a = lattice_value(cell + offset, slice) * 6.2831853;
        corners[k] = dot(vec2(cos(a), sin(a)), f - offset);
    }
    float n = mix(mix(corners[0], corners[1], u.x), mix(corners[2], corners[3], u.x), u.y);
    return clamp(0.5 + n * 0.5, 0.0, 1.0);
}

// 奥行き（z）は整数の層の間を補間して、時間で滑らかに変わるようにする
float layered(vec2 q, float z, float salt) {
    float lower = floor(z) + salt * 31.0;
    return mix(smooth_value(q, lower), smooth_value(q, lower + 1.0), fract(z));
}

float fractal_sum(vec2 q, float z, float salt) {
    float total = 0.0;
    float weight = 0.5;
    float norm = 0.0;
    for (int i = 0; i < clamp(noise_octaves, 1, 8); ++i) {
        float n = layered(q, z, salt + float(i));
        // 乱流は 0.5 からの隔たりを裏返して足す 谷が白い筋になる（YMM4 の絵は明るい地に筋）
        total += weight * (turbulence ? 1.0 - abs(n * 2.0 - 1.0) : n);
        norm += weight;
        weight *= 0.5;
        q = q * 2.0 + 13.7;
    }
    return total / max(norm, 1e-4);
}

float cell_noise(vec2 q, float z, float salt, bool distance_only) {
    vec2 cell = floor(q);
    float nearest = 10.0;
    float value = 0.0;
    float slice = floor(z) + salt * 31.0;
    for (int j = -1; j <= 1; ++j) {
        for (int i = -1; i <= 1; ++i) {
            vec2 c = cell + vec2(i, j);
            vec2 point = c + vec2(lattice_value(c, slice + 0.3), lattice_value(c, slice + 0.7));
            float d = length(q - point);
            if (d < nearest) { nearest = d; value = lattice_value(c, slice + 0.9); }
        }
    }
    return distance_only ? clamp(nearest, 0.0, 1.0) : value;
}

// YMM4 のノイズのブラシ 大きさ 100% の目の粗さは、YMM4 に描かせた絵の粒の数から決めた
float shaped_noise(float v, float strength);

float noise_raw(vec2 p, float salt) {
    // 粒の大きさ（下の 2〜60）は画面の画素 画質を落とした下書きでも粒が絵に対して同じ大きさに
    // 見えるよう、画面 1 画素あたりの画素数を掛ける 掛けないと下書きだけ粒が 2〜4 倍に粗くなる
    vec2 scale = max(vec2(noise_scale_x, noise_scale_y) * 0.01, vec2(0.0001)) * u_pixel_scale;
    vec2 moved = rotated(p, -angle) + vec2(noise_x, noise_y) + vec2(speed_x, speed_y) * u_time;
    float z = noise_z + speed_z * u_time;
    float v;
    if (noise_kind == 1) {
        // 砂嵐は升ごとにばらばらの値 勾配ノイズを升の角（整数の点）で引くと、どこでも 0.5 の
        // 一様な灰になっていた（#177 大きさ 1000% の不透明度が YMM4 は 0.03〜0.97 に散り、
        // こちらは 0.5 の一枚だった） 升の大きさは大きさ 100% で 1 画素 YMM4 の絵は
        // 大きさ 1000% で隣の画素との相関が 0.98、8 画素離れて 0.22 だった
        vec2 cell = floor(moved / scale);
        float depth = z * 7.0;
        float lower = floor(depth) + salt * 31.0;
        v = mix(lattice_value(cell, lower), lattice_value(cell, lower + 1.0), fract(depth));
    } else if (noise_kind == 4 || noise_kind == 5) {
        float grain = noise_kind == 5 ? 60.0 : 25.0;
        v = cell_noise(moved / (grain * scale), z, salt, noise_kind == 5);
    } else if (noise_kind == 6) {
        v = lattice_value(floor(moved / (30.0 * scale)), floor(z) + salt * 31.0);
    } else if (noise_kind == 2) {
        v = fractal_sum(moved / (35.0 * scale), z, salt);
    } else {
        // なめらかな種類の値は 0.35〜0.65 ほどに収まる（YMM4 の絵と同じ散らばり）
        v = fractal_sum(moved / (48.0 * scale), z, salt);
    }
    return v;
}

float noise_value(vec2 p, float salt) {
    return shaped_noise(noise_raw(p, salt), max(noise_strength, 0.0) * 0.01);
}

// 強さは値に掛ける しきい値を上げると明るい側へ寄る 段階は値を量子化する
float shaped_noise(float v, float strength) {
    v *= strength;
    v /= max(1.0 - clamp(noise_threshold * 0.01, 0.0, 0.99), 0.01);
    float levels = max(noise_levels, 2.0);
    if (levels < 255.5) v = floor(clamp(v, 0.0, 0.9999) * levels) / (levels - 1.0);
    return clamp(v, 0.0, 1.0);
}

// 模様の色（sRGB、ストレートアルファ） p は絵の中心からの画素で Y は下が正
vec4 pattern_color(vec2 p) {
    float z = max(zoom, 0.0001) * 0.01;
    vec2 q = p - vec2(center_x, center_y);
    if (pattern == 0) return ramp(0.0);
    if (pattern == 7) {
        if (!colorful) return ramp(noise_value(q, 0.0));
        // 色つきは色の通り道ごとに別のノイズを引く
        vec3 channels = vec3(noise_value(q, 0.0), noise_value(q, 1.0), noise_value(q, 2.0));
        vec4 low = ramp(0.0);
        vec4 high = ramp(1.0);
        return vec4(mix(low.rgb, high.rgb, channels), mix(low.a, high.a, channels.g));
    }
    if (pattern == 1 || pattern == 3) {
        vec2 direction = vec2(cos(radians(angle)), sin(radians(angle)));
        // 割合で指定したときは絵の幅が 100（YMM4 の CoordinateMode が Relative）
        // 長さとずらしは画素の値として画質の倍率を掛けて届くが、割合のときは倍率が要らない
        // 絵の幅がもう下書きの画素で測られているので、ここで倍率を戻さないと二重に縮む
        float unit = relative ? object_size().x * 0.01 / max(u_pixel_scale, 0.0001) : 1.0;
        float along = dot(q, direction) - offset * unit;
        float length_ = max(size * unit, 1.0);
        // 山形は中心で終わりの色、長さの半分で始まりの色に戻る
        if (pattern == 3) return ramp(extended(1.0 - abs(along) / (length_ * 0.5)));
        return ramp(extended(along / length_ + 0.5));
    }
    if (pattern == 2) {
        vec2 r = rotated(q, -angle);
        vec2 radius = max(vec2(radius_x, radius_y) * z, vec2(1.0));
        radius.x *= 1.0 - max(aspect, 0.0) * 0.01;
        radius.y *= 1.0 + min(aspect, 0.0) * 0.01;
        return ramp(extended(length(r / radius)));
    }
    if (pattern == 4) {
        vec2 direction = vec2(cos(radians(angle)), sin(radians(angle)));
        float a = max(width_a * z, 0.0);
        float b = max(width_b * z, 0.0);
        float period = max(a + b, 1.0);
        // 1 本目の帯は絵の中心にまたがる（YMM4 の絵で中心の画素が帯の真ん中だった）
        // 中心のずらしはほかの模様と同じく q（中心を引いたあと）で測る
        float m = mod(dot(q, direction) - offset * z + a * 0.5, period);
        return m < a ? ramp(0.0) : ramp(1.0);
    }
    vec2 r = rotated(q, -angle);
    if (pattern == 5) {
        // 縦横比は模様ごと縮める 正なら横、負なら縦（間隔も水玉の形も同じ割合）
        vec2 squash = vec2(1.0 - max(aspect, 0.0) * 0.01, 1.0 + min(aspect, 0.0) * 0.01);
        squash = max(squash, vec2(0.01));
        vec2 cell = vec2(max(span * z, 1.0)) * squash;
        // 水玉は中心から半マスずれた所に並ぶ 反転の設定は YMM4 の絵でも変わらなかった
        vec2 local = mod(r, cell) - cell * 0.5;
        bool inside = length(local / squash) <= dot_radius * z;
        return inside ? ramp(0.0) : ramp(1.0);
    }
    vec2 cell = max(vec2(cell_width, cell_height) * z, vec2(1.0));
    // 格子の線は中心を通り、線の太さの真ん中が中心に来る
    vec2 local = mod(r + thickness * z * 0.5, cell);
    bool line = min(local.x, local.y) < thickness * z;
    if (inverted) line = !line;
    return line ? ramp(0.0) : ramp(1.0);
}

// 縁のある模様（ストライプ・水玉・格子）は、画素の中を 4x4 に分けて平均する
// 画素ごとに「乗る・乗らない」で描くと、太さ 1.3 の斜めの線が場所によって 1 画素にも
// 2 画素にもなり、菱形の大きな濃淡（モアレ）が浮く（SFっぽい吹き出し(右) の格子 #179）
// YMM4 は線の縁をなめらかに描く 事前乗算で平均するので、透明な色の RGB は混ざらない
// グラデーションとノイズはもともとなめらかで、ノイズは重いので 1 点のまま
vec4 pattern_smooth(vec2 p) {
    if (pattern < 4 || pattern > 6) return pattern_color(p);
    vec4 sum = vec4(0.0);
    for (int j = 0; j < 4; ++j) {
        for (int i = 0; i < 4; ++i) {
            vec4 c = pattern_color(p + (vec2(float(i), float(j)) + 0.5) * 0.25 - 0.5);
            sum += vec4(c.rgb * c.a, c.a);
        }
    }
    sum *= 1.0 / 16.0;
    return sum.a > 0.0 ? vec4(sum.rgb / sum.a, sum.a) : vec4(0.0);
}


float luminance(vec3 c) { return dot(c, vec3(0.3, 0.59, 0.11)); }
vec3 with_luminance(vec3 c, float l) {
    c += l - luminance(c);
    float n = min(min(c.r, c.g), c.b);
    float x = max(max(c.r, c.g), c.b);
    float lum = luminance(c);
    if (n < 0.0) c = lum + (c - lum) * lum / max(lum - n, 1e-5);
    if (x > 1.0) c = lum + (c - lum) * (1.0 - lum) / max(x - lum, 1e-5);
    return c;
}
float saturation_of(vec3 c) { return max(max(c.r, c.g), c.b) - min(min(c.r, c.g), c.b); }
vec3 with_saturation(vec3 c, float s) {
    float n = min(min(c.r, c.g), c.b);
    float x = max(max(c.r, c.g), c.b);
    return x > n ? (c - n) * s / (x - n) : vec3(0.0);
}

float soft_light(float b, float s) {
    if (s <= 0.5) return b - (1.0 - 2.0 * s) * b * (1.0 - b);
    float d = b <= 0.25 ? ((16.0 * b - 12.0) * b + 4.0) * b : sqrt(b);
    return b + (2.0 * s - 1.0) * (d - b);
}
float color_dodge(float b, float s) {
    return b <= 0.0 ? 0.0 : (s >= 1.0 ? 1.0 : min(1.0, b / (1.0 - s)));
}
float color_burn(float b, float s) {
    return b >= 1.0 ? 1.0 : (s <= 0.0 ? 0.0 : 1.0 - min(1.0, (1.0 - b) / s));
}
vec3 dodge3(vec3 b, vec3 s) {
    return vec3(color_dodge(b.r, s.r), color_dodge(b.g, s.g), color_dodge(b.b, s.b));
}
vec3 burn3(vec3 b, vec3 s) {
    return vec3(color_burn(b.r, s.r), color_burn(b.g, s.g), color_burn(b.b, s.b));
}

// b は下、s は上 どちらも sRGB mode は BLEND_MODES の並びの番号
vec3 blend_colors(int mode, vec3 b, vec3 s) {
    if (mode == 1) return min(b + s, 1.0);
    if (mode == 2) return max(b - s, 0.0);
    if (mode == 3) return b * s;
    if (mode == 4) return b + s - b * s;
    if (mode == 5) return mix(2.0 * b * s, 1.0 - 2.0 * (1.0 - b) * (1.0 - s), step(0.5, b));
    if (mode == 6) return vec3(soft_light(b.r, s.r), soft_light(b.g, s.g), soft_light(b.b, s.b));
    if (mode == 7) return mix(2.0 * b * s, 1.0 - 2.0 * (1.0 - b) * (1.0 - s), step(0.5, s));
    if (mode == 8) return dodge3(b, s);
    if (mode == 9) return burn3(b, s);
    if (mode == 10) return max(b, s);
    if (mode == 11) return min(b, s);
    if (mode == 12) return luminance(s) > luminance(b) ? s : b;
    if (mode == 13) return luminance(s) < luminance(b) ? s : b;
    if (mode == 14) return abs(b - s);
    if (mode == 15) return b + s - 2.0 * b * s;
    if (mode == 16) return max(b + s - 1.0, 0.0);
    if (mode == 17) return clamp(b + 2.0 * s - 1.0, 0.0, 1.0);
    if (mode == 18) {
        return mix(burn3(b, 2.0 * s), dodge3(b, 2.0 * s - 1.0), step(0.5, s));
    }
    if (mode == 19) return mix(min(b, 2.0 * s), max(b, 2.0 * s - 1.0), step(0.5, s));
    if (mode == 20) return step(1.0, b + s);
    if (mode == 21) return clamp(b / max(s, 1e-5), 0.0, 1.0);
    if (mode == 22) return with_luminance(with_saturation(s, saturation_of(b)), luminance(b));
    if (mode == 23) return with_luminance(with_saturation(b, saturation_of(s)), luminance(b));
    if (mode == 24) return with_luminance(s, luminance(b));
    if (mode == 25) return with_luminance(b, luminance(s));
    return s;
}

vec3 blended(vec3 b, vec3 s) { return blend_colors(blend, b, s); }

void main() {
    vec4 base = texture(u_texture, v_uv);
    vec2 pixel = v_uv * u_size - object_center();
    pixel.y = -pixel.y;
    if (noise_mask != 0) {
        // YMM4 のノイズ（NoiseEffect） 模様の値だけ薄める 1 で消え 0 で元のまま
        // 不透明度なら不透明度に、色なら符号化した色に 1 - 値 を掛ける（#177 の探りで
        // 強さ 100 の乱数が不透明度の平均 0.50、強さ 50 が 0.75、灰 128 の色が 64 になった）
        // 強さ s が 1 まではノイズ n で 1 - s n、1 を越えると (2 - s)(1 - n) 強さ 120・150・200 で
        // 不透明度の平均が 0.41・0.25・0（散らばりは平均に比例して縮む）だった 値を s 倍して
        // 切り詰めると 0.42・0.33・0.25 になり、強い所が消えきらない
        float n = shaped_noise(noise_raw(pixel - vec2(center_x, center_y), 0.0), 1.0);
        float s = max(noise_strength, 0.0) * 0.01;
        float keep = s <= 1.0 ? 1.0 - s * n : max(2.0 - s, 0.0) * (1.0 - n);
        if (noise_mask == 1) { frag_color = vec4(base.rgb, base.a * keep); return; }
        frag_color = vec4(to_linear(to_srgb(base.rgb) * keep), base.a);
        return;
    }
    vec4 paint = pattern_smooth(pixel);
    float amount = clamp(opacity * 0.01, 0.0, 1.0) * paint.a;
    if (pattern_only) {
        // 模様だけで塗る 元の絵の色は使わず、形（不透明度）だけを借りる
        // 透明な所は色も 0 にする 後ろの変形は隣の画素と RGB のまま混ぜるので、
        // 透明な白（YMM4 の格子の背景に多い）を残すと、縮めた格子が白っぽい板になる
        float alpha = base.a * amount;
        frag_color = alpha > 0.0 ? vec4(to_linear(paint.rgb), alpha) : vec4(0.0);
        return;
    }
    vec3 under = to_srgb(base.rgb);
    if (key_only) {
        // 目印の色で塗った所だけを模様に替える share は画素のうち塗りが占める割合
        vec3 key = to_srgb(key_color.rgb);
        if (keep_color.a <= 0.0) {
            // 線の色を持たない前の版の保存（目印はマゼンタ）は、前の版と同じ絵にする
            // 目印からの近さで混ぜ、α は元の絵のまま 下の透かし方を当てると、透明な模様の
            // 所で開き直した作品の塗りが消える
            float near = clamp(1.0 - length(under - key) * 3.0, 0.0, 1.0);
            frag_color = vec4(to_linear(mix(under, paint.rgb, amount * near)), base.a);
            return;
        }
        // 絵は線の色と目印の色の 2 色だけ（縁はその間の色）なので、画素の色が
        // 2 色を結ぶ線のどこにあるかで割合が決まる 目印からの近さで決めると、
        // 境の中間色で割合がずれて目印の色が残り、目印に近い線の色は模様に食われる
        vec3 kept = to_srgb(keep_color.rgb);
        vec3 axis = key - kept;
        float share = clamp(dot(under - kept, axis) / max(dot(axis, axis), 1e-6), 0.0, 1.0);
        // 塗りの分は模様の不透明度で透かす 目印と混ぜると、透明な模様から目印が透ける
        float kept_alpha = base.a * (1.0 - share);
        float fill_alpha = base.a * share * amount;
        float alpha = kept_alpha + fill_alpha;
        vec3 rgb = (kept * kept_alpha + paint.rgb * fill_alpha) / max(alpha, 1e-6);
        frag_color = alpha > 0.0 ? vec4(to_linear(rgb), alpha) : vec4(0.0);
        return;
    }
    vec3 rgb = mix(under, blended(under, paint.rgb), amount);
    frag_color = vec4(to_linear(rgb), base.a);
}
