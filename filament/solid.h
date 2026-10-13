// solid.h: a 3D file (glTF: .glb, .gltf) in the scene. All of it is Filament's own: its loader (gltfio) reads the
// file, its ready-made materials for glTF give the looks the file names (lit by the stage's room), and the file's own
// movement, if it has one, goes on as the stage's time does (gltfio's Animator).
#include <gltfio/Animator.h>
#include <gltfio/AssetLoader.h>
#include <gltfio/FilamentAsset.h>
#include <gltfio/ResourceLoader.h>
#include <gltfio/TextureProvider.h>
#include <gltfio/materials/uberarchive.h>

struct Solid {
    gltfio::FilamentAsset* made = nullptr;
    mat4f fit;             // the file's own size and middle brought to: 1 tall, its middle at the place
    float seconds = 0.f;   // how long its own movement is (none: 0)
    float starts = 0.f;    // how far into it it is when the stage's time is 0
};

static gltfio::AssetLoader* solid_loader(Stage& s) {
    static gltfio::AssetLoader* loader = gltfio::AssetLoader::create({s.engine, gltfio::createUbershaderProvider(s.engine, UBERARCHIVE_DEFAULT_DATA, UBERARCHIVE_DEFAULT_SIZE)});
    return loader;
}

// Where it is (and how large: a place may scale).
static void solid_to(Stage& s, Solid* d, const mat4f& place) {
    auto& tm = s.engine->getTransformManager();
    tm.setTransform(tm.getInstance(d->made->getRoot()), place * d->fit);
}

// The 3D file put in the scene, `high` tall, its middle at `place`. nullptr: it could not be read.
static Solid* solid(Stage& s, const char* file, const mat4f& place, float high) {
    const std::vector<uint8_t> bytes = load_file(file);
    Solid* d = new Solid;
    d->made = bytes.empty() ? nullptr : solid_loader(s)->createAsset(bytes.data(), (uint32_t)bytes.size());
    if (!d->made) return nullptr;
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wdeprecated-declarations"   // (the file's own place, for a .gltf whose pictures lie beside it: gltfio's way until mid 2027)
    gltfio::ResourceLoader pictures({s.engine, file, true});
#pragma clang diagnostic pop
    static gltfio::TextureProvider* reader = gltfio::createStbProvider(s.engine);
    pictures.addTextureProvider("image/png", reader);
    pictures.addTextureProvider("image/jpeg", reader);
    if (!pictures.loadResources(d->made)) return nullptr;
    d->made->releaseSourceData();
    const Aabb box = d->made->getBoundingBox();
    d->fit = mat4f::scaling(high / std::max(box.max.y - box.min.y, 1e-6f)) * mat4f::translation(-0.5f * (box.min + box.max));
    solid_to(s, d, place);
    s.scene->addEntities(d->made->getRenderableEntities(), d->made->getRenderableEntityCount());
    gltfio::Animator* moves = d->made->getInstance()->getAnimator();
    if (moves->getAnimationCount()) {   // (its own movement: round and round, as the stage's time goes)
        d->seconds = moves->getAnimationDuration(0);
        s.playing[d] = [d, moves](double now) {
            moves->applyAnimation(0, std::fmod((float)now + d->starts, d->seconds));
            moves->updateBoneMatrices();
        };
    }
    return d;
}

// Whether it is in the picture; and taken off the stage for good.
static void solid_seen(Stage& s, Solid* d, bool in) {
    if (in) s.scene->addEntities(d->made->getRenderableEntities(), d->made->getRenderableEntityCount());
    else s.scene->removeEntities(d->made->getRenderableEntities(), d->made->getRenderableEntityCount());
}
static void solid_off(Stage& s, Solid* d) {
    s.playing.erase(d);
    solid_seen(s, d, false);
    solid_loader(s)->destroyAsset(d->made);
    delete d;
}
