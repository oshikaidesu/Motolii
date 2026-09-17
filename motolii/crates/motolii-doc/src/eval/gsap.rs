//! GSAP v3 の ease(MIT)。`src/gsap-core.js` の EASING 節(`_insertEase` / `_configElastic` / `_configBack` /
//! `SteppedEase` / Power・Expo・Circ・Sine・Bounce)と `src/EasePack.js` の `_createSlowMo` を式のまま写す。
//! 名前と既定も GSAP の語: `"power2.out"` `"back.out(1.7)"` `"elastic.out(1, 0.3)"` `"steps(5)"` `"slow(0.7, 0.7, false)"` `"none"`。
//! 文法は `_parseEase` → `_configEaseFromString`: `family[.in|.out|.inOut][(a, b, c)]`、向きが無ければ GSAP と同じく `.out`。
use serde::{Deserialize, Serialize};
use std::f64::consts::{FRAC_PI_2, TAU};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GsapFamily {
    None,
    Power1,
    Power2,
    Power3,
    Power4,
    Expo,
    Circ,
    Sine,
    Back,
    Elastic,
    Bounce,
    Steps,
    Slow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EaseDir {
    In,
    Out,
    InOut,
}

/// `p1..p3` は family ごとの引数(GSAP の config の順): back = overshoot、elastic = amplitude, period、
/// steps = steps, immediateStart、slow = linearRatio, power, yoyoMode。使わない物は 0。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GsapEase {
    pub family: GsapFamily,
    pub dir: EaseDir,
    pub p1: f64,
    pub p2: f64,
    pub p3: f64,
}

pub const FAMILIES: &[&str] = &[
    "none", "power1", "power2", "power3", "power4", "expo", "circ", "sine", "back", "elastic", "bounce", "steps", "slow",
];

impl GsapEase {
    /// `_parseEase` の文字列の読み。`gsap-core.js` の `_easeMap` の別名(linear / quad / cubic / quart / quint / strong /
    /// power0、`Power2.easeOut` の大文字形)も受ける。
    pub fn parse(text: &str) -> Result<GsapEase, String> {
        let text = text.trim();
        let (head, args) = match text.split_once('(') {
            Some((head, rest)) => (head.trim(), rest.trim_end_matches(')').split(',').map(str::trim).filter(|s| !s.is_empty()).collect::<Vec<_>>()),
            None => (text, Vec::new()),
        };
        let (family_text, dir_text) = match head.split_once('.') {
            Some((f, d)) => (f, Some(d)),
            None => (head, None),
        };
        let family = match family_text.to_ascii_lowercase().as_str() {
            "none" | "linear" | "power0" => GsapFamily::None,
            "power1" | "quad" => GsapFamily::Power1,
            "power2" | "cubic" => GsapFamily::Power2,
            "power3" | "quart" => GsapFamily::Power3,
            "power4" | "quint" | "strong" => GsapFamily::Power4,
            "expo" => GsapFamily::Expo,
            "circ" => GsapFamily::Circ,
            "sine" => GsapFamily::Sine,
            "back" => GsapFamily::Back,
            "elastic" => GsapFamily::Elastic,
            "bounce" => GsapFamily::Bounce,
            "steps" | "steppedease" => GsapFamily::Steps,
            "slow" | "slowmo" => GsapFamily::Slow,
            _ => return Err(format!("Unknown GSAP ease {text:?}. Families: {} (with .in / .out / .inOut)", FAMILIES.join(", "))),
        };
        let dir = match dir_text.map(|d| d.to_ascii_lowercase()).as_deref() {
            None => EaseDir::Out,
            Some("in" | "easein") => EaseDir::In,
            Some("out" | "easeout") => EaseDir::Out,
            Some("inout" | "easeinout") => EaseDir::InOut,
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
        let (p1, p2, p3) = match family {
            GsapFamily::Back => (number(0, 1.70158)?, 0.0, 0.0),
            // `_configElastic`: period || (type ? .3 : .45) — inOut は type 無しで呼ばれる。
            GsapFamily::Elastic => (number(0, 1.0)?, number(1, if dir == EaseDir::InOut { 0.45 } else { 0.3 })?, 0.0),
            GsapFamily::Steps => (number(0, 1.0)?, number(1, 0.0)?, 0.0),
            // `_createSlowMo`: linearRatio || 0.7、power は 0 も有効なので既定は「無ければ 0.7」。
            GsapFamily::Slow => (number(0, 0.7)?, number(1, 0.7)?, number(2, 0.0)?),
            _ => (0.0, 0.0, 0.0),
        };
        let ease = GsapEase { family, dir, p1, p2, p3 };
        ease.validate()?;
        Ok(ease)
    }

    pub fn validate(&self) -> Result<(), String> {
        if ![self.p1, self.p2, self.p3].iter().all(|v| v.is_finite()) {
            return Err(format!("{self}: every argument must be finite"));
        }
        match self.family {
            GsapFamily::Elastic if self.p1 <= 0.0 || self.p2 <= 0.0 => Err(format!("{self}: amplitude and period must be positive")),
            GsapFamily::Steps if self.p1 < 1.0 => Err(format!("{self}: steps must be 1 or more")),
            GsapFamily::Slow if self.p1 <= 0.0 => Err(format!("{self}: linearRatio must be positive")),
            _ => Ok(()),
        }
    }

    /// 終点を越えるか(Back と Elastic)。
    pub fn overshoots(&self) -> bool {
        matches!(self.family, GsapFamily::Back | GsapFamily::Elastic)
    }

    pub fn ease(&self, p: f64) -> f64 {
        let p = if p.is_finite() { p.clamp(0.0, 1.0) } else { 0.0 };
        match self.family {
            GsapFamily::None => p,
            GsapFamily::Power1 => power(2, self.dir, p),
            GsapFamily::Power2 => power(3, self.dir, p),
            GsapFamily::Power3 => power(4, self.dir, p),
            GsapFamily::Power4 => power(5, self.dir, p),
            // `_insertEase("Expo", p => (2 ** (10 * (p - 1))) * p + p * p * p * p * p * p * (1-p))`
            GsapFamily::Expo => from_in(self.dir, p, |p| 2f64.powf(10.0 * (p - 1.0)) * p + p.powi(6) * (1.0 - p)),
            // `_insertEase("Circ", p => -(_sqrt(1 - (p * p)) - 1))`
            GsapFamily::Circ => from_in(self.dir, p, |p| -((1.0 - p * p).sqrt() - 1.0)),
            // `_insertEase("Sine", p => p === 1 ? 1 : -_cos(p * _HALF_PI) + 1)`
            GsapFamily::Sine => from_in(self.dir, p, |p| if p == 1.0 { 1.0 } else { -(p * FRAC_PI_2).cos() + 1.0 }),
            // `_configBack`: easeOut = p => p ? ((--p) * p * ((overshoot + 1) * p + overshoot) + 1) : 0
            GsapFamily::Back => {
                let s = self.p1;
                from_out(self.dir, p, |p| if p == 0.0 { 0.0 } else { let p = p - 1.0; p * p * ((s + 1.0) * p + s) + 1.0 })
            }
            // `_configElastic`: p1 = amplitude >= 1 ? amplitude : 1; p2 = period / (amplitude < 1 ? amplitude : 1);
            // p3 = p2 / 2π · asin(1 / p1); easeOut = p => p === 1 ? 1 : p1 · 2^(−10p) · sin((p − p3) · 2π / p2) + 1
            GsapFamily::Elastic => {
                let (amplitude, period) = (self.p1, self.p2);
                let a = if amplitude >= 1.0 { amplitude } else { 1.0 };
                let per = period / if amplitude < 1.0 { amplitude } else { 1.0 };
                let shift = per / TAU * (1.0 / a).asin();
                let shift = if shift.is_finite() { shift } else { 0.0 };
                from_out(self.dir, p, |p| if p == 1.0 { 1.0 } else { a * 2f64.powf(-10.0 * p) * ((p - shift) * (TAU / per)).sin() + 1.0 })
            }
            // (n, c) = (7.5625, 2.75); n1 = 1/c, n2 = 2/c, n3 = 2.5/c; easeIn = p => 1 − easeOut(1 − p)、inOut は easeIn から。
            GsapFamily::Bounce => {
                let (n, c) = (7.5625, 2.75);
                let out = move |p: f64| {
                    if p < 1.0 / c { n * p * p }
                    else if p < 2.0 / c { let p = p - 1.5 / c; n * p * p + 0.75 }
                    else if p < 2.5 / c { let p = p - 2.25 / c; n * p * p + 0.9375 }
                    else { let p = p - 2.625 / c; n * p * p + 0.984375 }
                };
                from_in(self.dir, p, move |p| 1.0 - out(1.0 - p))
            }
            // `SteppedEase.config(steps = 1, immediateStart)`: p1 = 1/steps, p2 = steps + (immediateStart ? 0 : 1),
            // p3 = immediateStart ? 1 : 0, max = 1 − 1e−8; p => (((p2 · clamp(0, max, p)) | 0) + p3) · p1
            GsapFamily::Steps => {
                let immediate = self.p2 != 0.0;
                let q1 = 1.0 / self.p1;
                let q2 = self.p1 + if immediate { 0.0 } else { 1.0 };
                let q3 = if immediate { 1.0 } else { 0.0 };
                (((q2 * p.clamp(0.0, 1.0 - 1e-8)).trunc()) + q3) * q1
            }
            // `_createSlowMo(linearRatio, power, yoyoMode)`: linearRatio = min(1, linearRatio || 0.7); pow = linearRatio < 1 ? power : 0;
            // p1 = (1 − linearRatio) / 2; p3 = p1 + linearRatio; r = p + (0.5 − p) · pow;
            // p < p1: yoyo ? 1 − q² : r − q⁴ · r (q = 1 − p / p1); p > p3: yoyo ? (p === 1 ? 0 : 1 − q²) : r + (p − r) · q⁴ (q = (p − p3) / p1); else yoyo ? 1 : r
            GsapFamily::Slow => {
                let linear = (if self.p1 == 0.0 { 0.7 } else { self.p1 }).min(1.0);
                let pow = if linear < 1.0 { self.p2 } else { 0.0 };
                let yoyo = self.p3 != 0.0;
                let a = (1.0 - linear) / 2.0;
                let b = a + linear;
                let r = p + (0.5 - p) * pow;
                if p < a {
                    let q = 1.0 - p / a;
                    if yoyo { 1.0 - q * q } else { r - q * q * q * q * r }
                } else if p > b {
                    let q = (p - b) / a;
                    if yoyo { if p == 1.0 { 0.0 } else { 1.0 - q * q } } else { r + (p - r) * q * q * q * q }
                } else if yoyo { 1.0 } else { r }
            }
        }
    }
}

/// `_insertEase(name, easeIn, easeOut = p => 1 − easeIn(1 − p), easeInOut = p => p < .5 ? easeIn(p·2)/2 : 1 − easeIn((1 − p)·2)/2)`
fn from_in(dir: EaseDir, p: f64, ease_in: impl Fn(f64) -> f64) -> f64 {
    match dir {
        EaseDir::In => ease_in(p),
        EaseDir::Out => 1.0 - ease_in(1.0 - p),
        EaseDir::InOut => if p < 0.5 { ease_in(p * 2.0) / 2.0 } else { 1.0 - ease_in((1.0 - p) * 2.0) / 2.0 },
    }
}

/// `_easeInOutFromOut = easeOut => p => p < .5 ? (1 − easeOut(1 − p·2)) / 2 : .5 + easeOut((p − .5)·2) / 2`、in は `1 − easeOut(1 − p)`
fn from_out(dir: EaseDir, p: f64, ease_out: impl Fn(f64) -> f64) -> f64 {
    match dir {
        EaseDir::Out => ease_out(p),
        EaseDir::In => 1.0 - ease_out(1.0 - p),
        EaseDir::InOut => if p < 0.5 { (1.0 - ease_out(1.0 - p * 2.0)) / 2.0 } else { 0.5 + ease_out((p - 0.5) * 2.0) / 2.0 },
    }
}

/// Linear,Quad,Cubic,Quart,Quint = 累乗 1..5: in = p^n、out = 1 − (1 − p)^n、inOut = p < .5 ? (p·2)^n / 2 : 1 − ((1 − p)·2)^n / 2
fn power(n: i32, dir: EaseDir, p: f64) -> f64 {
    match dir {
        EaseDir::In => p.powi(n),
        EaseDir::Out => 1.0 - (1.0 - p).powi(n),
        EaseDir::InOut => if p < 0.5 { (p * 2.0).powi(n) / 2.0 } else { 1.0 - ((1.0 - p) * 2.0).powi(n) / 2.0 },
    }
}

impl std::fmt::Display for GsapEase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dir = match self.dir { EaseDir::In => "in", EaseDir::Out => "out", EaseDir::InOut => "inOut" };
        let family = FAMILIES[self.family as usize];
        match self.family {
            GsapFamily::None => write!(f, "none"),
            GsapFamily::Back => write!(f, "{family}.{dir}({})", self.p1),
            GsapFamily::Elastic => write!(f, "{family}.{dir}({}, {})", self.p1, self.p2),
            GsapFamily::Steps if self.p2 != 0.0 => write!(f, "steps({}, true)", self.p1),
            GsapFamily::Steps => write!(f, "steps({})", self.p1),
            GsapFamily::Slow => write!(f, "slow({}, {}, {})", self.p1, self.p2, self.p3 != 0.0),
            _ => write!(f, "{family}.{dir}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 参照値は本物の GSAP(gsap-core.js + EasePack.js、node で `gsap.parseEase(name)(t)`、t = 0, .25, .5, .75, 1)。
    /// 式は各 family の腕の注釈にある。
    const REFERENCE: &[(&str, [f64; 5])] = &[
        ("none", [0.0, 0.25, 0.5, 0.75, 1.0]),
        ("power1.in", [0.0, 0.0625, 0.25, 0.5625, 1.0]),
        ("power1.out", [0.0, 0.4375, 0.75, 0.9375, 1.0]),
        ("power1.inOut", [0.0, 0.125, 0.5, 0.875, 1.0]),
        ("power2.in", [0.0, 0.015625, 0.125, 0.421875, 1.0]),
        ("power2.out", [0.0, 0.578125, 0.875, 0.984375, 1.0]),
        ("power2.inOut", [0.0, 0.0625, 0.5, 0.9375, 1.0]),
        ("power3.in", [0.0, 0.00390625, 0.0625, 0.31640625, 1.0]),
        ("power3.out", [0.0, 0.68359375, 0.9375, 0.99609375, 1.0]),
        ("power3.inOut", [0.0, 0.03125, 0.5, 0.96875, 1.0]),
        ("power4.in", [0.0, 0.0009765625, 0.03125, 0.2373046875, 1.0]),
        ("power4.out", [0.0, 0.7626953125, 0.96875, 0.9990234375, 1.0]),
        ("power4.inOut", [0.0, 0.015625, 0.5, 0.984375, 1.0]),
        ("expo.in", [0.0, 0.001564173401, 0.0234375, 0.177077150379, 1.0]),
        ("expo.out", [0.0, 0.822922849621, 0.9765625, 0.998435826599, 1.0]),
        ("expo.inOut", [0.0, 0.01171875, 0.5, 0.98828125, 1.0]),
        ("circ.in", [0.0, 0.031754163448, 0.133974596216, 0.338562172234, 1.0]),
        ("circ.out", [0.0, 0.661437827766, 0.866025403784, 0.968245836552, 1.0]),
        ("circ.inOut", [0.0, 0.066987298108, 0.5, 0.933012701892, 1.0]),
        ("sine.in", [0.0, 0.076120467489, 0.292893218813, 0.617316567635, 1.0]),
        ("sine.out", [0.0, 0.382683432365, 0.707106781187, 0.923879532511, 1.0]),
        ("sine.inOut", [0.0, 0.146446609407, 0.5, 0.853553390593, 1.0]),
        ("back.in", [0.0, -0.0641365625, -0.0876975, 0.1825903125, 1.0]),
        ("back.out", [0.0, 0.8174096875, 1.0876975, 1.0641365625, 1.0]),
        ("back.inOut", [0.0, -0.04384875, 0.5, 1.04384875, 1.0]),
        ("back.out(1.7)", [0.0, 0.8171875, 1.0875, 1.0640625, 1.0]),
        ("elastic.in", [0.0, -0.005524271728, -0.015625, 0.088388347648, 1.0]),
        ("elastic.out", [0.0, 0.911611652352, 1.015625, 1.005524271728, 1.0]),
        ("elastic.inOut", [0.0, 0.011969444424, 0.5, 0.988030555576, 1.0]),
        ("elastic.out(1, 0.3)", [0.0, 0.911611652352, 1.015625, 1.005524271728, 1.0]),
        ("elastic.out(0.5, 0.5)", [0.0, 1.0, 1.03125, 1.0, 1.0]),
        ("bounce.in", [0.0, 0.02734375, 0.234375, 0.52734375, 1.0]),
        ("bounce.out", [0.0, 0.47265625, 0.765625, 0.97265625, 1.0]),
        ("bounce.inOut", [0.0, 0.1171875, 0.5, 0.8828125, 1.0]),
        ("steps(5)", [0.0, 0.2, 0.6, 0.8, 1.0]),
        ("steps(1)", [0.0, 0.0, 1.0, 1.0, 1.0]),
        ("slow", [0.0, 0.425, 0.5, 0.575, 1.0]),
        ("slow(0.7, 0.7, false)", [0.0, 0.425, 0.5, 0.575, 1.0]),
        ("slow(0.5, 0.8, true)", [0.0, 1.0, 1.0, 1.0, 0.0]),
        ("slow(0.3, 0, false)", [0.0, 0.248334027489, 0.5, 0.75, 1.0]),
    ];

    #[test]
    fn every_gsap_ease_matches_gsap_at_five_points() {
        for (name, expected) in REFERENCE {
            let ease = GsapEase::parse(name).unwrap_or_else(|e| panic!("{name}: {e}"));
            for (i, t) in [0.0, 0.25, 0.5, 0.75, 1.0].into_iter().enumerate() {
                let got = ease.ease(t);
                assert!((got - expected[i]).abs() < 1e-9, "{name} at {t}: got {got}, GSAP {}", expected[i]);
            }
        }
    }

    /// 向きが無ければ `.out`(`_easeMap[lowercase] = easeOut`)、別名と大文字形も同じ物、表示は読み直せる。
    #[test]
    fn the_string_grammar_is_gsaps() {
        let same = |a: &str, b: &str| assert_eq!(GsapEase::parse(a).unwrap(), GsapEase::parse(b).unwrap(), "{a} = {b}");
        same("power2", "power2.out");
        same("back", "back.out(1.70158)");
        same("elastic", "elastic.out(1, 0.3)");
        same("elastic.inOut", "elastic.inOut(1, 0.45)");
        same("Power2.easeInOut", "power2.inOut");
        same("cubic.in", "power2.in");
        same("strong.out", "power4.out");
        same("linear", "none");
        for (name, _) in REFERENCE {
            let ease = GsapEase::parse(name).unwrap();
            assert_eq!(GsapEase::parse(&ease.to_string()).unwrap(), ease, "{name} → {ease}");
        }
        assert!(GsapEase::parse("swing").unwrap_err().contains("power1"));
        assert!(GsapEase::parse("power2.up").unwrap_err().contains("inOut"));
        assert!(GsapEase::parse("steps(0)").is_err());
    }
}
