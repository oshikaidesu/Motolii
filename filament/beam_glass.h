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
#ifdef __APPLE__
#include <pthread.h>
#endif
#include <atomic>
#include <cmath>
#include <condition_variable>
#include <cstdint>
#include <cstring>
#include <memory>
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

struct Outline;  // beam_extrude.h
struct Solid {
    // kind FLAT: a convex solid with flat faces
    std::vector<Plane> planes;
    std::vector<Poly> faces;  // faces[i] lies on planes[i]; may be empty (n == 0) if the plane cuts nothing
    // Colour films inside the glass (kind FLAT only, at most 4): a film passes some colours straight on and mirrors
    // the rest, as the crossed films of a colour-splitting cube do. Which colours a film mirrors goes by its place
    // in this list (glass_scene.mm, FILMBANDS). Known to the light on the GPU (flat_walk.mat) and to the body
    // (glass_solid.mat).
    std::vector<Plane> films;
    // round kinds (only the cheap trace and the renderer know them; the exact beam trace is for flat faces)
    enum Kind { FLAT = 0, BALL = 1, ROD = 2, CONE = 3, EXTRUDE = 4 };
    int kind = FLAT;
    V3 c{0, 0, 0};    // BALL: centre. ROD: middle of the axis. CONE: middle of the base
    V3 ax{0, 1, 0};   // ROD: unit axis. CONE: unit axis, from the base to the tip
    double r = 0;     // BALL, ROD: radius. CONE: radius of the base
    double h = 0;     // ROD: half length. CONE: height
    // kind EXTRUDE (beam_extrude.h): an outline with dents and holes pushed through a thickness. c: where the
    // outline's own origin is, ax: the direction of the thickness, h: half the thickness.
    V3 ux{1, 0, 0};   // where the outline's x axis points (its y axis is ax x ux)
    double sc = 1;    // how much larger than the outline as drawn
    std::shared_ptr<const Outline> shape;
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
    s.ux = R[0] * in.ux.x + R[1] * in.ux.y + R[2] * in.ux.z;
    s.sc = in.sc * scale;
    s.shape = in.shape;
    for (const Plane& pl : in.planes) {
        V3 n = R[0] * pl.n.x + R[1] * pl.n.y + R[2] * pl.n.z;
        s.planes.push_back({n, pl.o * scale + dot(n, at)});
    }
    for (const Plane& pl : in.films) {
        V3 n = R[0] * pl.n.x + R[1] * pl.n.y + R[2] * pl.n.z;
        s.films.push_back({n, pl.o * scale + dot(n, at)});
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

inline void extrude_bound(const Solid& so, V3& centre, double& radius);  // beam_extrude.h
// Several solids. They must not overlap; they may touch or rest on each other (leave a hair of air between).
struct Scene {
    std::vector<Solid> solids;
    std::vector<V3> centre;      // bounding spheres, filled by bound()
    std::vector<double> radius;
    void bound() {
        centre.assign(solids.size(), V3{0, 0, 0});
        radius.assign(solids.size(), 0.0);
        for (size_t s = 0; s < solids.size(); ++s) {
            if (solids[s].kind == Solid::EXTRUDE) {
                extrude_bound(solids[s], centre[s], radius[s]);
                continue;
            }
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
}  // namespace beam

#include "beam_glass_2.h"
#include "beam_glass_3.h"
#include "beam_glass_4.h"
#include "beam_glass_5.h"
#include "beam_glass_6.h"
#include "beam_glass_7.h"
#include "beam_glass_8.h"
