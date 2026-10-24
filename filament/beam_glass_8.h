// beam_glass.h, part 8 (cut there by size, as it was): read only through beam_glass.h, in order.
#pragma once

namespace beam {

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
    if (solid.kind == Solid::EXTRUDE) {
        extrude_silhouette(solid, sun, floorY, tri);
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


// ---- Receivers: where light lands besides the floor --------------------------------------------------------------
// Everything above drops its light on ONE plane, the floor. A Stage is everything flat light can land on: the room
// (the floor and the walls standing on it; convex, seen from inside) and matte blocks standing in it. It is made
// once, when the scene is set up, and handed to the materials as two small tables (`room`, `blocks`); NOTHING here
// runs per frame. Where a piece of light lands is decided in the shaders:
//   - recv_plane.glsl, recvChoose: the plane a ray is dropped on (the first plane of the room it meets; the face
//     through which it enters the first block it meets). ball_light.mat and cone_walk.mat ask it for every corner.
//   - recv_lands.glsl, recvOn / recvBlocked: per pixel, is the light really there: on the part of its plane that is
//     surface, and with no block in its way. In a convex room a ray meets a plane first exactly where its landing
//     point lies in front of all the other planes, and a face of a block exactly where its landing point lies
//     within the block. So every ray is kept on one receiver and one only: what the floor loses at the foot of a
//     wall the wall gets, what a block catches is missing behind it.
// stage_choose / stage_on / stage_blocked below are that rule on the CPU, line for line, for the desk check only.
struct Block {
    V3 c;         // middle
    V3 ax[3];     // unit axes, at right angles
    double h[3];  // half sizes along them
};
struct Stage {
    std::vector<Half> room;      // inside the room: all of these hold. The floor first, then the walls, and LAST where
                                 // the walls end (nothing lands on that one)
    std::vector<Block> blocks;
};
// walls: the room's side of each wall as a.x + b >= 0, a of length 1 and level (walls stand upright). They end at y = top.
inline Stage make_stage(double floorY, const std::vector<Half>& walls, double top, const std::vector<Block>& blocks) {
    Stage st;
    st.room.push_back(Half{V3{0, 1, 0}, -floorY});
    for (const Half& w : walls) st.room.push_back(w);
    st.room.push_back(Half{V3{0, -1, 0}, top});
    st.blocks = blocks;
    return st;
}
// recvChoose of recv_plane.glsl. kind 0: the room, kind 1: the blocks. The plane a ray from E along D (unit) is
// dropped on (unit normal towards the light, its points dot(n, x) = o), and what it belongs to (-1: the room, i: block i).
inline bool stage_choose(const Stage& st, int kind, V3 E, V3 D, Plane& N, int& owner) {
    owner = -1;
    if (kind == 0) {
        double best = 1e30;
        bool found = false;
        for (size_t j = 0; j + 1 < st.room.size(); ++j) {
            const double heading = -dot(st.room[j].a, D);
            if (heading < 1e-5) continue;
            const double t = (dot(st.room[j].a, E) + st.room[j].b) / heading;
            if (t < best) {
                best = t;
                N = Plane{st.room[j].a, -st.room[j].b};
                found = true;
            }
        }
        return found;
    }
    double bestKey = 1e30;
    for (size_t i = 0; i < st.blocks.size(); ++i) {
        const Block& B = st.blocks[i];
        double t0 = -1e30, t1 = 1e30, half = 0;
        V3 enter{0, 1, 0};
        bool past = false;
        for (int k = 0; k < 3; ++k) {
            const double o = dot(B.ax[k], E - B.c), d = dot(B.ax[k], D);
            if (std::fabs(d) < 1e-6) {
                if (std::fabs(o) > B.h[k]) past = true;
                continue;
            }
            const double ta = (-B.h[k] - o) / d, tb = (B.h[k] - o) / d;
            if (std::min(ta, tb) > t0) {
                t0 = std::min(ta, tb);
                enter = d > 0 ? B.ax[k] * -1.0 : B.ax[k];
                half = B.h[k];
            }
            t1 = std::min(t1, std::max(ta, tb));
        }
        if (past) continue;
        const double key = (t0 < t1 && t1 > 0) ? t0 : 1e6 + (t0 - t1);
        if (key < bestKey) {
            bestKey = key;
            N = Plane{enter, dot(enter, B.c) + half};
            owner = (int)i;
        }
    }
    return bestKey < 1e29;
}
// recvThrough, recvBlocked, recvOn of recv_lands.glsl.
inline bool stage_through(const Block& B, V3 a, V3 ab, double far) {
    double t0 = 0, t1 = far;
    for (int k = 0; k < 3; ++k) {
        const double o = dot(B.ax[k], a - B.c), d = dot(B.ax[k], ab);
        if (std::fabs(d) < 1e-7) {
            if (std::fabs(o) > B.h[k]) return false;
            continue;
        }
        const double ta = (-B.h[k] - o) / d, tb = (B.h[k] - o) / d;
        t0 = std::max(t0, std::min(ta, tb));
        t1 = std::min(t1, std::max(ta, tb));
    }
    return t0 < t1;
}
inline bool stage_blocked(const Stage& st, V3 X, V3 E, V3 sun) {
    for (const Block& B : st.blocks) {
        if (stage_through(B, E, X - E, 0.9999)) return true;  // (0.9999 of the way: light that lands ON a block has not gone through it)
        if (stage_through(B, E, sun * -1.0, 1e9)) return true;
    }
    return false;
}
inline bool stage_on(const Stage& st, V3 X, int owner) {
    if (owner < 0) {
        for (const Half& h : st.room)
            if (dot(h.a, X) + h.b < -1e-4) return false;
        return true;
    }
    const Block& B = st.blocks[(size_t)owner];
    for (int k = 0; k < 3; ++k)
        if (std::fabs(dot(B.ax[k], X - B.c)) > B.h[k] + 1e-4) return false;
    return true;
}

}  // namespace beam

#include "beam_extrude.h"
