// interp.h: how a value goes through the stretch between two keys. Motolii's own nine kinds, brought as they are from
// the past repository (rust_ae/Motolii, motolii/crates/motolii-doc/src/eval/track.rs and bezier.rs): the same names,
// the same numbers in the same order, the same formulas, the same numbers refused. `interp_ease` is told u (how far
// through the stretch, 0 to 1) and gives how far the value has gone from the one key's to the other's: 0 at the
// start, 1 at the end (Cyclic ends where its wave is), and on the way it may pass the end and come back.
#include <cmath>
#include <sstream>
#include <string>

struct Interp {
    enum Kind { Hold, Linear, Bezier, Bounce, Elastic, Cyclic, Random, Steps, ElasticSteps } kind = Linear;
    double p[5] = {0, 0, 0, 0, 0};   // (its numbers, in the order of interp_kinds' names)
};
// name, how many numbers, their names, and the usual numbers when none are said: the past repository's own, as it
// found them good on the machine (rust_ae/Motolii, motolii/ui/native/src/editor/ease_kinds.rs).
// lo / hi: how far a hand takes each number, the past skin's own (motolii/ui/native/src/editor/ease_kinds.rs, its handles' clamps).
static const struct { const char* name; int numbers; const char* names; double usual[5], lo[5], hi[5]; } interp_kinds[] = {
    {"Hold", 0, "", {}, {}, {}},
    {"Linear", 0, "", {}, {}, {}},
    {"Bezier", 4, "x1 y1 x2 y2", {0.42, 0.0, 0.58, 1.0}, {0, -0.4, 0, -0.4}, {1, 1.6, 1, 1.6}},
    {"Bounce", 2, "first_dip dip", {0.27, 0.2}, {0.06, 0}, {0.7, 0.9}},
    {"Elastic", 3, "limit period damp", {1.5, 0.3, 0.35}, {1.02, 0.12, 0}, {2, 0.7, 0.97}},
    {"Cyclic", 4, "period peak linear envelope_end", {2.0 / 7.0, 0.5, 0.0, 0.0}, {0.08, 0.02, 0, 0}, {0.95, 0.98, 1, 0.95}},
    {"Random", 5, "seed grain center_u center_v bias", {500.0, 0.15, 0.5, 0.75, 0.5}, {1, 0, 0.05, -0.2, 0}, {999, 0.5, 0.95, 1.2, 1}},
    {"Steps", 2, "width smooth", {0.178, 0.0}, {0.06, 0}, {0.5, 0.45}},
    {"ElasticSteps", 2, "width elasticity", {0.2, 0.5}, {0.09, 0}, {0.5, 1}},
};

static double interp_smoothstep(double edge0, double edge1, double x) {
    if (std::fabs(edge1 - edge0) < 2.220446049250313e-16) return x < edge0 ? 0.0 : 1.0;
    const double t = std::fmin(std::fmax((x - edge0) / (edge1 - edge0), 0.0), 1.0);
    return t * t * (3.0 - 2.0 * t);
}
static double interp_mix(double a, double b, double t) { return a + (b - a) * t; }
static double interp_fract(double x) { return x - std::floor(x); }
static double interp_clamp(double x, double low, double high) { return std::fmin(std::fmax(x, low), high); }

// (bezier.rs: the CSS easing curve. x back to the curve's own parameter: Newton eight times, then halving: the past
//  repository's own earlier solver, kept there as the one its present solver is checked against)
static double interp_sample(double p1, double p2, double s) { const double inv = 1.0 - s; return 3.0 * inv * inv * s * p1 + 3.0 * inv * s * s * p2 + s * s * s; }
static double interp_bezier(double x1, double y1, double x2, double y2, double x) {
    if (x <= 0.0) return 0.0;
    if (x >= 1.0) return 1.0;
    double s = x;
    for (int i = 0; i < 8; ++i) {
        const double err = interp_sample(x1, x2, s) - x, inv = 1.0 - s, d = 3.0 * inv * inv * x1 + 6.0 * inv * s * (x2 - x1) + 3.0 * s * s * (1.0 - x2);
        if (std::fabs(err) < 1e-7) return interp_sample(y1, y2, s);
        if (std::fabs(d) < 1e-6) break;
        s -= err / d;
    }
    double lo = 0.0, hi = 1.0;
    s = interp_clamp(x, lo, hi);
    while (hi - lo > 1e-7) {
        if (interp_sample(x1, x2, s) < x) lo = s; else hi = s;
        s = (lo + hi) / 2.0;
    }
    return interp_sample(y1, y2, s);
}
// (a bounce like itself at every size: `first_dip` is when the first valley is, `dip` how deep; each bounce is 1 - dip of the one before, in height and in length)
static double interp_bounce(double first_dip, double dip, double u) {
    if (!std::isfinite(u) || u <= 0.0) return 0.0;
    if (u >= 1.0) return 1.0;
    const double d = interp_clamp(1.0 - dip, 0.02, 1.0), t = std::fmax(first_dip / (1.0 + d), 0.02);
    if (u <= t) return (u / t) * (u / t);
    double cusp = t, scale = d;
    for (int i = 0; i < 24; ++i) {
        const double width = 2.0 * t * scale;
        if (cusp + width > 1.0) return 1.0;
        if (u <= cusp + width) { const double local = (u - cusp - t * scale) / (t * scale); return 1.0 - scale * (1.0 - local * local); }
        cusp += width;
        scale *= d;
    }
    return 1.0;
}
// (v = 1 - (A - 1)(1 - u)^n cos(2 pi u / p): `limit` is the ceiling it passes the end by, `period` where the first valley is, `damp` how fast it dies)
static double interp_elastic(double limit, double period, double damp, double u) {
    if (!std::isfinite(u) || u <= 0.0) return 0.0;
    if (u >= 1.0) return 1.0;
    period = std::isfinite(period) && period > 0.0 ? period : 0.3;
    const double n = damp >= 0.97 ? 400.0 : 1.0 / ((1.0 - damp) * (1.0 - damp));
    const double closed = (limit - 1.0) * std::pow(1.0 - u, n), envelope = interp_mix(1.0, closed, interp_smoothstep(0.0, 0.45 * period, u));
    return 1.0 - envelope * std::cos((6.283185307179586 * u) / period);
}
// (a wave repeated: `peak` is the phase of its top, `linear` mixes cosine with triangle, `envelope_end` the end of the line its valleys ride on)
static double interp_wave(double peak, double linear, double phi) {
    const double s = interp_clamp(peak, 0.02, 0.98), tri = phi < s ? phi / s : (1.0 - phi) / (1.0 - s);
    const double smooth = phi < s ? (1.0 - std::cos((3.141592653589793 * phi) / s)) / 2.0 : (1.0 + std::cos((3.141592653589793 * (phi - s)) / (1.0 - s))) / 2.0;
    return interp_mix(smooth, tri, interp_clamp(linear, 0.0, 1.0));
}
static double interp_cyclic(double period, double peak, double linear, double envelope_end, double u) {
    if (!std::isfinite(u)) return 0.0;
    period = std::isfinite(period) && period > 0.0 ? period : 2.0 / 7.0;
    if (u >= 1.0) return envelope_end + (1.0 - envelope_end) * interp_wave(peak, linear, interp_fract(1.0 / period));
    const double floor = envelope_end * u;
    return floor + (1.0 - floor) * interp_wave(peak, linear, interp_fract(u / period));
}
// (shaken: `grain` is how fine, `center_u` and `center_v` where the shaking is strongest and how strong, `bias` pushes the band; it is still at both ends)
static double interp_lattice(double seed, double index) { const double value = std::sin((seed + index * 97.13) * 12.9898) * 43758.5453; return (value - std::floor(value)) * 2.0 - 1.0; }
static double interp_noise(double x, double seed) {
    const double index = std::floor(x), local = x - index;
    return interp_mix(interp_lattice(seed, index), interp_lattice(seed, index + 1.0), (1.0 - std::cos(3.141592653589793 * local)) / 2.0);
}
static double interp_random(double seed, double grain, double center_u, double center_v, double bias, double u) {
    if (!std::isfinite(u) || u <= 0.0) return 0.0;
    if (u >= 1.0) return 1.0;
    seed = std::round(seed);
    const double frequency = interp_mix(40.0, 8.0, interp_clamp(grain / 0.5, 0.0, 1.0));
    const double noise = 0.8 * interp_noise(u * frequency, seed) + 0.2 * interp_noise(u * frequency * 2.7, seed + 37.0);
    const double amplitude = interp_clamp(1.7 * std::fabs(center_v - 0.47), 0.0, 0.9);
    const double envelope = interp_smoothstep(0.0, 1.0, interp_clamp(1.0 - std::fabs(u - center_u) / 0.55, 0.0, 1.0));
    const double fade = interp_smoothstep(0.0, 0.05, u) * interp_smoothstep(1.0, 0.95, u);
    const double push = (0.5 - bias) * (1.0 - std::exp(-u / 0.15)) * (1.0 - std::exp(-(1.0 - u) / 0.15));
    return u + amplitude * envelope * fade * noise + push;
}
// (steps along the diagonal: `width` of a step, `smooth` how wide the smoothing that ARRIVES at each step's moment is)
static double interp_steps(double width, double smooth, double u) {
    if (!std::isfinite(u) || u <= 0.0) return 0.0;
    if (u >= 1.0) return 1.0;
    width = std::isfinite(width) && width > 0.0 ? width : 0.178;
    smooth = std::fmax(smooth, 0.0);
    const double index = std::floor(u / width), local = u - index * width, ramp_start = width - smooth;
    const double progress = smooth <= 0.0001 ? 0.0 : interp_smoothstep(0.0, 1.0, interp_clamp((local - ramp_start) / smooth, 0.0, 1.0));
    return interp_clamp(width * (index + progress), 0.0, 1.0);
}
// (springing steps: each change BEGINS at its step's moment, the other way round from Steps)
static double interp_elastic_steps(double width, double elasticity, double u) {
    if (!std::isfinite(u) || u <= 0.0) return 0.0;
    if (u >= 1.0) return 1.0;
    width = std::isfinite(width) && width > 0.0 ? width : 0.2;
    const double e = interp_clamp(elasticity, 0.0, 1.0);
    if (u < width) return 0.0;
    const double index = std::floor(u / width) - 1.0, tau = (u - (index + 1.0) * width) / width, rise = interp_mix(0.45, 0.03, e);
    double response = interp_smoothstep(0.0, rise, tau);
    if (tau > rise && e > 0.01) {
        const double ring_t = tau - rise, ring_amp = 0.36 * std::pow(e, 3.2);
        response += ring_amp * std::exp(-5.9 * ring_t) * std::sin((6.283185307179586 * ring_t) / 0.112);
    }
    response -= 0.045 * std::sin(3.141592653589793 * e) * interp_smoothstep(0.78, 0.99, tau);
    return interp_clamp(width * (index + response), -0.2, 1.35);
}

static double interp_ease(const Interp& i, double u) {
    const double* p = i.p;
    switch (i.kind) {
        case Interp::Hold: return 0.0;
        case Interp::Linear: return interp_clamp(u, 0.0, 1.0);
        case Interp::Bezier: return interp_bezier(interp_clamp(p[0], 0.0, 1.0), p[1], interp_clamp(p[2], 0.0, 1.0), p[3], u);
        case Interp::Bounce: return interp_bounce(p[0], p[1], u);
        case Interp::Elastic: return interp_elastic(p[0], p[1], p[2], u);
        case Interp::Cyclic: return interp_cyclic(p[0], p[1], p[2], p[3], u);
        case Interp::Random: return interp_random(p[0], p[1], p[2], p[3], p[4], u);
        case Interp::Steps: return interp_steps(p[0], p[1], u);
        case Interp::ElasticSteps: return interp_elastic_steps(p[0], p[1], u);
    }
    return u;
}

// A kind said back: its name as the table writes it, then all its numbers. (However it was said, it reads back so.)
static std::string interp_words(const Interp& is) {
    const auto& kind = interp_kinds[(size_t)is.kind];
    std::string said = kind.name;
    char one[40];
    for (int j = 0; j < kind.numbers; ++j) std::snprintf(one, sizeof one, " %.9g", is.p[j]), said += one;
    return said;
}

// A kind as it is said: its name (capitals or not) and then either none of its numbers (the usual ones) or all of
// them. 0: said rightly. -1: not the name of a kind. 1: a kind, but not rightly said (`why` tells how it is said).
static int interp_said(const std::string& text, Interp* is, std::string* why = nullptr) {
    std::istringstream said(text);
    std::string name, more;
    said >> name;
    for (size_t k = 0; k < sizeof interp_kinds / sizeof *interp_kinds; ++k) {
        const auto& kind = interp_kinds[k];
        if (strcasecmp(name.c_str(), kind.name)) continue;
        Interp made;
        made.kind = (Interp::Kind)k;
        int n = 0;
        for (double one = 0; n < 5 && said >> one;) made.p[n++] = one;
        said.clear();
        const bool right = !(said >> more) && (n == 0 || n == kind.numbers);
        if (n == 0) for (int j = 0; j < 5; ++j) made.p[j] = kind.usual[j];
        const double* p = made.p;
        bool finite = true;
        for (int j = 0; j < kind.numbers; ++j) finite = finite && std::isfinite(p[j]);
        const char* rule = !right ? "" : !finite ? "every number is a real one"   // (the past repository's own rules for the numbers)
            : made.kind == Interp::Bezier && (p[0] < 0.0 || p[0] > 1.0 || p[2] < 0.0 || p[2] > 1.0) ? "x1 and x2 are from 0 to 1"
            : made.kind == Interp::Bounce && p[0] <= 0.0 ? "first_dip is above 0"
            : (made.kind == Interp::Elastic && p[1] <= 0.0) || (made.kind == Interp::Cyclic && p[0] <= 0.0) ? "period is above 0"
            : (made.kind == Interp::Steps || made.kind == Interp::ElasticSteps) && p[0] <= 0.0 ? "width is above 0"
            : made.kind == Interp::Steps && p[1] < 0.0 ? "smooth is 0 or more" : nullptr;
        if (rule) {
            if (why) *why = std::string(kind.name) + (kind.numbers ? std::string(" is said alone or with its ") + std::to_string(kind.numbers) + " numbers (" + kind.names + ")" : " takes no numbers") + (*rule ? std::string(": ") + rule : "");
            return 1;
        }
        *is = made;
        return 0;
    }
    return -1;
}
