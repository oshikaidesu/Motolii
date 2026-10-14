// Things of the document falling and striking one another: Jolt (physics_bench chose it), behind a door that shows
// none of its own types. Every picture of the stage is a step of 1/60 s from the document's beginning, kept once made,
// so going back in time is reading what was kept, and the same document moves the same way every time.
#pragma once
#include <functional>
#include <memory>
#include <vector>

struct PhysThing {
    bool still = false;          // a wall: where the document puts it (moved only by its keys)
    bool flat = false;           // held to its own plane, turning only about the eye's line (Jolt's Plane2D)
    bool ball = false;           // a ball of radius half[0]; else a box of half sizes half
    float half[3] = {0.5f, 0.5f, 0.5f};
    float bounce = 0.3f, friction = 0.5f;
    double until = 0.0;          // before this moment it goes as its keys say; from it the world has it (with the speed it had)
};
struct PhysPose { float at[3] = {0, 0, 0}, turn[4] = {0, 0, 0, 1}; };   // where its middle is, and how it is turned (x y z w)

// Where the document puts a thing's middle at a moment, and how it is turned (its keys, as the stage reads them).
using PhysKeyed = std::function<PhysPose(int index, double at)>;

class PhysWorld {
public:
    PhysWorld(const std::vector<PhysThing>& things, float gravity, PhysKeyed keyed);
    ~PhysWorld();
    // Each thing at picture n (n / 60 s from the beginning): stepped there once, then kept.
    PhysPose pose(int index, int picture);
private:
    struct Own;
    std::unique_ptr<Own> own;
};
