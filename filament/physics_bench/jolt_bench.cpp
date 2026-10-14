// The same scenes as rapier_bench, in Jolt (cross-platform deterministic build, one thread): a pile of thin pieces,
// a grid of joints, a rope, the dancing cat's outline as walls with balls knocked about it, and overlaps as a hand
// makes them. Each scene run twice: the time a step takes, and whether the two runs end the same to the bit.
//   jolt_bench <cat outline file>
#include <Jolt/Jolt.h>
#include <Jolt/RegisterTypes.h>
#include <Jolt/Core/Factory.h>
#include <Jolt/Core/TempAllocator.h>
#include <Jolt/Core/JobSystemSingleThreaded.h>
#include <Jolt/Physics/PhysicsSettings.h>
#include <Jolt/Physics/PhysicsSystem.h>
#include <Jolt/Physics/Collision/Shape/BoxShape.h>
#include <Jolt/Physics/Collision/Shape/SphereShape.h>
#include <Jolt/Physics/Collision/Shape/CapsuleShape.h>
#include <Jolt/Physics/Body/BodyCreationSettings.h>
#include <Jolt/Physics/Constraints/PointConstraint.h>
#include <chrono>
#include <cmath>
#include <cstdio>
#include <fstream>
#include <sstream>
#include <string>
#include <vector>
#include <algorithm>
using namespace JPH;

namespace Layers { static constexpr ObjectLayer STILL = 0, MOVING = 1, COUNT = 2; }
struct Pairs final : ObjectLayerPairFilter {
    bool ShouldCollide(ObjectLayer a, ObjectLayer b) const override { return a == Layers::MOVING || b == Layers::MOVING; }
};
struct Broad final : BroadPhaseLayerInterface {
    uint GetNumBroadPhaseLayers() const override { return 2; }
    BroadPhaseLayer GetBroadPhaseLayer(ObjectLayer l) const override { return BroadPhaseLayer(l); }
#if defined(JPH_EXTERNAL_PROFILE) || defined(JPH_PROFILE_ENABLED)
    const char* GetBroadPhaseLayerName(BroadPhaseLayer) const override { return "layer"; }
#endif
};
struct ObjectVsBroad final : ObjectVsBroadPhaseLayerFilter {
    bool ShouldCollide(ObjectLayer a, BroadPhaseLayer b) const override { return a == Layers::MOVING || (BroadPhaseLayer::Type)b == Layers::MOVING; }
};

struct Lcg {
    uint64_t s;
    float next() { s = s * 6364136223846793005ull + 1442695040888963407ull; return (float)(s >> 40) / 16777216.f; }
};

struct World {
    Pairs pairs; Broad broad; ObjectVsBroad ovb;
    TempAllocatorImpl temp{64 * 1024 * 1024};
    JobSystemSingleThreaded jobs{cMaxPhysicsJobs};
    PhysicsSystem ps;
    std::vector<BodyID> moving;
    World(float gravity) {
        ps.Init(8192, 0, 65536, 20480, broad, ovb, pairs);
        ps.SetGravity(Vec3(0, gravity, 0));
    }
    BodyInterface& bi() { return ps.GetBodyInterface(); }
    bool flat = false;   // (every moving body held to the picture's plane: Jolt's Plane2D, 2D in the 3D engine)
    BodyID add(const Shape* shape, Vec3 at, Quat turn, bool dynamic, float friction = 0.5f, float bounce = 0.f, float density = 1.f) {
        BodyCreationSettings s(shape, RVec3(at), turn, dynamic ? EMotionType::Dynamic : EMotionType::Static, dynamic ? Layers::MOVING : Layers::STILL);
        s.mFriction = friction, s.mRestitution = bounce, s.mLinearDamping = 0, s.mAngularDamping = 0;
        if (dynamic && flat) s.mAllowedDOFs = EAllowedDOFs::Plane2D;
        if (dynamic) { s.mOverrideMassProperties = EOverrideMassProperties::CalculateInertia; s.mMassPropertiesOverride.mMass = density * shape->GetMassProperties().mMass; }
        const BodyID id = bi().CreateAndAddBody(s, dynamic ? EActivation::Activate : EActivation::DontActivate);
        if (dynamic) moving.push_back(id);
        return id;
    }
    void joint(BodyID a, BodyID b, Vec3 pa, Vec3 pb) {
        PointConstraintSettings c;
        c.mSpace = EConstraintSpace::LocalToBodyCOM, c.mPoint1 = RVec3(pa), c.mPoint2 = RVec3(pb);
        ps.AddConstraint(bi().CreateConstraint(&c, a, b));
    }
    void step() { ps.Update(1.f / 60.f, 1, &temp, &jobs); }
    uint64_t hash() {
        uint64_t h = 1469598103934665603ull;
        for (BodyID id : moving) {
            const RVec3 p = bi().GetPosition(id);
            const Vec3 v = bi().GetLinearVelocity(id);
            for (float f : {(float)p.GetX(), (float)p.GetY(), (float)p.GetZ(), v.GetX(), v.GetY(), v.GetZ()}) {
                uint32_t bits; std::memcpy(&bits, &f, 4);
                for (int k = 0; k < 4; k++) h = (h ^ ((bits >> (8 * k)) & 255)) * 1099511628211ull;
            }
        }
        return h;
    }
    float energy() { float e = 0; for (BodyID id : moving) e += bi().GetLinearVelocity(id).LengthSq(); return e; }
    float top() { float e = 0; for (BodyID id : moving) e = std::max(e, bi().GetLinearVelocity(id).Length()); return e; }
};

static int pile(World& w) {
    w.add(new BoxShape(Vec3(20, 0.5f, 5)), Vec3(0, -0.5f, 0), Quat::sIdentity(), false);
    for (float x : {-10.5f, 10.5f}) w.add(new BoxShape(Vec3(0.5f, 10, 5)), Vec3(x, 10, 0), Quat::sIdentity(), false);
    for (float z : {-0.6f, 0.6f}) w.add(new BoxShape(Vec3(20, 10, 0.5f)), Vec3(0, 10, z + (z > 0 ? 0.5f : -0.5f)), Quat::sIdentity(), false);
    Lcg r{7};
    for (int i = 0; i < 200; i++) {
        const float hx = 0.1f + 0.25f * r.next(), hy = 0.1f + 0.25f * r.next();
        const Vec3 p(-8.f + 0.8f * (i % 20) + 0.1f * r.next(), 1.f + 0.8f * (i / 20), 0);
        w.add(new BoxShape(Vec3(hx, hy, 0.08f), 0.f), p, Quat::sIdentity(), true, 0.6f, 0.2f);
    }
    return 600;
}
static int grid(World& w) {
    const int n = 20;
    std::vector<BodyID> b(n * n);
    for (int k = 0; k < n; k++)
        for (int i = 0; i < n; i++) {
            const BodyID h = w.add(new SphereShape(0.4f), Vec3((float)k, -(float)i, 0), Quat::sIdentity(), i > 0);
            if (i > 0) w.joint(b[k * n + i - 1], h, Vec3(0, -0.5f, 0), Vec3(0, 0.5f, 0));
            if (k > 0) w.joint(b[(k - 1) * n + i], h, Vec3(0.5f, 0, 0), Vec3(-0.5f, 0, 0));
            b[k * n + i] = h;
        }
    return 600;
}
static int rope(World& w) {
    BodyID last;
    const Quat along = Quat::sRotation(Vec3::sAxisZ(), 0.5f * JPH_PI);   // (a capsule lies along its y: turned to lie along x)
    for (int i = 0; i < 100; i++) {
        const BodyID h = w.add(new CapsuleShape(0.1f, 0.05f), Vec3(0.3f * i, 10, 0), along, i > 0, 0.5f, 0.f, 2.f);
        if (i > 0) w.joint(last, h, Vec3(0, -0.15f, 0), Vec3(0, 0.15f, 0));   // (turned: the body's -y is the world's +x)
        last = h;
    }
    return 600;
}

struct Cat { double fps = 30; std::vector<std::vector<std::vector<std::pair<float, float>>>> pictures; };
static Cat read_cat(const char* path) {
    Cat c;
    std::ifstream f(path);
    std::string line;
    std::getline(f, line);
    std::sscanf(line.c_str(), "fps %lf", &c.fps);
    while (std::getline(f, line)) {
        std::vector<std::vector<std::pair<float, float>>> rings(1);
        std::istringstream s(line);
        std::string word;
        s >> word;   // (its moment)
        std::vector<float> v;
        while (s >> word) {
            if (word == "|") { for (size_t i = 0; i + 1 < v.size(); i += 2) rings.back().push_back({v[i] * 19.2f, (1.f - v[i + 1]) * 10.8f}); v.clear(); rings.emplace_back(); continue; }
            v.push_back(std::stof(word));
        }
        for (size_t i = 0; i + 1 < v.size(); i += 2) rings.back().push_back({v[i] * 19.2f, (1.f - v[i + 1]) * 10.8f});
        rings.erase(std::remove_if(rings.begin(), rings.end(), [](auto& r) { return r.size() < 3; }), rings.end());
        c.pictures.push_back(rings);
    }
    return c;
}
static bool inside(const std::vector<std::pair<float, float>>& r, float x, float y) {
    bool in = false;
    for (size_t i = 0, j = r.size() - 1; i < r.size(); j = i++)
        if ((r[i].second > y) != (r[j].second > y) && x < (r[j].first - r[i].first) * (y - r[i].second) / (r[j].second - r[i].second) + r[i].first) in = !in;
    return in;
}
static void run_cat(World& w, const Cat& cat, std::vector<double>& times) {
    const float W = 19.2f, H = 10.8f;
    Lcg r{7};
    std::vector<std::pair<BodyID, float>> balls;
    for (int i = 0; i < 40; i++) {
        const float rad = 0.22f + 0.28f * r.next();
        const Vec3 p(W * (0.3f + 0.4f * r.next()), H + 0.7f * i, -0.2f + 0.4f * r.next());
        balls.push_back({w.add(new SphereShape(rad), p, Quat::sIdentity(), true, 0.3f, 0.7f), rad});
    }
    const int steps = (int)(cat.pictures.size() / cat.fps * 60.0);
    std::vector<BodyID> walls;
    size_t showing = SIZE_MAX;
    for (int k = 0; k < steps; k++) {
        const auto t0 = std::chrono::steady_clock::now();
        const size_t now = std::min((size_t)(k / 60.0 * cat.fps), cat.pictures.size() - 1);
        if (now != showing) {
            showing = now;
            for (BodyID id : walls) w.bi().RemoveBody(id), w.bi().DestroyBody(id);
            walls.clear();
            for (auto& ring : cat.pictures[now])
                for (size_t a = 0; a < ring.size(); a++) {
                    const auto p = ring[a], q = ring[(a + 1) % ring.size()];
                    const float dx = q.first - p.first, dy = q.second - p.second, len = std::max(std::sqrt(dx * dx + dy * dy), 1e-3f);
                    walls.push_back(w.add(new BoxShape(Vec3(0.5f * len, 0.03f, 0.3f), 0.f), Vec3(0.5f * (p.first + q.first), 0.5f * (p.second + q.second), 0), Quat::sRotation(Vec3::sAxisZ(), std::atan2(dy, dx)), false, 0.5f, 0.8f));
                }
            for (auto [b, rad] : balls) {
                const RVec3 t = w.bi().GetPosition(b);
                if (std::fabs((float)t.GetZ()) > 0.3f) continue;
                for (auto& ring : cat.pictures[now]) {
                    if (!inside(ring, (float)t.GetX(), (float)t.GetY())) continue;
                    float best = 1e9f, ox = 0, oy = 0, nx = 0, ny = 1;
                    for (size_t a = 0; a < ring.size(); a++) {
                        const auto p = ring[a], q = ring[(a + 1) % ring.size()];
                        const float dx = q.first - p.first, dy = q.second - p.second;
                        const float s = std::clamp((((float)t.GetX() - p.first) * dx + ((float)t.GetY() - p.second) * dy) / std::max(dx * dx + dy * dy, 1e-6f), 0.f, 1.f);
                        const float ax = p.first + s * dx, ay = p.second + s * dy, d = std::hypot((float)t.GetX() - ax, (float)t.GetY() - ay);
                        if (d < best) { const float l = std::max(std::hypot(dy, dx), 1e-6f); best = d, ox = ax, oy = ay, nx = dy / l, ny = -dx / l; }
                    }
                    if (inside(ring, ox + nx * 0.05f, oy + ny * 0.05f)) nx = -nx, ny = -ny;
                    const Vec3 v = w.bi().GetLinearVelocity(b);
                    w.bi().SetPosition(b, RVec3(ox + nx * (rad + 0.02f), oy + ny * (rad + 0.02f), t.GetZ()), EActivation::Activate);
                    w.bi().SetLinearVelocity(b, Vec3(v.GetX() + 9.f * nx, v.GetY() + 9.f * ny, v.GetZ()));
                    break;
                }
            }
        }
        if (k % 2 == 0)
            for (auto [b, rad] : balls) {
                const RVec3 t = w.bi().GetPosition(b);
                if (t.GetY() < -1 || t.GetX() < -1 || t.GetX() > W + 1) {
                    w.bi().SetPosition(b, RVec3(W * (0.3f + 0.4f * r.next()), H + 0.5f + 2.f * r.next(), -0.2f + 0.4f * r.next()), EActivation::Activate);
                    w.bi().SetLinearAndAngularVelocity(b, Vec3(0, -2, 0), Vec3::sZero());
                }
            }
        w.step();
        times.push_back(std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count());
    }
}

static std::pair<float, int> overlap(World& w, bool born, std::vector<double>& times) {
    if (born) {
        w.add(new BoxShape(Vec3(20, 0.5f, 5)), Vec3(0, -0.5f, 0), Quat::sIdentity(), false);
        Lcg r{11};
        for (int i = 0; i < 50; i++) w.add(new BoxShape(Vec3(0.3f, 0.3f, 0.08f), 0.f), Vec3(0.01f * r.next(), 1, 0), Quat::sIdentity(), true);
    } else {
        pile(w);
        for (int i = 0; i < 300; i++) w.step();
        Lcg r{13};
        for (int i = 0; i < 20; i++) w.bi().SetPosition(w.moving[i], RVec3(-0.5f + r.next(), 0.6f + 0.5f * r.next(), 0), EActivation::Activate);
    }
    float most = 0;
    int calm = 0;
    for (int k = 0; k < 240; k++) {
        const auto t0 = std::chrono::steady_clock::now();
        w.step();
        times.push_back(std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count());
        const float v = w.top();
        if (k < 120) most = std::max(most, v);
        if (v > 0.5f) calm = k + 1;
    }
    // (still one in another: pairs whose middles are nearer than 0.15; and how far apart the first and last are)
    int stacked = 0;
    float lo = 1e9f, hi = -1e9f;
    for (size_t i = 0; i < w.moving.size(); i++) {
        const RVec3 a = w.bi().GetPosition(w.moving[i]);
        lo = std::min(lo, (float)a.GetX()), hi = std::max(hi, (float)a.GetX());
        for (size_t j = i + 1; j < w.moving.size(); j++) if ((float)(a - w.bi().GetPosition(w.moving[j])).Length() < 0.15f) stacked++;
    }
    std::printf("   (after 4 s: %d pairs still one in another, spread %.1f m)\n", stacked, hi - lo);
    return {most, calm};
}

int main(int argc, char** argv) {
    if (argc < 2) return std::fprintf(stderr, "jolt_bench <cat outline>\n"), 1;
    RegisterDefaultAllocator();
    Factory::sInstance = new Factory();
    RegisterTypes();
    const Cat cat = read_cat(argv[1]);
    for (float limit : {0.f, 1.f / 60.f})
        for (bool born : {true, false}) {
            World w(-9.81f);
            if (limit > 0) { PhysicsSettings s = w.ps.GetPhysicsSettings(); s.mMaxPenetrationDistance = limit; w.ps.SetPhysicsSettings(s); }   // (1 m/s at 60 a second)
            std::vector<double> times;
            const auto [most, calm] = overlap(w, born, times);
            std::printf("jolt overlap %s (push-out limit %s): fastest %.1f m/s, calm after %d steps\n", born ? "born in one place" : "thrown into a pile", limit > 0 ? "1 m/s" : "as it comes", most, calm);
        }
    for (const char* name : {"pile", "grid", "rope", "cat", "cat2d", "pile2d"}) {
        uint64_t hashes[2];
        char report[256] = "";
        for (int run = 0; run < 2; run++) {
            World w(std::string(name).rfind("cat", 0) == 0 ? -14.f : -9.81f);
            w.flat = std::string(name).find("2d") != std::string::npos;
            std::vector<double> times;
            if (std::string(name).rfind("cat", 0) == 0) run_cat(w, cat, times);
            else {
                const int steps = std::string(name).rfind("pile", 0) == 0 ? pile(w) : std::string(name) == "grid" ? grid(w) : rope(w);
                for (int i = 0; i < steps; i++) {
                    const auto t0 = std::chrono::steady_clock::now();
                    w.step();
                    times.push_back(std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count());
                }
            }
            hashes[run] = w.hash();
            double sum = 0, most = 0;
            for (double t : times) sum += t, most = std::max(most, t);
            std::snprintf(report, sizeof report, "jolt %s: %zu steps, %.3f ms a step, most %.3f ms, end energy %.3f", name, times.size(), sum / times.size(), most, w.energy());
        }
        std::printf("%s, the same twice: %s\n", report, hashes[0] == hashes[1] ? "yes" : "NO");
    }
    return 0;
}
