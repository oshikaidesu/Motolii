/*{
 "DESCRIPTION": "Sashimono Edit's 縁の反射 (bevel_light, 装飾; MIT, src/sashimono/effects/builtin.py), made an ISF effect by filament/effects/from_sashimono.py",
 "CREDIT": "Sashimono Edit (MIT)",
 "CATEGORIES": [
  "Stylize"
 ],
 "ISFVSN": "2",
 "INPUTS": [
  {
   "NAME": "inputImage",
   "TYPE": "image"
  },
  {
   "NAME": "lighting",
   "LABEL": "光の当て方",
   "TYPE": "long",
   "VALUES": [
    0,
    1
   ],
   "LABELS": [
    "照り返し",
    "拡散"
   ],
   "DEFAULT": 0
  },
  {
   "NAME": "azimuth",
   "LABEL": "光の方位",
   "TYPE": "float",
   "DEFAULT": -90,
   "MIN": -360,
   "MAX": 360
  },
  {
   "NAME": "elevation",
   "LABEL": "光の高さ",
   "TYPE": "float",
   "DEFAULT": 0,
   "MIN": 0,
   "MAX": 90
  },
  {
   "NAME": "constant",
   "LABEL": "強さ",
   "TYPE": "float",
   "DEFAULT": 50,
   "MIN": 0,
   "MAX": 1000
  },
  {
   "NAME": "exponent",
   "LABEL": "鋭さ",
   "TYPE": "float",
   "DEFAULT": 1,
   "MIN": 0.01,
   "MAX": 128
  },
  {
   "NAME": "color",
   "LABEL": "光の色",
   "TYPE": "color",
   "DEFAULT": [
    1.0,
    1.0,
    1.0,
    1.0
   ]
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
   "DEFAULT": 1
  },
  {
   "NAME": "surface_scale",
   "LABEL": "高さの倍率",
   "TYPE": "float",
   "DEFAULT": 10,
   "MIN": 0,
   "MAX": 100
  },
  {
   "NAME": "profile",
   "LABEL": "縁の形",
   "TYPE": "long",
   "VALUES": [
    0,
    1,
    2,
    3
   ],
   "LABELS": [
    "直線",
    "丸",
    "くぼんだ丸",
    "段"
   ],
   "DEFAULT": 0
  },
  {
   "NAME": "thickness",
   "LABEL": "縁の太さ",
   "TYPE": "float",
   "DEFAULT": 10,
   "MIN": 1,
   "MAX": 128
  },
  {
   "NAME": "blur",
   "LABEL": "ぼかし",
   "TYPE": "float",
   "DEFAULT": 0,
   "MIN": 0,
   "MAX": 96
  },
  {
   "NAME": "inverted",
   "LABEL": "へこませる",
   "TYPE": "bool",
   "DEFAULT": false
  }
 ],
 "PASSES": [
  {
   "TARGET": "sm_pass0"
  },
  {
   "TARGET": "sm_pass1"
  },
  {
   "TARGET": "sm_pass2"
  },
  {
   "TARGET": "sm_pass3"
  },
  {}
 ]
}*/
#define u_texture inputImage
#define u_source inputImage
#define v_uv isf_FragNormCoord
#define frag_color gl_FragColor
#define u_size RENDERSIZE
#define u_pass PASSINDEX
#define u_time TIME
#define u_frame (TIME * 60.0)
#define u_fps 60.0
#define u_duration 1.0e9
#define u_object vec4(0.0, 0.0, RENDERSIZE)
#define u_origin (RENDERSIZE * 0.5)
#define u_content vec4(0.0, 0.0, RENDERSIZE)
#define u_pixel_scale 1.0
#define texelFetch(t, p, l) texelFetch(t, ivec2((p).x, textureSize(t, 0).y - 1 - (p).y), l)
vec4 sm_prev_texture(vec2 uv) {
    if (PASSINDEX == 1) return texture(isf_img_sm_pass0, uv);
    if (PASSINDEX == 2) return texture(isf_img_sm_pass1, uv);
    if (PASSINDEX == 3) return texture(isf_img_sm_pass2, uv);
    if (PASSINDEX == 4) return texture(isf_img_sm_pass3, uv);
    return texture(inputImage, uv);
}
vec4 sm_prev_texelFetch(ivec2 p, int l) {
    if (PASSINDEX == 1) return texelFetch(isf_img_sm_pass0, p, l);
    if (PASSINDEX == 2) return texelFetch(isf_img_sm_pass1, p, l);
    if (PASSINDEX == 3) return texelFetch(isf_img_sm_pass2, p, l);
    if (PASSINDEX == 4) return texelFetch(isf_img_sm_pass3, p, l);
    return texelFetch(inputImage, p, l);
}
ivec2 sm_prev_textureSize(int l) { return textureSize(inputImage, l); }
const vec3 LUMA = vec3(0.2126, 0.7152, 0.0722);  // Rec.709
const float PI = 3.14159265358979;
// 画面からカメラまでの距離 sashimono.engine.gpu.projection.CAMERA_DISTANCE と同じ値
// 画面の画素で数えた距離なので、合成の画素へ縮める 縮めないと、画質を落としたプレビューで
// 奥行きの付き方（傾けた板の遠近・奥へ置いた絵の縮み方）が書き出しと変わる
#define CAMERA (1024.0 * u_pixel_scale)

// 1 画素を事前乗算で読む 外は端の画素（CLAMP_TO_EDGE と同じ） 外を透明にすると、
// 画面いっぱいの素材を拡大して置いたときに端の 1 列が薄れる
vec4 texel_premul(ivec2 p, bool premultiplied) {
    ivec2 size = sm_prev_textureSize( 0);
    vec4 c = sm_prev_texelFetch( clamp(p, ivec2(0), size - 1), 0);
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
    vec2 size = vec2(sm_prev_textureSize( 0));
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
    if (radius < 0.5) return sm_prev_texture( uv);
    float sigma = max(radius * 0.5, 0.5);
    vec2 step = direction / u_size;
    vec4 sum = premul(sm_prev_texture( uv));
    float total = 1.0;
    int taps = int(min(radius, 96.0));
    for (int i = 1; i <= taps; ++i) {
        float offset = float(i);
        float w = exp(-0.5 * offset * offset / (sigma * sigma));
        sum += premul(sm_prev_texture( uv + step * offset)) * w;
        sum += premul(sm_prev_texture( uv - step * offset)) * w;
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
    return sm_prev_texture( uv);
}

vec3 to_srgb(vec3 c) {
    c = clamp(c, 0.0, 1.0);
    return mix(c * 12.92, 1.055 * pow(c, vec3(1.0 / 2.4)) - 0.055, step(0.0031308, c));
}
vec3 to_linear(vec3 c) {
    return mix(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), step(0.04045, c));
}
mat2 turn(float degrees) {
    float r = radians(degrees);
    return mat2(cos(r), sin(r), -sin(r), cos(r));
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














float profile_height(float t) {
    t = clamp(t, 0.0, 1.0);
    float h = t;
    if (profile == 1) h = sqrt(max(1.0 - (1.0 - t) * (1.0 - t), 0.0));
    if (profile == 2) h = 1.0 - sqrt(max(1.0 - t * t, 0.0));
    if (profile == 3) h = step(0.5, t);
    return inverted ? -h : h;
}

float height_at(vec2 uv) {
    return profile_height(sm_prev_texture( uv).a);
}

float shade(vec3 normal, vec3 light, float k) {
    if (lighting == 0) {
        vec3 halfway = normalize(light + vec3(0.0, 0.0, 1.0));
        return k * pow(max(dot(normal, halfway), 0.0), max(exponent, 0.01));
    }
    return k * max(dot(normal, light), 0.0);
}

// YMM4 は光の色を sRGB の値へ足す（加算） 平らな面にも光が当たって明るくなる
vec3 lit_color(vec3 under, float amount) {
    if (blend == 0) return mix(under, color.rgb, clamp(amount * color.a, 0.0, 1.0));
    return blend_colors(blend, under, clamp(color.rgb * amount * color.a, 0.0, 1.0));
}

// 画質を落とした合成で、書き出しの画素を 1 つずつ思い描いて光を当て、色で平均するか
// 書き出しの帯は、縁から画面の 1 画素ずつ刻んだ段になる 合成の 1 画素には、急な段・
// 緩い段・平らな面が太さの端数や縁の向きに応じて入り交じる 合成の画素で刻み直すと、
// 端数（6 の縁を 1/4 で 1.5 画素など）で段の数と傾きが書き出しと合わず、縮めた書き出しと
// 光り方がずれる（#182） 光は傾きに比例しないので、傾きを平均してから光を当てても
// 合わない 覆う書き出しの画素ごとに段の高さを求め、光を当てて色で平均する
// 書き出しの画素で 1 に満たない太さ（キーフレームの 0 や負の値も）は、書き出しが 1 へ
// 切り上げて別の式で描くので思い描けない
// 画質の分母が 8 を超えると、覆う書き出しの画素を 8 × 8 までしか数えられず平均が偏る
bool fine_rings() {
    return u_pixel_scale < 0.999 && u_pixel_scale >= 0.125 && thickness >= u_pixel_scale;
}

// 1 辺で数える書き出しの画素の数 画質の分母と同じ
int fine_count() {
    return int(clamp(floor(1.0 / u_pixel_scale + 0.5), 1.0, 8.0));
}

// 書き出しの太さ（書き出しの画素） 書き出しの pass 0 と同じ範囲へ収める
float fine_width() {
    return clamp(thickness / u_pixel_scale, 1.0, 128.0);
}

// 合成の画素 1 つが形に掛かる割合 絵の外は端の画素を読む（テクスチャの読み方と同じ）
float coverage_at(int x, int y) {
    ivec2 last = ivec2(u_size) - 1;
    return sm_prev_texelFetch( clamp(ivec2(x, y), ivec2(0), last), 0).a;
}

// 縁が 1 行（swap なら 1 列）の真ん中の高さを通る位置（合成の画素）
// 直線の縁なら、高さ 1 の帯に入る形の面積は、帯の真ん中の線の上で形に入る長さに等しい
// 帯の 7 画素の割合を足せば、アンチエイリアスの階調から縁の位置が端数まで分かる
float strip_edge(int line, int start, bool swap, bool inward_positive) {
    float sum = 0.0;
    for (int i = 0; i < 7; ++i) {
        sum += swap ? coverage_at(line, start + i) : coverage_at(start + i, line);
    }
    return inward_positive ? float(start + 7) - sum : float(start) + sum;
}

// crossing（合成の画素の座標）の近くの縁を直線で近づける 近づけられなければ偽
// アルファ 0.5 の境を補って読むと、斜めや曲がった縁では割合が距離に比例しないので、
// 境が本当の縁から 1 合成画素の 1 割ほど（書き出しの画素で 0.2〜0.5）ずれる 書き出しは
// 自分の細かい画素で境を読むので、ずれは書き出しの 1 画素の段 1 つ分に近い
// 縁の 3 行ぶんの面積から直線を求めると、ずれが残らない
bool edge_line(vec2 crossing, out vec2 point, out vec2 inward) {
    vec2 uv = crossing / u_size;
    vec2 step_ = 1.0 / u_size;
    vec2 across = vec2(step_.x, 0.0);
    vec2 down = vec2(0.0, step_.y);
    vec2 slope = vec2(
        sm_prev_texture( uv + across).a - sm_prev_texture( uv - across).a,
        sm_prev_texture( uv + down).a - sm_prev_texture( uv - down).a
    );
    point = crossing;
    inward = vec2(0.0);
    if (length(slope) < 1e-3) return false;
    // 縁が横に近ければ列ごとに足す 行ごとに足すと、縁が帯の外へ抜ける
    bool swap = abs(slope.y) > abs(slope.x);
    vec2 p = swap ? crossing.yx : crossing;
    bool inward_positive = (swap ? slope.y : slope.x) > 0.0;
    int start = int(floor(p.x)) - 3;
    int line = int(floor(p.y));
    float before = strip_edge(line - 1, start, swap, inward_positive);
    float middle = strip_edge(line, start, swap, inward_positive);
    float after = strip_edge(line + 1, start, swap, inward_positive);
    // 3 行が 1 本の線に乗らない所（角や細い所）は直線で近づけられない
    if (abs(after - 2.0 * middle + before) > 0.25) return false;
    vec2 tangent = normalize(vec2(after - before, 2.0));
    vec2 normal = vec2(tangent.y, -tangent.x);
    if ((normal.x > 0.0) != inward_positive) normal = -normal;
    vec2 at = vec2(middle, float(line) + 0.5);
    point = swap ? at.yx : at;
    inward = swap ? normal.yx : normal;
    return true;
}

// 縁の内向きが n の直線の縁で、24 方向のうち縁へいちばん真っ直ぐ向く方向の余弦
// 書き出しはその方向で縁を見つけるので、縁までの距離をこれで割った値が測りになる
float facing(vec2 n) {
    float step_ = PI / 12.0;
    return cos(abs(mod(atan(n.y, n.x) + step_ * 0.5, step_) - step_ * 0.5));
}

// 縁の向きを持たない（縁が測る範囲に無かった）印 角度は -π..π なので重ならない
const float NO_EDGE = 10.0;

// 書き出しの pass 0 は、画素から 24 方向へ輪を広げ、形の外（アルファ 0.5 未満）に掛かった
// 最初の輪で縁までを測る 斜めや曲がった縁では、いちばん近い方向が縁へ真っ直ぐ向かない
// ので、縁までの本当の距離より少し遠く出る 合成の画素で輪を刻むと、この測り方の癖と
// 1 合成画素（書き出しの 2〜4 画素）より細かい縁の位置が消え、楕円の段が書き出しとずれる
// （#194） 同じ 24 方向で縁を越える所を刻まずに探し、書き出しの画素で数えて置く
// 返すのは測りと、縁の内向きの角度 間の書き出しの画素は、周りの合成の画素の測りを縁の
// 向きへ伸ばして求める（measure_at） 伸ばせるように、形の外は形までの距離を負で
// 3 合成画素まで置き、太さの先も 2 合成画素ぶんまで測る
vec2 edge_measure() {
    bool inside = sm_prev_texture( v_uv).a >= 0.5;
    float reach = inside ? fine_width() * u_pixel_scale + 2.0 : 3.0;
    int rings = int(ceil(reach));
    vec2 centre = v_uv * u_size;
    for (int ring = 1; ring <= rings; ++ring) {
        float best = -1.0;
        vec2 best_toward = vec2(0.0);
        for (int k = 0; k < 24; ++k) {
            float a = PI * 2.0 * float(k) / 24.0;
            vec2 toward = vec2(cos(a), sin(a));
            if ((sm_prev_texture( v_uv + toward * float(ring) / u_size).a >= 0.5) == inside) {
                continue;
            }
            // 越えた輪と 1 つ内の輪の間を半分ずつ詰める 10 回で 1/1024 合成画素
            float near_ = float(ring - 1);
            float far_ = float(ring);
            for (int i = 0; i < 10; ++i) {
                float middle = (near_ + far_) * 0.5;
                if ((sm_prev_texture( v_uv + toward * middle / u_size).a >= 0.5) == inside) {
                    near_ = middle;
                } else {
                    far_ = middle;
                }
            }
            float crossing = (near_ + far_) * 0.5;
            if (best < 0.0 || crossing < best) {
                best = crossing;
                best_toward = toward;
            }
        }
        // 先に越えた輪の方向より、後の輪で越える方向が近いことは無い
        if (best < 0.0) continue;
        vec2 point;
        vec2 inward;
        vec2 direction = inside ? -best_toward : best_toward;
        bool straight = edge_line(centre + best_toward * best, point, inward);
        // 縁とほぼ平行に見つけた方向は、直線との交わりが遠くへ飛ぶので決め直さない
        if (straight && abs(dot(best_toward, inward)) > 0.5) {
            // 見つけた方向のまま、越える所だけを縁の直線で決め直す 縁までの距離を 24 方向の
            // 癖で割る形にすると、曲がりのきつい縁（楕円の長い軸の端）から離れた所で外れる
            float refined = dot(point - centre, inward) / dot(best_toward, inward);
            // 補った境から大きく離れるなら、直線で近づけたのが外れている
            if (abs(refined - best) < 0.5) {
                best = refined;
                direction = inward;
            }
        }
        return vec2((inside ? best : -best) / u_pixel_scale, atan(direction.y, direction.x));
    }
    return vec2((inside ? reach : -reach) / u_pixel_scale, NO_EDGE);
}

// 書き出しの画素（uv）の縁までの測り
// 周りの合成の画素 4 つの測りを、それぞれの縁の向きへ伸ばして小さい方を取る 直線の縁なら
// どれから伸ばしても同じ値になる 補う（線形に混ぜる）と、上下の縁から等しく離れた
// 真ん中の尾根で測りが低く出る 尾根は太い縁の反射でよく掛かり、段が 1 つずれる
float measure_at(vec2 uv) {
    vec2 p = uv * u_size;
    ivec2 corner = ivec2(floor(p - 0.5));
    ivec2 last = ivec2(u_size) - 1;
    float best = 1e9;
    for (int j = 0; j < 2; ++j) {
        for (int i = 0; i < 2; ++i) {
            ivec2 texel = clamp(corner + ivec2(i, j), ivec2(0), last);
            vec4 field = sm_prev_texelFetch( texel, 0);
            float extended = field.r;
            if (field.b < NO_EDGE - 1.0) {
                vec2 n = vec2(cos(field.b), sin(field.b));
                extended += dot(p - (vec2(texel) + 0.5), n) / (facing(n) * u_pixel_scale);
            }
            best = min(best, extended);
        }
    }
    return best;
}

// 書き出しの pass 0 が、縁から measure 画素（24 方向で測った値）の所へ置く値
// 形の外は 0、縁の外に掛かった最初の輪の半径を太さで割る 太さの先は 1
// 輪は太さ（上限 128）を 48 までの数で刻む
float export_value(float measure, float width) {
    if (measure <= 0.0) return 0.0;
    float rings = ceil(min(width, 48.0));
    // 輪の半径ちょうどに縁がある所は、書き出しではその輪が形の外に掛からない（アルファが
    // ちょうど 0.5） 太さが 48 を超えて輪の間が 1 画素でなくなると、四角の縁から画素の
    // 中心までの距離が輪の半径と揃う所が出る 測りの丸めで下へ外れないよう少し足す
    return min((floor(measure * rings / width + 0.002) + 1.0) / rings, 1.0);
}

// 書き出しがぼかすか 書き出しのぼかしは 1 画素に満たない半径では何もしない
bool fine_blurred() {
    return blur / u_pixel_scale >= 1.0;
}

float normal_cdf(float z) {
    // erf の近似（Abramowitz と Stegun 7.1.26 誤差 1.5e-7） GLSL に erf が無い
    float x = abs(z) * 0.70710678;
    float t = 1.0 / (1.0 + 0.3275911 * x);
    float poly = ((((1.061405429 * t - 1.453152027) * t + 1.421413741) * t - 0.284496736) * t
        + 0.254829592) * t;
    float erf_ = 1.0 - poly * exp(-x * x);
    return 0.5 * (1.0 + sign(z) * erf_);
}

// 書き出しのぼかし（1 次元）で、縁の内へ x 画素の所の段 1 つがどれだけ上がって見えるか
// 書き出しのぼかしは半径の半分を標準偏差とし、半径で打ち切った重み
float blurred_step(float x) {
    float taps = floor(min(blur / u_pixel_scale, 96.0));
    float reach = taps + 0.5;
    if (x >= reach) return 1.0;
    if (x <= -reach) return 0.0;
    float sigma = max(blur / u_pixel_scale * 0.5, 0.5);
    float low = normal_cdf(-reach / sigma);
    float high = normal_cdf(reach / sigma);
    return clamp((normal_cdf(x / sigma) - low) / (high - low), 0.0, 1.0);
}

// 縁を直線と見て、書き出しの段を縁に垂直な向きにだけぼかした値
// 段は縁から 1 輪ごとに 1/輪の数ずつ上がる ぼかした段を足し合わせる
// 24 方向の測りと縁までの本当の距離の違い（1 % 未満）は見ない
float blurred_value(float measure, float width) {
    float rings = ceil(min(width, 48.0));
    float spacing = width / rings;
    float sum = 0.0;
    for (int k = 0; k < int(rings); ++k) {
        sum += blurred_step(measure - float(k) * spacing);
    }
    return sum / rings;
}

// ぼかしのある時の縁の坂の広さ（太さとぼかしの広がり 合成の画素）
float blurred_slope() {
    float sigma = max(blur / u_pixel_scale * 0.5, 0.5);
    return (fine_width() + 2.5 * sigma) * u_pixel_scale;
}

// 坂がこれより広いと、書き出しの画素を思い描かずに前の描き方で描く（draw_mode）
const float LEGACY_SLOPE = 9.0;

// ぼかしのある時に、合成の画素でぼかした値を補うだけで足りる割合
// 坂が合成の画素 3 つより狭いと、補うと坂が均されて緩む所が内へずれる 広いほど補うだけの
// 方が合う 縁を直線と見る値は、上下の縁や角をまたぐぼかしで折れ目が出て、補っても
// 消えない 坂の広さでなめらかに混ぜて、ぼかしを動かした時に光り方が跳ねないようにする
float coarse_share() {
    return clamp((blurred_slope() - 3.0) / (LEGACY_SLOPE - 3.0), 0.0, 1.0);
}

// 描き方 書き出しと同じ式・前の思い描き方（#182）・今の思い描き方（#194）
const int DRAW_PLAIN = 0;
const int DRAW_LEGACY = 1;
const int DRAW_FINE = 2;

// 今の思い描き方が前の描き方より縮めた書き出しから離れる所は、前の描き方のまま描く
// 四角と楕円・太さ 1〜128・ぼかし 0〜96 の 592 通りで前と今を比べて、どれも前より
// 離れないように決めた（#194）
// ぼかしの無い 48 を超える太さは、輪の間が 1 画素でなくなり、輪の半径と画素の中心が
// 揃う所で書き出しの丸め方を当てきれない ぼかしの坂が広い所は、前の合成の画素で刻んで
// ぼかす描き方の方が四角の角や尾根で合う 坂の広さの境で描き方が切り替わるが、境の
// 両側はどちらも縮めた書き出しに近いので、跳ねは小さい
int draw_mode() {
    if (!fine_rings()) return DRAW_PLAIN;
    bool legacy = !fine_blurred() ? fine_width() > 48.0 : blurred_slope() >= LEGACY_SLOPE;
    if (!legacy) return DRAW_FINE;
    // 前の思い描き方は、ぼかしの無い 46 合成画素までの太さだけだった
    return blur <= 0.0 && thickness <= 46.0 ? DRAW_LEGACY : DRAW_PLAIN;
}

// 書き出しの画素 1 つの高さ
// ぼかしが無ければ、縁までの測りを求めてから書き出しと同じく段に刻む 刻んだ段を補うと、
// 段の境が合成の画素の升目に揃い、急な段が書き出しの 2〜4 倍の幅へ緩む
// ぼかしがあれば、縁を直線と見てぼかした段に、合成の画素でぼかした値との違いを足す
// 直線と見た値は角で外れるが、外れは合成の画素でぼかした値と比べると分かり、
// なだらかなので補っても崩れない（pass 1 の緑とアルファ）
float fine_height(vec2 uv) {
    vec4 field = sm_prev_texture( uv);
    if (fine_blurred()) {
        float share = coarse_share();
        float straight = blurred_value(measure_at(uv), fine_width()) + field.a - field.g;
        return profile_height(mix(straight, field.a, share));
    }
    return profile_height(export_value(measure_at(uv), fine_width()));
}

// 合成の 1 画素が覆う書き出しの画素に光を当てて、色で平均する
// 書き出しでは形の縁の画素も、周りの段の傾きで光ってから覆う割合だけ混ざる 書き出しの
// 画素が形に掛かる割合（縁からの距離 + 0.5）で重みを付ける
// 高さは覆う書き出しの画素と、その上下左右の 1 画素ずつで求める 隣どうしで同じ画素を
// 何度も読まないよう、先に升目へ溜める（1/8 で 10 × 10）
vec3 fine_lit(vec3 under, vec3 light, float k) {
    int count = fine_count();
    float strength = surface_scale / u_pixel_scale;
    float heights[10][10];
    for (int y = 0; y < count + 2; ++y) {
        for (int x = 0; x < count + 2; ++x) {
            // 四隅は傾きに使わない
            if ((x == 0 || x == count + 1) && (y == 0 || y == count + 1)) continue;
            vec2 offset = (vec2(float(x - 1), float(y - 1)) + 0.5) / float(count) - 0.5;
            heights[y][x] = fine_height(v_uv + offset / u_size);
        }
    }
    vec3 sum = vec3(0.0);
    float taken = 0.0;
    for (int y = 1; y <= count; ++y) {
        for (int x = 1; x <= count; ++x) {
            vec2 offset = (vec2(float(x - 1), float(y - 1)) + 0.5) / float(count) - 0.5;
            float weight = clamp(measure_at(v_uv + offset / u_size) + 0.5, 0.0, 1.0);
            if (weight <= 0.0) continue;
            float gx = (heights[y][x + 1] - heights[y][x - 1]) * 0.5;
            float gy = (heights[y + 1][x] - heights[y - 1][x]) * 0.5;
            vec3 normal = normalize(vec3(-gx * strength, -gy * strength, 1.0));
            sum += lit_color(under, shade(normal, light, k)) * weight;
            taken += weight;
        }
    }
    if (taken <= 0.0) return lit_color(under, shade(vec3(0.0, 0.0, 1.0), light, k));
    return sum / taken;
}

// 前の思い描き方（#182）で、縁から distance 画素の所に書き出しが置く高さ
float legacy_height(float distance, float width) {
    if (distance <= 0.0) return profile_height(0.0);
    float rings = ceil(min(width, 48.0));
    float ring = floor(distance * rings / width) + 1.0;
    return profile_height(min(ring / rings, 1.0));
}

// 前の思い描き方（#182） 縁からの距離と向きを合成の画素で刻んだ輪から取り、縁を直線と
// 見て覆う書き出しの画素に光を当てる 四角の縁ならどの画素も書き出しと同じ段に入る
vec3 legacy_lit(vec3 under, vec3 light, float k, float width) {
    vec2 step_ = 1.0 / u_size;
    float here = sm_prev_texture( v_uv).a * width - 0.5;
    vec2 slope = vec2(
        sm_prev_texture( v_uv + vec2(step_.x, 0.0)).a
            - sm_prev_texture( v_uv - vec2(step_.x, 0.0)).a,
        sm_prev_texture( v_uv + vec2(0.0, step_.y)).a
            - sm_prev_texture( v_uv - vec2(0.0, step_.y)).a
    );
    vec2 inward = length(slope) > 1e-6 ? normalize(slope) : vec2(0.0);
    float scale = 1.0 / u_pixel_scale;
    int count = fine_count();
    float full = min(thickness * scale, 128.0);
    float strength = surface_scale * scale;
    vec3 sum = vec3(0.0);
    float taken = 0.0;
    for (int y = 0; y < count; ++y) {
        for (int x = 0; x < count; ++x) {
            vec2 offset = (vec2(float(x), float(y)) + 0.5) / float(count) - 0.5;
            float distance = (here + dot(offset, inward)) * scale;
            // 縁の外に掛かる書き出しの画素は、書き出しでも光らない（形の外）
            if (distance <= 0.0) continue;
            float g = (legacy_height(distance + 1.0, full) - legacy_height(distance - 1.0, full))
                * 0.5;
            vec3 normal = normalize(vec3(-inward * g * strength, 1.0));
            sum += lit_color(under, shade(normal, light, k));
            taken += 1.0;
        }
    }
    if (taken <= 0.0) return lit_color(under, shade(vec3(0.0, 0.0, 1.0), light, k));
    return sum / taken;
}

void main() {
    int mode = draw_mode();
    bool fine = mode == DRAW_FINE;
    // 前の思い描き方は、縁から帯の外の平らな所まで（太さ + 2 画素）を 1 画素ずつ刻む
    float legacy_width = ceil(thickness) + 2.0;
    if (u_pass == 0) {
        // 書き出しの画素を思い描く時は、縁までの測りを赤へ、縁の内向きの角度を青へ置く
        if (fine) {
            vec2 measured = edge_measure();
            frag_color = vec4(measured.x, 0.0, measured.y, 1.0);
            return;
        }
        // 縁からの距離を太さで割った値（0 が縁、1 が太さ以上の内側）を作る
        float inside = sm_prev_texture( v_uv).a;
        if (inside < 0.5) { frag_color = vec4(1.0, 1.0, 1.0, 0.0); return; }
        float width = mode == DRAW_LEGACY ? legacy_width : clamp(thickness, 1.0, 128.0);
        float nearest = width;
        int rings = int(ceil(min(width, 48.0)));
        for (int ring = 1; ring <= rings; ++ring) {
            float r = width * float(ring) / float(rings);
            if (r >= nearest) break;
            for (int k = 0; k < 24; ++k) {
                float a = PI * 2.0 * float(k) / 24.0;
                if (sm_prev_texture( v_uv + vec2(cos(a), sin(a)) * r / u_size).a < 0.5) {
                    nearest = min(nearest, r);
                }
            }
        }
        frag_color = vec4(1.0, 1.0, 1.0, nearest / width);
        return;
    }
    vec4 here = sm_prev_texelFetch( ivec2(gl_FragCoord.xy), 0);
    bool blurred = fine && fine_blurred();
    if (u_pass == 1) {
        // ぼかす値は、書き出しと同じく覆う書き出しの画素で段に刻んでから平均する（縮める）
        // 縁を直線と見てぼかした値も同じ所で平均して緑へ置く 赤と青（測りと縁の向き）は残す
        if (blurred) {
            int count = fine_count();
            float width = fine_width();
            float steps = 0.0;
            float straight = 0.0;
            for (int y = 0; y < count; ++y) {
                for (int x = 0; x < count; ++x) {
                    vec2 offset = (vec2(float(x), float(y)) + 0.5) / float(count) - 0.5;
                    float measure = measure_at(v_uv + offset / u_size);
                    steps += export_value(measure, width);
                    straight += blurred_value(measure, width);
                }
            }
            float taken = float(count * count);
            frag_color = vec4(here.r, straight / taken, here.b, steps / taken);
            return;
        }
        // 書き出しでは何もしない（画質を落とした時の刻み直しのための段） 画素をそのまま
        // 写す 補って読むと、画素の中心でも値が丸めで揺れ、等倍の絵が前と変わりうる
        frag_color = here;
        return;
    }
    if (u_pass == 2 || u_pass == 3) {
        if (fine && !blurred) { frag_color = here; return; }
        vec2 direction = u_pass == 2 ? vec2(1.0, 0.0) : vec2(0.0, 1.0);
        // blur は合成の画素へ縮めて渡される（TrackSpec の px） ぼかす絵も合成の大きさなので、
        // そのまま渡すと書き出しの半径を縮めた幅でぼかす ここで u_pixel_scale で割ると
        // 2 回縮める逆になり、2 倍・4 倍の幅でぼかしてしまう
        // blurred_step は書き出しの画素で数えるので、同じ blur を u_pixel_scale で割って
        // 半径を揃える 求め方は別（こちらは画素ごとの重みの畳み込み、あちらは段 1 つを
        // 同じ重みでぼかした形を正規分布の累積で近づけた物）
        vec4 spread = blur1d(v_uv, direction, blur);
        // 思い描く時にぼかすのは段の値（アルファ）だけ 赤・緑・青は光を当てる所で読む
        frag_color = blurred ? vec4(here.rgb, spread.a) : spread;
        return;
    }
    vec4 base = texture(u_source, v_uv);
    if (base.a <= 0.0001) { frag_color = base; return; }
    // 方位は画面で右が 0、時計回り（YMM4 に描かせた絵で、-85 は上から当たった）
    float a = radians(azimuth);
    float e = radians(elevation);
    vec3 light = normalize(vec3(cos(e) * cos(a), -cos(e) * sin(a), sin(e)));
    float k = max(constant, 0.0) * 0.01;
    vec3 under = to_srgb(base.rgb);
    if (fine) {
        frag_color = vec4(to_linear(fine_lit(under, light, k)), base.a);
        return;
    }
    if (mode == DRAW_LEGACY) {
        frag_color = vec4(to_linear(legacy_lit(under, light, k, legacy_width)), base.a);
        return;
    }
    vec2 step_ = 1.0 / u_size;
    float gx = (height_at(v_uv + vec2(step_.x, 0.0)) - height_at(v_uv - vec2(step_.x, 0.0))) * 0.5;
    float gy = (height_at(v_uv + vec2(0.0, step_.y)) - height_at(v_uv - vec2(0.0, step_.y))) * 0.5;
    // 1 画素に満たない太さ（画質を落とした合成で、書き出しの画素を思い描けない時）は、
    // 1 画素の帯として描いてから、帯の画素に占める縁の割合だけ平らな面の光と混ぜる
    // 書き出しを縮めると、細い縁の急な面と内側の平らな面が 1 画素の中で平均される 1 画素に
    // 切り上げたままだと、面が緩く帯が太い別の光り方になる 傾きは太さで割って、書き出しの
    // 急な面に合わせる
    float share = clamp(thickness, 0.0001, 1.0);
    vec3 normal = normalize(vec3(-gx * surface_scale, -gy * surface_scale, share));
    vec3 lit = lit_color(under, shade(normal, light, k));
    // 混ぜるのは色にしてから 光の量で混ぜると、強く当てて白く飽和する所が縁の割合より濃く出る
    if (share < 1.0) {
        lit = mix(lit_color(under, shade(vec3(0.0, 0.0, 1.0), light, k)), lit, share);
    }
    frag_color = vec4(to_linear(lit), base.a);
}
