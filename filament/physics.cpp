// physics.h's door, Jolt behind it (built as physics_bench/jolt_bench.cpp is: cross-platform deterministic, one thread).
#include "physics.h"
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
#include <mutex>
using namespace JPH;

namespace {
constexpr ObjectLayer STILL = 0, MOVING = 1;
struct Pairs final : ObjectLayerPairFilter { bool ShouldCollide(ObjectLayer a, ObjectLayer b) const override { return a == MOVING || b == MOVING; } };
struct Broad final : BroadPhaseLayerInterface {
    uint GetNumBroadPhaseLayers() const override { return 2; }
    BroadPhaseLayer GetBroadPhaseLayer(ObjectLayer l) const override { return BroadPhaseLayer((BroadPhaseLayer::Type)l); }
#if defined(JPH_EXTERNAL_PROFILE) || defined(JPH_PROFILE_ENABLED)
    const char* GetBroadPhaseLayerName(BroadPhaseLayer) const override { return "layer"; }
#endif
};
struct ObjectVsBroad final : ObjectVsBroadPhaseLayerFilter { bool ShouldCollide(ObjectLayer a, BroadPhaseLayer b) const override { return a == MOVING || (BroadPhaseLayer::Type)b == MOVING; } };
void once() {
    static std::once_flag done;
    std::call_once(done, [] { RegisterDefaultAllocator(); Factory::sInstance = new Factory(); RegisterTypes(); });
}
PhysPose read(BodyInterface& bi, BodyID id) {
    PhysPose p;
    const RVec3 at = bi.GetCenterOfMassPosition(id);
    const Quat q = bi.GetRotation(id);
    p.at[0] = (float)at.GetX(), p.at[1] = (float)at.GetY(), p.at[2] = (float)at.GetZ();
    p.turn[0] = q.GetX(), p.turn[1] = q.GetY(), p.turn[2] = q.GetZ(), p.turn[3] = q.GetW();
    return p;
}
}

struct PhysWorld::Own {
    Pairs pairs; Broad broad; ObjectVsBroad ovb;
    TempAllocatorImpl temp{16 * 1024 * 1024};
    JobSystemSingleThreaded jobs{cMaxPhysicsJobs};
    PhysicsSystem ps;
    std::vector<PhysThing> things;
    std::vector<BodyID> bodies;
    std::vector<bool> handed;                      // (a keyed thing the world has taken over)
    PhysKeyed keyed;
    std::vector<std::vector<PhysPose>> kept;       // (picture by picture, every thing)
};

PhysWorld::PhysWorld(const std::vector<PhysThing>& things, float gravity, PhysKeyed keyed) : own((once(), new Own)) {   // (Jolt made ready before anything of it is made)
    Own& o = *own;
    o.things = things, o.keyed = std::move(keyed);
    o.ps.Init(4096, 0, 16384, 8192, o.broad, o.ovb, o.pairs);
    o.ps.SetGravity(Vec3(0, -gravity, 0));
    BodyInterface& bi = o.ps.GetBodyInterface();
    for (size_t i = 0; i < things.size(); ++i) {
        const PhysThing& t = things[i];
        const PhysPose p = o.keyed((int)i, 0.0);
        const Shape* shape = t.ball ? (Shape*)new SphereShape(std::max(t.half[0], 1e-3f))
                                    : (Shape*)new BoxShape(Vec3(std::max(t.half[0], 1e-3f), std::max(t.half[1], 1e-3f), std::max(t.half[2], 1e-3f)), std::min({0.05f, t.half[0], t.half[1], t.half[2]}));
        const bool keyedNow = t.still || t.until > 0.0;   // (a wall, or a thing still going as its keys say: moved by them, struck by nothing)
        BodyCreationSettings s(shape, RVec3(p.at[0], p.at[1], p.at[2]), Quat(p.turn[0], p.turn[1], p.turn[2], p.turn[3]).Normalized(),
                               keyedNow ? EMotionType::Kinematic : EMotionType::Dynamic, MOVING);
        s.mRestitution = t.bounce, s.mFriction = t.friction;
        s.mAllowDynamicOrKinematic = true;
        if (t.flat) s.mAllowedDOFs = EAllowedDOFs::Plane2D;
        o.bodies.push_back(bi.CreateAndAddBody(s, EActivation::Activate));
        o.handed.push_back(!keyedNow);
    }
    o.kept.push_back({});
    for (BodyID id : o.bodies) o.kept.back().push_back(read(bi, id));
}
PhysWorld::~PhysWorld() = default;

PhysPose PhysWorld::pose(int index, int picture) {
    Own& o = *own;
    picture = std::max(picture, 0);
    if (index < 0 || index >= (int)o.bodies.size()) return {};
    BodyInterface& bi = o.ps.GetBodyInterface();
    const float dt = 1.f / 60.f;
    while ((int)o.kept.size() <= picture) {
        const double at = (double)o.kept.size() / 60.0;   // (the moment this step ends)
        for (size_t i = 0; i < o.bodies.size(); ++i) {
            const PhysThing& t = o.things[i];
            if (o.handed[i]) continue;
            if (!t.still && at > t.until) {   // (its last key passed: the world has it now, with the speed its keys gave it)
                const PhysPose a = o.keyed((int)i, t.until - dt), b = o.keyed((int)i, t.until);
                bi.SetMotionType(o.bodies[i], EMotionType::Dynamic, EActivation::Activate);
                bi.SetLinearVelocity(o.bodies[i], Vec3(b.at[0] - a.at[0], b.at[1] - a.at[1], b.at[2] - a.at[2]) / dt);
                o.handed[i] = true;
                continue;
            }
            const PhysPose p = o.keyed((int)i, at);
            bi.MoveKinematic(o.bodies[i], RVec3(p.at[0], p.at[1], p.at[2]), Quat(p.turn[0], p.turn[1], p.turn[2], p.turn[3]).Normalized(), dt);
        }
        o.ps.Update(dt, 1, &o.temp, &o.jobs);
        o.kept.push_back({});
        for (BodyID id : o.bodies) o.kept.back().push_back(read(bi, id));
    }
    return o.kept[(size_t)picture][(size_t)index];
}
