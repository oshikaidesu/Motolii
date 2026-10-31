/*{
 "DESCRIPTION": "Sashimono Edit's 内側の縁取り (inner_outline, 装飾; MIT, src/sashimono/effects/builtin.py), made an ISF effect by filament/effects/from_sashimono.py",
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
   "NAME": "thickness",
   "LABEL": "太さ",
   "TYPE": "float",
   "DEFAULT": 4,
   "MIN": 0,
   "MAX": 256
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
   "NAME": "opacity",
   "LABEL": "濃さ",
   "TYPE": "float",
   "DEFAULT": 100,
   "MIN": 0,
   "MAX": 100
  },
  {
   "NAME": "color",
   "LABEL": "色",
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
   "DEFAULT": 0
  },
  {
   "NAME": "outline_only",
   "LABEL": "縁だけ残す",
   "TYPE": "bool",
   "DEFAULT": false
  },
  {
   "NAME": "angular",
   "LABEL": "角ばらせる",
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
  {}
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
vec4 sm_prev_texture(vec2 uv) {
    if (PASSINDEX == 1) return texture(isf_img_sm_pass0, uv);
    if (PASSINDEX == 2) return texture(isf_img_sm_pass1, uv);
    return texture(inputImage, uv);
}
vec4 sm_prev_texelFetch(ivec2 p, int l) {
    if (PASSINDEX == 1) return texelFetch(isf_img_sm_pass0, p, l);
    if (PASSINDEX == 2) return texelFetch(isf_img_sm_pass1, p, l);
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

vec3 to_srgb(vec3 c) { return clamp(c, 0.0, 1.0); }   // (what it is given is sRGB already: from_sashimono.py)
vec3 to_linear(vec3 c) { return c; }

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









void main() {
    if (u_pass == 0) {
        // 縁から thickness 以内の内側を帯にする 角ばらせるなら正方形、そうでなければ円で削る
        float t = min(max(thickness, 0.0), 256.0);
        float inside = sm_prev_texture( v_uv).a;
        float kept = inside;
        if (t >= 0.5) {
            int rings = int(ceil(min(t, 64.0)));
            for (int ring = 1; ring <= rings; ++ring) {
                float r = t * float(ring) / float(rings);
                for (int k = 0; k < 32; ++k) {
                    float a = PI * 2.0 * float(k) / 32.0;
                    vec2 d = vec2(cos(a), sin(a));
                    if (angular) d /= max(abs(d.x), abs(d.y));
                    kept = min(kept, sm_prev_texture( v_uv + d * r / u_size).a);
                }
            }
        }
        frag_color = vec4(1.0, 1.0, 1.0, clamp(inside - kept, 0.0, 1.0));
        return;
    }
    if (u_pass == 1) {
        frag_color = blur1d(v_uv, vec2(1.0, 0.0), blur);
        return;
    }
    float band = blur1d(v_uv, vec2(0.0, 1.0), blur).a;
    vec4 base = texture(u_source, v_uv);
    // ぼかした帯は絵の外へはみ出さない 内側の縁取りなので
    float amount = clamp(band * base.a * color.a * opacity * 0.01, 0.0, 1.0);
    if (outline_only) {
        frag_color = vec4(color.rgb, amount);
        return;
    }
    vec3 under = to_srgb(base.rgb);
    vec3 mixed = blend_colors(blend, under, to_srgb(color.rgb));
    frag_color = vec4(to_linear(mix(under, mixed, amount / max(base.a, 1e-4))), base.a);
}
