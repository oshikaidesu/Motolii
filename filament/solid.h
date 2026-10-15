// solid.h: a 3D file (glTF: .glb, .gltf) in the scene. All of it is Filament's own: its loader (gltfio) reads the
// file, its ready-made materials for glTF give the looks the file names (lit by the stage's room), and the file's own
// movement, if it has one, goes on as the stage's time does (gltfio's Animator).
#include <filament/Color.h>
#include <filesystem>
#include <gltfio/Animator.h>
#include <gltfio/AssetLoader.h>
#include <gltfio/FilamentAsset.h>
#include <gltfio/ResourceLoader.h>
#include <gltfio/TextureProvider.h>
#include <gltfio/materials/uberarchive.h>
#include "third_party/filament_src/third_party/cgltf/cgltf.h"   // (the reader of glTF that Filament's own loader is built on: already in its library)

struct Solid {
    gltfio::FilamentAsset* made = nullptr;
    mat4f fit;             // the file's own size and middle brought to: 1 tall, its middle at the place
    float seconds = 0.f;   // how long its own movement is (none: 0)
    float starts = 0.f;    // how far into it it is when the stage's time is 0
    std::vector<std::string> moves;   // the names of its own movements (gltfio's Animator: a file may have several)
};

// A file's movement at a moment of its own: the one named (none, or not one of its own: its first), round and round.
static void solid_at(Solid* d, const std::string& name, double seconds) {
    gltfio::Animator* moves = d->made->getInstance()->getAnimator();
    if (!moves->getAnimationCount()) return;
    size_t which = 0;
    for (size_t i = 0; i < d->moves.size(); ++i) if (d->moves[i] == name) which = i;
    const float lasts = moves->getAnimationDuration(which);
    if (lasts <= 0.f) return;
    moves->applyAnimation(which, (float)(seconds - lasts * std::floor(seconds / lasts)));
    moves->updateBoneMatrices();
}

static gltfio::AssetLoader* solid_loader(Stage& s) {
    if (!s.solids) s.solids = gltfio::AssetLoader::create({s.engine, gltfio::createUbershaderProvider(s.engine, UBERARCHIVE_DEFAULT_DATA, UBERARCHIVE_DEFAULT_SIZE)});
    return (gltfio::AssetLoader*)s.solids;
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
    gltfio::ResourceLoader pictures({.engine = s.engine, .normalizeSkinningWeights = true});
    std::vector<std::vector<uint8_t>> beside(d->made->getResourceUriCount());   // (a .gltf's parts in files beside it: read and handed over, as gltfio asks)
    for (size_t i = 0; i < beside.size(); ++i) {
        const char* part = d->made->getResourceUris()[i];
        beside[i] = load_file((std::filesystem::path(file).parent_path() / part).string().c_str());
        pictures.addResourceData(part, gltfio::ResourceLoader::BufferDescriptor(beside[i].data(), beside[i].size()));
    }
    if (!s.solidPictures) s.solidPictures = gltfio::createStbProvider(s.engine);
    gltfio::TextureProvider* reader = (gltfio::TextureProvider*)s.solidPictures;
    pictures.addTextureProvider("image/png", reader);
    pictures.addTextureProvider("image/jpeg", reader);
    if (!pictures.loadResources(d->made)) return nullptr;
    d->made->releaseSourceData();
    const Aabb box = d->made->getBoundingBox();
    d->fit = mat4f::scaling(high / std::max(box.max.y - box.min.y, 1e-6f)) * mat4f::translation(-0.5f * (box.min + box.max));
    solid_to(s, d, place);
    s.scene->addEntities(d->made->getRenderableEntities(), d->made->getRenderableEntityCount());
    gltfio::Animator* moves = d->made->getInstance()->getAnimator();
    for (size_t i = 0; i < moves->getAnimationCount(); ++i) {   // (its movements by name; one with none, by its number)
        const char* name = moves->getAnimationName(i);
        std::string word = name && *name ? name : "move" + std::to_string(i + 1);
        std::replace(word.begin(), word.end(), ' ', '_');
        d->moves.push_back(word);
    }
    if (moves->getAnimationCount()) {   // (its own movement: round and round, as the stage's time goes)
        d->seconds = moves->getAnimationDuration(0);
        s.playing[d] = [d, moves](double now) {
            moves->applyAnimation(0, std::fmod((float)now + d->starts, d->seconds));
            moves->updateBoneMatrices();
        };
    }
    return d;
}

// A 3D file's shape alone, as a mesh of the kit: every triangle of it where the file puts it, brought to 1 tall with
// its middle at the middle (as `solid` shows it). For a look of the kit's own that draws a shape more than once
// (glass). It stands as the file has it at rest: a movement of its own is not gone through. nullptr: not read.
static Mesh* solid_mesh(const char* file) {
    cgltf_options how{};
    cgltf_data* read = nullptr;
    if (cgltf_parse_file(&how, file, &read) != cgltf_result_success) return nullptr;
    if (cgltf_load_buffers(&how, read, file) != cgltf_result_success) return cgltf_free(read), nullptr;
    Mesh* m = new Mesh;
    for (cgltf_size n = 0; n < read->nodes_count; ++n) {
        const cgltf_node& node = read->nodes[n];
        if (!node.mesh) continue;
        float to[16];
        cgltf_node_transform_world(&node, to);
        const mat4f world(float4{to[0], to[1], to[2], to[3]}, float4{to[4], to[5], to[6], to[7]}, float4{to[8], to[9], to[10], to[11]}, float4{to[12], to[13], to[14], to[15]});
        for (cgltf_size p = 0; p < node.mesh->primitives_count; ++p) {
            const cgltf_primitive& part = node.mesh->primitives[p];
            const cgltf_accessor* places = nullptr;
            const cgltf_accessor* ways = nullptr;
            for (cgltf_size a = 0; a < part.attributes_count; ++a) {
                if (part.attributes[a].type == cgltf_attribute_type_position) places = part.attributes[a].data;
                if (part.attributes[a].type == cgltf_attribute_type_normal) ways = part.attributes[a].data;
            }
            if (part.type != cgltf_primitive_type_triangles || !places) continue;
            const uint32_t first = (uint32_t)m->pos.size();
            for (cgltf_size v = 0; v < places->count; ++v) {
                float at[3] = {0, 0, 0}, faces[3] = {0, 0, 1};
                cgltf_accessor_read_float(places, v, at, 3);
                if (ways) cgltf_accessor_read_float(ways, v, faces, 3);
                m->pos.push_back((world * float4{at[0], at[1], at[2], 1.f}).xyz);
                m->nrm.push_back(normalize((world * float4{faces[0], faces[1], faces[2], 0.f}).xyz));
                m->uv.push_back(float2{0.f});
            }
            const cgltf_size corners = part.indices ? part.indices->count : places->count;
            for (cgltf_size i = 0; i < corners; ++i) m->idx.push_back(first + (uint32_t)(part.indices ? cgltf_accessor_read_index(part.indices, i) : i));
            if (!ways)   // (a file that does not say which way its faces look: each corner looks the way its triangles do)
                for (size_t i = m->idx.size() - corners; i + 2 < m->idx.size(); i += 3) {
                    const float3 flat = normalize(cross(m->pos[m->idx[i + 1]] - m->pos[m->idx[i]], m->pos[m->idx[i + 2]] - m->pos[m->idx[i]]));
                    for (int k = 0; k < 3; ++k) m->nrm[m->idx[i + (size_t)k]] = flat;
                }
        }
    }
    cgltf_free(read);
    if (m->idx.empty()) return delete m, nullptr;
    float3 low = m->pos[0], high = m->pos[0];
    for (const float3& p : m->pos) low = min(low, p), high = max(high, p);
    const float fit = 1.f / std::max(high.y - low.y, 1e-6f);
    for (float3& p : m->pos) p = (p - 0.5f * (low + high)) * fit;
    return m;
}

// A plain look in one colour (floor_lit.mat: lit by the stage's room); and every face of a file's shape given it, the
// looks the file names set aside. Its surface, Filament's own: how metal, how rough, how much it reflects, a clear coat
// over it (how much, how rough), and how much it lights itself (on the picture: 1 its own colour, more brighter).
// Filament's shading model of it: lit (floor_lit.mat), cloth (solid_cloth.mat) or subsurface (solid_subsurface.mat), each
// told only what it has.
struct Surface { float metallic = 0.08f, roughness = 0.6f, reflectance = 0.5f, coat = 0.f, coatRoughness = 0.f, glow = 0.f, sheen = 0.f, anisotropy = 0.f, thickness = 0.5f; };
static const char* surface_model(const std::string& shading) { return shading == "cloth" ? "solid_cloth.filamat" : shading == "subsurface" ? "solid_subsurface.filamat" : "floor_lit.filamat"; }
static void plain(MaterialInstance* look, float3 colour, const Surface& is = {}) {
    const auto put = [&](const char* name, float v) { if (look->getMaterial()->hasParameter(name)) look->setParameter(name, v); };
    look->setParameter("color", float4{Color::toLinear<ACCURATE>(sRGBColor{colour}), 1.f});
    put("roughness", is.roughness);
    put("metalOff", is.metallic - 0.08f);
    put("reflectanceOff", is.reflectance - 0.5f);
    put("clearCoat", is.coat);
    put("clearCoatRoughness", is.coatRoughness);
    put("glowing", is.glow);
    put("sheen", is.sheen);
    put("anisotropy", is.anisotropy);
    put("thickness", is.thickness);
}
static void solid_fill(Stage& s, Solid* d, float3 colour, const Surface& is = {}, const char* model = "floor_lit.filamat") {
    Material* lit = material(s, model);
    if (!lit) return;
    MaterialInstance* look = lit->createInstance();
    plain(look, colour, is);
    auto& faces = s.engine->getRenderableManager();
    for (size_t i = 0; i < d->made->getRenderableEntityCount(); ++i) {
        const auto one = faces.getInstance(d->made->getRenderableEntities()[i]);
        for (size_t part = 0; part < faces.getPrimitiveCount(one); ++part) faces.setMaterialInstanceAt(one, part, look);
    }
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
