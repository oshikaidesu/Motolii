@description("Many things, four shapes to stand in — table, circle, helix, grid — and time walks through them (three.js css3d periodic table, 2011). Put everything at one point; the index says where each thing stands, the clock says which shape")

@label("Hold") @range(0.1, 120.0)
override hold: f32 = 3.0;
@label("Travel") @range(0.05, 60.0)
override travel: f32 = 1.2;
@label("Cell") @range(1.0, 10000.0)
override cell: f32 = 84.0;
@label("Radius") @range(1.0, 10000.0)
override radius: f32 = 420.0;
@label("Stagger") @range(0.0, 1.0)
override stagger: f32 = 0.004;

// 4 つの並び。番号 k と物の数 n から、その物の立ち位置(共通の点からのずれ)。
fn table(k: u32, n: u32, cell: f32) -> vec2f {
    let cols = 18u;
    let rows = (n + cols - 1u) / cols;
    let c = f32(k % cols) - f32(cols - 1u) * 0.5;
    let r = f32(k / cols) - f32(rows - 1u) * 0.5;
    return vec2f(c * cell, r * cell * 1.3);
}
fn circle(k: u32, n: u32, radius: f32) -> vec2f {
    // 2 重の環: 外に 2/3、内に 1/3(球を正面から見た密度の写し)。
    let outer = (n * 2u) / 3u;
    if k < outer {
        let a = 6.2831853 * f32(k) / f32(max(outer, 1u));
        return vec2f(cos(a), sin(a)) * radius;
    }
    let a = 6.2831853 * f32(k - outer) / f32(max(n - outer, 1u)) + 0.3;
    return vec2f(cos(a), sin(a)) * radius * 0.55;
}
fn helix(k: u32, n: u32, radius: f32, cell: f32) -> vec2f {
    let a = f32(k) * 0.42;
    let y = (f32(k) - f32(n) * 0.5) * cell * 0.12;
    return vec2f(cos(a) * radius, y + sin(a) * cell * 0.5);
}
fn grid(k: u32, n: u32, cell: f32) -> vec2f {
    let cols = 12u;
    let rows = (n + cols - 1u) / cols;
    let c = f32(k % cols) - f32(cols - 1u) * 0.5;
    let r = f32(k / cols) - f32(rows - 1u) * 0.5;
    return vec2f(c * cell * 1.1, r * cell * 1.1);
}
fn shape(i: u32, k: u32, n: u32) -> vec2f {
    switch i % 4u {
        case 0u: { return table(k, n, cell); }
        case 1u: { return circle(k, n, radius); }
        case 2u: { return helix(k, n, radius, cell); }
        default: { return grid(k, n, cell); }
    }
}

fn block(k: u32) -> Offset {
    let n = max(host.members, 1u);
    let period = hold + travel;
    // 番号で少し遅らせる(from center ではなく順): 群れが波のように移る。
    let t = max(host.time - f32(k) * stagger, 0.0);
    let i = u32(floor(t / period));
    let u = clamp((t - f32(i) * period - hold) / travel, 0.0, 1.0);
    let e = u * u * (3.0 - 2.0 * u);
    let here = shape(i, k, n);
    let next = shape(i + 1u, k, n);
    // 移る間は少し小さく薄く(手前を通る印象)。
    let dip = sin(e * 3.1415926);
    return Offset(mix(here, next, e), 0.0, 1.0 - 0.15 * dip, vec4f(1.0, 1.0, 1.0, 1.0 - 0.35 * dip));
}
