// dawn_probe: Filament (WebGPU backend, Dawn) runs a compute shader that writes the vertices of TRIS triangles into a
// GPU buffer every frame, and Filament draws that same buffer as an ordinary Renderable. The CPU never writes that
// buffer: it sends 16 bytes a frame (the time and the count). Nothing is read back while playing.
//
// Needs Filament built with filament_dawn_compute.patch (the WebGPU backend's dispatchCompute is empty in 1.77.2).
// The compute program, its buffers and the dispatch go through the engine's own driver commands (private headers):
// Filament has no public way to them yet.
//
//   dawn_probe <material.filamat> [out dir for pictures]
//   TRIS=4096  W=1280 H=720  FRAMES=600
//   WARM=60    frames run (and waited for) before the timed ones: the pipelines are built there
//   SYNC=0     1: wait for the GPU after every frame (a second way to time a frame; never done when playing)
//   HZ=0       frames are started this many times a second (0: as fast as they can be queued)
//   COMPUTE=1  0: never (the buffer stays as WebGPU made it, all zero), 2: the first frame only
//   DRAW=1     0: the computed triangles are not in the scene
// Bottom row: eight triangles uploaded once from the CPU the usual way (VertexBuffer::setBufferAt), to see that the
// ordinary path is not disturbed.
#include <filament/BufferObject.h>
#include <filament/Camera.h>
#include <filament/Engine.h>
#include <filament/IndexBuffer.h>
#include <filament/Material.h>
#include <filament/MaterialInstance.h>
#include <filament/RenderableManager.h>
#include <filament/Renderer.h>
#include <filament/Scene.h>
#include <filament/SwapChain.h>
#include <filament/VertexBuffer.h>
#include <filament/View.h>
#include <filament/Viewport.h>

#include <backend/PixelBufferDescriptor.h>
#include <backend/Program.h>
#include <utils/EntityManager.h>

#include "details/BufferObject.h"
#include "details/Engine.h"

#define STB_IMAGE_WRITE_IMPLEMENTATION
#include <stb_image_write.h>

#include <algorithm>
#include <chrono>
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <string>
#include <thread>
#include <vector>

using namespace filament;
using utils::Entity;
using utils::EntityManager;

namespace {

// One invocation per vertex. Binding numbers: Filament's binding b is WebGPU's 2b (2b + 1 is a sampler's).
const char* const COMPUTE_WGSL = R"(
struct Params { time : f32, count : u32, tris : f32, pad : f32 };
struct Vertex { pos : vec4f, col : vec4f };
@group(0) @binding(0) var<uniform> params : Params;
@group(0) @binding(2) var<storage, read_write> verts : array<Vertex>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id : vec3u) {
    let i = id.x;
    if (i >= params.count) { return; }
    let tri = f32(i / 3u);
    let corner = f32(i % 3u);
    let t = params.time;
    let r0 = sqrt((tri + 0.5) / params.tris);
    let r = r0 * 0.78 * (1.0 + 0.08 * sin(t * 2.0 + r0 * 9.0));
    let a = tri * 2.39996323 + t * (0.25 + 0.9 * (1.0 - r0));
    let c = vec2f(r * cos(a), 0.12 + r * sin(a));
    let s = (0.9 / sqrt(params.tris)) * (1.0 + 0.5 * sin(t * 3.0 + tri * 0.37));
    let b = a * 3.0 + t * 2.0 + corner * 2.0943951;
    let h = r0 * 0.9 + t * 0.15;
    verts[i].pos = vec4f(c + s * vec2f(cos(b), sin(b)), 0.0, 1.0);
    verts[i].col = vec4f(0.5 + 0.5 * cos(6.2831853 * (h + vec3f(0.0, 0.33, 0.67))), 1.0);
}
)";

int envInt(const char* name, int fallback) {
    const char* v = std::getenv(name);
    return v ? std::atoi(v) : fallback;
}

double now() {
    return std::chrono::duration<double>(std::chrono::steady_clock::now().time_since_epoch()).count();
}

struct Shot {
    std::string path;
    int w, h;
};

}  // namespace

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: dawn_probe <material.filamat> [out dir]\n");
        return 2;
    }
    const std::string outDir = argc > 2 ? argv[2] : "";
    const int W = envInt("W", 1280), H = envInt("H", 720);
    const int tris = envInt("TRIS", 4096), frames = envInt("FRAMES", 600);
    const int computeMode = envInt("COMPUTE", 1), drawOn = envInt("DRAW", 1);
    const int warm = envInt("WARM", 60), syncEach = envInt("SYNC", 0), hz = envInt("HZ", 0);
    const uint32_t count = uint32_t(tris) * 3;

    Engine* engine = Engine::Builder().backend(Engine::Backend::WEBGPU).build();
    if (!engine || engine->getBackend() != Engine::Backend::WEBGPU) {
        std::fprintf(stderr, "FAIL: no WebGPU engine\n");
        return 1;
    }
    SwapChain* swapChain = engine->createSwapChain(uint32_t(W), uint32_t(H));
    Renderer* renderer = engine->createRenderer();
    renderer->setClearOptions({.clearColor = {0.02f, 0.02f, 0.03f, 1.0f}, .clear = true});
    Scene* scene = engine->createScene();
    View* view = engine->createView();
    view->setViewport({0, 0, uint32_t(W), uint32_t(H)});
    view->setScene(scene);
    Entity camEntity = EntityManager::get().create();
    Camera* camera = engine->createCamera(camEntity);
    const double aspect = double(W) / H;
    camera->setProjection(Camera::Projection::ORTHO, -aspect, aspect, -1.0, 1.0, -1.0, 1.0);
    view->setCamera(camera);

    std::ifstream file(argv[1], std::ios::binary);
    std::vector<char> package((std::istreambuf_iterator<char>(file)), std::istreambuf_iterator<char>());
    if (package.empty()) {
        std::fprintf(stderr, "FAIL: cannot read %s\n", argv[1]);
        return 1;
    }
    Material* material = Material::Builder().package(package.data(), package.size()).build(*engine);

    // The computed triangles: one buffer, 32 bytes a vertex (place, colour). The CPU never fills it.
    BufferObject* gpuVerts = BufferObject::Builder().size(count * 32).bindingType(BufferObject::BindingType::VERTEX).build(*engine);
    VertexBuffer* vb = VertexBuffer::Builder()
            .vertexCount(count).bufferCount(1).enableBufferObjects()
            .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT4, 0, 32)
            .attribute(VertexAttribute::COLOR, 0, VertexBuffer::AttributeType::FLOAT4, 16, 32)
            .build(*engine);
    vb->setBufferObjectAt(*engine, 0, gpuVerts);
    static std::vector<uint32_t> order;   // 0, 1, 2, ...: set once
    order.resize(count);
    for (uint32_t i = 0; i < count; ++i) order[i] = i;
    IndexBuffer* ib = IndexBuffer::Builder().indexCount(count).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(order.data(), order.size() * 4));
    Entity computed = EntityManager::get().create();
    RenderableManager::Builder(1)
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib, 0, count)
            .material(0, material->getDefaultInstance())
            .culling(false).castShadows(false).receiveShadows(false)
            .build(*engine, computed);
    if (drawOn) scene->addEntity(computed);

    // The usual path beside it: eight triangles, uploaded once.
    static float usual[8 * 3 * 8];
    for (int k = 0; k < 8; ++k) {
        const float cx = -1.4f + 0.4f * float(k), cy = -0.86f;
        const float rgb[3] = {0.25f + 0.1f * float(k), 0.9f - 0.1f * float(k), 0.6f};
        const float corner[3][2] = {{-0.12f, -0.08f}, {0.12f, -0.08f}, {0.0f, 0.1f}};
        for (int c = 0; c < 3; ++c) {
            float* v = usual + (k * 3 + c) * 8;
            v[0] = cx + corner[c][0]; v[1] = cy + corner[c][1]; v[2] = 0.0f; v[3] = 1.0f;
            v[4] = rgb[0]; v[5] = rgb[1]; v[6] = rgb[2]; v[7] = 1.0f;
        }
    }
    VertexBuffer* usualVb = VertexBuffer::Builder()
            .vertexCount(24).bufferCount(1)
            .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT4, 0, 32)
            .attribute(VertexAttribute::COLOR, 0, VertexBuffer::AttributeType::FLOAT4, 16, 32)
            .build(*engine);
    usualVb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(usual, sizeof(usual)));
    IndexBuffer* usualIb = IndexBuffer::Builder().indexCount(24).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    usualIb->setBuffer(*engine, IndexBuffer::BufferDescriptor(order.data(), 24 * 4));
    Entity usualEntity = EntityManager::get().create();
    RenderableManager::Builder(1)
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, usualVb, usualIb, 0, 24)
            .material(0, material->getDefaultInstance())
            .culling(false).castShadows(false).receiveShadows(false)
            .build(*engine, usualEntity);
    scene->addEntity(usualEntity);

    // The compute side, through the engine's driver commands. (COMPUTE=0 makes none of it: that runs on an unpatched
    // Filament too.)
    backend::DriverApi& driver = downcast(engine)->getDriverApi();
    using namespace backend;
    ProgramHandle computeProgram;
    DescriptorSetHandle set;
    BufferObjectHandle params;
    if (computeMode != 0) {
        Program program;
        program.shaderLanguage(ShaderLanguage::WGSL);
        program.diagnostics(utils::CString("dawn_probe_compute"),
                [](utils::CString const&, utils::io::ostream& out) -> utils::io::ostream& { return out; });
        program.shader(ShaderStage::COMPUTE, COMPUTE_WGSL, std::strlen(COMPUTE_WGSL) + 1);
        program.descriptorBindings(0, {
                {utils::CString("params"), DescriptorType::UNIFORM_BUFFER, 0},
                {utils::CString("verts"), DescriptorType::SHADER_STORAGE_BUFFER, 1}});
        computeProgram = driver.createProgram(std::move(program), utils::ImmutableCString{"dawn_probe_compute"});
        backend::DescriptorSetLayout layout;
        layout.label = utils::StaticString("dawn_probe_compute");
        layout.descriptors = {
                {DescriptorType::UNIFORM_BUFFER, ShaderStageFlags::COMPUTE, 0},
                {DescriptorType::SHADER_STORAGE_BUFFER, ShaderStageFlags::COMPUTE, 1}};
        DescriptorSetLayoutHandle layoutHandle = driver.createDescriptorSetLayout(std::move(layout), utils::ImmutableCString{"dawn_probe_layout"});
        set = driver.createDescriptorSet(layoutHandle, utils::ImmutableCString{"dawn_probe_set"});
        params = driver.createBufferObject(16, BufferObjectBinding::UNIFORM, BufferUsage::DYNAMIC, utils::ImmutableCString{"dawn_probe_params"});
        driver.updateDescriptorSetBuffer(set, 0, params, 0, 16);
        driver.updateDescriptorSetBuffer(set, 1, downcast(gpuVerts)->getHwHandle(), 0, count * 32);
    }

    std::vector<double> cpuMs;
    cpuMs.reserve(size_t(frames));
    int shotsAsked = 0;
    int toldToSkip = 0, late = 0;
    double begin = now();
    for (int f = -warm; f < frames; ++f) {
        if (f == 0) {
            engine->flushAndWait();
            begin = now();
        }
        if (hz > 0 && f >= 0) {
            const double due = begin + double(f) / hz;
            if (now() > due + 0.5 / hz) ++late;
            while (now() < due) std::this_thread::sleep_for(std::chrono::microseconds(200));
        }
        const double t0 = now();
        if (!renderer->beginFrame(swapChain) && f >= 0) ++toldToSkip;   // (it is drawn all the same)
        if (computeMode == 1 || (computeMode == 2 && f == -warm)) {
            struct { float time; uint32_t count; float tris; float pad; }* p = static_cast<decltype(p)>(std::malloc(16));
            *p = {float(f + warm) / 60.0f, count, float(tris), 0.0f};
            driver.updateBufferObject(params, BufferDescriptor(p, 16, [](void* b, size_t, void*) { std::free(b); }), 0);
            driver.bindDescriptorSet(set, 0, {});
            driver.dispatchCompute(computeProgram, {(count + 63) / 64, 1, 1});
            driver.bindDescriptorSet({}, 0, {});
        }
        renderer->render(view);
        if (!outDir.empty() && (f == 0 || f == 30 || f == 31 || f == 90)) {   // pictures only when asked for: not part of playing
            auto* shot = new Shot{outDir + "/dawn_probe_" + std::to_string(f) + ".png", W, H};
            auto* pixels = new uint8_t[size_t(W) * H * 4];
            ++shotsAsked;
            renderer->readPixels(0, 0, uint32_t(W), uint32_t(H),
                    PixelBufferDescriptor(pixels, size_t(W) * H * 4, PixelDataFormat::RGBA, PixelDataType::UBYTE,
                            [](void* buffer, size_t, void* user) {
                                Shot* s = static_cast<Shot*>(user);
                                stbi_write_png(s->path.c_str(), s->w, s->h, 4, buffer, s->w * 4);
                                delete[] static_cast<uint8_t*>(buffer);
                                delete s;
                            }, shot));
        }
        renderer->endFrame();
        if (f >= 0) cpuMs.push_back((now() - t0) * 1000.0);
        if (syncEach) engine->flushAndWait();
    }
    const double queued = now();
    engine->flushAndWait();
    const double end = now();

    std::sort(cpuMs.begin(), cpuMs.end());
    std::printf("dawn_probe: %d triangles (%u vertices), %dx%d, %d frames, compute %d, draw %d, pictures %d\n",
            tris, count, W, H, frames, computeMode, drawOn, shotsAsked);
    std::printf("  whole frame      %.3f ms  (%.0f fps; %s)\n", (end - begin) * 1000.0 / frames, frames / (end - begin),
            syncEach ? "the GPU waited for after every frame" : hz ? "frames started on the clock" : "all frames queued, one wait at the very end");
    if (hz) std::printf("  on the clock     %d Hz asked, %d of %d frames started late\n", hz, late, frames);
    std::printf("  told to skip     %d of %d frames (beginFrame said false)\n", toldToSkip, frames);
    std::printf("  CPU in the frame %.3f ms median, %.3f ms worst  (begin..end of frame on the app's thread)\n",
            cpuMs[cpuMs.size() / 2], cpuMs.back());
    std::printf("  final wait       %.1f ms once, after the last frame\n", (end - queued) * 1000.0);
    engine->destroy(&engine);
    return 0;
}
