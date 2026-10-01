// 関数の棚の下の段(WESL の module `package::processing`): Processing の語彙(reference の名前と引数の順のまま)。WGSL の組み込みと同じ物は書かない
// (lerp = mix、constrain = clamp、dist = distance、sq = x * x)。上の段(cavalry.wgsl)はこれを import して呼ぶ。
// 頭の JSON も `fn block(` も無い .wgsl は札でなく module。札は `import package::processing::{ map, random };` で引く。

/// map(value, start1, stop1, start2, stop2): 範囲の写し替え。
fn map(value: f32, start1: f32, stop1: f32, start2: f32, stop2: f32) -> f32 {
    return start2 + (stop2 - start2) * (value - start1) / (stop1 - start1);
}

/// norm(value, start, stop): 範囲を 0..1 に。
fn norm(value: f32, start: f32, stop: f32) -> f32 {
    return (value - start) / (stop - start);
}

/// random(index, seed): 番号と種から [0, 1)。同じ番号は毎コマ同じ(randomSeed の代わりに seed を渡す)。
fn random(index: u32, seed: u32) -> f32 {
    var h = (index + 1u) * 2654435761u + seed * 2246822519u;
    h = (h ^ (h >> 15u)) * 2246822519u;
    h = (h ^ (h >> 13u)) * 3266489917u;
    return f32(h >> 8u) / 16777216.0;
}

fn noise_hash(c: vec2f, seed: u32) -> f32 {
    return random(u32(abs(c.x) * 7.0 + abs(c.y) * 131.0), seed) * 2.0 - 1.0;
}

/// noise(x, y): 2D の値ノイズ、[-1, 1]。Processing の noise は [0, 1] だが、ずれに掛けるので中心 0 にしてある。
fn noise(x: f32, y: f32) -> f32 {
    let q = vec2f(x, y);
    let i = floor(q);
    let f = fract(q);
    let u = f * f * (3.0 - 2.0 * f);
    let a = noise_hash(i, 9u);
    let b = noise_hash(i + vec2f(1.0, 0.0), 9u);
    let c = noise_hash(i + vec2f(0.0, 1.0), 9u);
    let d = noise_hash(i + vec2f(1.0, 1.0), 9u);
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

/// TWO_PI と degrees/radians は WGSL にもあるが、名前で探せるように。
const TWO_PI: f32 = 6.2831853;
const HALF_PI: f32 = 1.5707963;
