// tsl_on_dawn.cpp: does a three.js TSL effect, turned to WGSL by three.js's own builder, draw on Dawn (the Dawn that
// Filament's WebGPU backend is built with)? Everything three.js would give the GPU is read from a folder written by
// three.js's own code (WGSLNodeBuilder's shaders, each bind group's uniforms as three packs them, the plane's arrays:
// reports/shaderlab_origins.md), and drawn here the plain WebGPU way, with the shaders' own layout ("auto"), as Dawn's
// samples/HelloTriangle.cpp draws. Device and reading back as tvg_on_dawn.cpp.
//   ./dawn_probe/tsl_on_dawn.sh <folder> <width> <height>   (folder: vert.wgsl frag.wgsl bindings.txt g*b*.bin
//   position.f32 uv.f32 index.u16 picture.rgba; writes drawn.rgba there)
#include <dawn/webgpu_cpp.h>
#include <cstdio>
#include <fstream>
#include <sstream>
#include <string>
#include <vector>

static int errors = 0;

static std::string slurp(const std::string& path) {
    std::ifstream f(path, std::ios::binary);
    std::stringstream s;
    s << f.rdbuf();
    return s.str();
}

int main(int argc, char** argv) {
    if (argc < 4) { std::printf("usage: tsl_on_dawn <folder> <width> <height>\n"); return 2; }
    const std::string dir = std::string(argv[1]) + "/";
    const uint32_t W = (uint32_t)std::atoi(argv[2]), H = (uint32_t)std::atoi(argv[3]);
    const wgpu::InstanceFeatureName wanted[] = {wgpu::InstanceFeatureName::TimedWaitAny};
    wgpu::InstanceDescriptor instance_wish{};
    instance_wish.requiredFeatureCount = 1;
    instance_wish.requiredFeatures = wanted;
    wgpu::Instance instance = wgpu::CreateInstance(&instance_wish);
    wgpu::Adapter adapter;
    wgpu::RequestAdapterOptions any{};
    instance.WaitAny(instance.RequestAdapter(&any, wgpu::CallbackMode::WaitAnyOnly,
                             [&](wgpu::RequestAdapterStatus, wgpu::Adapter got, wgpu::StringView) { adapter = got; }), UINT64_MAX);
    if (!adapter) { std::printf("FAIL no adapter\n"); return 1; }
    wgpu::DeviceDescriptor device_wish{};
    device_wish.SetUncapturedErrorCallback([](const wgpu::Device&, wgpu::ErrorType, wgpu::StringView said) {
        ++errors;
        std::printf("DEVICE ERROR: %.*s\n", (int)said.length, said.data);
    });
    wgpu::Device device;
    instance.WaitAny(adapter.RequestDevice(&device_wish, wgpu::CallbackMode::WaitAnyOnly,
                             [&](wgpu::RequestDeviceStatus, wgpu::Device got, wgpu::StringView) { device = got; }), UINT64_MAX);
    if (!device) { std::printf("FAIL no device\n"); return 1; }
    wgpu::Queue queue = device.GetQueue();

    // the shaders, as three.js's builder wrote them
    const std::string vert = slurp(dir + "vert.wgsl"), frag = slurp(dir + "frag.wgsl");
    const auto module = [&](const std::string& code) {
        wgpu::ShaderSourceWGSL wgsl{};
        wgsl.code = code.c_str();
        wgpu::ShaderModuleDescriptor made{};
        made.nextInChain = &wgsl;
        return device.CreateShaderModule(&made);
    };
    wgpu::ShaderModule vs = module(vert), fs = module(frag);

    // the picture it reads, and the one it draws into
    wgpu::TextureDescriptor like{};
    like.size = {W, H, 1};
    like.format = wgpu::TextureFormat::RGBA8Unorm;
    like.usage = wgpu::TextureUsage::TextureBinding | wgpu::TextureUsage::CopyDst;
    wgpu::Texture picture = device.CreateTexture(&like);
    const std::string pixels = slurp(dir + "picture.rgba");
    if (pixels.size() != (size_t)W * H * 4) { std::printf("FAIL picture.rgba is %zu bytes, not %u x %u x 4\n", pixels.size(), W, H); return 1; }
    wgpu::TexelCopyTextureInfo into{};
    into.texture = picture;
    wgpu::TexelCopyBufferLayout rows{};
    rows.bytesPerRow = W * 4;
    rows.rowsPerImage = H;
    const wgpu::Extent3D all{W, H, 1};
    queue.WriteTexture(&into, pixels.data(), pixels.size(), &rows, &all);
    like.usage = wgpu::TextureUsage::RenderAttachment | wgpu::TextureUsage::CopySrc;
    wgpu::Texture drawn = device.CreateTexture(&like);

    const auto buffer = [&](const std::string& bytes, wgpu::BufferUsage usage) {
        wgpu::BufferDescriptor room{};
        room.size = (bytes.size() + 3) & ~size_t(3);
        room.usage = usage | wgpu::BufferUsage::CopyDst;
        wgpu::Buffer b = device.CreateBuffer(&room);
        std::string padded = bytes;
        padded.resize(room.size, '\0');
        queue.WriteBuffer(b, 0, padded.data(), padded.size());
        return b;
    };
    // the mesh's arrays, one per vertex input in the order the builder gave them locations: attributes.txt, "<name> <type>"
    // a line, each array in <name>.f32
    const std::string index = slurp(dir + "index.u16");
    wgpu::Buffer indices = buffer(index, wgpu::BufferUsage::Index);
    std::vector<wgpu::Buffer> arrays;
    std::vector<wgpu::VertexAttribute> ats;
    std::vector<wgpu::VertexBufferLayout> layouts;
    std::istringstream inputs(slurp(dir + "attributes.txt"));
    for (std::string name, type; inputs >> name >> type;) {
        const uint32_t n = type == "vec4" ? 4 : type == "vec3" ? 3 : type == "vec2" ? 2 : 1;
        arrays.push_back(buffer(slurp(dir + name + ".f32"), wgpu::BufferUsage::Vertex));
        wgpu::VertexAttribute at{};
        at.format = n == 4 ? wgpu::VertexFormat::Float32x4 : n == 3 ? wgpu::VertexFormat::Float32x3 : n == 2 ? wgpu::VertexFormat::Float32x2 : wgpu::VertexFormat::Float32;
        at.shaderLocation = (uint32_t)ats.size();
        ats.push_back(at);
    }
    for (size_t i = 0; i < ats.size(); ++i) {
        wgpu::VertexBufferLayout l{};
        l.arrayStride = 4 * (ats[i].format == wgpu::VertexFormat::Float32x4 ? 4 : ats[i].format == wgpu::VertexFormat::Float32x3 ? 3 : ats[i].format == wgpu::VertexFormat::Float32x2 ? 2 : 1);
        l.attributeCount = 1, l.attributes = &ats[i];
        layouts.push_back(l);
    }

    wgpu::ColorTargetState target{};
    target.format = wgpu::TextureFormat::RGBA8Unorm;
    wgpu::FragmentState fragment{};
    fragment.module = fs;
    fragment.targetCount = 1;
    fragment.targets = &target;
    wgpu::RenderPipelineDescriptor descriptor{};
    descriptor.vertex.module = vs;
    descriptor.vertex.bufferCount = layouts.size();
    descriptor.vertex.buffers = layouts.data();
    descriptor.fragment = &fragment;
    wgpu::RenderPipeline pipeline = device.CreateRenderPipeline(&descriptor);

    // the bind groups as three.js's bindings say: "<group> <binding> uniform <bytes>" (its g<group>b<binding>.bin) or "texture"
    std::vector<std::vector<wgpu::BindGroupEntry>> groups;
    std::vector<wgpu::Buffer> keep;
    std::istringstream plan(slurp(dir + "bindings.txt"));
    for (uint32_t g, b; plan >> g >> b;) {
        std::string kind;
        size_t bytes = 0;
        plan >> kind >> bytes;
        if (groups.size() <= g) groups.resize(g + 1);
        wgpu::BindGroupEntry e{};
        e.binding = b;
        if (kind == "uniform") {
            keep.push_back(buffer(slurp(dir + "g" + std::to_string(g) + "b" + std::to_string(b) + ".bin"), wgpu::BufferUsage::Uniform));
            e.buffer = keep.back();
        } else if (kind == "texture") {
            e.textureView = picture.CreateView();
        } else if (kind == "sampler") {   // "<mag> <min> <wrapS> <wrapT>" as three.js's constants: 1003 nearest, 1006 linear; 1000 repeat, 1001 clamp, 1002 mirror
            int mag, min, ws, wt;
            plan >> mag >> min >> ws >> wt;
            const auto filter = [](int f) { return f == 1003 || f == 1004 || f == 1005 ? wgpu::FilterMode::Nearest : wgpu::FilterMode::Linear; };
            const auto wrap = [](int w) { return w == 1000 ? wgpu::AddressMode::Repeat : w == 1002 ? wgpu::AddressMode::MirrorRepeat : wgpu::AddressMode::ClampToEdge; };
            wgpu::SamplerDescriptor how{};
            how.magFilter = filter(mag), how.minFilter = filter(min), how.addressModeU = wrap(ws), how.addressModeV = wrap(wt);
            e.sampler = device.CreateSampler(&how);
        } else { std::printf("FAIL a binding of kind %s\n", kind.c_str()); return 1; }
        groups[g].push_back(e);
    }
    std::vector<wgpu::BindGroup> made;
    for (uint32_t g = 0; g < groups.size(); ++g) {
        wgpu::BindGroupDescriptor bg{};
        bg.layout = pipeline.GetBindGroupLayout(g);
        bg.entryCount = groups[g].size();
        bg.entries = groups[g].data();
        made.push_back(device.CreateBindGroup(&bg));
    }

    wgpu::RenderPassColorAttachment color{};
    color.view = drawn.CreateView();
    color.loadOp = wgpu::LoadOp::Clear;
    color.storeOp = wgpu::StoreOp::Store;
    color.clearValue = {0, 0, 0, 1};
    wgpu::RenderPassDescriptor pass_wish{};
    pass_wish.colorAttachmentCount = 1;
    pass_wish.colorAttachments = &color;
    wgpu::CommandEncoder coder = device.CreateCommandEncoder();
    wgpu::RenderPassEncoder pass = coder.BeginRenderPass(&pass_wish);
    pass.SetPipeline(pipeline);
    for (uint32_t g = 0; g < made.size(); ++g) pass.SetBindGroup(g, made[g]);
    for (uint32_t i = 0; i < arrays.size(); ++i) pass.SetVertexBuffer(i, arrays[i]);
    pass.SetIndexBuffer(indices, wgpu::IndexFormat::Uint16);
    pass.DrawIndexed((uint32_t)(index.size() / 2));
    pass.End();

    // read the texture back (a row copied to a buffer takes a multiple of 256 bytes: WebGPU's copy rule)
    const uint32_t stride = (W * 4 + 255) & ~255u;
    wgpu::BufferDescriptor room{};
    room.size = stride * H;
    room.usage = wgpu::BufferUsage::CopyDst | wgpu::BufferUsage::MapRead;
    wgpu::Buffer back = device.CreateBuffer(&room);
    wgpu::TexelCopyTextureInfo from{};
    from.texture = drawn;
    wgpu::TexelCopyBufferInfo to{};
    to.buffer = back;
    to.layout.bytesPerRow = stride;
    to.layout.rowsPerImage = H;
    coder.CopyTextureToBuffer(&from, &to, &all);
    wgpu::CommandBuffer commands = coder.Finish();
    queue.Submit(1, &commands);
    bool mapped = false;
    instance.WaitAny(back.MapAsync(wgpu::MapMode::Read, 0, room.size, wgpu::CallbackMode::WaitAnyOnly,
                             [&](wgpu::MapAsyncStatus how, wgpu::StringView) { mapped = how == wgpu::MapAsyncStatus::Success; }), UINT64_MAX);
    if (!mapped) { std::printf("FAIL read back\n"); return 1; }
    const char* got = (const char*)back.GetConstMappedRange(0, room.size);
    std::ofstream drawn_file(dir + "drawn.rgba", std::ios::binary);
    for (uint32_t y = 0; y < H; ++y) drawn_file.write(got + (size_t)y * stride, W * 4);
    std::printf("%s: %u x %u drawn, device errors %d\n", errors ? "FAIL" : "PASS", W, H, errors);
    return errors ? 1 : 0;
}
