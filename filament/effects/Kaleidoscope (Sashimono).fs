/*{
 "DESCRIPTION": "Sashimono Edit's 万華鏡 (kaleidoscope, 変形; MIT, src/sashimono/effects/builtin.py), made an ISF effect by filament/effects/from_sashimono.py",
 "CREDIT": "Sashimono Edit (MIT)",
 "CATEGORIES": [
  "Distort"
 ],
 "ISFVSN": "2",
 "INPUTS": [
  {
   "NAME": "inputImage",
   "TYPE": "image"
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
   "NAME": "span",
   "LABEL": "長さ",
   "TYPE": "float",
   "DEFAULT": 100,
   "MIN": 1,
   "MAX": 4000
  },
  {
   "NAME": "angle",
   "LABEL": "回転",
   "TYPE": "float",
   "DEFAULT": 0,
   "MIN": -3600,
   "MAX": 3600
  },
  {
   "NAME": "corners",
   "LABEL": "角数",
   "TYPE": "float",
   "DEFAULT": 6,
   "MIN": 4,
   "MAX": 64
  },
  {
   "NAME": "repeats",
   "LABEL": "繰り返し回数",
   "TYPE": "float",
   "DEFAULT": 1,
   "MIN": 1,
   "MAX": 32
  },
  {
   "NAME": "fixed_size",
   "LABEL": "固定サイズ",
   "TYPE": "float",
   "DEFAULT": 0,
   "MIN": 0,
   "MAX": 8000
  },
  {
   "NAME": "circle_mask",
   "LABEL": "円形マスク",
   "TYPE": "bool",
   "DEFAULT": false
  },
  {
   "NAME": "spin_pattern",
   "LABEL": "回転同期",
   "TYPE": "bool",
   "DEFAULT": false
  },
  {
   "NAME": "clip_outside",
   "LABEL": "領域外を透過",
   "TYPE": "bool",
   "DEFAULT": false
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












void main() {
    vec2 origin = object_center();
    vec2 p = v_uv * u_size - origin;

    // 角数は偶数に丸める 鏡を交互に返して 1 周させるので、奇数では継ぎ目が合わない
    // 下限を 4 にする 2 だと三角の頂角が 180 度になり、範囲の内接円の半径が 0 で何も描けない
    // （AviUtl2 で 2 を描かせた見本はまだ無い 設定 UI も 4 からにしてある）
    float n = max(4.0, floor(corners * 0.5 + 0.5) * 2.0);
    float half_ = PI / n;
    float length_ = max(span, 1.0);
    // 覆う範囲は、鏡の三角を 繰り返し回数 + 1 段ぶん並べた正多角形
    // 実測で 繰り返し 2・長さ 100 の端が 300、3・200 が 800 の辺に乗った
    float outer = (max(floor(repeats + 0.5), 1.0) + 1.0) * length_;
    // 固定サイズ は覆う範囲の差し渡しをその大きさへ縮める（実測 200 で半分）
    float zoom = fixed_size > 0.0 ? fixed_size / (2.0 * outer) : 1.0;
    p /= zoom;

    // 回転 は元の絵を時計回りに回してから鏡に映す 範囲と鏡の向きは動かない
    // 回転同期 を入れると、模様（範囲と鏡）もいっしょに時計回りに回る
    // 実測 回転 30 で範囲は横 600 のまま（差 13.5 → 4.0）、同期ありで縦 600 に
    // 変わった（差 14.9 → 4.0） 同期ありで元の絵を回さない読み方は差 12.8
    float turn = radians(angle);
    mat2 counter = mat2(cos(turn), sin(turn), -sin(turn), cos(turn));
    if (spin_pattern) {
        p = counter * p;
    }

    // 鏡の軸は真下 三角の頂点は軸から ±180/角数 の線の上に乗る
    // 田の縦棒が軸に沿って残り、横棒は鏡の線に消えることから軸が縦だと読んだ
    // 上か下かは、田の上半分と下半分の違いで決めた（真上だと差が 8.0、真下で 3.3）
    const float AXIS = -PI * 0.5;

    // 範囲の外は描かない 角を鏡の線へ畳んでから、軸へ下ろした長さで測る
    // 畳む式は、もともと軸の側の三角にある角をそのまま返す形にする
    // 符号を逆にすると軸の側の三角が左右に返り、底辺で折り返す回数の偶奇で
    // 返ったり返らなかったりする 回した絵で模様が崩れる（差 8.6）
    float phi = atan(p.y, p.x) - AXIS;
    float folded = half_ - abs(mod(phi + half_, 4.0 * half_) - 2.0 * half_);
    float along = length(p) * cos(folded);
    float apothem = outer * cos(half_);
    if (circle_mask ? length(p) > apothem : along > apothem) {
        frag_color = vec4(0.0);
        return;
    }

    // 角数 の三角で畳むのは中心のまわりだけで、その先は角数 6 の万華鏡として読む
    // 鏡の三角（頂角 360/角数、辺 長さ）を、辺 長さ の正三角形へ引き伸ばしてから畳む
    // 横は 1/(2 sin(180/角数)) 倍、軸に沿っては cos 30/cos(180/角数) 倍で、角数 6 では両方 1
    //
    // AviUtl2 の 角数 4 と 12 では、中心の外の頂点に集まる三角がどれも 6 枚で、
    // 横か縦へ潰れた六角形の模様が並び、角数 4 の中心の田は 0.8 倍に縮んでいた
    // 角数 の三角のまま底辺で折り返すと、外の頂点にも 角数 枚の三角が集まり、
    // 差は角数 4 で 10.4、12 で 12.3 引き伸ばすと 2.5 と 5.2
    // 三角の高さを保つ引き伸ばし方（横だけ tan 30/tan(180/角数) 倍）は 8.5 と 10.7、
    // 軸に沿った倍率を 3% 増減させると、どちらの角数でも差が増える
    // 中心ちょうどは角が決まらない（atan(0, 0) は実装しだいで NaN になる）ので畳まない
    if (length(p) >= 0.0001) {
        p = length(p) * vec2(cos(AXIS + folded), sin(AXIS + folded));
    }
    const float SIXTH = PI / 6.0;
    p *= vec2(0.5 / sin(half_), cos(SIXTH) / cos(half_));

    // 畳む 角を 1 つの三角へ折り返し、三角の底辺を越えたら底辺で折り返す
    // 繰り返せば、鏡を三角に組んだ万華鏡と同じ模様になる
    float base = length_ * cos(SIXTH);
    for (int i = 0; i < 256; ++i) {
        float radius = length(p);
        // 中心ちょうどは角が決まらない（atan(0, 0) は実装しだいで NaN になる）
        if (radius < 0.0001) break;
        phi = atan(p.y, p.x) - AXIS;
        folded = SIXTH - abs(mod(phi + SIXTH, 4.0 * SIXTH) - 2.0 * SIXTH);
        p = radius * vec2(cos(AXIS + folded), sin(AXIS + folded));
        // 軸が真下なので、軸に沿った長さは -p.y
        if (-p.y <= base) break;
        p.y = -2.0 * base - p.y;
    }

    // 読む所を反時計回りに回すと、映る絵は時計回りに回る
    p = counter * p;
    // 下の三角は左右を返して読む 田の字では見分けが付かず、F の字で分かった
    // （返さないと、三角の模様が中心の右へ逆向きに並んで差 4.4、返すと 2.6）
    p.x = -p.x;

    // 中心 は読む所をずらし、模様はオブジェクトの真ん中に置いたままにする
    // 実測 中心X 60・中心Y 30 で、範囲の端は動かず差は 1.7
    vec2 source = origin + vec2(center_x, center_y) + p;
    if (!clip_outside) {
        // 領域外を透過 を外すと、絵の外は縁の色を引き伸ばして埋める
        // 実測 白い四角で外すと範囲いっぱいが白く（光る割合 25.4%）、入れると
        // 四角の模様が並んだ（15.0%） どちらもこの読み方で差 0.1 未満
        source = clamp(source, u_object.xy + 0.5, u_object.zw - 0.5);
    }
    frag_color = sample_pixel(source);
}
