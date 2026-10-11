// Beam tracing of sunlight through flat-faced convex glass solids onto a floor (y = floorY).
// Any number of solids: light that leaves one and meets another goes on through it, solids take the sun from
// each other, and a solid may rest on another.
//
// Method: polygon beams (Heckbert & Hanrahan 1984, "Beam tracing polygonal objects"), one set of beams per
// wavelength, each exit beam projected onto the floor as a flat polygon whose brightness is the flux divided by
// the landed area (Sikachev, Tisevich, Ignatenko 2008, "Interactive caustics from polyhedra"). Nothing is
// sampled and nothing is blurred: a beam that enters one face and leaves another is an affine map, so every
// patch of light is an exact polygon with a constant brightness.
//
// Why a chamfer: a perfect right-angled block never splits colour. Parallel faces give back the entry direction,
// and at n > sqrt(2) light that entered one face is totally reflected by the adjacent one. Real cut glass has a
// 45 degree protective chamfer on every edge (0.1 to 0.5 mm on optical cubes); that is where the colour comes from.
//
// Glass: n(lambda) = nd + B (1/lambda^2 - 1/lambda_d^2), B = (nd - 1) / (V (1/lambda_F^2 - 1/lambda_C^2))
// (two-term Cauchy from the Abbe number V). Colour: CIE 1931 fit of Wyman, Sloan, Shirley 2013, weights
// normalised per channel so that all wavelengths together are white.
//
// Pure C++, no renderer types. Units: lengths as given; brightness is in units of the sun's irradiance on a
// surface facing it (1 = what the bare floor would get if it faced the sun).
#pragma once

#include <algorithm>
#include <array>
#include <atomic>
#include <cmath>
#include <condition_variable>
#include <cstdint>
#include <cstring>
#include <mutex>
#include <thread>
#include <type_traits>
#include <unordered_map>
#include <utility>
#include <vector>

namespace beam {

struct V3 {
    double x, y, z;
};
inline V3 operator+(V3 a, V3 b) { return {a.x + b.x, a.y + b.y, a.z + b.z}; }
inline V3 operator-(V3 a, V3 b) { return {a.x - b.x, a.y - b.y, a.z - b.z}; }
inline V3 operator*(V3 a, double s) { return {a.x * s, a.y * s, a.z * s}; }
inline double dot(V3 a, V3 b) { return a.x * b.x + a.y * b.y + a.z * b.z; }
inline V3 cross(V3 a, V3 b) { return {a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x}; }
inline double length(V3 a) { return std::sqrt(dot(a, a)); }
inline V3 normalize(V3 a) { return a * (1.0 / length(a)); }

// Half space n.p <= o. n is the outward unit normal of a face.
struct Plane {
    V3 n;
    double o;
};

struct Poly {
    static constexpr int CAP = 40;
    int n = 0;
    std::array<V3, CAP> p;
    Poly() {}
    Poly(const Poly& o) : n(o.n) { std::memcpy(p.data(), o.p.data(), (size_t)o.n * sizeof(V3)); }  // only the corners in use
    Poly& operator=(const Poly& o) {
        n = o.n;
        std::memcpy(p.data(), o.p.data(), (size_t)o.n * sizeof(V3));
        return *this;
    }
    void push(V3 v) {
        if (n < CAP) p[(size_t)n++] = v;
    }
};

inline double area(const Poly& P) {
    V3 s{0, 0, 0};
    for (int i = 1; i + 1 < P.n; ++i) s = s + cross(P.p[(size_t)i] - P.p[0], P.p[(size_t)i + 1] - P.p[0]);
    return 0.5 * length(s);
}

// Keeps the part of P where g >= 0. g is affine on the polygon's plane and is given at the vertices.
inline void clip(Poly& P, const double* g) {
    Poly out;
    for (int i = 0; i < P.n; ++i) {
        int j = (i + 1) % P.n;
        double a = g[i], b = g[j];
        if (a >= 0) out.push(P.p[(size_t)i]);
        if ((a >= 0) != (b >= 0)) {
            double t = a / (a - b);
            out.push(P.p[(size_t)i] + (P.p[(size_t)j] - P.p[(size_t)i]) * t);
        }
    }
    P = out;
}

struct Solid {
    // kind FLAT: a convex solid with flat faces
    std::vector<Plane> planes;
    std::vector<Poly> faces;  // faces[i] lies on planes[i]; may be empty (n == 0) if the plane cuts nothing
    // round kinds (only the cheap trace and the renderer know them; the exact beam trace is for flat faces)
    enum Kind { FLAT = 0, BALL = 1, ROD = 2, CONE = 3 };
    int kind = FLAT;
    V3 c{0, 0, 0};    // BALL: centre. ROD: middle of the axis. CONE: middle of the base
    V3 ax{0, 1, 0};   // ROD: unit axis. CONE: unit axis, from the base to the tip
    double r = 0;     // BALL, ROD: radius. CONE: radius of the base
    double h = 0;     // ROD: half length. CONE: height
};
inline Solid ball(double radius) {
    Solid s;
    s.kind = Solid::BALL;
    s.r = radius;
    return s;
}
// A round bar (cylinder) along +Y.
inline Solid rod(double radius, double halfLength) {
    Solid s;
    s.kind = Solid::ROD;
    s.r = radius;
    s.h = halfLength;
    return s;
}
// A cone along +Y, its own origin half way up (base at -height / 2, tip at +height / 2).
inline Solid cone(double radius, double height) {
    Solid s;
    s.kind = Solid::CONE;
    s.c = {0, -0.5 * height, 0};
    s.r = radius;
    s.h = height;
    return s;
}

inline void build_faces(Solid& s, double reach) {
    s.faces.assign(s.planes.size(), Poly{});
    for (size_t i = 0; i < s.planes.size(); ++i) {
        const Plane& pl = s.planes[i];
        V3 up = std::fabs(pl.n.y) < 0.9 ? V3{0, 1, 0} : V3{1, 0, 0};
        V3 u = normalize(cross(up, pl.n)), v = cross(pl.n, u), c = pl.n * pl.o;
        Poly P;
        P.push(c - u * reach - v * reach);
        P.push(c + u * reach - v * reach);
        P.push(c + u * reach + v * reach);
        P.push(c - u * reach + v * reach);
        for (size_t k = 0; k < s.planes.size() && P.n >= 3; ++k) {
            if (k == i) continue;
            double g[Poly::CAP];
            for (int q = 0; q < P.n; ++q) g[q] = s.planes[k].o - dot(s.planes[k].n, P.p[(size_t)q]);
            clip(P, g);
        }
        if (P.n >= 3 && area(P) > 1e-14 * reach * reach) s.faces[i] = P;
    }
}

// A box of half size `half`, every edge cut by a 45 degree chamfer whose flat is `chamferWidth` wide
// (0 = perfect box), turned by `yaw` about +Y and placed at `center`.
inline Solid chamfered_box(V3 center, double yaw, double half, double chamferWidth) {
    Solid s;
    const double cs = std::cos(yaw), sn = std::sin(yaw);
    const V3 ax{cs, 0, -sn}, ay{0, 1, 0}, az{sn, 0, cs};  // same turn as xcube_drop.mm's set_pose
    const V3 axes[3] = {ax, ay, az};
    auto add = [&](V3 n, double o) {
        n = normalize(n);
        s.planes.push_back({n, o + dot(n, center)});
    };
    for (int a = 0; a < 3; ++a)
        for (int sg = -1; sg <= 1; sg += 2) add(axes[a] * (double)sg, half);
    if (chamferWidth > 0) {
        const double leg = chamferWidth / std::sqrt(2.0);  // how far the cut reaches along each face
        for (int a = 0; a < 3; ++a)
            for (int b = a + 1; b < 3; ++b)
                for (int sa = -1; sa <= 1; sa += 2)
                    for (int sb = -1; sb <= 1; sb += 2) add(axes[a] * (double)sa + axes[b] * (double)sb, (2.0 * half - leg) / std::sqrt(2.0));
    }
    build_faces(s, half * 4.0);
    return s;
}

// ---- more solids. Any convex solid with flat faces works: give its faces as half spaces.
inline Solid from_planes(std::vector<Plane> planes, double reach) {
    Solid s;
    s.planes = std::move(planes);
    build_faces(s, reach);
    return s;
}
// Box with its own half sizes (slab, bar, pillar ...), centred, axis aligned.
inline Solid box(double hx, double hy, double hz) {
    std::vector<Plane> p = {{{1, 0, 0}, hx}, {{-1, 0, 0}, hx}, {{0, 1, 0}, hy}, {{0, -1, 0}, hy}, {{0, 0, 1}, hz}, {{0, 0, -1}, hz}};
    return from_planes(p, 4.0 * std::max(hx, std::max(hy, hz)));
}
// Regular prism with `sides` side faces; its axis is +Y, `radius` is the distance from the axis to a corner.
inline Solid prism(int sides, double radius, double halfHeight) {
    std::vector<Plane> p;
    const double pi = 3.14159265358979323846;
    const double inner = radius * std::cos(pi / sides);
    for (int i = 0; i < sides; ++i) {
        const double a = 2.0 * pi * (i + 0.5) / sides;
        p.push_back({{std::cos(a), 0, std::sin(a)}, inner});
    }
    p.push_back({{0, 1, 0}, halfHeight});
    p.push_back({{0, -1, 0}, halfHeight});
    return from_planes(p, 4.0 * std::max(radius, halfHeight));
}
// Pyramid on a regular base with `sides` corners at `radius`, apex `height` above the base; base centred at y = -height / 2.
inline Solid pyramid(int sides, double radius, double height) {
    std::vector<Plane> p;
    const double pi = 3.14159265358979323846;
    const V3 apex{0, 0.5 * height, 0};
    for (int i = 0; i < sides; ++i) {
        const double a0 = 2.0 * pi * i / sides, a1 = 2.0 * pi * (i + 1) / sides;
        const V3 b0{radius * std::cos(a0), -0.5 * height, radius * std::sin(a0)}, b1{radius * std::cos(a1), -0.5 * height, radius * std::sin(a1)};
        V3 n = normalize(cross(b1 - apex, b0 - apex));
        if (n.y < 0) n = n * -1.0;  // side faces look up and out
        p.push_back({n, dot(n, apex)});
    }
    p.push_back({{0, -1, 0}, 0.5 * height});
    return from_planes(p, 4.0 * std::max(radius, height));
}
// Octahedron with corners at distance `radius` on the axes.
inline Solid octahedron(double radius) {
    std::vector<Plane> p;
    for (int sx = -1; sx <= 1; sx += 2)
        for (int sy = -1; sy <= 1; sy += 2)
            for (int sz = -1; sz <= 1; sz += 2) p.push_back({normalize(V3{(double)sx, (double)sy, (double)sz}), radius / std::sqrt(3.0)});
    return from_planes(p, 4.0 * radius);
}

// Cuts every edge of the solid with a flat of the given width (the protective chamfer real cut glass has).
// Edges whose two faces meet at less than `minTurn` radians between their normals are left alone.
inline Solid chamfered(const Solid& in, double width, double minTurn = 0.6) {
    if (width <= 0) return in;
    std::vector<Plane> planes = in.planes;
    double reach = 0;
    for (const Poly& P : in.faces)
        for (int i = 0; i < P.n; ++i) reach = std::max(reach, length(P.p[(size_t)i]));
    const double eps = 1e-9 * (reach + 1.0);
    for (size_t i = 0; i < in.planes.size(); ++i)
        for (size_t j = i + 1; j < in.planes.size(); ++j) {
            // the two faces share an edge if two corners of face i lie on plane j
            V3 e[2];
            int n = 0;
            const Poly& P = in.faces[i];
            for (int q = 0; q < P.n && n < 2; ++q)
                if (std::fabs(dot(in.planes[j].n, P.p[(size_t)q]) - in.planes[j].o) < eps) e[n++] = P.p[(size_t)q];
            if (n < 2 || length(e[1] - e[0]) < 1e-6 * (reach + 1e-300)) continue;
            const V3 ni = in.planes[i].n, nj = in.planes[j].n;
            const double c = dot(ni, nj);
            if (c > std::cos(minTurn)) continue;
            const V3 m = normalize(ni + nj);
            const double ca = dot(m, ni);                                // cos of half the turn between the faces
            const double depth = width * ca / (2.0 * std::sqrt(std::max(1e-12, 1.0 - ca * ca)));  // width / (2 tan)
            planes.push_back({m, dot(m, e[0]) - depth});
        }
    return from_planes(planes, 4.0 * reach);
}

// Turns (columns of R are where the solid's x, y, z axes go), scales and moves a solid.
inline Solid placed(const Solid& in, const V3 R[3], double scale, V3 at) {
    Solid s;
    s.kind = in.kind;
    s.c = (R[0] * in.c.x + R[1] * in.c.y + R[2] * in.c.z) * scale + at;
    s.ax = R[0] * in.ax.x + R[1] * in.ax.y + R[2] * in.ax.z;
    s.r = in.r * scale;
    s.h = in.h * scale;
    for (const Plane& pl : in.planes) {
        V3 n = R[0] * pl.n.x + R[1] * pl.n.y + R[2] * pl.n.z;
        s.planes.push_back({n, pl.o * scale + dot(n, at)});
    }
    s.faces.resize(in.faces.size());
    for (size_t f = 0; f < in.faces.size(); ++f) {
        s.faces[f].n = in.faces[f].n;
        for (int i = 0; i < in.faces[f].n; ++i) {
            const V3 v = in.faces[f].p[(size_t)i];
            s.faces[f].p[(size_t)i] = (R[0] * v.x + R[1] * v.y + R[2] * v.z) * scale + at;
        }
    }
    return s;
}

struct Glass {
    double nd = 1.5168;   // BK7 crown glass at 587.56 nm
    double abbe = 64.17;  // BK7
    double exaggerate = 1.0;  // 1 = real glass; larger spreads the colours further (a deliberate lie)
    double n(double nm) const {
        const double ld = 587.56, lF = 486.13, lC = 656.27;
        const double B = (nd - 1.0) / ((abbe / exaggerate) * (1.0 / (lF * lF) - 1.0 / (lC * lC)));
        return nd + B * (1.0 / (nm * nm) - 1.0 / (ld * ld));
    }
};

// CIE 1931 colour matching functions, analytic fit (Wyman, Sloan, Shirley, JCGT 2(2), 2013).
inline V3 cie_xyz(double nm) {
    auto g = [](double x, double mu, double s1, double s2) {
        double t = (x - mu) * (x < mu ? s1 : s2);
        return std::exp(-0.5 * t * t);
    };
    double X = 1.056 * g(nm, 599.8, 0.0264, 0.0323) + 0.362 * g(nm, 442.0, 0.0624, 0.0374) - 0.065 * g(nm, 501.1, 0.0490, 0.0382);
    double Y = 0.821 * g(nm, 568.8, 0.0213, 0.0247) + 0.286 * g(nm, 530.9, 0.0613, 0.0322);
    double Z = 1.217 * g(nm, 437.0, 0.0845, 0.0278) + 0.681 * g(nm, 459.0, 0.0385, 0.0725);
    return {X, Y, Z};
}
inline V3 xyz_to_linear_srgb(V3 c) {
    return {3.2406 * c.x - 1.5372 * c.y - 0.4986 * c.z, -0.9689 * c.x + 1.8758 * c.y + 0.0415 * c.z, 0.0557 * c.x - 0.2040 * c.y + 1.0570 * c.z};
}

struct Band {
    double nm;
    V3 rgb;  // linear sRGB weight, never negative. All bands together are (1, 1, 1).
};
inline std::vector<Band> make_bands(int count, double nmMin = 400.0, double nmMax = 700.0) {
    std::vector<Band> b((size_t)count);
    V3 sum{0, 0, 0};
    for (int i = 0; i < count; ++i) {
        b[(size_t)i].nm = nmMin + (nmMax - nmMin) * (i + 0.5) / count;
        V3 c = xyz_to_linear_srgb(cie_xyz(b[(size_t)i].nm));
        // A pure spectral colour lies outside sRGB (one channel negative): move it toward white just enough.
        const double low = std::min(c.x, std::min(c.y, c.z));
        if (low < 0) c = c - V3{low, low, low};
        b[(size_t)i].rgb = c;
        sum = sum + c;
    }
    for (auto& e : b) e.rgb = {e.rgb.x / sum.x, e.rgb.y / sum.y, e.rgb.z / sum.z};
    return b;
}

// Unpolarised Fresnel reflectance going from index n1 to n2 at cos(incidence) ci. Returns 1 on total reflection.
inline double fresnel(double ci, double n1, double n2) {
    double s2 = (n1 / n2) * (n1 / n2) * (1.0 - ci * ci);
    if (s2 >= 1.0) return 1.0;
    double ct = std::sqrt(1.0 - s2);
    double rs = (n1 * ci - n2 * ct) / (n1 * ci + n2 * ct);
    double rp = (n1 * ct - n2 * ci) / (n1 * ct + n2 * ci);
    return 0.5 * (rs * rs + rp * rp);
}
// d: unit travel direction. n: unit normal pointing against d (d.n < 0). eta = n1 / n2.
inline bool refract(V3 d, V3 n, double eta, V3& out) {
    double ci = -dot(d, n);
    double k = 1.0 - eta * eta * (1.0 - ci * ci);
    if (k < 0) return false;
    out = normalize(d * eta + n * (eta * ci - std::sqrt(k)));
    return true;
}

// Keeps points p with a.p + b >= 0.
struct Half {
    V3 a;
    double b;
};
inline void clip(Poly& P, const Half& h) {
    double g[Poly::CAP];
    bool anyNeg = false;
    for (int q = 0; q < P.n; ++q) {
        g[q] = dot(h.a, P.p[(size_t)q]) + h.b;
        if (g[q] < 0) anyNeg = true;
    }
    if (anyNeg) clip(P, g);
}
// Q minus the convex region where all `nc` halves hold: the convex pieces of the difference are appended to
// `out`, and Q is left as the common part (possibly empty).
inline void split(Poly& Q, const Half* C, int nc, std::vector<Poly>& out, double tiny) {
    for (int k = 0; k < nc && Q.n >= 3; ++k) {
        Poly O = Q;
        clip(O, Half{C[k].a * -1.0, -C[k].b});
        if (O.n >= 3 && area(O) > tiny) out.push_back(O);
        clip(Q, C[k]);
    }
    if (Q.n < 3 || area(Q) <= tiny) Q.n = 0;
}

// Several solids. They must not overlap; they may touch or rest on each other (leave a hair of air between).
struct Scene {
    std::vector<Solid> solids;
    std::vector<V3> centre;      // bounding spheres, filled by bound()
    std::vector<double> radius;
    void bound() {
        centre.assign(solids.size(), V3{0, 0, 0});
        radius.assign(solids.size(), 0.0);
        for (size_t s = 0; s < solids.size(); ++s) {
            if (solids[s].kind == Solid::CONE) {
                // the ball through the rim and the tip (or just round the base, if the cone is flatter than it is wide)
                const double r = solids[s].r, h = solids[s].h, up = std::max(0.0, (h * h - r * r) / (2.0 * h));
                centre[s] = solids[s].c + solids[s].ax * up;
                radius[s] = std::sqrt(r * r + up * up);
                continue;
            }
            if (solids[s].kind != Solid::FLAT) {
                centre[s] = solids[s].c;
                radius[s] = solids[s].kind == Solid::BALL ? solids[s].r : std::sqrt(solids[s].r * solids[s].r + solids[s].h * solids[s].h);
                continue;
            }
            V3 lo{1e300, 1e300, 1e300}, hi{-1e300, -1e300, -1e300};
            for (const Poly& P : solids[s].faces)
                for (int i = 0; i < P.n; ++i) {
                    const V3 v = P.p[(size_t)i];
                    lo = {std::min(lo.x, v.x), std::min(lo.y, v.y), std::min(lo.z, v.z)};
                    hi = {std::max(hi.x, v.x), std::max(hi.y, v.y), std::max(hi.z, v.z)};
                }
            centre[s] = (lo + hi) * 0.5;
            for (const Poly& P : solids[s].faces)
                for (int i = 0; i < P.n; ++i) radius[s] = std::max(radius[s], length(P.p[(size_t)i] - centre[s]));
        }
    }
};
inline Scene one(const Solid& s) {
    Scene sc;
    sc.solids.push_back(s);
    sc.bound();
    return sc;
}

struct Params {
    V3 sun{0.6, -0.7, 0.3};  // direction the light travels (need not be unit)
    Glass glass;
    int bands = 64;           // colour resolution of the spectrum
    double nmMin = 400.0, nmMax = 700.0;
    double floorY = 0.0;
    int maxBounces = 4;       // internal reflections followed inside one solid
    int maxSolids = 4;        // how many times one beam may enter glass (the same solid again counts)
    double cutoff = 0.02;     // a branch carrying less than this share of the sun is dropped
    bool outerReflection = true;  // the few percent mirrored off the faces also travel on
    double minFlux = 0;       // a beam carrying less light than this (sun irradiance x area) is dropped;
                              // about 30 x (one pixel's area on the floor) removes half the patches and 0.3% of the light
    int roundSteps = 8;       // most colour steps drawn for the light of a round solid
    int coneSteps = 6;        // colour steps of a cone's or a rod's light (they are softened into one another, see sunTan; 8 and 16
                              // looked the same, 4 began to show the softening as faint blocks)
    double sunTan = 0.0087269; // tangent of half the sun's width as the renderer softens with it (0.5 degrees): a cone's light is told
                               // to soften by half the gap to the next colour step as well, so that the steps run into one another
    unsigned threads = 1;     // worker threads; the result does not depend on this
};

// One step of the way light took: which face of which solid. MIRRORED = it bounced off that face from outside.
// A face followed by another face of the same solid was an inner reflection; otherwise the light left through it.
inline uint16_t step(int solid, int face, bool mirrored = false) { return (uint16_t)((solid << 6) | face | (mirrored ? 0x8000 : 0)); }
inline int step_solid(uint16_t c) { return (c & 0x7FFF) >> 6; }
inline int step_face(uint16_t c) { return c & 63; }
inline bool step_mirrored(uint16_t c) { return (c & 0x8000) != 0; }

// One landed patch: a convex polygon on the floor (x, z), flat colour.
struct Patch {
    static constexpr int SEQ = 24;
    int first = 0, count = 0;  // corners in Result::xz
    V3 rgb{0, 0, 0};           // linear sRGB irradiance, in units of the sun's
    int band = 0;
    uint16_t seq[SEQ] = {};    // the way the light took (see step())
    uint8_t len = 0;
    uint64_t path = 0;         // hash of the above
};
struct Result {
    std::vector<double> xz;      // x0, z0, x1, z1, ...
    std::vector<double> travel;  // per corner: distance flown from the glass to the floor
    std::vector<Patch> patches;
    // Bookkeeping per unit of sun irradiance, summed over bands with each band counted as 1 / bands:
    double fluxIn = 0, fluxFloor = 0, fluxSky = 0, fluxDropped = 0;
    int beams = 0;
};

namespace detail {
// Helpers that are started once and then wait for work. (Starting threads anew for every frame cost nothing on a
// quiet machine and stalled for a tenth of a second now and then on a busy one.)
class Workers {
  public:
    static Workers& get() {
        static Workers w;
        return w;
    }
    // call(ctx, i) for every i below n, on the calling thread and up to threads - 1 helpers
    void run(size_t n, unsigned threads, void (*call)(void*, size_t), void* ctx) {
        static thread_local bool inside = false;
        std::unique_lock<std::mutex> one(turn_, std::defer_lock);
        if (inside || threads <= 1 || n <= 1 || pool_.empty() || !one.try_lock()) {
            // already inside a run (or another thread has the helpers): do it here
            for (size_t i = 0; i < n; ++i) call(ctx, i);
            return;
        }
        inside = true;
        {
            std::lock_guard<std::mutex> lock(m_);
            call_ = call;
            ctx_ = ctx;
            n_ = n;
            next_.store(0);
            wanted_ = (unsigned)std::min<size_t>({(size_t)threads - 1, pool_.size(), n - 1});
            ++round_;
        }
        wake_.notify_all();
        for (;;) {
            const size_t i = next_.fetch_add(1);
            if (i >= n) break;
            call(ctx, i);
        }
        {
            std::unique_lock<std::mutex> lock(m_);
            wanted_ = 0;  // helpers that have not woken yet are not needed any more
            done_.wait(lock, [&] { return busy_ == 0; });
        }
        inside = false;
    }

  private:
    Workers() {
        const unsigned count = std::max(1u, std::thread::hardware_concurrency()) - 1;
        for (unsigned w = 0; w < count; ++w) pool_.emplace_back([this] { loop(); });
    }
    ~Workers() {
        {
            std::lock_guard<std::mutex> lock(m_);
            quit_ = true;
        }
        wake_.notify_all();
        for (std::thread& t : pool_) t.join();
    }
    void loop() {
        std::unique_lock<std::mutex> lock(m_);
        uint64_t seen = 0;
        for (;;) {
            wake_.wait(lock, [&] { return quit_ || (round_ != seen && wanted_ > 0); });
            if (quit_) return;
            seen = round_;
            --wanted_;
            ++busy_;
            void (*call)(void*, size_t) = call_;
            void* ctx = ctx_;
            const size_t n = n_;
            lock.unlock();
            for (;;) {
                const size_t i = next_.fetch_add(1);
                if (i >= n) break;
                call(ctx, i);
            }
            lock.lock();
            if (--busy_ == 0) done_.notify_all();
        }
    }
    std::mutex m_, turn_;
    std::condition_variable wake_, done_;
    std::vector<std::thread> pool_;
    void (*call_)(void*, size_t) = nullptr;
    void* ctx_ = nullptr;
    size_t n_ = 0;
    std::atomic<size_t> next_{0};
    unsigned wanted_ = 0, busy_ = 0;
    uint64_t round_ = 0;
    bool quit_ = false;
};
}  // namespace detail
template <class F>
inline void parallel_for(size_t n, unsigned threads, F&& f) {
    if (threads <= 1 || n <= 1) {
        for (size_t i = 0; i < n; ++i) f(i);
        return;
    }
    detail::Workers::get().run(n, threads, [](void* ctx, size_t i) { (*static_cast<std::remove_reference_t<F>*>(ctx))(i); }, (void*)&f);
}

namespace detail {

struct Way {
    uint16_t seq[Patch::SEQ];
    uint8_t len;
    bool full;  // more steps than fit: such a branch is dropped
    void add(uint16_t c) {
        if (len < Patch::SEQ) seq[len++] = c;
        else full = true;
    }
    uint64_t hash() const {
        uint64_t h = 1469598103934665603ull;
        for (int i = 0; i < len; ++i) h = (h ^ (uint64_t)(seq[i] + 1)) * 1099511628211ull;
        return h;
    }
};

struct Ctx {
    const Scene* scene;
    const Params* par;
    Result* res;
    double n;       // index of this band
    V3 rgb;
    int band;
    double share;   // 1 / bands
    // When set, only this one way is followed and nothing is counted; its landed polygons go to `out`.
    const Patch* only = nullptr;
    std::vector<Poly>* out = nullptr;
    std::vector<double>* outE = nullptr;
};

// Where the beam lands. Returns false if it goes up.
// Each landed corner is (x, travel, z): its place on the floor and, in y, how far the light flew from the glass
// to get there. A renderer uses the travel to soften the light as a sun with some width would.
inline bool to_floor(const Poly& P, V3 dir, double floorY, Poly& F) {
    if (dir.y > -1e-9) return false;
    F.n = 0;
    for (int i = 0; i < P.n; ++i) {
        V3 v = P.p[(size_t)i];
        double t = (floorY - v.y) / dir.y;
        if (t < 0) t = 0;  // a face resting on the floor
        V3 q = v + dir * t;
        F.push({q.x, t, q.z});
    }
    return true;
}

// The beam (polygon P on some plane, leaving along dir, carrying `irr` per unit area of that plane) reaches the floor.
inline void land(Ctx& c, const Poly& P, V3 dir, double irr, double cosAtPlane, const Way& way) {
    const double flux = irr * area(P);
    Poly F;
    if (!to_floor(P, dir, c.par->floorY, F)) {
        if (!c.only) c.res->fluxSky += flux * c.share;
        return;
    }
    const double e = irr * (-dir.y) / cosAtPlane;  // flux / landed area
    if (c.only) {
        c.out->push_back(F);
        c.outE->push_back(e);
        return;
    }
    Patch p;
    p.first = (int)(c.res->xz.size() / 2);
    p.count = F.n;
    p.rgb = c.rgb * e;
    p.band = c.band;
    std::memcpy(p.seq, way.seq, sizeof(p.seq));
    p.len = way.len;
    p.path = way.hash();
    for (int i = 0; i < F.n; ++i) {
        c.res->xz.push_back(F.p[(size_t)i].x);
        c.res->xz.push_back(F.p[(size_t)i].z);
        c.res->travel.push_back(F.p[(size_t)i].y);
    }
    c.res->patches.push_back(p);
    c.res->fluxFloor += flux * c.share;
}
inline void drop(Ctx& c, double flux) {
    if (!c.only) c.res->fluxDropped += flux * c.share;
}

// Face G of some solid as seen from plane (pn, po) along d: the halves that bound its shadow on that plane.
// Returns the number of halves (0 if the face is degenerate).
inline int shadow_halves(const Poly& G, V3 pn, double po, V3 d, Half* out) {
    const double den = dot(pn, d);
    Poly S;
    for (int i = 0; i < G.n; ++i) {
        const V3 v = G.p[(size_t)i];
        S.push(v - d * ((dot(pn, v) - po) / den));
    }
    V3 nrm{0, 0, 0};
    for (int i = 1; i + 1 < S.n; ++i) nrm = nrm + cross(S.p[(size_t)i] - S.p[0], S.p[(size_t)i + 1] - S.p[0]);
    const double sg = dot(nrm, pn) >= 0 ? 1.0 : -1.0;
    for (int i = 0; i < S.n; ++i) {
        const V3 a = S.p[(size_t)i], b = S.p[(size_t)((i + 1) % S.n)];
        const V3 h = cross(pn, b - a) * sg;
        out[i] = Half{h, -dot(h, a)};
    }
    return S.n;
}

inline void inside(Ctx& c, int S, const Poly& P, int from, V3 d, double irr, double weight, int bounces, int entries, const Way& way);

// A face of another solid that a beam in the air may meet.
struct Cand {
    int S, g;
    Half region[Poly::CAP + 1];  // its shadow on the beam's plane, and "lies ahead"
    int nh;
    Half t;                      // distance along the beam to the face's plane, as a.p + b
};
// Working lists, one set per thread and per depth of the recursion, so that nothing is allocated per beam.
struct Scratch {
    std::vector<Cand> cand;
    std::vector<Poly> a, b, c;
};
inline Scratch& scratch(int depth) {
    static thread_local std::vector<Scratch> pool(Patch::SEQ + 4);
    return pool[(size_t)std::min(depth, Patch::SEQ + 3)];
}

// Light arrives from outside at face g of solid S (polygon R on that face, irr per unit face area): a few percent
// are mirrored and travel on through the air, the rest goes in.
inline void air(Ctx& c, const Poly& P, V3 pn, V3 d, double irr, int from, double weight, int entries, const Way& way);
inline void enter(Ctx& c, int S, int g, const Poly& R, V3 d, double irr, double weight, int entries, const Way& way) {
    const V3 ng = c.scene->solids[(size_t)S].planes[(size_t)g].n;
    const double ci = -dot(ng, d);
    const double Rf = fresnel(ci, 1.0, c.n);
    const uint16_t cIn = step(S, g), cMirror = step(S, g, true);
    const bool wantMirror = !c.only || (way.len < c.only->len && c.only->seq[way.len] == cMirror);
    const bool wantIn = !c.only || (way.len < c.only->len && c.only->seq[way.len] == cIn);
    if (wantMirror) {
        if (c.par->outerReflection && weight * Rf > c.par->cutoff * 0.25) {
            Way w = way;
            w.add(cMirror);
            if (!w.full) air(c, R, ng, d + ng * (2.0 * ci), irr * Rf, S, weight * Rf, entries, w);
            else drop(c, irr * Rf * area(R));
        } else {
            drop(c, irr * Rf * area(R));
        }
    }
    if (wantIn) {
        Way w = way;
        w.add(cIn);
        if (entries < c.par->maxSolids && !w.full) {
            V3 in;
            refract(d, ng, 1.0 / c.n, in);
            inside(c, S, R, g, in, irr * (1.0 - Rf), weight * (1.0 - Rf), 0, entries + 1, w);
        } else {
            drop(c, irr * (1.0 - Rf) * area(R));
        }
    }
}

// A beam in the air: polygon P on a plane with normal pn, travelling along d, carrying `irr` per unit area of
// that plane. `from` is the solid it just left (it cannot meet that one again right away), -1 for none.
// Whatever part meets another solid first goes into (or mirrors off) that solid; the rest lands on the floor.
inline void air(Ctx& c, const Poly& P, V3 pn, V3 d, double irr, int from, double weight, int entries, const Way& way) {
    const Scene& sc = *c.scene;
    if (!c.only) ++c.res->beams;
    const double areaP = area(P);
    if (irr * areaP < c.par->minFlux) {
        drop(c, irr * areaP);
        return;
    }
    const double po = dot(pn, P.p[0]);
    const double cosP = std::fabs(dot(pn, d));
    const double tiny = 1e-13 * areaP + 1e-300;
    Scratch& sx = scratch(way.len);
    // where the way being followed goes next
    const bool atEnd = c.only && way.len >= c.only->len;
    const uint16_t next = (c.only && !atEnd) ? c.only->seq[way.len] : 0;

    std::vector<Cand>& cand = sx.cand;
    cand.clear();
    if (sc.solids.size() > 1) {
        V3 mid{0, 0, 0};
        for (int i = 0; i < P.n; ++i) mid = mid + P.p[(size_t)i];
        mid = mid * (1.0 / P.n);
        double rb = 0;
        for (int i = 0; i < P.n; ++i) rb = std::max(rb, length(P.p[(size_t)i] - mid));
        for (int S = 0; S < (int)sc.solids.size(); ++S) {
            if (S == from) continue;
            const V3 rel = sc.centre[(size_t)S] - mid;
            const double along = dot(rel, d), reach = sc.radius[(size_t)S] + rb;
            if (along < -reach || dot(rel, rel) - along * along > reach * reach) continue;
            const Solid& so = sc.solids[(size_t)S];
            for (int g = 0; g < (int)so.planes.size(); ++g) {
                const Poly& G = so.faces[(size_t)g];
                const double den = dot(so.planes[(size_t)g].n, d);
                if (G.n < 3 || den > -1e-12) continue;  // only faces turned against the beam let it in
                cand.emplace_back();
                Cand& k = cand.back();
                k.S = S;
                k.g = g;
                k.nh = shadow_halves(G, pn, po, d, k.region);
                k.t = Half{so.planes[(size_t)g].n * (-1.0 / den), so.planes[(size_t)g].o / den};
                k.region[k.nh++] = Half{k.t.a, k.t.b + 1e-9};  // the face must lie ahead
                Poly Q = P;
                for (int h = 0; h < k.nh && Q.n >= 3; ++h) clip(Q, k.region[h]);
                if (!(Q.n >= 3 && area(Q) > tiny)) cand.pop_back();
            }
        }
    }
    // the part that meets nothing: on to the floor
    if (!c.only || atEnd) {
        std::vector<Poly>&rest = sx.a, &nextRest = sx.b;
        rest.clear();
        rest.push_back(P);
        for (const Cand& k : cand) {
            nextRest.clear();
            for (Poly& Q : rest) split(Q, k.region, k.nh, nextRest, tiny);
            rest.swap(nextRest);
        }
        for (const Poly& Q : rest) land(c, Q, d, irr, cosP, way);
    }
    if (atEnd) return;
    // the part that meets each face first
    for (size_t i = 0; i < cand.size(); ++i) {
        const Cand& k = cand[i];
        if (c.only && (step_solid(next) != k.S || step_face(next) != k.g)) continue;
        Poly first = P;
        for (int h = 0; h < k.nh && first.n >= 3; ++h) clip(first, k.region[h]);
        if (first.n < 3) continue;
        std::vector<Poly>&mine = sx.a, &tmp = sx.b;
        mine.clear();
        mine.push_back(first);
        for (size_t j = 0; j < cand.size() && !mine.empty(); ++j) {
            const Cand& o = cand[j];
            if (o.S == k.S) continue;  // faces of one solid never hide each other from the beam
            // where the other face is met earlier: inside its shadow and t_other < t_mine
            Half C[Poly::CAP + 2];
            for (int h = 0; h < o.nh; ++h) C[h] = o.region[h];
            C[o.nh] = Half{k.t.a - o.t.a, k.t.b - o.t.b};
            tmp.clear();
            for (Poly& Q : mine) split(Q, C, o.nh + 1, tmp, tiny);
            mine.swap(tmp);
        }
        const Plane& pg = sc.solids[(size_t)k.S].planes[(size_t)k.g];
        const double den = dot(pg.n, d);
        // (the pieces are moved to their own list: `mine` is reused for the next face)
        std::vector<Poly>& go = sx.c;
        go.clear();
        for (const Poly& Q : mine) {
            go.emplace_back();
            Poly& R = go.back();
            for (int q = 0; q < Q.n; ++q) {
                const V3 v = Q.p[(size_t)q];
                R.push(v + d * ((pg.o - dot(pg.n, v)) / den));
            }
        }
        const Cand kk = k;  // `cand` belongs to this depth only, but keep what is needed across the calls anyway
        for (size_t q = 0; q < go.size(); ++q) {
            const Poly R = go[q];
            enter(c, kk.S, kk.g, R, d, irr * (-den) / cosP, weight, entries, way);
        }
    }
}

// A beam inside solid S: polygon P on face `from`, travelling along d, carrying `irr` per unit area of that face.
// `weight` is the share of the sun this branch still carries (Fresnel factors only), used for the cutoff.
inline void inside(Ctx& c, int S, const Poly& P, int from, V3 d, double irr, double weight, int bounces, int entries, const Way& way) {
    const Solid& s = c.scene->solids[(size_t)S];
    if (!c.only) ++c.res->beams;
    const double cosFrom = std::fabs(dot(s.planes[(size_t)from].n, d));
    int cand[64];
    double den[64];
    int nc = 0;
    for (int j = 0; j < (int)s.planes.size() && nc < 64; ++j) {
        if (j == from || s.faces[(size_t)j].n == 0) continue;
        const double dj = dot(s.planes[(size_t)j].n, d);
        if (dj > 1e-12) {
            cand[nc] = j;
            den[nc] = dj;
            ++nc;
        }
    }
    const bool haveNext = c.only && way.len < c.only->len;
    if (c.only && !haveNext) return;  // the way ends inside the glass: not a way that lands
    int live[64], nl = 0;
    if (c.only) {
        // one known face is wanted: cut against every other candidate, no sorting out needed
        for (int a = 0; a < nc; ++a) live[nl++] = a;
    } else {
        // Distance to every candidate at every corner. A plane that some other plane beats at every corner is never
        // reached first anywhere on the polygon, and the cuts against it are implied by the cuts against its winner.
        double T[64][Poly::CAP];
        for (int a = 0; a < nc; ++a) {
            const Plane& pa = s.planes[(size_t)cand[a]];
            for (int q = 0; q < P.n; ++q) T[a][q] = (pa.o - dot(pa.n, P.p[(size_t)q])) / den[a];
        }
        for (int a = 0; a < nc; ++a) {
            bool beaten = false;
            for (int b = 0; b < nc && !beaten; ++b) {
                if (b == a) continue;
                bool all = true, strict = false;
                for (int q = 0; q < P.n; ++q) {
                    if (T[b][q] > T[a][q]) { all = false; break; }
                    if (T[b][q] < T[a][q]) strict = true;
                }
                if (all && (strict || b < a)) beaten = true;
            }
            if (!beaten) live[nl++] = a;
        }
    }
    const double tiny = 1e-13 * area(P) + 1e-300;
    for (int ia = 0; ia < nl; ++ia) {
        const int a = live[ia];
        const int j = cand[a];
        const uint16_t cj = step(S, j);
        if (c.only && c.only->seq[way.len] != cj) continue;
        Poly Q = P;
        const Plane& pj = s.planes[(size_t)j];
        for (int ib = 0; ib < nl && Q.n >= 3; ++ib) {
            if (ib == ia) continue;
            const Plane& pb = s.planes[(size_t)cand[live[ib]]];
            clip(Q, Half{pb.n * (-1.0 / den[live[ib]]) - pj.n * (-1.0 / den[a]), pb.o / den[live[ib]] - pj.o / den[a]});
        }
        if (Q.n < 3 || area(Q) <= tiny) continue;
        Poly R;
        for (int q = 0; q < Q.n; ++q) {
            const V3 v = Q.p[(size_t)q];
            R.push(v + d * ((pj.o - dot(pj.n, v)) / den[a]));
        }
        const double ci = den[a];
        const double irrJ = irr * ci / cosFrom;  // same flux, spread over the footprint on face j
        if (irrJ * area(R) < c.par->minFlux) {
            drop(c, irrJ * area(R));
            continue;
        }
        Way wj = way;
        wj.add(cj);
        const double Rf = fresnel(ci, c.n, 1.0);
        // what the way being followed does after this face
        bool wantOut = true, wantOn = true;
        if (c.only) {
            const bool more = wj.len < c.only->len;
            const bool sameSolid = more && step_solid(c.only->seq[wj.len]) == S && !step_mirrored(c.only->seq[wj.len]);
            wantOn = sameSolid;
            wantOut = !sameSolid;
        }
        if (Rf < 1.0 && wantOut) {
            V3 out;
            refract(d, pj.n * -1.0, c.n, out);
            if (!wj.full) air(c, R, pj.n, out, irrJ * (1.0 - Rf), S, weight * (1.0 - Rf), entries, wj);
            else drop(c, irrJ * (1.0 - Rf) * area(R));
        }
        if (wantOn) {
            const double w = weight * Rf;
            if (bounces < c.par->maxBounces && w > c.par->cutoff && !wj.full) inside(c, S, R, j, d - pj.n * (2.0 * ci), irrJ * Rf, w, bounces + 1, entries, wj);
            else drop(c, irrJ * Rf * area(R));
        }
    }
}

// Sunlight on one face of one solid, minus what other solids standing in the sun's way take first.
inline void sun_on_face(Ctx& c, int A, int f) {
    const Scene& sc = *c.scene;
    const V3 L = normalize(c.par->sun);
    const Solid& sa = sc.solids[(size_t)A];
    const Poly& F = sa.faces[(size_t)f];
    if (F.n < 3) return;
    const V3 nf = sa.planes[(size_t)f].n;
    const double ci = -dot(nf, L);
    if (ci <= 1e-9) return;  // faces away from the sun
    const double tiny = 1e-13 * area(F) + 1e-300;
    std::vector<Poly> lit{F}, tmp;
    if (sc.solids.size() > 1) {
        const double po = sa.planes[(size_t)f].o;
        V3 mid{0, 0, 0};
        for (int i = 0; i < F.n; ++i) mid = mid + F.p[(size_t)i];
        mid = mid * (1.0 / F.n);
        double rb = 0;
        for (int i = 0; i < F.n; ++i) rb = std::max(rb, length(F.p[(size_t)i] - mid));
        for (int S = 0; S < (int)sc.solids.size() && !lit.empty(); ++S) {
            if (S == A) continue;
            const V3 rel = sc.centre[(size_t)S] - mid;
            const double along = dot(rel, L), reach = sc.radius[(size_t)S] + rb;
            if (along > reach || dot(rel, rel) - along * along > reach * reach) continue;  // not upstream of this face
            const Solid& so = sc.solids[(size_t)S];
            for (int g = 0; g < (int)so.planes.size() && !lit.empty(); ++g) {
                const Poly& G = so.faces[(size_t)g];
                const double den = dot(so.planes[(size_t)g].n, L);
                if (G.n < 3 || den > -1e-12) continue;
                Half C[Poly::CAP + 1];
                int nh = shadow_halves(G, nf, po, L, C);
                // the sun met that face before it could reach this one: t < 0 seen from here
                C[nh++] = Half{so.planes[(size_t)g].n * (1.0 / den), -so.planes[(size_t)g].o / den};
                tmp.clear();
                for (Poly& Q : lit) {
                    split(Q, C, nh, tmp, tiny);
                }
                lit.swap(tmp);
            }
        }
    }
    detail::Way way{};
    for (const Poly& Q : lit) {
        if (!c.only) c.res->fluxIn += ci * area(Q) * c.share;
        enter(c, A, f, Q, L, ci, 1.0, 0, way);
    }
}

// Everything one wavelength (index n) does, for the faces [first, last) of the flat list of all faces.
inline void trace_one(const Scene& sc, const Params& par, double n, V3 rgb, int band, double share, Result& res, int onlySolid = -1, int onlyFace = -1) {
    Ctx c{&sc, &par, &res, n, rgb, band, share};
    for (int A = 0; A < (int)sc.solids.size(); ++A) {
        if (onlySolid >= 0 && A != onlySolid) continue;
        for (int f = 0; f < (int)sc.solids[(size_t)A].planes.size(); ++f) {
            if (onlyFace >= 0 && f != onlyFace) continue;
            sun_on_face(c, A, f);
        }
    }
}

// One known way through the glass, at index n: the exact landed polygons (there can be several pieces when
// other solids cut the beam) and their brightness.
inline void follow(const Scene& sc, const Params& par, double n, const Patch& way, std::vector<Poly>& F, std::vector<double>& e) {
    F.clear();
    e.clear();
    if (way.len == 0) return;
    Result none;
    Ctx c{&sc, &par, &none, n, V3{1, 1, 1}, 0, 1.0};
    c.only = &way;
    c.out = &F;
    c.outE = &e;
    sun_on_face(c, step_solid(way.seq[0]), step_face(way.seq[0]));
}

}  // namespace detail

// Reference: every band traced in full. Exact for each band, slow (bands x everything).
inline Result trace(const Scene& sc, const Params& par) {
    const std::vector<Band> bands = make_bands(par.bands, par.nmMin, par.nmMax);
    std::vector<Result> part((size_t)par.bands);
    parallel_for((size_t)par.bands, par.threads, [&](size_t b) {
        detail::trace_one(sc, par, par.glass.n(bands[b].nm), bands[b].rgb, (int)b, 1.0 / par.bands, part[b]);
    });
    Result res;
    for (Result& r : part) {
        const int base = (int)(res.xz.size() / 2);
        res.xz.insert(res.xz.end(), r.xz.begin(), r.xz.end());
        res.travel.insert(res.travel.end(), r.travel.begin(), r.travel.end());
        for (Patch p : r.patches) {
            p.first += base;
            res.patches.push_back(p);
        }
        res.fluxIn += r.fluxIn;
        res.fluxFloor += r.fluxFloor;
        res.fluxSky += r.fluxSky;
        res.fluxDropped += r.fluxDropped;
        res.beams += r.beams;
    }
    return res;
}
inline Result trace(const Solid& solid, const Params& par) { return trace(one(solid), par); }

constexpr int TRI = 6;  // floats per triangle corner: x, z, r, g, b, travel

struct FastStats {
    int ways = 0, coloured = 0, smooth = 0, patches = 0, follows = 0;
};

// The same light, for a renderer: triangles x, z, r, g, b, travel (TRI floats per corner).
// Three full traces (both ends and the middle of the spectrum) find every way the light takes and where its
// patch lands at those three indices. A patch that moves less than its `detail` (world units, about one pixel
// on the floor; one value per solid, taken from the solid the light left last) between red and blue is drawn
// once in white. One that moves further is drawn as that many steps of the spectrum. Between two indices where
// it is known exactly, a step is read off the straight line between them, but only after that line has been
// checked against an exact follow in the middle (detail::follow); where the check fails, or the patch appears,
// vanishes or changes shape, the interval is halved and followed exactly again, down to single bands.
inline void trace_fast(const Scene& sc, const Params& par, const std::vector<double>& detailPerSolid, std::vector<float>& tri, FastStats* stats = nullptr) {
    tri.clear();
    const int N = par.bands;
    const std::vector<Band> bands = make_bands(N, par.nmMin, par.nmMax);
    std::vector<double> nb((size_t)N);
    for (int i = 0; i < N; ++i) nb[(size_t)i] = par.glass.n(bands[(size_t)i].nm);
    const double nBlue = nb[0], nRed = nb[(size_t)N - 1];
    constexpr int G = 64;  // the index range is cut into G parts; knot 0 is red, knot G is blue
    auto knot_n = [&](int k) { return nRed + (nBlue - nRed) * (double)k / G; };
    const int traced[3] = {0, G / 2, G};

    // the three full traces, one task per (index, solid)
    const size_t ns = sc.solids.size();
    std::vector<Result> part(3 * ns);
    parallel_for(3 * ns, par.threads, [&](size_t k) {
        detail::trace_one(sc, par, knot_n(traced[k / ns]), V3{1, 1, 1}, 0, 1.0, part[k], (int)(k % ns));
    });
    // every way seen at any of the three, once (middle first), with its patches at each of the three
    struct WayRef {
        const Patch* any;
        std::vector<std::pair<const Result*, const Patch*>> at[3];
    };
    std::vector<WayRef> ways;
    {
        std::unordered_map<uint64_t, size_t> index;
        const int order[3] = {1, 0, 2};
        for (int oi = 0; oi < 3; ++oi)
            for (size_t s = 0; s < ns; ++s) {
                const Result& res = part[(size_t)order[oi] * ns + s];
                for (const Patch& p : res.patches) {
                    auto it = index.find(p.path);
                    if (it == index.end()) {
                        it = index.emplace(p.path, ways.size()).first;
                        ways.push_back(WayRef{&p, {}});
                    }
                    ways[it->second].at[order[oi]].push_back({&res, &p});
                }
            }
    }

    struct Sample {
        bool known = false;          // followed (or traced) at this knot
        std::vector<Poly> F;         // empty = no light takes this way here
        std::vector<double> e;
    };
    auto alike = [](const Sample& a, const Sample& b) {
        if (a.F.empty() || a.F.size() != b.F.size()) return false;
        for (size_t p = 0; p < a.F.size(); ++p)
            if (a.F[p].n != b.F[p].n) return false;
        return true;
    };
    std::vector<std::vector<float>> outTri(ways.size());
    std::vector<FastStats> outSt(ways.size());
    parallel_for(ways.size(), par.threads, [&](size_t wi) {
        const WayRef& wr = ways[wi];
        const Patch* w = wr.any;
        std::vector<float>& tri = outTri[wi];
        FastStats& st = outSt[wi];
        auto emit = [&](const Poly& F, V3 rgb) {
            for (int i = 1; i + 1 < F.n; ++i) {
                const int idx[3] = {0, i, i + 1};
                for (int k = 0; k < 3; ++k) {
                    tri.push_back((float)F.p[(size_t)idx[k]].x);
                    tri.push_back((float)F.p[(size_t)idx[k]].z);
                    tri.push_back((float)rgb.x);
                    tri.push_back((float)rgb.y);
                    tri.push_back((float)rgb.z);
                    tri.push_back((float)F.p[(size_t)idx[k]].y);
                }
            }
        };
        const int lastSolid = step_solid(w->seq[w->len - 1]);
        const double detail_ = detailPerSolid.empty() ? 1e-3 : detailPerSolid[std::min((size_t)lastSolid, detailPerSolid.size() - 1)];
        static thread_local std::vector<Sample> samp(G + 1);
        for (Sample& sm : samp) {
            sm.known = false;
            sm.F.clear();
            sm.e.clear();
        }
        for (int k = 0; k < 3; ++k) {
            Sample& sm = samp[(size_t)traced[k]];
            sm.known = true;
            for (const auto& rp : wr.at[k]) {
                Poly F;
                for (int i = 0; i < rp.second->count; ++i)
                    F.push({rp.first->xz[(size_t)(rp.second->first + i) * 2], rp.first->travel[(size_t)(rp.second->first + i)], rp.first->xz[(size_t)(rp.second->first + i) * 2 + 1]});
                sm.F.push_back(F);
                sm.e.push_back(rp.second->rgb.x);
            }
        }
        const Sample &red = samp[0], &mid = samp[(size_t)G / 2], &blue = samp[(size_t)G];
        double move = 0;
        if (alike(red, blue)) {
            for (size_t p = 0; p < red.F.size(); ++p)
                for (int i = 0; i < red.F[p].n; ++i) {
                    const V3 dv = red.F[p].p[(size_t)i] - blue.F[p].p[(size_t)i];
                    move = std::max(move, std::sqrt(dv.x * dv.x + dv.z * dv.z));
                }
        } else {
            // not the same shape across the spectrum: judge by how far apart whatever exists lies
            double x0 = 1e300, x1 = -1e300, z0 = 1e300, z1 = -1e300;
            for (int k = 0; k < 3; ++k)
                for (const Poly& P : samp[(size_t)traced[k]].F)
                    for (int i = 0; i < P.n; ++i) {
                        x0 = std::min(x0, P.p[(size_t)i].x); x1 = std::max(x1, P.p[(size_t)i].x);
                        z0 = std::min(z0, P.p[(size_t)i].z); z1 = std::max(z1, P.p[(size_t)i].z);
                    }
            move = x1 > x0 ? std::max(x1 - x0, z1 - z0) : 0;
        }
        const int m = move <= detail_ ? 1 : (int)std::min<double>(N, std::ceil(move / detail_));
        if (m == 1) {
            // (a patch that exists in only part of the spectrum and does not move: draw it where it is known, in
            // the colour of that part)
            const bool r_ = !red.F.empty(), m_ = !mid.F.empty(), b_ = !blue.F.empty();
            const Sample& use = m_ ? mid : (r_ ? red : blue);
            V3 colour{0, 0, 0};
            for (int i = 0; i < N; ++i) {
                const bool lowHalf = i >= N / 2;  // band 0 is blue, band N - 1 is red
                const bool on = lowHalf ? (m_ || r_) && (r_ || (m_ && !b_ ? true : m_)) : (m_ || b_);
                if (on) colour = colour + bands[(size_t)i].rgb;
            }
            for (size_t p = 0; p < use.F.size(); ++p) {
                emit(use.F[p], colour * use.e[p]);
                ++st.patches;
            }
            return;
        }
        ++st.coloured;
        auto know = [&](int k) {
            Sample& sm = samp[(size_t)k];
            if (sm.known) return;
            sm.known = true;
            ++st.follows;
            detail::follow(sc, par, knot_n(k), *w, sm.F, sm.e);
        };
        // segments of the index range, left to right: (a, b, may the patch be read off the line between them)
        struct Seg {
            int a, b;
            bool line;
        };
        Seg segs[G];
        int nseg = 0;
        const double tol = 0.35 * detail_;
        // the smallest interval worth cutting: finer than the steps that will be drawn makes no difference
        int finest = 1;
        while (finest * 2 * m <= G) finest *= 2;
        auto resolve = [&](auto&& self, int a, int b) -> void {
            const Sample &A = samp[(size_t)a], &B = samp[(size_t)b];
            if (b - a <= finest) {
                segs[nseg++] = Seg{a, b, alike(A, B)};
                return;
            }
            const int h = (a + b) / 2;
            know(h);
            const Sample& H = samp[(size_t)h];
            if (A.F.empty() && B.F.empty() && H.F.empty()) {
                segs[nseg++] = Seg{a, b, false};  // nothing there
                return;
            }
            bool ok = alike(A, B) && alike(A, H);
            for (size_t p = 0; ok && p < H.F.size(); ++p) {
                for (int i = 0; i < H.F[p].n; ++i) {
                    const V3 dv = (A.F[p].p[(size_t)i] + B.F[p].p[(size_t)i]) * 0.5 - H.F[p].p[(size_t)i];
                    if (std::sqrt(dv.x * dv.x + dv.z * dv.z) > tol) ok = false;
                }
                if (std::fabs(0.5 * (A.e[p] + B.e[p]) - H.e[p]) > 0.03 * H.e[p]) ok = false;
            }
            if (ok) {
                ++st.smooth;
                segs[nseg++] = Seg{a, h, true};
                segs[nseg++] = Seg{h, b, true};
                return;
            }
            self(self, a, h);
            self(self, h, b);
        };
        resolve(resolve, 0, G / 2);
        resolve(resolve, G / 2, G);
        for (int j = 0; j < m; ++j) {
            const int lo = j * N / m, hi = (j + 1) * N / m;
            if (hi <= lo) continue;
            V3 colour{0, 0, 0};
            double n = 0;
            for (int i = lo; i < hi; ++i) {
                colour = colour + bands[(size_t)i].rgb;
                n += nb[(size_t)i];
            }
            n /= (hi - lo);
            const double x = std::min((double)G, std::max(0.0, G * (n - nRed) / (nBlue - nRed)));
            int si = 0;
            while (si + 1 < nseg && x > segs[si].b) ++si;
            const Seg& sg = segs[si];
            const Sample &A = samp[(size_t)sg.a], &B = samp[(size_t)sg.b];
            if (sg.line) {
                const double t = (x - sg.a) / (double)(sg.b - sg.a);
                for (size_t p = 0; p < A.F.size(); ++p) {
                    Poly H;
                    H.n = A.F[p].n;
                    for (int i = 0; i < H.n; ++i) H.p[(size_t)i] = A.F[p].p[(size_t)i] * (1.0 - t) + B.F[p].p[(size_t)i] * t;
                    emit(H, colour * (A.e[p] * (1.0 - t) + B.e[p] * t));
                    ++st.patches;
                }
            } else {
                // no line to read from: the nearer end that has light
                const Sample& U = (x - sg.a <= sg.b - x) ? (A.F.empty() ? B : A) : (B.F.empty() ? A : B);
                for (size_t p = 0; p < U.F.size(); ++p) {
                    emit(U.F[p], colour * U.e[p]);
                    ++st.patches;
                }
            }
        }
    });
    FastStats st;
    st.ways = (int)ways.size();
    size_t total = 0;
    for (const auto& t : outTri) total += t.size();
    tri.reserve(total);
    for (size_t i = 0; i < outTri.size(); ++i) {
        tri.insert(tri.end(), outTri[i].begin(), outTri[i].end());
        st.coloured += outSt[i].coloured;
        st.smooth += outSt[i].smooth;
        st.patches += outSt[i].patches;
        st.follows += outSt[i].follows;
    }
    if (stats) *stats = st;
}
inline void trace_fast(const Solid& solid, const Params& par, double detail_, std::vector<float>& tri, FastStats* stats = nullptr) {
    trace_fast(one(solid), par, std::vector<double>{detail_}, tri, stats);
}

// ---- the cheap version: plausible, not exact.
// The exact trace above splits every beam wherever its parts go different ways; with a few solids that is
// hundreds of ways and far too slow for a frame. Here each lit face (or each piece of it, see `pieces`) sends ONE
// ray from its middle. Wherever that ray goes - into the glass, mirrored inside, out, on through another solid,
// down to the floor - the whole piece is sent the same way as one flat patch. Nothing is clipped, nothing is
// split, so pieces of light can overlap or leave gaps where the true beam would have divided. The cost is
// (lit pieces) x (colour steps), whatever the number of solids.
struct CheapStats {
    int pieces = 0, patches = 0, rays = 0;
    int alike = 0;  // balls: their light is the one shared shape (see "A ball by likeness")
};
namespace detail {

// A ray in the air meets one solid: how far, the outward normal there, and which face (FLAT: its index;
// ROD: 0 side, 1 the +axis end, 2 the -axis end; BALL: 0).
// A cone, a cone with its tip cut off, or (both ends as wide) a rod: one shape for the formulas of light.
struct Taper {
    V3 c, ax;     // the middle of the base, and the unit axis from there to the top
    double r, rt; // radius of the base and of the top (0: a cone with its tip)
    double h;     // height
};
inline Taper taper_of(const Solid& so) {
    if (so.kind == Solid::ROD) return Taper{so.c - so.ax * so.h, so.ax, so.r, so.r, 2.0 * so.h};
    return Taper{so.c, so.ax, so.r, 0.0, so.h};
}
// Every place a line o + t d crosses its surface (side: face 0, base: face 1, top: face 2): t and the outward
// normal. Returns how many (0 to 2, the shape is convex; touching points may be missing).
inline int taper_crossings(const Taper& so, V3 o, V3 d, double t[2], V3 n[2], int face[2]) {
    const V3 w = o - so.c;
    const double k = (so.r - so.rt) / so.h, y0 = dot(w, so.ax), da = dot(d, so.ax);
    const V3 wp = w - so.ax * y0, dp = d - so.ax * da;
    int count = 0;
    auto add = [&](double tt, V3 nn, int f) {
        if (count < 2) {
            t[count] = tt;
            n[count] = nn;
            face[count++] = f;
        }
    };
    // the side: |across| = r - k y, for y between base and top
    const double A = dot(dp, dp) - k * k * da * da, B = dot(wp, dp) + k * da * (so.r - k * y0), C = dot(wp, wp) - (so.r - k * y0) * (so.r - k * y0);
    double root[2];
    int roots = 0;
    if (std::fabs(A) > 1e-14) {
        const double disc = B * B - A * C;
        if (disc > 0) {
            const double sq = std::sqrt(disc);
            root[0] = (-B - sq) / A;
            root[1] = (-B + sq) / A;
            roots = 2;
        }
    } else if (std::fabs(B) > 1e-14) {
        root[0] = -C / (2.0 * B);
        roots = 1;
    }
    const double slope = 1.0 / std::sqrt(1.0 + k * k);
    for (int i = 0; i < roots; ++i) {
        const double y = y0 + root[i] * da;
        if (y < 0 || y > so.h) continue;
        const V3 q = wp + dp * root[i];
        const double ql = length(q);
        add(root[i], ql > 1e-12 * so.r ? (q * (1.0 / ql) + so.ax * k) * slope : so.ax, 0);
    }
    // the base and the top
    if (std::fabs(da) > 1e-14) {
        const double tb = -y0 / da;
        const V3 q = wp + dp * tb;
        if (dot(q, q) <= so.r * so.r) add(tb, so.ax * -1.0, 1);
        if (so.rt > 0) {
            const double tu = (so.h - y0) / da;
            const V3 qu = wp + dp * tu;
            if (dot(qu, qu) <= so.rt * so.rt) add(tu, so.ax, 2);
        }
    }
    if (count == 2 && t[1] < t[0]) {
        std::swap(t[0], t[1]);
        std::swap(n[0], n[1]);
        std::swap(face[0], face[1]);
    }
    return count;
}
inline int cone_crossings(const Solid& so, V3 o, V3 d, double t[2], V3 n[2], int face[2]) { return taper_crossings(taper_of(so), o, d, t, n, face); }
inline bool meet(const Solid& so, V3 o, V3 d, double& t, V3& n, int& face) {
    if (so.kind == Solid::CONE) {
        double tt[2];
        V3 nn[2];
        int ff[2];
        if (cone_crossings(so, o, d, tt, nn, ff) < 2 || tt[0] < -1e-9 || dot(nn[0], d) >= 0) return false;
        t = tt[0];
        n = nn[0];
        face = ff[0];
        return true;
    }
    if (so.kind == Solid::FLAT) {
        double tin = -1e300, tout = 1e300;
        int gin = -1;
        for (int j = 0; j < (int)so.planes.size(); ++j) {
            if (so.faces[(size_t)j].n == 0) continue;
            const double dj = dot(so.planes[(size_t)j].n, d), off = so.planes[(size_t)j].o - dot(so.planes[(size_t)j].n, o);
            if (dj < -1e-12) {
                if (off / dj > tin) {
                    tin = off / dj;
                    gin = j;
                }
            } else if (dj > 1e-12) {
                tout = std::min(tout, off / dj);
            } else if (off < 0) {
                return false;
            }
        }
        if (gin < 0 || tin >= tout || tin < -1e-9) return false;
        t = tin;
        n = so.planes[(size_t)gin].n;
        face = gin;
        return true;
    }
    if (so.kind == Solid::BALL) {
        const V3 w = o - so.c;
        const double b = dot(w, d), cc = dot(w, w) - so.r * so.r, disc = b * b - cc;
        if (disc <= 0) return false;
        t = -b - std::sqrt(disc);
        if (t < -1e-9) return false;
        n = (o + d * t - so.c) * (1.0 / so.r);
        face = 0;
        return true;
    }
    // ROD: the part of an endless cylinder between two planes
    const V3 w = o - so.c;
    const double wa = dot(w, so.ax), da = dot(d, so.ax);
    const V3 wp = w - so.ax * wa, dp = d - so.ax * da;
    double tinS = -1e300, toutS = 1e300, tinC = -1e300, toutC = 1e300;
    if (std::fabs(da) > 1e-12) {
        const double t1 = (-so.h - wa) / da, t2 = (so.h - wa) / da;
        tinS = std::min(t1, t2);
        toutS = std::max(t1, t2);
    } else if (std::fabs(wa) > so.h) {
        return false;
    }
    const double A = dot(dp, dp), B = dot(wp, dp), C = dot(wp, wp) - so.r * so.r;
    if (A > 1e-18) {
        const double disc = B * B - A * C;
        if (disc <= 0) return false;
        const double sq = std::sqrt(disc);
        tinC = (-B - sq) / A;
        toutC = (-B + sq) / A;
    } else if (C > 0) {
        return false;
    }
    const double tin = std::max(tinS, tinC), tout = std::min(toutS, toutC);
    if (tin >= tout || tin < -1e-9) return false;
    t = tin;
    if (tinC >= tinS) {
        const V3 q = o + d * t - so.c;
        n = normalize(q - so.ax * dot(q, so.ax));
        face = 0;
    } else {
        n = da < 0 ? so.ax : so.ax * -1.0;
        face = da < 0 ? 1 : 2;
    }
    return true;
}
// A ray inside a solid (starting on face `from`, or anywhere inside with from = -1) reaches its surface.
inline bool leave(const Solid& so, V3 p, V3 d, int from, double& t, V3& n, int& face) {
    if (so.kind == Solid::CONE) {
        double tt[2];
        V3 nn[2];
        int ff[2];
        const int count = cone_crossings(so, p, d, tt, nn, ff);
        for (int i = 0; i < count; ++i)
            if (tt[i] > 1e-9 * so.r && dot(nn[i], d) > 0) {
                t = tt[i];
                n = nn[i];
                face = ff[i];
                return true;
            }
        return false;
    }
    if (so.kind == Solid::FLAT) {
        int best = -1;
        double tb = 1e300;
        for (int j = 0; j < (int)so.planes.size(); ++j) {
            if (j == from || so.faces[(size_t)j].n == 0) continue;
            const double dj = dot(so.planes[(size_t)j].n, d);
            if (dj <= 1e-12) continue;
            const double tt = (so.planes[(size_t)j].o - dot(so.planes[(size_t)j].n, p)) / dj;
            if (tt < tb) {
                tb = tt;
                best = j;
            }
        }
        if (best < 0) return false;
        t = tb;
        n = so.planes[(size_t)best].n;
        face = best;
        return true;
    }
    if (so.kind == Solid::BALL) {
        const V3 w = p - so.c;
        const double b = dot(w, d), disc = b * b - (dot(w, w) - so.r * so.r);
        t = -b + std::sqrt(std::max(0.0, disc));
        if (t <= 1e-12 * so.r) return false;
        n = (p + d * t - so.c) * (1.0 / so.r);
        face = 0;
        return true;
    }
    const V3 w = p - so.c;
    const double wa = dot(w, so.ax), da = dot(d, so.ax);
    const V3 wp = w - so.ax * wa, dp = d - so.ax * da;
    double tS = 1e300, tC = 1e300;
    if (da > 1e-12) tS = (so.h - wa) / da;
    else if (da < -1e-12) tS = (-so.h - wa) / da;
    const double A = dot(dp, dp), B = dot(wp, dp), C = dot(wp, wp) - so.r * so.r;
    if (A > 1e-18) tC = (-B + std::sqrt(std::max(0.0, B * B - A * C))) / A;
    t = std::min(tS, tC);
    if (!(t < 1e299) || t <= 1e-12 * so.r) return false;
    if (tC <= tS) {
        const V3 q = p + d * t - so.c;
        n = normalize(q - so.ax * dot(q, so.ax));
        face = 0;
    } else {
        n = da > 0 ? so.ax : so.ax * -1.0;
        face = da > 0 ? 1 : 2;
    }
    return true;
}
// First solid a ray in the air meets (other than `from`): which, how far, through which face, and the normal there.
inline bool first_solid(const Scene& sc, V3 o, V3 d, int from, int& S, int& g, double& t, V3& n) {
    t = 1e300;
    S = -1;
    for (int k = 0; k < (int)sc.solids.size(); ++k) {
        if (k == from) continue;
        const V3 rel = sc.centre[(size_t)k] - o;
        const double along = dot(rel, d);
        if (along < -sc.radius[(size_t)k] || dot(rel, rel) - along * along > sc.radius[(size_t)k] * sc.radius[(size_t)k]) continue;
        double tk;
        V3 nk;
        int gk;
        if (meet(sc.solids[(size_t)k], o, d, tk, nk, gk) && tk < t) {
            t = tk;
            S = k;
            g = gk;
            n = nk;
        }
    }
    return S >= 0;
}

// The way one ray takes from a point on solid A, as a list of planes to slide along and the direction between
// them. Returns false if the light never reaches the floor.
struct Leg {
    V3 d;        // direction of travel on this leg
    Plane to;    // the plane the leg ends on (the floor for the last one)
};
struct Route {
    Leg leg[Patch::SEQ + 2];
    int n = 0;
    double gain = 0;  // floor irradiance per unit of sun irradiance (flux through the piece / landed area)
    double weight = 1; // the share of the light that is left after every face on the way (Fresnel only)
    uint64_t key = 0; // which faces were touched: two indices with the same key take the same way
};
// `start` lies on solid A where its outward normal is `nStart` (face `fStart`).
inline bool route(const Scene& sc, const Params& par, double n, int A, int fStart, V3 start, V3 nStart, bool mirrored, Route& r) {
    const V3 L = normalize(par.sun);
    r.n = 0;
    r.weight = 1.0;
    r.key = 1469598103934665603ull;
    auto touch = [&](int s, int g, int kind) { r.key = (r.key ^ (uint64_t)(((s << 6 | g) << 2 | kind) + 1)) * 1099511628211ull; };
    V3 p = start, d = L;
    double ci = -dot(nStart, d);
    if (ci <= 1e-9) return false;
    double irr = ci;       // flux per unit area of the plane the light is on now
    double cosOn = ci;     // |n.d| of that plane for the leg that follows
    int S = A, g = fStart, entries = 0;
    V3 ng = nStart;
    bool outside = true;   // about to meet solid S (normal ng) from outside
    for (int guard = 0; guard < Patch::SEQ; ++guard) {
        if (outside) {
            ci = -dot(ng, d);
            const double Rf = fresnel(ci, 1.0, n);
            if (mirrored && guard == 0) {
                irr *= Rf;
                r.weight *= Rf;
                d = d + ng * (2.0 * ci);
                touch(S, g, 1);
                // mirrored off the first face: on through the air
            } else {
                if (entries >= par.maxSolids) return false;
                ++entries;
                irr *= 1.0 - Rf;
                r.weight *= 1.0 - Rf;
                V3 in;
                refract(d, ng, 1.0 / n, in);
                d = in;
                touch(S, g, 0);
                cosOn = std::fabs(dot(ng, d));
                // inside: mirror until the glass lets the light out
                const Solid& so = sc.solids[(size_t)S];
                int from = so.kind == Solid::FLAT ? g : -1;
                bool out = false;
                for (int b = 0; b <= par.maxBounces; ++b) {
                    double tb;
                    V3 nj;
                    int best;
                    if (!leave(so, p, d, from, tb, nj, best)) return false;
                    p = p + d * tb;
                    r.leg[r.n++] = Leg{d, Plane{nj, dot(nj, p)}};  // (a round face counts as its tangent plane here)
                    const double db = dot(nj, d);
                    irr *= db / cosOn;
                    const double Ri = fresnel(db, n, 1.0);
                    if (Ri < 1.0) {
                        irr *= 1.0 - Ri;
                        r.weight *= 1.0 - Ri;
                        V3 o;
                        refract(d, nj * -1.0, n, o);
                        d = o;
                        cosOn = std::fabs(dot(nj, d));
                        touch(S, best, 2);
                        out = true;
                        break;
                    }
                    d = d - nj * (2.0 * db);
                    cosOn = db;
                    if (so.kind == Solid::FLAT) from = best;
                    touch(S, best, 3);
                }
                if (!out) return false;  // still inside after the allowed mirrors: lost
            }
            if (mirrored && guard == 0) cosOn = ci;
            outside = false;
        }
        // in the air: another solid, or the floor
        int S2, g2;
        double t2;
        V3 n2;
        if (sc.solids.size() > 1 && first_solid(sc, p, d, S, S2, g2, t2, n2)) {
            p = p + d * t2;
            r.leg[r.n++] = Leg{d, Plane{n2, dot(n2, p)}};
            irr *= -dot(n2, d) / cosOn;
            S = S2;
            g = g2;
            ng = n2;
            outside = true;
            continue;
        }
        if (d.y > -1e-9) return false;  // up into the sky
        r.leg[r.n++] = Leg{d, Plane{V3{0, 1, 0}, par.floorY}};
        r.gain = irr * (-d.y) / cosOn;
        return true;
    }
    return false;
}
// Sends a polygon lying on the first face along a route. Corners come out as (x, travel, z).
inline void send(const Route& r, const Poly& P, double floorY, Poly& F) {
    F.n = 0;
    for (int i = 0; i < P.n; ++i) {
        V3 v = P.p[(size_t)i];
        double last = 0;
        for (int k = 0; k < r.n; ++k) {
            const double den = dot(r.leg[k].to.n, r.leg[k].d);
            double t = (r.leg[k].to.o - dot(r.leg[k].to.n, v)) / den;
            if (k == r.n - 1 && t < 0) t = 0;  // already on the floor
            v = v + r.leg[k].d * t;
            last = t;
        }
        (void)floorY;
        F.push({v.x, last, v.z});
    }
}

}  // namespace detail

namespace detail {
// The light of one flat face (of a flat solid, or the flat end of a rod): the face is cut into pieces x pieces
// parts along its first edge, and each part goes wherever one ray from its middle goes.
// A face cut into pieces x pieces parts along its first edge (thin faces, the chamfers, along their length only).
inline void face_pieces(const Poly& face, V3 nf, int pieces, std::vector<Poly>& list) {
    list.clear();
    if (face.n < 3) return;
    if (pieces <= 1) {
        list.push_back(face);
        return;
    }
    const V3 u = normalize(face.p[1] - face.p[0]), v = cross(nf, u);
    double u0 = 1e300, u1 = -1e300, v0 = 1e300, v1 = -1e300;
    for (int i = 0; i < face.n; ++i) {
        u0 = std::min(u0, dot(face.p[(size_t)i], u)); u1 = std::max(u1, dot(face.p[(size_t)i], u));
        v0 = std::min(v0, dot(face.p[(size_t)i], v)); v1 = std::max(v1, dot(face.p[(size_t)i], v));
    }
    // thin faces (chamfers) are cut along their length only
    const int kv = (v1 - v0) * 6.0 < (u1 - u0) ? 1 : pieces, ku = (u1 - u0) * 6.0 < (v1 - v0) ? 1 : pieces;
    const double tiny = 1e-12 * area(face);
    for (int a = 0; a < ku; ++a)
        for (int b = 0; b < kv; ++b) {
            Poly t = face;
            clip(t, Half{u, -(u0 + (u1 - u0) * a / ku)});
            clip(t, Half{u * -1.0, u0 + (u1 - u0) * (a + 1) / ku});
            clip(t, Half{v, -(v0 + (v1 - v0) * b / kv)});
            clip(t, Half{v * -1.0, v0 + (v1 - v0) * (b + 1) / kv});
            if (t.n >= 3 && area(t) > tiny) list.push_back(t);
        }
}
inline void flat_light(const Scene& sc, const Params& par, int A, int f, const Poly& face, V3 nf, int pieces, double detail_, const std::vector<Band>& bands,
                       const std::vector<double>& nb, std::vector<float>& tri, CheapStats& st) {
    const int N = par.bands;
    const double nBlue = nb[0], nRed = nb[(size_t)N - 1];
    const V3 L = normalize(par.sun);
    const size_t ns = sc.solids.size();
    auto emit = [&](const Poly& F, V3 rgb) {
        for (int i = 1; i + 1 < F.n; ++i) {
            const int idx[3] = {0, i, i + 1};
            for (int k = 0; k < 3; ++k) {
                tri.push_back((float)F.p[(size_t)idx[k]].x);
                tri.push_back((float)F.p[(size_t)idx[k]].z);
                tri.push_back((float)rgb.x);
                tri.push_back((float)rgb.y);
                tri.push_back((float)rgb.z);
                tri.push_back((float)F.p[(size_t)idx[k]].y);
            }
        }
        ++st.patches;
    };
    static thread_local std::vector<Poly> list;
    face_pieces(face, nf, pieces, list);
    for (const Poly& P : list) {
        V3 mid{0, 0, 0};
        for (int i = 0; i < P.n; ++i) mid = mid + P.p[(size_t)i];
        mid = mid * (1.0 / P.n);
        // does another solid take the sun before it gets here?
        if (ns > 1) {
            int S, g;
            double t;
            V3 nn;
            if (first_solid(sc, mid - L * 1e3, L, -1, S, g, t, nn) && (S != A || std::fabs(t - 1e3) > 1e-6 * 1e3 + 1e-7)) continue;
        }
        ++st.pieces;
        Route rr, rb, r;
        Poly F;
        // mirrored off the face: one white patch
        if (par.outerReflection) {
            ++st.rays;
            if (route(sc, par, 0.5 * (nRed + nBlue), A, f, mid, nf, true, r)) {
                send(r, P, par.floorY, F);
                emit(F, V3{1, 1, 1} * r.gain);
            }
        }
        // through the glass: how far apart do red and blue land?
        st.rays += 2;
        const bool okR = route(sc, par, nRed, A, f, mid, nf, false, rr), okB = route(sc, par, nBlue, A, f, mid, nf, false, rb);
        if (!okR && !okB) continue;
        int m = N / 4;  // red and blue go different ways: a fair number of steps
        if (okR && okB && rr.key == rb.key) {
            Poly Fr, Fb;
            send(rr, P, par.floorY, Fr);
            send(rb, P, par.floorY, Fb);
            double move = 0;
            for (int i = 0; i < Fr.n; ++i) {
                const V3 dv = Fr.p[(size_t)i] - Fb.p[(size_t)i];
                move = std::max(move, std::sqrt(dv.x * dv.x + dv.z * dv.z));
            }
            m = move <= detail_ ? 1 : (int)std::min<double>(N, std::ceil(move / detail_));
        }
        for (int j = 0; j < m; ++j) {
            const int lo = j * N / m, hi = (j + 1) * N / m;
            if (hi <= lo) continue;
            V3 colour{0, 0, 0};
            double n = 0;
            for (int i = lo; i < hi; ++i) {
                colour = colour + bands[(size_t)i].rgb;
                n += nb[(size_t)i];
            }
            n /= (hi - lo);
            ++st.rays;
            if (!route(sc, par, n, A, f, mid, nf, false, r)) continue;
            send(r, P, par.floorY, F);
            emit(F, colour * r.gain);
        }
    }
}
}  // namespace detail

// The sunlit side of a rod as a mesh of points with normals (world space), for its light and its outline.
// (A ball has its own, simpler way: see "A ball by likeness".)
struct LitMesh {
    std::vector<V3> p, n;
    std::vector<int> face;
    std::vector<int> tri;  // three indices each
};
inline void lit_mesh(const Solid& so, V3 L, LitMesh& m, bool roundPartOnly = false) {
    m.p.clear();
    m.n.clear();
    m.face.clear();
    m.tri.clear();
    const double pi = 3.14159265358979323846, edge = 0.497 * pi;  // stop just short of the grazing rim
    auto frame = [](V3 w, V3& e1, V3& e2) {
        const V3 up = std::fabs(w.y) < 0.9 ? V3{0, 1, 0} : V3{1, 0, 0};
        e1 = normalize(cross(up, w));
        e2 = cross(w, e1);
    };
    auto disc = [&](V3 mid, V3 nrm, V3 e1, V3 e2, double rad, int rings, int sectors, int faceId) {
        // rings around `mid`
        const int base = (int)m.p.size();
        m.p.push_back(mid);
        m.n.push_back(nrm);
        m.face.push_back(faceId);
        for (int i = 1; i <= rings; ++i)
            for (int j = 0; j < sectors; ++j) {
                const double ph = 2.0 * pi * j / sectors, u = (double)i / rings;
                const V3 dir = e1 * std::cos(ph) + e2 * std::sin(ph);
                m.p.push_back(mid + dir * (rad * u));
                m.n.push_back(nrm);
                m.face.push_back(faceId);
            }
        for (int j = 0; j < sectors; ++j) {
            m.tri.insert(m.tri.end(), {base, base + 1 + j, base + 1 + (j + 1) % sectors});
            for (int i = 1; i < rings; ++i) {
                const int a = base + 1 + (i - 1) * sectors + j, b = base + 1 + (i - 1) * sectors + (j + 1) % sectors;
                const int c2 = a + sectors, d2 = b + sectors;
                m.tri.insert(m.tri.end(), {a, c2, d2, a, d2, b});
            }
        }
    };
    if (so.kind == Solid::ROD) {
        // the side: the half that faces the sun
        const V3 w = L * -1.0;
        const V3 wp = w - so.ax * dot(w, so.ax);
        if (length(wp) > 1e-6) {
            const V3 u = normalize(wp), v = cross(so.ax, u);
            const int na = 8, ns = 28, base = (int)m.p.size();
            for (int i = 0; i <= na; ++i)
                for (int j = 0; j <= ns; ++j) {
                    const double ps = -edge + 2.0 * edge * j / ns, sAx = -so.h + 2.0 * so.h * i / na;
                    const V3 nn = u * std::cos(ps) + v * std::sin(ps);
                    m.p.push_back(so.c + so.ax * sAx + nn * so.r);
                    m.n.push_back(nn);
                    m.face.push_back(0);
                }
            for (int i = 0; i < na; ++i)
                for (int j = 0; j < ns; ++j) {
                    const int a = base + i * (ns + 1) + j, b = a + 1, c2 = a + ns + 1, d2 = c2 + 1;
                    m.tri.insert(m.tri.end(), {a, c2, d2, a, d2, b});
                }
        }
        // the end that faces the sun
        for (int sgn = -1; sgn <= 1 && !roundPartOnly; sgn += 2) {
            const V3 nn = so.ax * (double)sgn;
            if (dot(nn, L) > -1e-6) continue;
            V3 e1, e2;
            frame(nn, e1, e2);
            disc(so.c + so.ax * (so.h * sgn), nn, e1, e2, so.r, 8, 32, sgn > 0 ? 1 : 2);
        }
    }
}

// ---- A ball by likeness ------------------------------------------------------------------------------------------
// A ball is the same from every side, and every ball is the same ball at another size. So a ball's light is ONE
// shape, copied: the sunlit side as rings around the middle of the sunbeam (ball_disc), every corner sent through
// a ball of radius 1 by a closed formula that depends on its ring alone (ball_row), turned about the sun's axis,
// scaled by the radius and dropped on the floor (ball_land). Nothing is followed per ball. A renderer keeps the
// mesh once and draws it for every ball and colour step (ball_steps), doing ball_corner per corner on the GPU
// (ball_light.mat); all the CPU hands over per ball is its middle and its radius.
// What is left out on purpose: other solids. A ball's light is the same copy whatever stands in its sun or in
// its light; where lights and shadows overlap on the floor they simply add up. (Following every corner against
// the solids near it was tried: 30 balls looked the same to 0.1 / 255 and it brought back thresholds that can
// make a moving picture jump.) Also left out: the faint light mirrored off the outside of the ball (0.02 / 255).
// Everything here is smooth in the ball's place, its size and the sun, so nothing jumps from frame to frame.
constexpr int BALL_RINGS = 14, BALL_SECTORS = 56;
constexpr double BALL_BEND = 0.497 * 3.14159265358979323846;  // the outermost ring: this far round from the middle of the sunlit side (just short of grazing)
constexpr double BALL_CAP = 40.0;   // a point of light may be this many times the sun, no more
constexpr double BALL_FLAT = 0.02;  // light that leaves upwards, or flatter than this, is not drawn (it lands nowhere near, or never)
constexpr double BALL_REACH = 8.0;  // light that has flown further than this many radii is not drawn: past the ball's focus it thins out with
                                    // the square of the distance, and there it is under half a percent of the sun spread over a huge patch of floor
// (towards both limits the light is faded to nothing first, see ball_fade, so that nothing is cut off visibly)

// The mesh: (ring, sector) per corner, three indices per triangle. Corner 0 is the middle (ring 0).
inline void ball_disc(std::vector<float>& ringSector, std::vector<uint32_t>& idx) {
    ringSector.assign({0.f, 0.f});
    idx.clear();
    for (int i = 1; i <= BALL_RINGS; ++i)
        for (int j = 0; j < BALL_SECTORS; ++j) {
            ringSector.push_back((float)i);
            ringSector.push_back((float)j);
        }
    for (int j = 0; j < BALL_SECTORS; ++j) {
        const uint32_t n1 = (uint32_t)((j + 1) % BALL_SECTORS);
        idx.insert(idx.end(), {0u, 1u + (uint32_t)j, 1u + n1});
        for (int i = 1; i < BALL_RINGS; ++i) {
            const uint32_t a = 1u + (uint32_t)((i - 1) * BALL_SECTORS + j), b = 1u + (uint32_t)(i - 1) * BALL_SECTORS + n1;
            const uint32_t c2 = a + BALL_SECTORS, d2 = b + BALL_SECTORS;
            idx.insert(idx.end(), {a, c2, d2, a, d2, b});
        }
    }
}
inline int ball_corner_index(int ring, int sector) { return ring == 0 ? 0 : 1 + (ring - 1) * BALL_SECTORS + sector; }
// The corners around (ring, sector), in order round it: each one with the next (and the last with the first) makes
// one of the triangles of the mesh that meet at this corner. Around the middle, 8 of its 56. Returns how many;
// a ring beyond the last means "no such corner".
inline int ball_around(int ring, int sector, int at[8][2]) {
    const int S = BALL_SECTORS;
    if (ring == 0) {
        for (int k = 0; k < 8; ++k) {
            at[k][0] = 1;
            at[k][1] = k * S / 8;
        }
        return 8;
    }
    const int next = (sector + 1) % S, prev = (sector + S - 1) % S;
    const int six[6][2] = {{ring + 1, sector}, {ring + 1, next}, {ring, next}, {ring - 1, sector}, {ring - 1, prev}, {ring, prev}};
    for (int k = 0; k < 6; ++k) {
        at[k][0] = six[k][0];
        at[k][1] = six[k][1];
    }
    return 6;
}
// The colour steps a ball's light is drawn in.
struct BallStep {
    V3 rgb;
    double n;
};
inline std::vector<BallStep> ball_steps(const Params& par, int count = 0) {
    const int N = par.bands, steps = std::max(1, std::min(N, count > 0 ? count : par.roundSteps));
    const std::vector<Band> bands = make_bands(N, par.nmMin, par.nmMax);
    std::vector<BallStep> out;
    for (int j = 0; j < steps; ++j) {
        const int lo = j * N / steps, hi = (j + 1) * N / steps;
        BallStep s{V3{0, 0, 0}, 0};
        for (int i = lo; i < hi; ++i) {
            s.rgb = s.rgb + bands[(size_t)i].rgb;
            s.n += par.glass.n(bands[(size_t)i].nm);
        }
        s.n /= std::max(1, hi - lo);
        out.push_back(s);
    }
    return out;
}
// One ray through a ball of radius 1, in the plane of the sun's axis (first number) and the ray (second): where
// it leaves the ball, which way it goes, and how much of the light is left. It depends on the ring alone.
struct BallRow {
    double a, b, da, db, weight;
    bool ok;
};
inline BallRow ball_row(int ring, double n) {
    const double th = (double)ring / BALL_RINGS * BALL_BEND, cs = std::cos(th), sn = std::sin(th);
    const V3 p0{cs, sn, 0}, d{-1, 0, 0};
    const double R0 = fresnel(cs, 1.0, n);
    V3 in, out;
    if (!refract(d, p0, 1.0 / n, in)) return BallRow{0, 0, 0, 0, 0, false};
    const V3 p1 = p0 + in * (-2.0 * dot(p0, in));              // straight across to the far side
    if (!refract(in, p1 * -1.0, n, out)) return BallRow{0, 0, 0, 0, 0, false};
    return BallRow{p1.x, p1.y, out.x, out.y, (1.0 - R0) * (1.0 - R0), true};  // the way out mirrors as much as the way in
}
// Everything about the shape that is the same for every ball of a frame.
struct BallShape {
    V3 w, e1, e2;               // towards the sun, and two directions across the sunbeam
    V3 across[BALL_SECTORS];    // the direction of each sector
    double out[BALL_RINGS + 1]; // how far from the middle of the sunbeam each ring is (ball of radius 1)
    double cs[BALL_SECTORS], sn[BALL_SECTORS];
    std::vector<BallStep> steps;
    struct Rows {
        BallRow ring[BALL_RINGS + 1];
    };
    std::vector<Rows> rows;     // per step
};
inline void sun_frame(V3 sun, V3& w, V3& e1, V3& e2) {
    w = normalize(sun) * -1.0;
    const V3 up = std::fabs(w.y) < 0.9 ? V3{0, 1, 0} : V3{1, 0, 0};
    e1 = normalize(cross(up, w));
    e2 = cross(w, e1);
}
inline BallShape ball_shape(const Params& par) {
    BallShape sh;
    sun_frame(par.sun, sh.w, sh.e1, sh.e2);
    for (int j = 0; j < BALL_SECTORS; ++j) {
        const double ph = 2.0 * 3.14159265358979323846 * j / BALL_SECTORS;
        sh.cs[j] = std::cos(ph);
        sh.sn[j] = std::sin(ph);
        sh.across[j] = sh.e1 * sh.cs[j] + sh.e2 * sh.sn[j];
    }
    for (int i = 0; i <= BALL_RINGS; ++i) sh.out[i] = std::sin((double)i / BALL_RINGS * BALL_BEND);
    sh.steps = ball_steps(par);
    sh.rows.resize(sh.steps.size());
    for (size_t s = 0; s < sh.steps.size(); ++s)
        for (int i = 0; i <= BALL_RINGS; ++i) sh.rows[s].ring[i] = ball_row(i, sh.steps[s].n);
    return sh;
}
// The rows as a table for a renderer (RGBA float, BALL_RINGS + 1 wide, two lines per step): line 2s holds
// (a, b, da, db) of step s, line 2s + 1 holds (weight, 1 if the ray exists, how far out the ring is, 0).
// It depends on the glass alone, not on the sun and not on any ball.
inline void ball_table(const BallShape& sh, std::vector<float>& rgba, int& width, int& height) {
    width = BALL_RINGS + 1;
    height = 2 * (int)sh.steps.size();
    rgba.assign((size_t)width * height * 4, 0.f);
    for (size_t s = 0; s < sh.steps.size(); ++s)
        for (int i = 0; i <= BALL_RINGS; ++i) {
            const BallRow& r = sh.rows[s].ring[i];
            float* o = rgba.data() + ((2 * s) * (size_t)width + (size_t)i) * 4;
            o[0] = (float)r.a; o[1] = (float)r.b; o[2] = (float)r.da; o[3] = (float)r.db;
            o += (size_t)width * 4;
            o[0] = (float)r.weight; o[1] = r.ok ? 1.f : 0.f; o[2] = (float)sh.out[i];
        }
}
// How much of a corner's light is still drawn: 1, falling smoothly to 0 where it would be left out (flown too far
// for its radius, or leaving too flat), so that light never appears or disappears at full strength.
inline double ball_fade(double flownOverRadius, double down) {
    auto smooth = [](double x) {
        x = std::min(1.0, std::max(0.0, x));
        return x * x * (3.0 - 2.0 * x);
    };
    return smooth((BALL_REACH - flownOverRadius) / (0.5 * BALL_REACH)) * smooth((down - BALL_FLAT) / (4.0 * BALL_FLAT));
}
// Corner (ring, sector) of a ball (middle c, radius r) at one step: where its light lands (x, distance flown from
// the ball, z) and how much of it is still drawn (ball_fade). False if it is left out.
inline bool ball_land(const BallShape& sh, size_t step, V3 c, double r, int ring, int sector, double floorY, V3& q, double& fade) {
    fade = 0;
    const BallRow& row = sh.rows[step].ring[ring];
    if (!row.ok) return false;
    const V3& ac = sh.across[sector];
    const V3 D = sh.w * row.da + ac * row.db;
    if (!(D.y < -BALL_FLAT)) return false;
    const V3 E = c + (sh.w * row.a + ac * row.b) * r;
    const double t = std::max(0.0, (floorY - E.y) / D.y);
    if (t > BALL_REACH * r) return false;
    q = V3{E.x + D.x * t, t, E.z + D.z * t};
    fade = ball_fade(t / r, -D.y);
    return true;
}
// How bright the corner that landed at q is, as a multiple of what is left of the sun: the triangles that meet at
// it (ball_around; p, lands and fades are those corners), the sunbeam they catch over the floor they land on.
// Measured on the very triangles that are drawn, so the light adds up to what went in. Times the least `fade`
// of the corner and its neighbours: a triangle that is about to be left out is already dark at all three corners.
inline double ball_squeeze(const BallShape& sh, double r, int ring, int sector, V3 q, double fade, const int at[8][2], int count, const V3* p, const bool* lands,
                           const double* fades) {
    const double sx = sh.out[ring] * sh.cs[sector], sy = sh.out[ring] * sh.sn[sector];
    double before = 0, after = 0;
    for (int k = 0; k < count; ++k) {
        if (at[k][0] <= BALL_RINGS) fade = std::min(fade, lands[k] ? fades[k] : 0.0);
        const int m = (k + 1) % count;
        if (!lands[k] || !lands[m]) continue;
        const double ax = sh.out[at[k][0]] * sh.cs[at[k][1]] - sx, ay = sh.out[at[k][0]] * sh.sn[at[k][1]] - sy;
        const double bx = sh.out[at[m][0]] * sh.cs[at[m][1]] - sx, by = sh.out[at[m][0]] * sh.sn[at[m][1]] - sy;
        before += std::fabs(ax * by - ay * bx);
        after += std::fabs((p[k].x - q.x) * (p[m].z - q.z) - (p[k].z - q.z) * (p[m].x - q.x));
    }
    before *= r * r;
    if (!(before > 0)) return 0.0;
    return fade * (after * BALL_CAP > before ? before / after : BALL_CAP);
}
// One corner in full: where it lands and how bright it is there, as a multiple of the sun. This is what
// ball_light.mat does per corner.
inline bool ball_corner(const BallShape& sh, size_t step, V3 c, double r, int ring, int sector, double floorY, V3& q, double& light) {
    double fade;
    if (!ball_land(sh, step, c, r, ring, sector, floorY, q, fade)) return false;
    int at[8][2];
    const int count = ball_around(ring, sector, at);
    V3 p[8];
    bool lands[8];
    double fades[8];
    for (int k = 0; k < count; ++k) lands[k] = at[k][0] <= BALL_RINGS && ball_land(sh, step, c, r, at[k][0], at[k][1], floorY, p[k], fades[k]);
    light = sh.rows[step].ring[ring].weight * ball_squeeze(sh, r, ring, sector, q, fade, at, count, p, lands, fades);
    return true;
}
// A ball's light as triangles, worked out on the CPU exactly as a renderer draws the shared shape (for a renderer
// that does not, and for the desk check).
inline void ball_light(const BallShape& sh, V3 c, double r, double floorY, std::vector<float>& tri, int* patches = nullptr) {
    static const std::pair<std::vector<float>, std::vector<uint32_t>> disc = [] {
        std::pair<std::vector<float>, std::vector<uint32_t>> d;
        ball_disc(d.first, d.second);
        return d;
    }();
    const size_t nv = disc.first.size() / 2, nt = disc.second.size() / 3;
    const size_t base = tri.size();
    tri.resize(base + sh.steps.size() * nt * 3 * (size_t)TRI);
    float* o = tri.data() + base;
    std::vector<V3> q(nv);
    std::vector<double> e(nv);
    std::vector<char> ok(nv);
    for (size_t s = 0; s < sh.steps.size(); ++s) {
        for (size_t v = 0; v < nv; ++v) ok[v] = ball_corner(sh, s, c, r, (int)disc.first[v * 2], (int)disc.first[v * 2 + 1], floorY, q[v], e[v]);
        for (size_t t = 0; t < nt; ++t) {
            const uint32_t v[3] = {disc.second[t * 3], disc.second[t * 3 + 1], disc.second[t * 3 + 2]};
            if (!ok[v[0]] || !ok[v[1]] || !ok[v[2]]) continue;
            for (int k = 0; k < 3; ++k) {
                *o++ = (float)q[v[k]].x;
                *o++ = (float)q[v[k]].z;
                *o++ = (float)(sh.steps[s].rgb.x * e[v[k]]);
                *o++ = (float)(sh.steps[s].rgb.y * e[v[k]]);
                *o++ = (float)(sh.steps[s].rgb.z * e[v[k]]);
                *o++ = (float)q[v[k]].y;
            }
            if (patches) ++*patches;
        }
    }
    tri.resize((size_t)(o - tri.data()));
}

// ---- A cone, and a rod, by formula --------------------------------------------------------------------------------
// The light of a cone (and of a rod, which for this is a cone with its tip cut off so far down that both ends are
// as wide: `Taper`), like a ball's, is one mesh drawn many times, with nothing worked out per solid on the CPU.
// Each corner of the mesh is a point of the sunlit surface, sent through the solid by closed formulas (cone_walk)
// and dropped on the floor. What is different from a ball:
// - Light in a cone goes several ways. They are drawn as separate sheets, one instance each (solid, colour step,
//   way): way 0 leaves at the first face it meets inside, way 1 is mirrored there and leaves at the second.
//   Every sheet is the whole mesh; how much light takes a way is what the glass itself says (Fresnel), so a sheet
//   goes dark by itself, smoothly, where the glass lets nothing out or mirrors nothing, and its coloured edge stays.
// - A sheet tears where its light crosses a rim inside: on one side it meets the far side first, on the other an
//   end. Where that happens is found exactly (cone_split; light entering along one line from top to base all
//   bends the same way, which makes it one quadratic), and the mesh is laid so that a row lies on it: the side
//   is two patches, above and below the tear, which slide as the solid or the sun turns. The same for the two
//   lines where the sun grazes the side (cone_frame): the mesh ends on them. So the ends of the sheets are
//   where they belong at any moment and move smoothly.
// - Tears further in (after a mirror) are not found exactly: there the corners carry a `fade` that falls to
//   nothing towards the rims, and a corner is only as bright as the dimmest of itself and its neighbours, so
//   those ends fade out instead of jumping.
// - Light that leaves upwards, too flat or too far is not cut off either: its share (`weight`) falls to nothing
//   first, and every corner keeps a place on the floor that moves smoothly whatever its light does (cone_walk),
//   so no triangle ever has to be dropped.
// Other solids are left out, as for the ball.
using detail::Taper;
using detail::taper_crossings;
using detail::taper_of;
constexpr int CONE_PATCHES = 4;                    // side above the tear, side below it, base, top
constexpr int CONE_ROWS[CONE_PATCHES] = {5, 5, 3, 3};
constexpr int CONE_COLS[CONE_PATCHES] = {32, 32, 24, 24};  // columns: across the sunlit part of the side; round an end
constexpr int CONE_WAYS = 2;                // sheets per colour step: out at the first face met inside, or at the second
                                            // (a third, after two mirrors, changed pictures of six cones by 0.01 / 255)
constexpr double CONE_TIP = 0.03;           // the mesh of a cone's side starts this far (of the way down) from the tip
constexpr double CONE_EDGE = 0.06;          // after a mirror: fade within this far (of the size) of a rim or the tip
constexpr double CONE_INSET = 0.999;        // the mesh stops this short of a rim (light entering on the rim itself has no way through)
constexpr double CONE_CAP = 40.0;

// The mesh: (row, column, patch) per corner; three indices per triangle.
inline int cone_corner_index(int patch, int i, int j) {
    int base = 0;
    for (int p = 0; p < patch; ++p) base += (CONE_ROWS[p] + 1) * (CONE_COLS[p] + 1);
    return base + i * (CONE_COLS[patch] + 1) + j;
}
// withTop = false: without the triangles of the top (a cone with its tip has none there).
inline void cone_mesh(std::vector<float>& corners, std::vector<uint32_t>& idx, bool withTop = true) {
    corners.clear();
    idx.clear();
    for (int patch = 0; patch < CONE_PATCHES; ++patch) {
        const int rows = CONE_ROWS[patch], cols = CONE_COLS[patch];
        for (int i = 0; i <= rows; ++i)
            for (int j = 0; j <= cols; ++j) corners.insert(corners.end(), {(float)i, (float)j, (float)patch});
        if (patch == 3 && !withTop) continue;
        for (int i = 0; i < rows; ++i)
            for (int j = 0; j < cols; ++j) {
                const uint32_t a = (uint32_t)cone_corner_index(patch, i, j), b = (uint32_t)cone_corner_index(patch, i, j + 1);
                const uint32_t c2 = (uint32_t)cone_corner_index(patch, i + 1, j), d2 = (uint32_t)cone_corner_index(patch, i + 1, j + 1);
                idx.insert(idx.end(), {a, c2, d2, a, d2, b});
            }
    }
}
// The corners around (i, j) of a patch, in order round it: each one with the next (and the last with the first)
// makes one of the triangles of the mesh that meet there. A corner outside the patch has row -1.
inline void cone_around(int patch, int i, int j, int at[6][2]) {
    const int rows = CONE_ROWS[patch], cols = CONE_COLS[patch];
    const int six[6][2] = {{i + 1, j}, {i + 1, j + 1}, {i, j + 1}, {i - 1, j}, {i - 1, j - 1}, {i, j - 1}};
    for (int k = 0; k < 6; ++k) {
        const bool in = six[k][0] >= 0 && six[k][0] <= rows && six[k][1] >= 0 && six[k][1] <= cols;
        at[k][0] = in ? six[k][0] : -1;
        at[k][1] = six[k][1];
    }
}
// How the solid stands to the sun.
struct ConeFrame {
    V3 e1, e2;    // across the axis: e1 points the way the sunlight goes (the far side), e2 = axis x e1
    double half;  // the sunlit part of the side reaches this far round to either side of the line facing the sun (0: none, pi: all)
};
inline ConeFrame cone_frame(const Taper& so, V3 sun) {
    const V3 L = normalize(sun);
    const double La = dot(L, so.ax), k = (so.r - so.rt) / so.h;
    const V3 Lp = L - so.ax * La;
    const double Lr = length(Lp);
    ConeFrame f;
    if (Lr > 1e-9) {
        f.e1 = Lp * (1.0 / Lr);
    } else {
        const V3 up = std::fabs(so.ax.y) < 0.9 ? V3{0, 1, 0} : V3{1, 0, 0};
        f.e1 = normalize(cross(up, so.ax));
    }
    f.e2 = cross(so.ax, f.e1);
    const double q = Lr > 1e-9 ? -k * La / Lr : (La < 0 ? 2.0 : -2.0);
    f.half = q >= 1.0 ? 3.14159265358979323846 : (q <= -1.0 ? 0.0 : 3.14159265358979323846 - std::acos(q));
    return f;
}
// The side between `lo` and `hi` of the way down from top (0) to base (1) is what the mesh covers.
inline double cone_lo(const Taper& so) { return so.rt > 0 ? 1.0 - CONE_INSET : CONE_TIP; }
// Along the line from top to base at `across` on the side: how far down the tear is. Light (index of refraction n)
// that enters above it and light that enters below it meet different faces first: if it travels towards the
// base, the far side above and the base below; if towards the top, the top above and the far side below.
inline double cone_split(const Taper& so, V3 sun, V3 across, double n) {
    const V3 L = normalize(sun);
    const double k = (so.r - so.rt) / so.h, lo = cone_lo(so), hi = CONE_INSET;
    const V3 N = (across + so.ax * k) * (1.0 / std::sqrt(1.0 + k * k));
    V3 d;
    if (!(-dot(L, N) > 1e-5) || !refract(L, N, 1.0 / n, d)) return hi;
    const double da = dot(d, so.ax);
    if (da < -1e-9) {
        // towards the base: from height y the light meets the base's plane at across * r + y * g; on the rim at y = 0 and once more
        const V3 g = (d - so.ax * da) * (1.0 / -da) - across * k;
        const double gg = dot(g, g);
        if (!(gg > 1e-18)) return hi;
        const double y = -2.0 * so.r * dot(across, g) / gg;
        return std::min(hi, std::max(lo, 1.0 - y / so.h));
    }
    if (da > 1e-9 && so.rt > 0) {
        // towards the top: from u below the top it meets the top's plane at across * rt + u * g; on the rim at u = 0 and once more
        const V3 g = (d - so.ax * da) * (1.0 / da) + across * k;
        const double gg = dot(g, g);
        if (!(gg > 1e-18)) return lo;
        const double u = -2.0 * so.rt * dot(across, g) / gg;
        return std::min(hi, std::max(lo, u / so.h));
    }
    return da > 0 ? lo : hi;
}
// Where corner (i, j) of a patch is on the solid, and the outward normal there. n: the index of refraction of the
// colour step (the tear, and with it the rows of the side, lie a little differently for every colour).
inline void cone_entry(const Taper& so, const ConeFrame& f, V3 sun, double n, int patch, int i, int j, V3& P, V3& N) {
    const double pi = 3.14159265358979323846;
    if (patch >= 2) {
        const double ph = 2.0 * pi * j / CONE_COLS[patch], rad = (patch == 2 ? so.r : so.rt) * CONE_INSET * i / CONE_ROWS[patch];
        P = so.c + (patch == 2 ? V3{0, 0, 0} : so.ax * so.h) + (f.e1 * std::cos(ph) + f.e2 * std::sin(ph)) * rad;
        N = patch == 2 ? so.ax * -1.0 : so.ax;
        return;
    }
    const double ph = pi + (2.0 * j / CONE_COLS[patch] - 1.0) * f.half, k = (so.r - so.rt) / so.h;
    const V3 across = f.e1 * std::cos(ph) + f.e2 * std::sin(ph);
    const double split = cone_split(so, sun, across, n), part = (double)i / CONE_ROWS[patch], lo = cone_lo(so);
    const double down = patch == 0 ? lo + (split - lo) * part : split + (CONE_INSET - split) * part;
    P = so.c + so.ax * (so.h * (1.0 - down)) + across * (so.rt + (so.r - so.rt) * down);
    N = (across + so.ax * k) * (1.0 / std::sqrt(1.0 + k * k));
}
struct ConeHit {
    V3 q;           // where the corner is put on the floor: x, distance flown from the solid, z. Where its light lands; if that is
                    // nowhere near (it leaves upwards, too flat, too far), as far along its way as light is drawn at all, so
                    // that the place moves smoothly whatever the light does. If !ok: under the middle of the base.
    double weight;  // how much of the light takes this way (Fresnel at every face), and of that how much is still drawn
                    // (ball_fade: nothing once it leaves too flat or lands too far)
    double fade;    // 1, falling to 0 towards a tear that the mesh does not follow (the rims and the tip, after a mirror)
    int faces;      // which faces it met inside: corners with the same number belong to the same piece of the sheet
    bool ok;
};
// Corner at P (outward normal N) of patch `patch`, index of refraction n, `way` mirrors inside before it leaves.
// The two patches of the side are told which face they meet first (so that the row on the tear belongs to both
// sides exactly).
inline ConeHit cone_walk(const Taper& so, V3 sun, V3 P, V3 N, int patch, double n, int way, double floorY) {
    auto smooth = [](double x) {
        x = std::min(1.0, std::max(0.0, x));
        return x * x * (3.0 - 2.0 * x);
    };
    const V3 L = normalize(sun);
    const double size = std::max(so.r, so.h), k = (so.r - so.rt) / so.h;
    ConeHit hit;
    hit.q = V3{so.c.x, 0, so.c.z};  // a corner without light: all of them on one spot, so that their triangles cover no pixels
    hit.weight = 0;
    hit.fade = 0;
    hit.faces = 0;
    hit.ok = false;
    const double ci = -dot(L, N);
    if (!(ci > 1e-5)) return hit;
    double weight = 1.0 - fresnel(ci, 1.0, n), fade = 1.0;
    V3 d, D{0, 0, 0}, p = P;
    if (!refract(L, N, 1.0 / n, d)) return hit;
    int faces = 1;
    for (int b = 0; b <= way; ++b) {
        double t = 0;
        V3 nj;
        int face = 0;
        const double da = dot(d, so.ax), y0 = dot(p - so.c, so.ax);
        // the two parts of the side are told what they meet first: 1 the base, 2 the top, 0 the far side
        const int told = b > 0 || patch >= 2 ? -1 : (patch == 0 ? (da > 1e-9 && so.rt > 0 ? 2 : 0) : (da < -1e-9 ? 1 : 0));
        if (told == 1) {
            t = y0 / -da;
            nj = so.ax * -1.0;
            face = 1;
        } else if (told == 2) {
            t = (so.h - y0) / da;
            nj = so.ax;
            face = 2;
        } else {
            double tt[2];
            V3 nn[2];
            int ff[2];
            const int count = taper_crossings(so, p, d, tt, nn, ff);
            bool found = false;
            for (int i = 0; i < count && !found; ++i)
                if ((told < 0 || ff[i] == 0) && tt[i] > 1e-7 * size && dot(nn[i], d) > 0) {
                    t = tt[i];
                    nj = nn[i];
                    face = ff[i];
                    found = true;
                }
            if (!found) {
                if (told < 0) return hit;
                // on the tear itself the far side is met at a rim, where the search may just miss it
                if (da < -1e-9) t = y0 / -da;
                else if (da > 1e-9) t = (so.h - y0) / da;
                else return hit;
                const V3 w = p + d * t - so.c;
                const V3 out = normalize(w - so.ax * dot(w, so.ax));
                nj = (out + so.ax * k) * (1.0 / std::sqrt(1.0 + k * k));
                face = 0;
            }
        }
        p = p + d * t;
        if (b > 0) {
            const V3 w = p - so.c;
            const double y = dot(w, so.ax), rho = length(w - so.ax * y);
            // how far from a rim (or the tip)
            const double margin = face == 1 ? 1.0 - rho / so.r : (face == 2 ? 1.0 - rho / so.rt : std::min(y, so.h - y) / so.h);
            fade *= smooth(margin / CONE_EDGE);
        }
        faces = faces * 3 + face;
        const double co = dot(d, nj), mirrored = fresnel(co, n, 1.0);  // 1 when the glass lets nothing out
        if (b < way) {
            weight *= mirrored;
            d = d - nj * (2.0 * co);
        } else {
            weight *= 1.0 - mirrored;
            // Where the glass lets nothing out, the corner still needs a place (its light is 0, but the sheet must
            // not tear there): it is sent out as steeply as the last light that does get out.
            const double s2 = n * n * (1.0 - co * co), most = 0.98;
            if (s2 < most) {
                if (!refract(d, nj * -1.0, n, D)) return hit;
            } else {
                V3 along = d - nj * co;
                const double al = length(along);
                if (!(al > 1e-12)) return hit;
                D = along * (std::sqrt(most) / al) + nj * std::sqrt(1.0 - most);
            }
        }
    }
    hit.faces = faces;
    hit.ok = true;
    hit.fade = fade;
    const double t = D.y < -1e-6 ? std::max(0.0, (floorY - p.y) / D.y) : 1e30, far = std::min(t, BALL_REACH * size);
    hit.q = V3{p.x + D.x * far, far, p.z + D.z * far};
    hit.weight = weight * (D.y < -BALL_FLAT ? ball_fade(t / size, -D.y) : 0.0);
    return hit;
}
// How bright a corner is, as a multiple of the sun: the triangles that meet at it and belong to the same piece of
// the sheet, the sunbeam they catch over the floor they land on; times how much of the light takes this way; times
// the least `fade` of the corner and its neighbours. P: where the corners are on the solid, hit: what became of
// them; at: cone_around.
inline double cone_bright(V3 L, V3 Pv, const ConeHit& hv, const int at[6][2], const V3* Pk, const ConeHit* hk) {
    if (!hv.ok || !(hv.weight > 0)) return 0.0;
    double before = 0, after = 0, fade = hv.fade;
    for (int k = 0; k < 6; ++k) {
        if (at[k][0] < 0) continue;
        const bool same = hk[k].ok && hk[k].faces == hv.faces;
        fade = std::min(fade, same ? hk[k].fade : 0.0);
        const int m = (k + 1) % 6;
        if (at[m][0] < 0 || !same || !hk[m].ok || hk[m].faces != hv.faces) continue;
        before += std::fabs(dot(cross(Pk[k] - Pv, Pk[m] - Pv), L));
        after += std::fabs((hk[k].q.x - hv.q.x) * (hk[m].q.z - hv.q.z) - (hk[k].q.z - hv.q.z) * (hk[m].q.x - hv.q.x));
    }
    if (!(before > 0)) return 0.0;
    return hv.weight * fade * (after * CONE_CAP > before ? before / after : CONE_CAP);
}
// The light of a cone or a rod as triangles, worked out on the CPU exactly as cone_walk.mat + cone_light.mat draw
// it (for a renderer that does not, and for the desk check).
inline void cone_light(const Solid& solid, const Params& par, std::vector<float>& tri, int* patches = nullptr) {
    struct Mesh {
        std::vector<float> corners;
        std::vector<uint32_t> idx;
    };
    static const Mesh mesh = [] {
        Mesh m;
        cone_mesh(m.corners, m.idx);
        return m;
    }();
    const Taper so = taper_of(solid);
    const V3 L = normalize(par.sun);
    const ConeFrame f = cone_frame(so, par.sun);
    const size_t nv = mesh.corners.size() / 3, nt = mesh.idx.size() / 3;
    const std::vector<BallStep> steps = ball_steps(par, par.coneSteps);
    const size_t base = tri.size();
    tri.resize(base + steps.size() * (size_t)CONE_WAYS * nt * 3 * (size_t)TRI);
    float* o = tri.data() + base;
    std::vector<V3> P(nv), N(nv);
    std::vector<ConeHit> hit(nv);
    std::vector<double> e(nv), fly(nv);
    for (size_t si = 0; si < steps.size(); ++si) {
        const BallStep& st = steps[si];
        const double nNext = steps[si + 1 < steps.size() ? si + 1 : (si > 0 ? si - 1 : si)].n;  // the neighbouring colour
        for (size_t v = 0; v < nv; ++v) cone_entry(so, f, par.sun, st.n, (int)mesh.corners[v * 3 + 2], (int)mesh.corners[v * 3], (int)mesh.corners[v * 3 + 1], P[v], N[v]);
        for (int way = 0; way < CONE_WAYS; ++way) {
            for (size_t v = 0; v < nv; ++v) {
                const int patch = (int)mesh.corners[v * 3 + 2];
                hit[v] = cone_walk(so, par.sun, P[v], N[v], patch, st.n, way, par.floorY);
                // soften by half the gap to where the next colour lands, on top of what the distance flown asks for
                fly[v] = hit[v].q.y;
                if (hit[v].ok && hit[v].weight > 0) {
                    V3 Pn, Nn;
                    cone_entry(so, f, par.sun, nNext, patch, (int)mesh.corners[v * 3], (int)mesh.corners[v * 3 + 1], Pn, Nn);
                    const ConeHit next = cone_walk(so, par.sun, Pn, Nn, patch, nNext, way, par.floorY);
                    if (next.ok && next.weight > 0) fly[v] += 0.5 * std::sqrt((next.q.x - hit[v].q.x) * (next.q.x - hit[v].q.x) + (next.q.z - hit[v].q.z) * (next.q.z - hit[v].q.z)) / par.sunTan;
                }
            }
            for (size_t v = 0; v < nv; ++v) {
                const int patch = (int)mesh.corners[v * 3 + 2];
                int at[6][2];
                cone_around(patch, (int)mesh.corners[v * 3], (int)mesh.corners[v * 3 + 1], at);
                V3 Pk[6];
                ConeHit hk[6];
                for (int k = 0; k < 6; ++k) {
                    if (at[k][0] < 0) continue;
                    const size_t u = (size_t)cone_corner_index(patch, at[k][0], at[k][1]);
                    Pk[k] = P[u];
                    hk[k] = hit[u];
                }
                e[v] = cone_bright(L, P[v], hit[v], at, Pk, hk);
            }
            for (size_t t = 0; t < nt; ++t) {
                const uint32_t v[3] = {mesh.idx[t * 3], mesh.idx[t * 3 + 1], mesh.idx[t * 3 + 2]};
                if (!hit[v[0]].ok || !hit[v[1]].ok || !hit[v[2]].ok || hit[v[0]].faces != hit[v[1]].faces || hit[v[1]].faces != hit[v[2]].faces) continue;
                if (e[v[0]] + e[v[1]] + e[v[2]] <= 0) continue;
                for (int k = 0; k < 3; ++k) {
                    *o++ = (float)hit[v[k]].q.x;
                    *o++ = (float)hit[v[k]].q.z;
                    *o++ = (float)(st.rgb.x * e[v[k]]);
                    *o++ = (float)(st.rgb.y * e[v[k]]);
                    *o++ = (float)(st.rgb.z * e[v[k]]);
                    *o++ = (float)fly[v[k]];
                }
                if (patches) ++*patches;
            }
        }
    }
    tri.resize((size_t)(o - tri.data()));
}

// pieces: 1 = every lit face is one piece; k = each face is cut into a k x k grid of pieces.
// balls: if given, it comes back as the list of all balls and cones, and they get no triangles here: the renderer
// draws them from their shared shapes (see "A ball by likeness", "A cone by formula"). If not given, their light
// is worked out here as triangles, exactly as those shapes would be drawn.
// leave: which kinds those are (one bit per Solid::Kind). Flat-faced solids can be left to the renderer too (see
// "Flat faces on the GPU"); then this function has nothing to do for them.
inline void trace_cheap(const Scene& sc, const Params& par, const std::vector<double>& detailPerSolid, int pieces, std::vector<float>& tri, CheapStats* stats = nullptr,
                        std::vector<int>* balls = nullptr, unsigned leave = (1u << Solid::BALL) | (1u << Solid::CONE) | (1u << Solid::ROD)) {
    tri.clear();
    if (balls) balls->clear();
    if (!balls) leave = 0;
    const int N = par.bands;
    const std::vector<Band> bands = make_bands(N, par.nmMin, par.nmMax);
    std::vector<double> nb((size_t)N);
    for (int i = 0; i < N; ++i) nb[(size_t)i] = par.glass.n(bands[(size_t)i].nm);
    const V3 L = normalize(par.sun);
    const size_t ns = sc.solids.size();
    std::vector<std::vector<float>> outTri(ns);
    std::vector<CheapStats> outSt(ns);
    BallShape shape;  // the one shape of every ball's light, if a ball's triangles are wanted here
    if (!((leave >> Solid::BALL) & 1u))
        for (const Solid& so : sc.solids)
            if (so.kind == Solid::BALL) {
                shape = ball_shape(par);
                break;
            }
    parallel_for(ns, par.threads, [&](size_t A) {
        std::vector<float>& tri = outTri[A];
        CheapStats& st = outSt[A];
        const Solid& so = sc.solids[A];
        const double detail_ = detailPerSolid.empty() ? 1e-3 : detailPerSolid[std::min(A, detailPerSolid.size() - 1)];
        if ((leave >> so.kind) & 1u) {
            st.alike = 1;
            return;
        }
        if (so.kind == Solid::BALL) {
            ball_light(shape, so.c, so.r, par.floorY, tri, &st.patches);
            return;
        }
        if (so.kind == Solid::CONE || so.kind == Solid::ROD) {
            cone_light(so, par, tri, &st.patches);
            return;
        }
        for (int f = 0; f < (int)so.planes.size(); ++f) {
            const Poly& face = so.faces[(size_t)f];
            if (face.n < 3 || dot(so.planes[(size_t)f].n, L) > -1e-9) continue;
            detail::flat_light(sc, par, (int)A, f, face, so.planes[(size_t)f].n, pieces, detail_, bands, nb, tri, st);
        }
    });
    CheapStats st;
    size_t total = 0;
    for (const auto& t : outTri) total += t.size();
    tri.reserve(total);
    for (size_t i = 0; i < ns; ++i) {
        tri.insert(tri.end(), outTri[i].begin(), outTri[i].end());
        st.pieces += outSt[i].pieces;
        st.patches += outSt[i].patches;
        st.rays += outSt[i].rays;
        st.alike += outSt[i].alike;
        if (balls && outSt[i].alike) balls->push_back((int)i);
    }
    if (stats) *stats = st;
}

// ---- Flat faces on the GPU -----------------------------------------------------------------------------------------
// The light of flat-faced solids as trace_cheap works it out (each lit face cut into pieces, one ray from the middle
// of a piece decides where the whole piece goes), but with nothing done on the CPU per frame: what is built here,
// once per shape, is a table of the pieces in the shape's own space and a mesh; per frame the renderer is told
// where each solid stands, and flat_walk.mat / flat_light.mat do the rest.
// A piece goes where its middle ray goes by one affine map (flat faces bend all light the same way), so three of
// its corners are enough: flat_walk.mat lands those three, flat_light.mat places every other corner between them.
// Colour: a piece is drawn once per colour step, not just where that colour lands but smeared half way to where
// the next and the previous colour land (every pixel gets as much of it as the share of that stretch for which
// the piece, sliding along it, covers the pixel). Neighbouring steps then cross-fade exactly, so the colours of a
// line of light join up without gaps however far the glass spreads them, and edges that the colours do not move
// across stay as sharp as the piece's own. For that the mesh has, besides each piece itself, a strip along every
// edge of it that the vertex shader pulls outwards to cover the smear.
struct FlatShape {
    int pieces = 0;
    std::vector<float> table;    // 5 RGBA texels per piece: its middle; the face's outward normal; three corners (c0, c1, c2)
    std::vector<float> hull;     // 5 RGBA texels per piece: its corners (a, b) two to a texel, up to 8 (the last one repeated),
                                 // counter-clockwise in (a, b); then its middle (a, b), 1 if the piece is the unit square in
                                 // (a, b) (a parallelogram: most are), how many corners
    std::vector<float> corners;  // the mesh, 5 numbers per corner: piece (+ 0.5: the outer side of an edge strip), a, b with
                                 // corner = c0 + a (c1 - c0) + b (c2 - c0), and for strips the edge's outward direction in (a, b)
    std::vector<uint32_t> idx;   // three per triangle
};
inline FlatShape flat_shape(const Solid& local, int piecesPerSide) {
    FlatShape sh;
    std::vector<Poly> list;
    for (size_t f = 0; f < local.planes.size(); ++f) {
        const V3 nf = local.planes[f].n;
        detail::face_pieces(local.faces[f], nf, piecesPerSide, list);
        for (const Poly& P0 : list) {
            Poly P = P0;
            while (P.n > 8) {  // (does not happen with the shapes at hand: drop the corner that matters least)
                int drop = 0;
                double least = 1e300;
                for (int i = 0; i < P.n; ++i) {
                    const double a2 = length(cross(P.p[(size_t)((i + 1) % P.n)] - P.p[(size_t)i], P.p[(size_t)((i + P.n - 1) % P.n)] - P.p[(size_t)i]));
                    if (a2 < least) {
                        least = a2;
                        drop = i;
                    }
                }
                for (int i = drop; i + 1 < P.n; ++i) P.p[(size_t)i] = P.p[(size_t)i + 1];
                --P.n;
            }
            V3 mid{0, 0, 0};
            for (int i = 0; i < P.n; ++i) mid = mid + P.p[(size_t)i];
            mid = mid * (1.0 / P.n);
            // three corners to carry: for a parallelogram the first and its two neighbours (then the piece is the unit
            // square in (a, b)); otherwise the first and the two that make the largest triangle with it
            int ib = 1, ic = 2;
            const bool square = P.n == 4 && length(P.p[0] + P.p[2] - P.p[1] - P.p[3]) < 1e-9 * (length(P.p[2] - P.p[0]) + 1e-30);
            if (square) {
                ib = 1;
                ic = 3;
            } else {
                double best = -1;
                for (int b = 1; b < P.n; ++b)
                    for (int c = b + 1; c < P.n; ++c) {
                        const double a2 = length(cross(P.p[(size_t)b] - P.p[0], P.p[(size_t)c] - P.p[0]));
                        if (a2 > best) {
                            best = a2;
                            ib = b;
                            ic = c;
                        }
                    }
            }
            const V3 c0 = P.p[0], e1 = P.p[(size_t)ib] - c0, e2 = P.p[(size_t)ic] - c0;
            for (V3 v : {mid, nf, c0, P.p[(size_t)ib], P.p[(size_t)ic]}) sh.table.insert(sh.table.end(), {(float)v.x, (float)v.y, (float)v.z, 0.f});
            // every corner between those three
            const double g11 = dot(e1, e1), g12 = dot(e1, e2), g22 = dot(e2, e2), det = g11 * g22 - g12 * g12;
            double a[8], b[8], am = 0, bm = 0, areaAB = 0;
            for (int i = 0; i < P.n; ++i) {
                const V3 w = P.p[(size_t)i] - c0;
                const double w1 = dot(w, e1), w2 = dot(w, e2);
                a[i] = (w1 * g22 - w2 * g12) / det;
                b[i] = (w2 * g11 - w1 * g12) / det;
                am += a[i] / P.n;
                bm += b[i] / P.n;
            }
            for (int i = 0; i < P.n; ++i) {
                const int k = (i + 1) % P.n;
                areaAB += 0.5 * (a[i] * b[k] - a[k] * b[i]);
            }
            if (areaAB < 0)  // counter-clockwise in (a, b)
                for (int i = 0; i < P.n / 2; ++i) {
                    std::swap(a[i], a[P.n - 1 - i]);
                    std::swap(b[i], b[P.n - 1 - i]);
                }
            for (int i = 0; i < 8; ++i) {
                const int k = std::min(i, P.n - 1);
                sh.hull.push_back((float)a[k]);
                sh.hull.push_back((float)b[k]);
            }
            sh.hull.insert(sh.hull.end(), {(float)am, (float)bm, square ? 1.f : 0.f, (float)P.n});
            // the piece itself
            const uint32_t base = (uint32_t)(sh.corners.size() / 5);
            for (int i = 0; i < P.n; ++i) sh.corners.insert(sh.corners.end(), {(float)sh.pieces, (float)a[i], (float)b[i], 0.f, 0.f});
            for (int i = 1; i + 1 < P.n; ++i) sh.idx.insert(sh.idx.end(), {base, base + (uint32_t)i, base + (uint32_t)i + 1});
            // a strip along every edge: two corners on the edge, two that are pulled outwards
            for (int i = 0; i < P.n; ++i) {
                const int k = (i + 1) % P.n;
                double ox = b[k] - b[i], oy = -(a[k] - a[i]);  // across the edge...
                if (ox * (0.5 * (a[i] + a[k]) - am) + oy * (0.5 * (b[i] + b[k]) - bm) < 0) {  // ...away from the middle
                    ox = -ox;
                    oy = -oy;
                }
                const uint32_t q = (uint32_t)(sh.corners.size() / 5);
                sh.corners.insert(sh.corners.end(), {(float)sh.pieces, (float)a[i], (float)b[i], (float)ox, (float)oy});
                sh.corners.insert(sh.corners.end(), {(float)sh.pieces, (float)a[k], (float)b[k], (float)ox, (float)oy});
                sh.corners.insert(sh.corners.end(), {(float)sh.pieces + 0.5f, (float)a[k], (float)b[k], (float)ox, (float)oy});
                sh.corners.insert(sh.corners.end(), {(float)sh.pieces + 0.5f, (float)a[i], (float)b[i], (float)ox, (float)oy});
                sh.idx.insert(sh.idx.end(), {q, q + 1, q + 2, q, q + 2, q + 3});
            }
            ++sh.pieces;
        }
    }
    return sh;
}

// A solid's shadow as if it were opaque: the lit faces slid along the sun onto the floor. They tile the outline
// exactly (the solid is convex). rgb is the direct sunlight the floor loses there, so a renderer can subtract it.
inline Result silhouette(const Solid& solid, V3 sun, double floorY) {
    Result res;
    const V3 L = normalize(sun);
    if (L.y > -1e-9) return res;
    for (size_t f = 0; f < solid.planes.size(); ++f) {
        const Poly& P = solid.faces[f];
        if (P.n < 3 || dot(solid.planes[f].n, L) > -1e-9) continue;
        Poly F;
        detail::to_floor(P, L, floorY, F);
        Patch p;
        p.first = (int)(res.xz.size() / 2);
        p.count = F.n;
        p.rgb = V3{1, 1, 1} * (-L.y);
        p.band = -1;
        for (int i = 0; i < F.n; ++i) {
            res.xz.push_back(F.p[(size_t)i].x);
            res.xz.push_back(F.p[(size_t)i].z);
        }
        res.patches.push_back(p);
    }
    return res;
}

// The shadow outline as triangles (x, z, 1, 1, 1, travel), appended to `tri`. A renderer that wants the shadow
// to soften exactly like the light draws these into a coverage buffer instead of using a shadow map. Outlines of
// several solids may overlap; the renderer must not count the sun as cut off more than once.
inline void silhouette_triangles(const Solid& solid, V3 sun, double floorY, std::vector<float>& tri) {
    const V3 L = normalize(sun);
    if (L.y > -1e-9) return;
    if (solid.kind == Solid::BALL) {
        // a ball's outline is its great circle across the sunbeam, slid along the sun onto the floor: one fan
        V3 w, e1, e2;
        sun_frame(L, w, e1, e2);
        auto put = [&](V3 v) {
            const double tt = std::max(0.0, (floorY - v.y) / L.y);
            const V3 q = v + L * tt;
            tri.insert(tri.end(), {(float)q.x, (float)q.z, 1.f, 1.f, 1.f, (float)tt});
        };
        auto rim = [&](int j) {
            const double ph = 2.0 * 3.14159265358979323846 * j / BALL_SECTORS;
            return solid.c + (e1 * std::cos(ph) + e2 * std::sin(ph)) * solid.r;
        };
        for (int j = 0; j < BALL_SECTORS; ++j) {
            put(solid.c);
            put(rim(j));
            put(rim(j + 1));
        }
        return;
    }
    if (solid.kind == Solid::CONE) {
        // the sunlit part of the side (between the two lines where the sun grazes it, found exactly) and the base
        // if that faces the sun: together they tile the outline once
        const ConeFrame f = cone_frame(taper_of(solid), L);
        const double pi = 3.14159265358979323846;
        auto put = [&](V3 v) {
            const double tt = std::max(0.0, (floorY - v.y) / L.y);
            const V3 q = v + L * tt;
            tri.insert(tri.end(), {(float)q.x, (float)q.z, 1.f, 1.f, 1.f, (float)tt});
        };
        const V3 tip = solid.c + solid.ax * solid.h;
        const int parts = 48;
        if (f.half > 0)
            for (int j = 0; j < parts; ++j) {
                const double a = pi + (2.0 * j / parts - 1.0) * f.half, b = pi + (2.0 * (j + 1) / parts - 1.0) * f.half;
                put(tip);
                put(solid.c + (f.e1 * std::cos(a) + f.e2 * std::sin(a)) * solid.r);
                put(solid.c + (f.e1 * std::cos(b) + f.e2 * std::sin(b)) * solid.r);
            }
        if (dot(solid.ax, L) > 0)
            for (int j = 0; j < parts; ++j) {
                const double a = 2.0 * pi * j / parts, b = 2.0 * pi * (j + 1) / parts;
                put(solid.c);
                put(solid.c + (f.e1 * std::cos(a) + f.e2 * std::sin(a)) * solid.r);
                put(solid.c + (f.e1 * std::cos(b) + f.e2 * std::sin(b)) * solid.r);
            }
        return;
    }
    if (solid.kind != Solid::FLAT) {
        // the sunlit side, slid along the sun onto the floor
        LitMesh m;
        lit_mesh(solid, L, m);
        for (size_t t = 0; t + 2 < m.tri.size(); t += 3)
            for (int k = 0; k < 3; ++k) {
                const V3 v = m.p[(size_t)m.tri[t + (size_t)k]];
                const double tt = std::max(0.0, (floorY - v.y) / L.y);
                const V3 q = v + L * tt;
                tri.push_back((float)q.x);
                tri.push_back((float)q.z);
                tri.push_back(1.f);
                tri.push_back(1.f);
                tri.push_back(1.f);
                tri.push_back((float)tt);
            }
        return;
    }
    for (size_t f = 0; f < solid.planes.size(); ++f) {
        const Poly& P = solid.faces[f];
        if (P.n < 3 || dot(solid.planes[f].n, L) > -1e-9) continue;
        Poly F;
        detail::to_floor(P, L, floorY, F);
        for (int i = 1; i + 1 < F.n; ++i) {
            const int idx[3] = {0, i, i + 1};
            for (int k = 0; k < 3; ++k) {
                tri.push_back((float)F.p[(size_t)idx[k]].x);
                tri.push_back((float)F.p[(size_t)idx[k]].z);
                tri.push_back(1.f);
                tri.push_back(1.f);
                tri.push_back(1.f);
                tri.push_back((float)F.p[(size_t)idx[k]].y);
            }
        }
    }
}
inline void silhouette_triangles(const Scene& sc, V3 sun, double floorY, std::vector<float>& tri) {
    for (const Solid& s : sc.solids) silhouette_triangles(s, sun, floorY, tri);
}

// Triangles for a renderer from the reference trace: x, z, r, g, b, travel per corner (fan per patch).
inline void to_triangles(const Result& r, std::vector<float>& out) {
    out.clear();
    for (const Patch& p : r.patches)
        for (int i = 1; i + 1 < p.count; ++i) {
            const int idx[3] = {p.first, p.first + i, p.first + i + 1};
            for (int k = 0; k < 3; ++k) {
                out.push_back((float)r.xz[(size_t)idx[k] * 2]);
                out.push_back((float)r.xz[(size_t)idx[k] * 2 + 1]);
                out.push_back((float)p.rgb.x);
                out.push_back((float)p.rgb.y);
                out.push_back((float)p.rgb.z);
                out.push_back(r.travel.empty() ? 0.f : (float)r.travel[(size_t)idx[k]]);
            }
        }
}

}  // namespace beam
