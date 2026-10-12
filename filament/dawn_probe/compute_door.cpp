// See compute_door.h.
#include "compute_door.h"

#include <filament/BufferObject.h>
#include <filament/Engine.h>

#include <backend/Program.h>

#include "details/BufferObject.h"
#include "details/Engine.h"

#include <cstdlib>
#include <cstring>
#include <vector>

namespace door {
using namespace filament;
using namespace filament::backend;

struct Program {
    ProgramHandle handle;
    DescriptorSetLayoutHandle layout;
    std::vector<Kind> slots;
};
struct Buffer {
    BufferObjectHandle handle;
    uint32_t bytes;
};
struct Set {
    DescriptorSetHandle handle;
};

static DriverApi& driver(Engine* engine) { return downcast(engine)->getDriverApi(); }

Program* program(Engine* engine, const char* name, const char* wgsl, const Kind* slots, int count) {
    backend::Program p;
    p.shaderLanguage(ShaderLanguage::WGSL);
    p.diagnostics(utils::CString(name), [](utils::CString const&, utils::io::ostream& out) -> utils::io::ostream& { return out; });
    p.shader(ShaderStage::COMPUTE, wgsl, std::strlen(wgsl) + 1);
    backend::Program::DescriptorBindingsInfo bindings = backend::Program::DescriptorBindingsInfo::with_capacity(size_t(count));
    backend::DescriptorSetLayout layout;
    layout.label = utils::CString(name);
    layout.descriptors = utils::FixedCapacityVector<DescriptorSetLayoutDescriptor>::with_capacity(size_t(count));
    for (int i = 0; i < count; ++i) {
        const DescriptorType type = slots[i] == Kind::UNIFORM ? DescriptorType::UNIFORM_BUFFER : DescriptorType::SHADER_STORAGE_BUFFER;
        bindings.push_back({utils::CString("slot"), type, descriptor_binding_t(i)});
        layout.descriptors.push_back({type, ShaderStageFlags::COMPUTE, descriptor_binding_t(i)});
    }
    p.descriptorBindings(0, std::move(bindings));
    auto* out = new Program;
    out->handle = driver(engine).createProgram(std::move(p), utils::ImmutableCString{name});
    out->layout = driver(engine).createDescriptorSetLayout(std::move(layout), utils::ImmutableCString{name});
    out->slots.assign(slots, slots + count);
    return out;
}

Buffer* buffer(Engine* engine, Kind kind, uint32_t bytes) {
    bytes = (bytes + 15u) & ~15u;
    return new Buffer{driver(engine).createBufferObject(bytes, kind == Kind::UNIFORM ? BufferObjectBinding::UNIFORM : BufferObjectBinding::SHADER_STORAGE,
                                                        BufferUsage::DYNAMIC, utils::ImmutableCString{"door"}),
                      bytes};
}

Buffer* buffer(Engine*, BufferObject* vertices) {
    return new Buffer{downcast(vertices)->getHwHandle(), uint32_t(vertices->getByteCount())};
}

void write(Engine* engine, Buffer* b, const void* data, uint32_t bytes, uint32_t at) {
    void* copy = std::malloc(bytes);
    std::memcpy(copy, data, bytes);
    driver(engine).updateBufferObject(b->handle, BufferDescriptor(copy, bytes, [](void* p, size_t, void*) { std::free(p); }), at);
}

Set* set(Engine* engine, Program* p, Buffer* const* buffers) {
    auto* out = new Set{driver(engine).createDescriptorSet(p->layout, utils::ImmutableCString{"door"})};
    for (size_t i = 0; i < p->slots.size(); ++i)
        driver(engine).updateDescriptorSetBuffer(out->handle, descriptor_binding_t(i), buffers[i]->handle, 0, buffers[i]->bytes);
    return out;
}

void run(Engine* engine, Program* p, Set* s, uint32_t groupsX, uint32_t groupsY, uint32_t groupsZ) {
    DriverApi& d = driver(engine);
    d.bindDescriptorSet(s->handle, 0, {});
    d.dispatchCompute(p->handle, {groupsX, groupsY, groupsZ});
    d.bindDescriptorSet({}, 0, {});
}

void read(Engine* engine, Buffer* b, uint32_t at, uint32_t bytes, void* into) {
    driver(engine).readBufferSubData(b->handle, at, bytes, BufferDescriptor(into, bytes));
    engine->flushAndWait();
}
}  // namespace door
