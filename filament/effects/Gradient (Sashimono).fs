/*{
 "DESCRIPTION": "Sashimono Edit's グラデーション (gradient, 装飾; MIT, src/sashimono/effects/builtin.py), made an ISF effect by filament/effects/from_sashimono.py",
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
   "NAME": "strength",
   "LABEL": "強さ",
   "TYPE": "float",
   "DEFAULT": 100,
   "MIN": 0,
   "MAX": 100
  },
  {
   "NAME": "start_color",
   "LABEL": "開始色",
   "TYPE": "color",
   "DEFAULT": [
    1.0,
    1.0,
    1.0,
    1.0
   ]
  },
  {
   "NAME": "end_color",
   "LABEL": "終了色",
   "TYPE": "color",
   "DEFAULT": [
    0.0,
    0.0,
    0.0,
    1.0
   ]
  },
  {
   "NAME": "mixing",
   "LABEL": "色の混ぜ方",
   "TYPE": "long",
   "VALUES": [
    0,
    1
   ],
   "LABELS": [
    "S 字（AviUtl2 と同じ）",
    "直線（Premiere Pro・テキストの塗りと同じ）"
   ],
   "DEFAULT": 0
  },
  {
   "NAME": "shape",
   "LABEL": "形状",
   "TYPE": "long",
   "VALUES": [
    0,
    1
   ],
   "LABELS": [
    "線形",
    "円形"
   ],
   "DEFAULT": 0
  },
  {
   "NAME": "angle",
   "LABEL": "角度",
   "TYPE": "float",
   "DEFAULT": 90,
   "MIN": -360,
   "MAX": 360
  },
  {
   "NAME": "span",
   "LABEL": "幅",
   "TYPE": "float",
   "DEFAULT": 100,
   "MIN": 1,
   "MAX": 4000
  },
  {
   "NAME": "center_x",
   "LABEL": "中心 X",
   "TYPE": "float",
   "DEFAULT": 0,
   "MIN": -4000,
   "MAX": 4000
  },
  {
   "NAME": "center_y",
   "LABEL": "中心 Y",
   "TYPE": "float",
   "DEFAULT": 0,
   "MIN": -4000,
   "MAX": 4000
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
const int MAX_GRADIENT_STOPS = 16;










// 色の止まりの並び（#364） 数が 2 より少なければ前からの開始色と終了色の 2 色で塗る
// 色は sRGB のまま届く（混ぜるのも sRGB なので、リニアへ往復させない）

vec4 gradient_colors[MAX_GRADIENT_STOPS];
float gradient_offsets[MAX_GRADIENT_STOPS];
// 色の混ぜ方 0 が S 字（既定 前の版と同じ）、1 が直線


float eased(float k) {
    return mixing == 1 ? k : k * k * (3.0 - 2.0 * k);
}

// 合成は符号化した値（sRGB）で計算する ここはリニアで持っているので往復する
vec3 to_srgb(vec3 c) {
    c = clamp(c, 0.0, 1.0);
    return mix(c * 12.92, 1.055 * pow(c, vec3(1.0 / 2.4)) - 0.055, step(0.0031308, c));
}
vec3 to_linear(vec3 c) {
    return mix(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), step(0.04045, c));
}

void main() {
    vec4 base = texture(u_texture, v_uv);

    // 中心を原点、右と下を正とした画素座標
    // v_uv の Y は上向きなので、ここで下向きに直す 設定の Y も上向きなので
    // 中心のずらし量も同じように反転する
    vec2 pixel = (v_uv - 0.5) * u_size;
    pixel.y = -pixel.y;
    vec2 centre = vec2(center_x, -center_y);
    // 下限は画面の 1 画素 合成の 1 画素で止めると、1/4 画質で幅 1 の帯が 4 倍に広がる
    float length_ = max(span, u_pixel_scale);

    float t;
    if (shape == 1) {
        // 円形 中心からの距離
        t = length(pixel - centre) / length_;
    } else {
        // 線形 角度 0 で上から下、90 で右から左（終了色の向き）
        //
        // AviUtl2 に 角度 0 と 90 のグラデーションを描かせて読み取った
        // 0 は上が開始色・下が終了色、90 は右が開始色・左が終了色だった
        // (cos, sin) のまま使うと 0 が左右、90 が上下になり、
        // 画面の端に寄せた配布物の文字が丸ごと終了色（多くは黒）で塗り潰される
        float radian = radians(angle);
        vec2 direction = vec2(-sin(radian), cos(radian));
        t = dot(pixel - centre, direction) / length_ + 0.5;
    }

    // 開始色から終了色への混ぜ方も **符号化した値（sRGB）で計算する**
    // AviUtl2 に赤から青のグラデーションを描かせると真ん中が (117, 0, 122) で、
    // 符号化した値の中点（127 付近）に当たる リニアで混ぜると真ん中が 186 の
    // 明るいマゼンタになり、配布物の中間色が全部派手になる
    //
    // 端をなだらかにしてから混ぜる（smoothstep） AviUtl2 に黒から白の階調を
    // 描かせて位置ごとの明るさを測ると、直線ではなく S 字だった
    // 直線で混ぜると、帯の両端が濃くなりすぎて中ほどが薄い別の絵になる
    float along = clamp(t, 0.0, 1.0);
    vec3 ramp_rgb;
    float ramp_alpha;
    if (gradient_count < 2) {
        // 前からの 2 色 前の版と同じ式のまま残す（読み込んだ作品の色を 1 段も動かさない）
        // 直線を選んだときだけ直線 既定の S 字は前と同じ式
        along = eased(along);
        ramp_rgb = mix(to_srgb(start_color.rgb), to_srgb(end_color.rgb), along);
        ramp_alpha = mix(start_color.a, end_color.a, along);
    } else {
        // 何色でも 止まりと止まりの間ごとに選んだ混ぜ方（既定の S 字か直線）で混ぜる
        // S 字を並び全体で 1 本にすると、
        // バーで 25% に置いた止まりの色が画面では 33% ほどの所に出る
        // 止まりが 2 つ（位置 0 と 1）なら上の 2 色の式と同じ絵になる
        int count = min(gradient_count, MAX_GRADIENT_STOPS);
        vec4 colour = gradient_colors[0];
        if (along > gradient_offsets[0]) {
            colour = gradient_colors[count - 1];
            for (int i = 1; i < count; ++i) {
                float right = gradient_offsets[i];
                if (along <= right) {
                    float left = gradient_offsets[i - 1];
                    float k = right > left ? (along - left) / (right - left) : 1.0;
                    k = clamp(k, 0.0, 1.0);
                    k = eased(k);
                    colour = mix(gradient_colors[i - 1], gradient_colors[i], k);
                    break;
                }
            }
        }
        ramp_rgb = colour.rgb;
        ramp_alpha = colour.a;
    }
    // 元の絵の不透明度はそのまま グラデーションは色だけを塗り替える
    float amount = clamp(strength * 0.01, 0.0, 1.0) * ramp_alpha;
    // 合成の仕方は塗りと同じ関数を使う AviUtl のグラデーションは
    // 加算や乗算で重ねる使い方が多く、通常だけだと配布物の見た目が出ない
    //
    // 合成は **符号化した値（sRGB）で計算する** ここはリニアで持っているので
    // 戻してから混ぜ、最後にリニアへ直す AviUtl も YMM4 も sRGB で混ぜており、
    // リニアのまま掛けると加算や乗算の見た目が別物になる
    vec3 under = to_srgb(base.rgb);
    vec3 rgb = mix(under, blend_colors(blend, under, ramp_rgb), amount);
    frag_color = vec4(to_linear(rgb), base.a);
}
