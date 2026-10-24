// beam_extrude.h, part 2 (cut there by size, as it was): read only through beam_extrude.h, in order.
#pragma once

namespace beam {

// ---- shapes -------------------------------------------------------------------------------------------------------
// Any finished outline as a solid: the outline in x, y, the thickness along z, centred.
inline Solid extruded(Outline ol) {
    Solid s;
    s.kind = Solid::EXTRUDE;
    s.c = {0, 0, 0};
    s.ax = {0, 0, 1};
    s.ux = {1, 0, 0};
    s.sc = 1;
    s.h = ol.h;
    s.r = ol.reach;
    s.shape = std::make_shared<const Outline>(std::move(ol));
    return s;
}
// A star with `points` tips at `outer` from the middle and the corners between them at `inner`; one tip points up (+y).
inline Solid star(int points, double outer, double inner, double halfThickness, double chamfer) {
    const double pi = 3.14159265358979323846;
    Outline ol;
    ol.h = halfThickness;
    ol.chamfer = chamfer;
    for (int i = 0; i < 2 * points; ++i) {
        const double a = 0.5 * pi + pi * i / points, rad = (i & 1) ? inner : outer;
        if (i == 0) ol.move(rad * std::cos(a), rad * std::sin(a));
        else ol.line(rad * std::cos(a), rad * std::sin(a));
    }
    ol.finish();
    return extruded(std::move(ol));
}
// A flat ring: a disc with a round hole.
inline Solid washer(double outer, double inner, double halfThickness, double chamfer) {
    Outline ol;
    ol.h = halfThickness;
    ol.chamfer = chamfer;
    ol.move(outer, 0);
    ol.arc(outer, 0, 0, 0, true);
    ol.move(inner, 0);
    ol.arc(inner, 0, 0, 0, false);
    ol.finish();
    return extruded(std::move(ol));
}
// A few capital letters, drawn here by hand from straight lines and arcs of circles (no font file): A B M O R.
// About 1 high. Returns an empty solid for any other letter.
inline Solid letter(char which, double halfThickness, double chamfer) {
    Outline ol;
    ol.h = halfThickness;
    ol.chamfer = chamfer;
    if (which == 'A') {
        // two legs that lean together under a flat top, a bar across, a three-cornered hole above the bar
        const double lean = 0.345, foot = 0.21, bar0 = 0.24, bar1 = 0.42, W = 0.90;
        ol.move(0, 0);
        ol.line(foot, 0);
        ol.line(foot + lean * bar0, bar0);
        ol.line(W - foot - lean * bar0, bar0);
        ol.line(W - foot, 0);
        ol.line(W, 0);
        ol.line(W - lean, 1);
        ol.line(lean, 1);
        const double top = (W - 2.0 * foot) / (2.0 * lean);  // where the insides of the legs meet
        ol.move(foot + lean * bar1, bar1);
        ol.line(0.5 * W, top);
        ol.line(W - foot - lean * bar1, bar1);
    } else if (which == 'M') {
        // two posts and a V between them, cut out above and below
        ol.move(0, 0);
        ol.line(0.2, 0);
        ol.line(0.2, 0.63);
        ol.line(0.5, 0.05);
        ol.line(0.8, 0.63);
        ol.line(0.8, 0);
        ol.line(1.0, 0);
        ol.line(1.0, 1);
        ol.line(0.8, 1);
        ol.line(0.5, 0.42);
        ol.line(0.2, 1);
        ol.line(0, 1);
    } else if (which == 'O') {
        // two half circles joined by short straight sides, and the same again inside
        ol.move(0.8, 0.4);
        ol.line(0.8, 0.6);
        ol.arc(0.0, 0.6, 0.4, 0.6, true);
        ol.line(0.0, 0.4);
        ol.arc(0.8, 0.4, 0.4, 0.4, true);
        ol.move(0.6, 0.6);
        ol.line(0.6, 0.4);
        ol.arc(0.2, 0.4, 0.4, 0.4, false);
        ol.line(0.2, 0.6);
        ol.arc(0.6, 0.6, 0.4, 0.6, false);
    } else if (which == 'B') {
        // a post with two bowls; the bowls are half circles that run into each other at the waist
        const double x1 = 0.40, y1 = 0.30, r1 = 0.30;  // lower bowl
        const double x2 = 0.36, y2 = 0.72, r2 = 0.28;  // upper bowl
        // where the two circles cross on the right
        const double dx = x2 - x1, dy = y2 - y1, d = std::sqrt(dx * dx + dy * dy);
        const double along = (d * d + r1 * r1 - r2 * r2) / (2.0 * d), off = std::sqrt(std::max(0.0, r1 * r1 - along * along));
        const double wx = x1 + dx * along / d + dy * off / d, wy = y1 + dy * along / d - dx * off / d;
        ol.move(0, 0);
        ol.line(x1, 0);
        ol.arc(wx, wy, x1, y1, true);
        ol.arc(x2, y2 + r2, x2, y2, true);
        ol.line(0, 1);
        const double in1 = 0.14, in2 = 0.12;
        ol.move(0.2, y1 - in1);
        ol.line(0.2, y1 + in1);
        ol.line(x1, y1 + in1);
        ol.arc(x1, y1 - in1, x1, y1, false);
        ol.move(0.2, y2 - in2);
        ol.line(0.2, y2 + in2);
        ol.line(x2, y2 + in2);
        ol.arc(x2, y2 - in2, x2, y2, false);
    } else if (which == 'R') {
        // a post, one bowl, and a leg that runs from under the bowl to the foot
        const double xb = 0.40, yb = 0.71, rb = 0.29, in = 0.13;
        const V2 f0{0.72, 0.0}, f1{0.52, 0.42};  // the outside of the leg, from the foot upwards
        // where that line meets the bowl's circle
        const V2 dir = f1 - f0, oc = f0 - V2{xb, yb};
        const double A = dot(dir, dir), B = dot(oc, dir), C = dot(oc, oc) - rb * rb, t = (-B - std::sqrt(std::max(0.0, B * B - A * C))) / A;
        const V2 meet = f0 + dir * t;
        ol.move(0, 0);
        ol.line(0.2, 0);
        ol.line(0.2, 0.42);
        ol.line(0.30, 0.42);
        ol.line(0.50, 0);
        ol.line(f0.x, f0.y);
        ol.line(meet.x, meet.y);
        ol.arc(xb, yb + rb, xb, yb, true);
        ol.line(0, 1);
        ol.move(0.2, yb - in);
        ol.line(0.2, yb + in);
        ol.line(xb, yb + in);
        ol.arc(xb, yb - in, xb, yb, false);
    } else {
        return Solid{};
    }
    ol.finish();
    return extruded(std::move(ol));
}

// ---- rays against the outline ---------------------------------------------------------------------------------------
struct OutlineHit {
    double t = 1e300;
    int edge = -1;
    V2 n{0, 0};  // out of the glass (unit)
};
// The nearest place beyond tmin where o + t d crosses the outline. want > 0: only where it leaves the glass,
// < 0: only where it goes in, 0: either.
inline bool outline_cast(const Outline& ol, V2 o, V2 d, double tmin, int want, OutlineHit& hit) {
    double best = 1e300;
    int be = -1;
    V2 bn{0, 0};
    bool straight = false;
    const size_t ne = ol.edges.size();
    for (size_t i = 0; i < ne; ++i) {
        const OutlineEdge& e = ol.edges[i];
        if (e.r == 0) {
            const V2 ab = e.b - e.a;
            const double den = cross(d, ab);  // d . (outward normal) times the length of the edge
            if (want > 0 ? den <= 0 : (want < 0 ? den >= 0 : den == 0)) continue;
            const V2 ao = e.a - o;
            const double t = cross(ao, ab) / den;
            if (!(t > tmin) || t >= best) continue;
            const double w = cross(ao, d) / den;
            if (w < -1e-5 || w > 1.0 + 1e-5) continue;
            best = t;
            be = (int)i;
            bn = V2{ab.y, -ab.x};
            straight = true;
        } else {
            const V2 oc = o - e.c;
            const double A = dot(d, d), B = dot(oc, d), C = dot(oc, oc) - e.r * e.r, disc = B * B - A * C;
            if (!(disc > 0)) continue;
            const double sq = std::sqrt(disc);
            const V2 ab = e.b - e.a;
            for (int k = 0; k < 2; ++k) {
                const double t = (k ? -B + sq : -B - sq) / A;
                if (!(t > tmin) || t >= best) continue;
                const V2 p = o + d * t;
                if (cross(ab, p - e.a) * e.r > 1e-5 * e.r * e.r) continue;  // on the circle, but not on this arc
                const V2 n = (p - e.c) * (1.0 / e.r);
                const double dn = dot(d, n);
                if (want > 0 ? dn <= 0 : (want < 0 ? dn >= 0 : false)) continue;
                best = t;
                be = (int)i;
                bn = n;
                straight = false;
            }
        }
    }
    if (be < 0) return false;
    hit.t = best;
    hit.edge = be;
    hit.n = straight ? bn * (1.0 / length(bn)) : bn;
    return true;
}
// For a ray from outside: how many times it crosses the outline beyond `from` (odd: it is inside the outline
// there), and the first place between `from` and `to` where it goes into the glass.
inline void outline_scan(const Outline& ol, V2 o, V2 d, double from, double to, int& crossings, OutlineHit& hit) {
    crossings = 0;
    hit.t = 1e300;
    hit.edge = -1;
    for (size_t i = 0; i < ol.edges.size(); ++i) {
        const OutlineEdge& e = ol.edges[i];
        if (e.r == 0) {
            const V2 ab = e.b - e.a;
            const double den = cross(d, ab);
            if (den == 0) continue;
            const V2 ao = e.a - o;
            const double t = cross(ao, ab) / den;
            if (!(t > from)) continue;
            const double w = cross(ao, d) / den;
            if (w < 0 || w >= 1.0) continue;  // (half open: a corner counts for one of its two edges)
            ++crossings;
            if (den < 0 && t < to && t < hit.t) {
                hit.t = t;
                hit.edge = (int)i;
                hit.n = V2{ab.y, -ab.x} * (1.0 / length(ab));
            }
        } else {
            const V2 oc = o - e.c;
            const double A = dot(d, d), B = dot(oc, d), C = dot(oc, oc) - e.r * e.r, disc = B * B - A * C;
            if (!(disc > 0)) continue;
            const double sq = std::sqrt(disc);
            const V2 ab = e.b - e.a;
            for (int k = 0; k < 2; ++k) {
                const double t = (k ? -B + sq : -B - sq) / A;
                if (!(t > from)) continue;
                const V2 p = o + d * t;
                if (cross(ab, p - e.a) * e.r > 0) continue;
                ++crossings;
                const V2 n = (p - e.c) * (1.0 / e.r);
                if (dot(d, n) < 0 && t < to && t < hit.t) {
                    hit.t = t;
                    hit.edge = (int)i;
                    hit.n = n;
                }
            }
        }
    }
}

// ---- for the rays of the other solids, and for the desk check: one ray against the whole solid ----------------------
// (the solid with sharp rims: the chamfers are too small to matter to a ray from elsewhere)
// Faces: the walls by the number of their edge; then the cap at +z, then the cap at -z.
inline bool extrude_leave(const Solid& so, V3 p, V3 d, double& t, V3& n, int& face) {
    const Outline& ol = *so.shape;
    const V3 ey = cross(so.ax, so.ux), w = (p - so.c) * (1.0 / so.sc);
    const V3 lo{dot(w, so.ux), dot(w, ey), dot(w, so.ax)}, ld{dot(d, so.ux), dot(d, ey), dot(d, so.ax)};
    double tc = 1e300;
    if (ld.z > 1e-12) tc = (ol.h - lo.z) / ld.z;
    else if (ld.z < -1e-12) tc = (-ol.h - lo.z) / ld.z;
    OutlineHit hit;
    if (outline_cast(ol, V2{lo.x, lo.y}, V2{ld.x, ld.y}, EXT_TMIN, 1, hit) && hit.t < tc) {
        t = hit.t * so.sc;
        n = so.ux * hit.n.x + ey * hit.n.y;
        face = hit.edge;
        return true;
    }
    if (!(tc < 1e299) || !(tc > EXT_TMIN)) return false;
    t = tc * so.sc;
    n = ld.z > 0 ? so.ax : so.ax * -1.0;
    face = (int)ol.edges.size() + (ld.z > 0 ? 0 : 1);
    return true;
}
inline bool extrude_meet(const Solid& so, V3 o, V3 d, double& t, V3& n, int& face) {
    const Outline& ol = *so.shape;
    const V3 ey = cross(so.ax, so.ux), w = (o - so.c) * (1.0 / so.sc);
    const V3 lo{dot(w, so.ux), dot(w, ey), dot(w, so.ax)}, ld{dot(d, so.ux), dot(d, ey), dot(d, so.ax)};
    // while the ray is between the planes of the caps
    double t0 = -1e300, t1 = 1e300;
    if (std::fabs(ld.z) > 1e-12) {
        const double a = (-ol.h - lo.z) / ld.z, b = (ol.h - lo.z) / ld.z;
        t0 = std::min(a, b);
        t1 = std::max(a, b);
    } else if (std::fabs(lo.z) >= ol.h) {
        return false;
    }
    const double tmin = 1e-7;
    if (!(t1 > tmin)) return false;
    const double from = std::max(t0, tmin);
    int crossings;
    OutlineHit hit;
    outline_scan(ol, V2{lo.x, lo.y}, V2{ld.x, ld.y}, from, t1, crossings, hit);
    if (crossings & 1) {
        if (!(t0 > tmin)) return false;  // it starts inside the glass
        t = t0 * so.sc;
        n = ld.z > 0 ? so.ax * -1.0 : so.ax;
        face = (int)ol.edges.size() + (ld.z > 0 ? 1 : 0);
        return true;
    }
    if (hit.edge < 0) return false;
    t = hit.t * so.sc;
    n = so.ux * hit.n.x + ey * hit.n.y;
    face = hit.edge;
    return true;
}
inline void extrude_bound(const Solid& so, V3& centre, double& radius) {
    const Outline& ol = *so.shape;
    centre = so.c;
    radius = std::sqrt(ol.reach * ol.reach + ol.h * ol.h) * so.sc;
}

// ---- the mesh a renderer draws the body with (chamfers included), in the solid's own frame --------------------------
// Flat faces: every triangle has its own three corners. pos and nrm get three entries per triangle.
inline void extrude_mesh(const Outline& ol, std::vector<V3>& pos, std::vector<V3>& nrm) {
    pos.clear();
    nrm.clear();
    const double leg = ol.chamfer / std::sqrt(2.0), h = ol.h;
    auto tri = [&](V3 a, V3 b, V3 c) {
        V3 n = cross(b - a, c - a);
        const double l = length(n);
        if (!(l > 1e-18)) return;
        n = n * (1.0 / l);
        pos.insert(pos.end(), {a, b, c});
        nrm.insert(nrm.end(), {n, n, n});
    };
    for (size_t t = 0; t + 2 < ol.insetTris.size(); t += 3) {
        const V2 a = ol.inset[(size_t)ol.insetTris[t]], b = ol.inset[(size_t)ol.insetTris[t + 1]], c = ol.inset[(size_t)ol.insetTris[t + 2]];
        tri(V3{a.x, a.y, h}, V3{b.x, b.y, h}, V3{c.x, c.y, h});
        tri(V3{a.x, a.y, -h}, V3{c.x, c.y, -h}, V3{b.x, b.y, -h});
    }
    for (const std::vector<int>& loop : ol.loops)
        for (size_t i = 0; i < loop.size(); ++i) {
            const int ia = loop[i], ib = loop[(i + 1) % loop.size()];
            const V2 a = ol.pts[(size_t)ia], b = ol.pts[(size_t)ib], ai = ol.inset[(size_t)ia], bi = ol.inset[(size_t)ib];
            const double w = h - leg;
            // the wall (seen from outside the glass is on the left of a -> b, so a, b, up is counter-clockwise)
            tri(V3{a.x, a.y, -w}, V3{b.x, b.y, -w}, V3{b.x, b.y, w});
            tri(V3{a.x, a.y, -w}, V3{b.x, b.y, w}, V3{a.x, a.y, w});
            if (leg > 0) {
                tri(V3{a.x, a.y, w}, V3{b.x, b.y, w}, V3{bi.x, bi.y, h});
                tri(V3{a.x, a.y, w}, V3{bi.x, bi.y, h}, V3{ai.x, ai.y, h});
                tri(V3{b.x, b.y, -w}, V3{a.x, a.y, -w}, V3{ai.x, ai.y, -h});
                tri(V3{b.x, b.y, -w}, V3{ai.x, ai.y, -h}, V3{bi.x, bi.y, -h});
            }
        }
}


// ---- the light --------------------------------------------------------------------------------------------------------
// What a renderer keeps per shape, built once. Three lines of a float texture, EXT_TABLE wide:
//   line 0: texel 0 = (edges, stretches, triangles, 0), texel 1 = (half thickness, leg of the chamfer, reach, 0),
//           then two texels per edge: (a.x, a.y, b.x, b.y), (c.x, c.y, r, sweep)
//   line 1: one texel per stretch: (edge, u0, u1, 0)
//   line 2: two texels per triangle of the outline: (x0, y0, x1, y1), (x2, y2, 0, 0)
inline bool ex_shape_table(const Outline& ol, float* rows /* 3 * EXT_TABLE * 4 */) {
    std::memset(rows, 0, (size_t)3 * EXT_TABLE * 4 * sizeof(float));
    if (2 + 2 * ol.edges.size() > (size_t)EXT_TABLE || ol.stretches.size() > (size_t)EXT_COLUMNS || 2 * (ol.tris.size() / 3) > (size_t)EXT_TABLE) return false;
    float* a = rows;
    a[0] = (float)ol.edges.size(); a[1] = (float)ol.stretches.size(); a[2] = (float)(ol.tris.size() / 3);
    a[4] = (float)ol.h; a[5] = (float)(ol.chamfer / std::sqrt(2.0)); a[6] = (float)ol.reach;
    for (size_t i = 0; i < ol.edges.size(); ++i) {
        const OutlineEdge& e = ol.edges[i];
        float* o = a + (2 + 2 * i) * 4;
        o[0] = (float)e.a.x; o[1] = (float)e.a.y; o[2] = (float)e.b.x; o[3] = (float)e.b.y;
        o[4] = (float)e.c.x; o[5] = (float)e.c.y; o[6] = (float)e.r; o[7] = (float)e.sweep;
    }
    float* b = rows + (size_t)EXT_TABLE * 4;
    for (size_t i = 0; i < ol.stretches.size(); ++i) {
        b[i * 4] = (float)ol.stretches[i].edge; b[i * 4 + 1] = (float)ol.stretches[i].u0; b[i * 4 + 2] = (float)ol.stretches[i].u1;
    }
    float* t = rows + (size_t)2 * EXT_TABLE * 4;
    for (size_t i = 0; i + 2 < ol.tris.size(); i += 3) {
        const V2 p0 = ol.pts[(size_t)ol.tris[i]], p1 = ol.pts[(size_t)ol.tris[i + 1]], p2 = ol.pts[(size_t)ol.tris[i + 2]];
        float* o = t + (i / 3) * 8;
        o[0] = (float)p0.x; o[1] = (float)p0.y; o[2] = (float)p1.x; o[3] = (float)p1.y; o[4] = (float)p2.x; o[5] = (float)p2.y;
    }
    return true;
}
// The one mesh of all strips: per corner (stretch, part * 4 + corner, slot).
// part: a stretch is cut in three at the (up to two) places where the light changes its way within it.
// corner: 0 = start of the part, near edge; 1 = start, far edge; 2 = end, far edge; 3 = end, near edge.
// slot: up to three strips lie side by side across (see ex_strip_vertex).
// Three lists of indices: `slim` draws slot 0 only (18 indices per stretch), `pair` slots 1 and 2 (36), `full` all
// three (54); in each the stretches come in order, so a shape with fewer stretches is drawn with the first so many.
inline void ex_strip_mesh(std::vector<float>& corners, std::vector<uint32_t>& slim, std::vector<uint32_t>& pair, std::vector<uint32_t>& full) {
    corners.clear();
    slim.clear();
    pair.clear();
    full.clear();
    for (int m = 0; m < EXT_COLUMNS; ++m)
        for (int slot = 0; slot < 3; ++slot)
            for (int part = 0; part < 3; ++part) {
                const uint32_t base = (uint32_t)(corners.size() / 3);
                for (int k = 0; k < 4; ++k) corners.insert(corners.end(), {(float)m, (float)(part * 4 + k), (float)slot});
                full.insert(full.end(), {base, base + 1, base + 2, base, base + 2, base + 3});
                (slot == 0 ? slim : pair).insert((slot == 0 ? slim : pair).end(), {base, base + 1, base + 2, base, base + 2, base + 3});
            }
}
// The one mesh of all outlines: per corner (triangle, corner, 0).
inline void ex_cap_mesh(std::vector<float>& corners, std::vector<uint32_t>& idx) {
    corners.clear();
    idx.clear();
    for (int t = 0; t < EXT_TABLE / 2; ++t)
        for (int k = 0; k < 3; ++k) {
            idx.push_back((uint32_t)(corners.size() / 3));
            corners.insert(corners.end(), {(float)t, (float)k, 0.f});
        }
}
// What a renderer hands over per solid and frame: three texels (middle + size, x axis + line of its shape in the
// table, y axis).
inline void ex_pose(const Solid& so, int shapeLine, float* a, float* b, float* c) {
    const V3 ey = cross(so.ax, so.ux);
    a[0] = (float)so.c.x; a[1] = (float)so.c.y; a[2] = (float)so.c.z; a[3] = (float)so.sc;
    b[0] = (float)so.ux.x; b[1] = (float)so.ux.y; b[2] = (float)so.ux.z; b[3] = (float)shapeLine;
    c[0] = (float)ey.x; c[1] = (float)ey.y; c[2] = (float)ey.z; c[3] = 0.f;
}

namespace detail {

}  // namespace detail
}  // namespace beam
