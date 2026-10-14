// Balls the keyed cat knocks about (Box2D, Erin Catto's, as Cavalry's Forge Dynamics and After Effects' Newton use):
// the cat's outline each picture of its film (bake.py) is a wall of the world for that picture, and the balls are
// baked to keys of the document, as a designer bakes dynamics to keyframes.
//   sim <outline> <balls> <seed> > calls.txt   (edit.sh calls: one shape circle a ball, its x and y keyed 30 a second)
#include <box2d/box2d.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

enum { WIDE = 1920, HIGH = 1080, MOST = 64, ROOM = 256 };
static const float PER_M = 100.f;     // (points of the picture to a metre of the world)
static const double POINT = 335.885;  // (points of the picture to a unit of the stage: motolii_point)

typedef struct { int n; b2Vec2 at[ROOM]; } Ring;
typedef struct { double t; int rings; Ring ring[8]; } Picture;

static int read_outlines(const char* path, Picture** out, double* fps) {
    FILE* f = fopen(path, "r");
    if (!f) return 0;
    static char line[1 << 16];
    int n = 0, room = 0;
    *out = NULL;
    if (!fgets(line, sizeof line, f) || sscanf(line, "fps %lf", fps) != 1) return 0;
    while (fgets(line, sizeof line, f)) {
        if (n == room) *out = realloc(*out, sizeof(Picture) * (room = room ? room * 2 : 512));
        Picture* p = &(*out)[n++];
        memset(p, 0, sizeof *p);
        char* s = line;
        p->t = strtod(s, &s);
        Ring* r = &p->ring[0];
        for (;;) {
            while (*s == ' ') s++;
            if (*s == '|') { if (r->n) r = &p->ring[++p->rings < 8 ? p->rings : 7]; s++; continue; }
            char* e;
            const double u = strtod(s, &e);
            if (e == s) break;
            const double v = strtod(e, &s);
            if (r->n < ROOM) r->at[r->n++] = (b2Vec2){(float)(u * WIDE / PER_M), (float)((1.0 - v) * HIGH / PER_M)};   // (y up, metres)
        }
        if (r->n) p->rings++;
    }
    fclose(f);
    return n;
}

static float area(const Ring* r) {
    float a = 0;
    for (int i = 0; i < r->n; i++) a += b2Cross(r->at[i], r->at[(i + 1) % r->n]);
    return 0.5f * a;
}
static int inside(const Ring* r, b2Vec2 p) {
    int in = 0;
    for (int i = 0, j = r->n - 1; i < r->n; j = i++)
        if ((r->at[i].y > p.y) != (r->at[j].y > p.y) && p.x < (r->at[j].x - r->at[i].x) * (p.y - r->at[i].y) / (r->at[j].y - r->at[i].y) + r->at[i].x) in = !in;
    return in;
}

int main(int argc, char** argv) {
    if (argc < 4) return fprintf(stderr, "sim <outline> <balls> <seed>\n"), 1;
    Picture* pics;
    double fps;
    const int pictures = read_outlines(argv[1], &pics, &fps), balls = atoi(argv[2]) < MOST ? atoi(argv[2]) : MOST;
    srand((unsigned)atoi(argv[3]));
    if (!pictures) return fprintf(stderr, "no outlines\n"), 1;

    b2WorldDef wd = b2DefaultWorldDef();
    wd.gravity = (b2Vec2){0.f, -14.f};
    wd.maxContactPushSpeed = 12.f;   // (a cat that grows into a ball shoves it this fast at most, metres a second)
    const b2WorldId world = b2CreateWorld(&wd);

    // No floor and no sides: they rain in over the cat from above, and one that leaves the picture rains in again.
    const float w = WIDE / PER_M, h = HIGH / PER_M;

    b2BodyId ball[MOST];
    float radius[MOST];
    for (int i = 0; i < balls; i++) {
        b2BodyDef bd = b2DefaultBodyDef();
        bd.type = b2_dynamicBody;
        radius[i] = 0.22f + 0.28f * (float)rand() / RAND_MAX;
        bd.position = (b2Vec2){w * (0.3f + 0.4f * (float)rand() / RAND_MAX), h + 0.7f * i};   // (raining in one after another, over the cat)
        ball[i] = b2CreateBody(world, &bd);
        b2ShapeDef bs = b2DefaultShapeDef();
        bs.density = 1.f, bs.material.restitution = 0.7f, bs.material.friction = 0.3f;
        const b2Circle c = {{0, 0}, radius[i]};
        b2CreateCircleShape(ball[i], &bs, &c);
    }

    // The film's pictures, 60 steps a second; the cat's outline is the wall of the picture the film shows then.
    const int steps = (int)floor(pictures / fps * 60.0);
    b2BodyId cat = b2_nullBodyId;
    int showing = -1;
    printf("# %d balls, %d steps\n", balls, steps);
    for (int i = 0; i < balls; i++) printf("add shape circle\n");
    for (int i = 0; i < balls; i++) printf("set %d size %.4f\n", i + 2, 2.0 * radius[i] * PER_M / POINT);
    for (int k = 0; k <= steps; k++) {
        const int now = (int)floor(k / 60.0 * fps) < pictures ? (int)floor(k / 60.0 * fps) : pictures - 1;
        if (now != showing) {
            showing = now;
            if (B2_IS_NON_NULL(cat)) b2DestroyBody(cat);
            b2BodyDef cd = b2DefaultBodyDef();
            cat = b2CreateBody(world, &cd);
            for (int r = 0; r < pics[now].rings; r++) {
                Ring* ring = &pics[now].ring[r];
                if (ring->n < 4) continue;
                if (area(ring) < 0) for (int a = 0, b = ring->n - 1; a < b; a++, b--) { const b2Vec2 t = ring->at[a]; ring->at[a] = ring->at[b], ring->at[b] = t; }   // (counter-clockwise: the side it is struck on is outside)
                b2ChainDef ch = b2DefaultChainDef();
                b2SurfaceMaterial m = {.friction = 0.4f, .restitution = 0.8f};
                ch.points = ring->at, ch.count = ring->n, ch.isLoop = true, ch.materials = &m, ch.materialCount = 1;
                b2CreateChain(cat, &ch);
            }
            // (a ball the cat has grown over is put out at its nearest edge and sent off that way: a knock)
            for (int i = 0; i < balls; i++) {
                const b2Vec2 p = b2Body_GetPosition(ball[i]);
                for (int r = 0; r < pics[now].rings; r++) {
                    const Ring* ring = &pics[now].ring[r];
                    if (ring->n < 4 || !inside(ring, p)) continue;
                    float best = 1e9f;
                    b2Vec2 out = p, normal = {0, 1};
                    for (int a = 0; a < ring->n; a++) {
                        const b2Vec2 e0 = ring->at[a], e1 = ring->at[(a + 1) % ring->n], d = b2Sub(e1, e0);
                        const float t = b2ClampFloat(b2Dot(b2Sub(p, e0), d) / fmaxf(b2Dot(d, d), 1e-6f), 0.f, 1.f);
                        const b2Vec2 q = b2MulAdd(e0, t, d);
                        const float dist = b2Distance(p, q);
                        if (dist < best) best = dist, out = q, normal = b2Normalize((b2Vec2){d.y, -d.x});   // (counter-clockwise: right of the edge is out)
                    }
                    b2Body_SetTransform(ball[i], b2MulAdd(out, radius[i] + 0.02f, normal), b2Rot_identity);
                    b2Body_SetLinearVelocity(ball[i], b2MulAdd(b2Body_GetLinearVelocity(ball[i]), 9.f, normal));
                    break;
                }
            }
        }
        if (k % 2 == 0) {   // (keys 30 a second)
            const double at = k / 60.0;
            for (int i = 0; i < balls; i++) {
                b2Vec2 p = b2Body_GetPosition(ball[i]);
                printf("key %d x %.4f %.4f\nkey %d y %.4f %.4f\n", i + 2, at, (p.x * PER_M - WIDE / 2.0) / POINT, i + 2, at, (p.y * PER_M - HIGH / 2.0) / POINT);
                if (p.y < -radius[i] - 0.2f || p.x < -1.f || p.x > w + 1.f) {   // (gone from the picture: in again from above; held till then, so it does not cross the picture)
                    printf("way %d x %.4f Hold\nway %d y %.4f Hold\n", i + 2, at, i + 2, at);
                    p = (b2Vec2){w * (0.3f + 0.4f * (float)rand() / RAND_MAX), h + 0.5f + 2.f * (float)rand() / RAND_MAX};
                    b2Body_SetTransform(ball[i], p, b2Rot_identity);
                    b2Body_SetLinearVelocity(ball[i], (b2Vec2){0.f, -2.f});
                }
            }
        }
        b2World_Step(world, 1.f / 60.f, 4);
    }
    b2DestroyWorld(world);
    return 0;
}
