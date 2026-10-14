// A ball pool the keyed hedgehog dances in (Jolt, in 3D): its film's outline each picture (bake.py) a wall reaching
// THICK towards the eye and away, the balls let settle first, then baked to keys of the document (x, y, z 15 a second).
//   pool <outline> <balls> > calls.txt   (edit.sh calls: one solid ball a ball)
#include <Jolt/Jolt.h>
#include <Jolt/RegisterTypes.h>
#include <Jolt/Core/Factory.h>
#include <Jolt/Core/TempAllocator.h>
#include <Jolt/Core/JobSystemSingleThreaded.h>
#include <Jolt/Physics/PhysicsSettings.h>
#include <Jolt/Physics/PhysicsSystem.h>
#include <Jolt/Physics/Collision/Shape/BoxShape.h>
#include <Jolt/Physics/Collision/Shape/SphereShape.h>
#include <Jolt/Physics/Body/BodyCreationSettings.h>
#include <algorithm>
#include <cmath>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <sstream>
#include <string>
#include <vector>
using namespace JPH;

namespace Layers { static constexpr ObjectLayer STILL = 0, MOVING = 1; }
struct Pairs final : ObjectLayerPairFilter { bool ShouldCollide(ObjectLayer a, ObjectLayer b) const override { return a == Layers::MOVING || b == Layers::MOVING; } };
struct Broad final : BroadPhaseLayerInterface {
    uint GetNumBroadPhaseLayers() const override { return 2; }
    BroadPhaseLayer GetBroadPhaseLayer(ObjectLayer l) const override { return BroadPhaseLayer(l); }
#if defined(JPH_EXTERNAL_PROFILE) || defined(JPH_PROFILE_ENABLED)
    const char* GetBroadPhaseLayerName(BroadPhaseLayer) const override { return "layer"; }
#endif
};
struct ObjectVsBroad final : ObjectVsBroadPhaseLayerFilter { bool ShouldCollide(ObjectLayer a, BroadPhaseLayer b) const override { return a == Layers::MOVING || (BroadPhaseLayer::Type)b == Layers::MOVING; } };
struct Lcg { uint64_t s; float next() { s = s * 6364136223846793005ull + 1442695040888963407ull; return (float)(s >> 40) / 16777216.f; } };

static const float W = 19.2f, H = 10.8f, DEEP = 1.f, THICK = 0.6f;   // (metres: the picture 100 points a metre; the pool reaches DEEP towards the eye and away)
static const double POINT = 335.885;                                   // (points of the picture to a unit of the stage: motolii_point)

typedef std::vector<std::vector<std::pair<float, float>>> Rings;
static std::vector<Rings> read_outlines(const char* path, double* fps) {
    std::vector<Rings> all;
    std::ifstream f(path);
    std::string line, word;
    std::getline(f, line);
    std::sscanf(line.c_str(), "fps %lf", fps);
    while (std::getline(f, line)) {
        Rings rings(1);
        std::istringstream s(line);
        s >> word;
        std::vector<float> v;
        const auto close = [&] { for (size_t i = 0; i + 1 < v.size(); i += 2) rings.back().push_back({v[i] * W, (1.f - v[i + 1]) * H}); v.clear(); };
        while (s >> word) { if (word == "|") { close(); rings.emplace_back(); } else v.push_back(std::stof(word)); }
        close();
        rings.erase(std::remove_if(rings.begin(), rings.end(), [](auto& r) { return r.size() < 3; }), rings.end());
        all.push_back(rings);
    }
    return all;
}
static bool inside(const std::vector<std::pair<float, float>>& r, float x, float y) {
    bool in = false;
    for (size_t i = 0, j = r.size() - 1; i < r.size(); j = i++)
        if ((r[i].second > y) != (r[j].second > y) && x < (r[j].first - r[i].first) * (y - r[i].second) / (r[j].second - r[i].second) + r[i].first) in = !in;
    return in;
}

int main(int argc, char** argv) {
    if (argc < 3) return std::fprintf(stderr, "pool <outline> <balls>\n"), 1;
    double fps = 30;
    const std::vector<Rings> pictures = read_outlines(argv[1], &fps);
    const int count = std::atoi(argv[2]);
    RegisterDefaultAllocator();
    Factory::sInstance = new Factory();
    RegisterTypes();
    Pairs pairs; Broad broad; ObjectVsBroad ovb;
    TempAllocatorImpl temp(64 * 1024 * 1024);
    JobSystemSingleThreaded jobs(cMaxPhysicsJobs);
    PhysicsSystem ps;
    ps.Init(8192, 0, 65536, 20480, broad, ovb, pairs);
    ps.SetGravity(Vec3(0, -14.f, 0));
    BodyInterface& bi = ps.GetBodyInterface();
    const auto still = [&](Vec3 half, Vec3 at, Quat turn, float bounce) {
        BodyCreationSettings s(new BoxShape(half, 0.f), RVec3(at), turn, EMotionType::Static, Layers::STILL);
        s.mRestitution = bounce, s.mFriction = 0.3f;
        return bi.CreateAndAddBody(s, EActivation::DontActivate);
    };
    // The pool: a floor at the picture's foot, sides at its edges and a little out, front and back DEEP away.
    still(Vec3(W, 0.5f, DEEP + 1), Vec3(0.5f * W, -0.5f, 0), Quat::sIdentity(), 0.3f);
    for (float x : {-0.5f, W + 0.5f}) still(Vec3(0.5f, H, DEEP + 1), Vec3(x, H, 0), Quat::sIdentity(), 0.3f);
    for (float z : {-THICK - 0.5f, DEEP + 0.5f}) still(Vec3(W, H, 0.5f), Vec3(0.5f * W, H, z), Quat::sIdentity(), 0.3f);   // (the back right behind the hedgehog: every ball is before it)

    Lcg r{7};
    std::vector<std::pair<BodyID, float>> balls;
    for (int i = 0; i < count; i++) {
        const float rad = 0.45f + 0.1f * r.next();
        const Vec3 at(0.6f + (W - 1.2f) * r.next(), 1.f + 0.35f * i, -THICK + 0.5f + (DEEP + THICK - 1.f) * r.next());
        BodyCreationSettings s(new SphereShape(rad), RVec3(at), Quat::sIdentity(), EMotionType::Dynamic, Layers::MOVING);
        s.mRestitution = 0.55f, s.mFriction = 0.3f, s.mLinearDamping = 0.05f, s.mAngularDamping = 0.2f;
        balls.push_back({bi.CreateAndAddBody(s, EActivation::Activate), rad});
    }
    const auto step = [&] { ps.Update(1.f / 60.f, 1, &temp, &jobs); };
    for (int k = 0; k < 240; k++) step();   // (settled before the film begins)

    std::printf("# %d balls\n", count);
    for (int i = 0; i < count; i++) std::printf("add solid ball\n");
    for (int i = 0; i < count; i++) std::printf("set %d size %.4f\n", i + 2, 2.0 * balls[i].second * 100.0 / POINT);
    const int steps = (int)(pictures.size() / fps * 60.0);
    std::vector<BodyID> walls;
    size_t showing = SIZE_MAX;
    for (int k = 0; k <= steps; k++) {
        const size_t now = std::min((size_t)(k / 60.0 * fps), pictures.size() - 1);
        if (now != showing) {
            showing = now;
            for (BodyID id : walls) bi.RemoveBody(id), bi.DestroyBody(id);
            walls.clear();
            for (auto& ring : pictures[now])
                for (size_t a = 0; a < ring.size(); a++) {
                    const auto p = ring[a], q = ring[(a + 1) % ring.size()];
                    const float dx = q.first - p.first, dy = q.second - p.second, len = std::max(std::sqrt(dx * dx + dy * dy), 1e-3f);
                    walls.push_back(still(Vec3(0.5f * len, 0.05f, THICK), Vec3(0.5f * (p.first + q.first), 0.5f * (p.second + q.second), 0), Quat::sRotation(Vec3::sAxisZ(), std::atan2(dy, dx)), 0.7f));
                }
            // (a ball the hedgehog has grown over, within its thickness, is put out at its nearest edge and sent off that way)
            for (auto [b, rad] : balls) {
                const RVec3 t = bi.GetPosition(b);
                if (std::fabs((float)t.GetZ()) > THICK + rad) continue;
                for (auto& ring : pictures[now]) {
                    if (!inside(ring, (float)t.GetX(), (float)t.GetY())) continue;
                    float best = 1e9f, ox = 0, oy = 0, nx = 0, ny = 1;
                    for (size_t a = 0; a < ring.size(); a++) {
                        const auto p = ring[a], q = ring[(a + 1) % ring.size()];
                        if (p.second < 0.01f && q.second < 0.01f) continue;   // (the picture's foot is not an edge a ball goes out by)
                        const float dx = q.first - p.first, dy = q.second - p.second;
                        const float s = std::clamp((((float)t.GetX() - p.first) * dx + ((float)t.GetY() - p.second) * dy) / std::max(dx * dx + dy * dy, 1e-6f), 0.f, 1.f);
                        const float ax = p.first + s * dx, ay = p.second + s * dy, d = std::hypot((float)t.GetX() - ax, (float)t.GetY() - ay);
                        if (d < best) { const float l = std::max(std::hypot(dy, dx), 1e-6f); best = d, ox = ax, oy = ay, nx = dy / l, ny = -dx / l; }
                    }
                    if (inside(ring, ox + nx * 0.05f, oy + ny * 0.05f)) nx = -nx, ny = -ny;
                    // (out of the way it is nearest: in front of or behind the hedgehog if that is nearer than its edge)
                    const float z = (float)t.GetZ(), toface = THICK + rad - std::fabs(z);
                    const Vec3 v = bi.GetLinearVelocity(b);
                    if (toface < best) {
                        const float side = 1.f;   // (out in front: nothing is behind it)
                        bi.SetPosition(b, RVec3(t.GetX(), t.GetY(), side * (THICK + rad + 0.02f)), EActivation::Activate);
                        bi.SetLinearVelocity(b, Vec3(v.GetX(), v.GetY() + 2.f, v.GetZ() + side * 6.f));
                    } else {
                        bi.SetPosition(b, RVec3(ox + nx * (rad + 0.02f), oy + ny * (rad + 0.02f), z), EActivation::Activate);
                        bi.SetLinearVelocity(b, Vec3(v.GetX() + 7.f * nx, v.GetY() + 7.f * ny, v.GetZ()));
                    }
                    break;
                }
            }
        }
        if (k % 4 == 0) {   // (keys 15 a second, in units of the stage: its middle at the picture's middle, z towards the eye)
            const double at = k / 60.0;
            for (size_t i = 0; i < balls.size(); i++) {
                const RVec3 p = bi.GetPosition(balls[i].first);
                std::printf("key %zu x %.4f %.4f\nkey %zu y %.4f %.4f\nkey %zu z %.4f %.4f\n", i + 2, at, ((float)p.GetX() - 0.5f * W) * 100.0 / POINT, i + 2, at, ((float)p.GetY() - 0.5f * H) * 100.0 / POINT, i + 2, at, (float)p.GetZ() * 100.0 / POINT);
            }
        }
        step();
    }
    return 0;
}
