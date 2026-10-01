//! GSAP の ease の**名前**を Motolii の既存の型へ写す表(利用者「ease は Motolii が持ってるだろ」— 評価器も型も足さない)。
//! 文法は GSAP `_parseEase` → `_configEaseFromString`(`gsap-core.js` 1016–1022): `family[.in|.out|.inOut][(a, b)]`、向きが無ければ `.out`。
//! 写し先: power1..4 / sine / expo / circ / back.inOut は CSS の cubic-bezier(easings.net の表)、power1・power2・back の in/out は
//! 多項式なので Bezier に**厳密に**乗る(x1 = 1/3, x2 = 2/3 で x(t) = t、Bernstein 係数から y1・y2)、elastic → 既存 Elastic、
//! bounce → 既存 Bounce、steps(n) → 既存 Steps、none → Linear。近似の誤差は test の表に記す(定規は本物の GSAP)。
use crate::doc::eval::Interp;

pub const FAMILIES: &[&str] = &["none", "power1", "power2", "power3", "power4", "sine", "expo", "circ", "back", "elastic", "bounce", "steps"];
/// GSAP にあって写せない物(既存の型に向きの反転が無い、式が別)。断る時に名指しする。
pub const UNMAPPED: &[&str] = &["elastic.in", "elastic.inOut", "bounce.in", "bounce.inOut", "steps(n, true)", "slow", "rough", "expoScale", "CustomEase"];

/// CSS の cubic-bezier の表(easings.net、`family: [in, out, inOut]`)。
const BEZIERS: &[(&str, [[f64; 4]; 3])] = &[
    ("power1", [[1.0 / 3.0, 0.0, 2.0 / 3.0, 1.0 / 3.0], [1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0, 1.0], [0.45, 0.0, 0.55, 1.0]]), // in/out = p² / 1 − (1 − p)² の厳密解
    ("power2", [[1.0 / 3.0, 0.0, 2.0 / 3.0, 0.0], [1.0 / 3.0, 1.0, 2.0 / 3.0, 1.0], [0.65, 0.0, 0.35, 1.0]]), // in/out = p³ / 1 − (1 − p)³ の厳密解
    ("power3", [[0.5, 0.0, 0.75, 0.0], [0.25, 1.0, 0.5, 1.0], [0.76, 0.0, 0.24, 1.0]]),
    ("power4", [[0.64, 0.0, 0.78, 0.0], [0.22, 1.0, 0.36, 1.0], [0.83, 0.0, 0.17, 1.0]]),
    ("sine", [[0.12, 0.0, 0.39, 0.0], [0.61, 1.0, 0.88, 1.0], [0.37, 0.0, 0.63, 1.0]]),
    ("expo", [[0.7, 0.0, 0.84, 0.0], [0.16, 1.0, 0.3, 1.0], [0.87, 0.0, 0.13, 1.0]]),
    ("circ", [[0.55, 0.0, 1.0, 0.45], [0.0, 0.55, 0.45, 1.0], [0.85, 0.0, 0.15, 1.0]]),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Dir {
    In,
    Out,
    InOut,
}

/// GSAP の ease の文字列を既存の型に。知らない名前・写せない名前は理由を付けて断る。
pub fn interp_for(text: &str) -> Result<Interp, String> {
    let text = text.trim();
    let (head, args) = match text.split_once('(') {
        Some((head, rest)) => (head.trim(), rest.trim_end_matches(')').split(',').map(str::trim).filter(|s| !s.is_empty()).collect::<Vec<_>>()),
        None => (text, Vec::new()),
    };
    let (family_text, dir_text) = head.split_once('.').map_or((head, None), |(f, d)| (f, Some(d)));
    let family = match family_text.to_ascii_lowercase().as_str() {
        "none" | "linear" | "power0" => "none",
        "power1" | "quad" => "power1",
        "power2" | "cubic" => "power2",
        "power3" | "quart" => "power3",
        "power4" | "quint" | "strong" => "power4",
        "sine" => "sine",
        "expo" => "expo",
        "circ" => "circ",
        "back" => "back",
        "elastic" => "elastic",
        "bounce" => "bounce",
        "steps" | "steppedease" => "steps",
        _ => return Err(format!("Unknown GSAP ease {text:?}. Families: {} (with .in / .out / .inOut); not mapped: {}", FAMILIES.join(", "), UNMAPPED.join(", "))),
    };
    let dir = match dir_text.map(|d| d.to_ascii_lowercase()).as_deref() {
        None => Dir::Out,
        Some("in" | "easein") => Dir::In,
        Some("out" | "easeout") => Dir::Out,
        Some("inout" | "easeinout") => Dir::InOut,
        Some(other) => return Err(format!("Unknown ease direction {other:?} in {text:?}: in, out or inOut")),
    };
    let number = |i: usize, default: f64| -> Result<f64, String> {
        match args.get(i) {
            None => Ok(default),
            Some(s) => match *s {
                "true" => Ok(1.0),
                "false" => Ok(0.0),
                s => s.parse::<f64>().map_err(|_| format!("{s:?} is not a number in {text:?}")),
            },
        }
    };
    let unmapped = |what: &str| Err(format!("{text:?} has no Motolii ease: {what}. Not mapped: {}", UNMAPPED.join(", ")));
    let bezier = |c: [f64; 4]| Interp::Bezier { x1: c[0], y1: c[1], x2: c[2], y2: c[3] };
    Ok(match family {
        "none" => Interp::Linear,
        // `_configBack`: out(p) = (p−1)²((s+1)(p−1) + s) + 1 = (s+1)p³ − (2s+3)p² + (s+3)p → Bezier(1/3, (s+3)/3, 2/3, 1)、in はその鏡。
        "back" => {
            let s = number(0, 1.70158)?;
            match dir {
                Dir::Out => bezier([1.0 / 3.0, (s + 3.0) / 3.0, 2.0 / 3.0, 1.0]),
                Dir::In => bezier([1.0 / 3.0, 0.0, 2.0 / 3.0, -s / 3.0]),
                Dir::InOut if args.is_empty() => bezier([0.68, -0.6, 0.32, 1.6]),
                Dir::InOut => return unmapped("back.inOut takes no overshoot here (easings.net has only the default 1.70158)"),
            }
        }
        // `_configElastic(amplitude = 1, period = .3)`: amplitude < 1 は period を amplitude で割って amplitude = 1 に。
        // 既存 Elastic{limit, period, damp}: period は同じ語(u の周期)、limit は最初の山の高さで合わせる(1 + amplitude / 2)、damp は既定 0.35。
        "elastic" => {
            if dir != Dir::Out {
                return unmapped("Elastic has no in / inOut");
            }
            let (amplitude, period) = (number(0, 1.0)?, number(1, 0.3)?);
            if amplitude <= 0.0 || period <= 0.0 {
                return Err(format!("{text:?}: amplitude and period must be positive"));
            }
            let (amplitude, period) = if amplitude >= 1.0 { (amplitude, period) } else { (1.0, period / amplitude) };
            Interp::Elastic { limit: 1.0 + amplitude / 2.0, period, damp: 0.35 }
        }
        // GSAP bounce.out (7.5625, 2.75): 最初の谷は p = 1.5 / 2.75 で値 .75。既存 Bounce{first_dip, dip} はその谷の (u, v)(減衰の則は別 — test の表)。
        "bounce" => {
            if dir != Dir::Out {
                return unmapped("Bounce has no in / inOut");
            }
            Interp::Bounce { first_dip: 1.5 / 2.75, dip: 0.75 }
        }
        // `SteppedEase.config(steps, immediateStart)`: 段の数 n → 既存 Steps{width: 1/n}(段の時刻は GSAP が k/(n+1)、ここは k/n — test の表)。
        "steps" => {
            let n = number(0, 1.0)?;
            if n < 1.0 {
                return Err(format!("{text:?}: steps must be 1 or more"));
            }
            if number(1, 0.0)? != 0.0 {
                return unmapped("steps(n, true) (immediateStart)");
            }
            Interp::Steps { width: 1.0 / n, smooth: 0.0 }
        }
        _ => {
            let (_, table) = BEZIERS.iter().find(|(f, _)| *f == family).expect("every family in BEZIERS");
            bezier(table[match dir { Dir::In => 0, Dir::Out => 1, Dir::InOut => 2 }])
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 定規は本物の GSAP(gsap-core.js を node で、`gsap.parseEase(name)(t)`、t = .25 / .5 / .75)。右端は写しの誤差の上限(最大 |Motolii − GSAP|)。
    /// 0 の行は厳密(power1・power2・back の in/out は多項式、none)。他は easings.net の cubic-bezier / 既存の型の近似。
    const RULER: &[(&str, [f64; 3], f64)] = &[
        ("none", [0.25, 0.5, 0.75], 0.0),
        ("power1.in", [0.0625, 0.25, 0.5625], 1e-6),
        ("power1.out", [0.4375, 0.75, 0.9375], 1e-6),
        ("power1.inOut", [0.125, 0.5, 0.875], 0.005),
        ("power2.in", [0.015625, 0.125, 0.421875], 1e-6),
        ("power2.out", [0.578125, 0.875, 0.984375], 1e-6),
        ("power2.inOut", [0.0625, 0.5, 0.9375], 0.009),
        ("power3.in", [0.00390625, 0.0625, 0.31640625], 0.005),
        ("power3.out", [0.68359375, 0.9375, 0.99609375], 0.005),
        ("power3.inOut", [0.03125, 0.5, 0.96875], 0.022),
        ("power4.in", [0.0009765625, 0.03125, 0.2373046875], 0.008),
        ("power4.out", [0.7626953125, 0.96875, 0.9990234375], 0.008),
        ("power4.inOut", [0.015625, 0.5, 0.984375], 0.029),
        ("sine.in", [0.076120467489, 0.292893218813, 0.617316567635], 0.008),
        ("sine.out", [0.382683432365, 0.707106781187, 0.923879532511], 0.008),
        ("sine.inOut", [0.146446609407, 0.5, 0.853553390593], 0.002),
        ("expo.in", [0.001564173401, 0.0234375, 0.177077150379], 0.005),
        ("expo.out", [0.822922849621, 0.9765625, 0.998435826599], 0.005),
        ("expo.inOut", [0.01171875, 0.5, 0.98828125], 0.029),
        ("circ.in", [0.031754163448, 0.133974596216, 0.338562172234], 0.002),
        ("circ.out", [0.661437827766, 0.866025403784, 0.968245836552], 0.002),
        ("circ.inOut", [0.066987298108, 0.5, 0.933012701892], 0.025),
        ("back.in", [-0.0641365625, -0.0876975, 0.1825903125], 1e-6),
        ("back.out", [0.8174096875, 1.0876975, 1.0641365625], 1e-6),
        ("back.out(1.7)", [0.8171875, 1.0875, 1.0640625], 1e-6),
        ("back.inOut", [-0.04384875, 0.5, 1.04384875], 0.054),
        ("elastic.out", [0.911611652352, 1.015625, 1.005524271728], 0.039),
        ("elastic.out(1, 0.3)", [0.911611652352, 1.015625, 1.005524271728], 0.039),
        ("bounce.out", [0.47265625, 0.765625, 0.97265625], 0.15),
        ("steps(5)", [0.2, 0.6, 0.8], 0.2),
    ];

    #[test]
    fn mapped_eases_stay_within_the_recorded_error_of_gsap() {
        let mut over = Vec::new();
        for (name, gsap, tolerance) in RULER {
            let interp = interp_for(name).unwrap_or_else(|e| panic!("{name}: {e}"));
            let worst = [0.25, 0.5, 0.75].iter().zip(gsap).map(|(t, g)| (interp.ease(*t) - g).abs()).fold(0.0, f64::max);
            eprintln!("{name}: max error {worst:.4} (recorded {tolerance})");
            if worst > *tolerance {
                over.push(format!("{name}: {worst:.4} > {tolerance}"));
            }
        }
        assert!(over.is_empty(), "over the recorded error: {over:?}");
    }

    /// 向きが無ければ `.out`(`_easeMap[lowercase] = easeOut`)、別名と大文字形は同じ物、写せない物は名指しで断る。
    #[test]
    fn the_string_grammar_is_gsaps_and_unmapped_names_are_refused_by_name() {
        let same = |a: &str, b: &str| assert_eq!(interp_for(a).unwrap(), interp_for(b).unwrap(), "{a} = {b}");
        same("power2", "power2.out");
        same("back", "back.out(1.70158)");
        same("elastic", "elastic.out(1, 0.3)");
        same("Power2.easeInOut", "power2.inOut");
        same("cubic.in", "power2.in");
        same("strong.out", "power4.out");
        same("linear", "none");
        assert!(interp_for("swing").unwrap_err().contains("power1"));
        assert!(interp_for("power2.up").unwrap_err().contains("inOut"));
        for name in ["elastic.in", "bounce.inOut", "steps(5, true)", "slow", "steps(0)"] {
            assert!(interp_for(name).is_err(), "{name} is not mapped");
        }
    }
}
