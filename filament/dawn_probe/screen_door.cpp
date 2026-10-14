// See screen_door.h.
#include "screen_door.h"

#include <filament/Engine.h>
#include <filament/RenderTarget.h>
#include <filament/Texture.h>

#include <backend/PipelineState.h>
#include <backend/Program.h>

#include "details/Engine.h"
#include "details/RenderTarget.h"
#include "details/Texture.h"

#include <cstdlib>
#include <cstring>

namespace door {
using namespace filament;
using namespace filament::backend;

// Filament's full-screen triangle (its POSITION at location 0, from -1,-1 past the far corners), as uv from the top left.
static const char* const sVertex = R"(
struct Out { @builtin(position) position: vec4f, @location(0) uv: vec2f };
@vertex fn main(@location(0) p: vec4f) -> Out {
  var o: Out;
  o.position = vec4f(p.xy, 0.0, 1.0);
  o.uv = vec2f(0.5 * p.x + 0.5, 0.5 - 0.5 * p.y);
  return o;
}
)";

struct Screen {
    ProgramHandle program;
    DescriptorSetLayoutHandle layout;
    DescriptorSetHandle set;
    BufferObjectHandle knobs;
    uint32_t knobBytes = 0;
};

static DriverApi& driver(Engine* engine) { return downcast(engine)->getDriverApi(); }

Screen* screen(Engine* engine, const char* name, const char* fragment, uint32_t knobBytes) {
    DriverApi& d = driver(engine);
    auto* s = new Screen;
    s->knobBytes = (knobBytes + 15u) & ~15u;
    backend::Program p;
    p.shaderLanguage(ShaderLanguage::WGSL);
    p.diagnostics(utils::CString(name), [](utils::CString const&, utils::io::ostream& out) -> utils::io::ostream& { return out; });
    p.shader(ShaderStage::VERTEX, sVertex, std::strlen(sVertex) + 1);
    p.shader(ShaderStage::FRAGMENT, fragment, std::strlen(fragment) + 1);
    backend::Program::DescriptorBindingsInfo bindings = backend::Program::DescriptorBindingsInfo::with_capacity(2);
    backend::DescriptorSetLayout layout;
    layout.label = utils::CString(name);
    layout.descriptors = utils::FixedCapacityVector<DescriptorSetLayoutDescriptor>::with_capacity(2);
    bindings.push_back({utils::CString("picture"), DescriptorType::SAMPLER_2D_FLOAT, descriptor_binding_t(0)});
    layout.descriptors.push_back({DescriptorType::SAMPLER_2D_FLOAT, ShaderStageFlags::FRAGMENT, descriptor_binding_t(0)});
    if (s->knobBytes) {
        bindings.push_back({utils::CString("knobs"), DescriptorType::UNIFORM_BUFFER, descriptor_binding_t(1)});
        layout.descriptors.push_back({DescriptorType::UNIFORM_BUFFER, ShaderStageFlags::FRAGMENT, descriptor_binding_t(1)});
    }
    p.descriptorBindings(0, std::move(bindings));
    s->program = d.createProgram(std::move(p), utils::ImmutableCString{name});
    s->layout = d.createDescriptorSetLayout(std::move(layout), utils::ImmutableCString{name});
    s->set = d.createDescriptorSet(s->layout, utils::ImmutableCString{name});
    if (s->knobBytes) {
        s->knobs = d.createBufferObject(s->knobBytes, BufferObjectBinding::UNIFORM, BufferUsage::DYNAMIC, utils::ImmutableCString{name});
        d.updateDescriptorSetBuffer(s->set, descriptor_binding_t(1), s->knobs, 0, s->knobBytes);
    }
    return s;
}

void knobs(Engine* engine, Screen* s, const void* data, uint32_t bytes) {
    if (!s->knobBytes || !bytes) return;
    bytes = bytes < s->knobBytes ? bytes : s->knobBytes;
    void* copy = std::calloc(1, s->knobBytes);
    std::memcpy(copy, data, bytes);
    driver(engine).updateBufferObject(s->knobs, BufferDescriptor(copy, s->knobBytes, [](void* p, size_t, void*) { std::free(p); }), 0);
}

void draw(Engine* engine, Screen* s, Texture* from, RenderTarget* to, uint32_t w, uint32_t h) {
    DriverApi& d = driver(engine);
    SamplerParams linear{};
    linear.filterMag = SamplerMagFilter::LINEAR, linear.filterMin = SamplerMinFilter::LINEAR;
    linear.wrapS = SamplerWrapMode::CLAMP_TO_EDGE, linear.wrapT = SamplerWrapMode::CLAMP_TO_EDGE;
    d.updateDescriptorSetTexture(s->set, descriptor_binding_t(0), downcast(from)->getHwHandle(), linear);
    RenderPassParams params{};
    params.viewport = {0, 0, w, h};
    params.flags.discardStart = TargetBufferFlags::ALL;
    PipelineState pipeline{};
    pipeline.program = s->program;
    pipeline.pipelineLayout.setLayout = {s->layout};
    pipeline.rasterState.culling = CullingMode::NONE;
    pipeline.rasterState.depthWrite = false;
    pipeline.rasterState.depthFunc = RasterState::DepthFunc::A;
    pipeline.rasterState.colorWrite = true;
    d.bindDescriptorSet(s->set, 0, {});
    d.beginRenderPass(to ? downcast(to)->getHwHandle() : downcast(engine)->getDefaultRenderTarget(), params);   // (none: the screen, as the frame's swap chain is)
    d.draw(pipeline, downcast(engine)->getFullScreenRenderPrimitive(), 0, 3, 1);
    d.endRenderPass();
    d.bindDescriptorSet({}, 0, {});
}

void drop(Engine* engine, Screen* s) {
    DriverApi& d = driver(engine);
    if (s->knobBytes) d.destroyBufferObject(s->knobs);
    d.destroyDescriptorSet(s->set);
    d.destroyDescriptorSetLayout(s->layout);
    d.destroyProgram(s->program);
    delete s;
}
}  // namespace door
