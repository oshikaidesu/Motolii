// tvg_on_dawn.cpp: does ThorVG's WebGPU engine run on Dawn (the Dawn that Filament's WebGPU backend is built with)?
// A Dawn device is made, ThorVG is given that device and a texture of it, draws a red disc on clear, and the texture is
// read back and looked at. ThorVG here is its own source (third_party/thorvg_src, v1.1.2, unchanged) built against
// Dawn's webgpu.h instead of wgpu-native's (dawn_probe/tvg_on_dawn.sh).   ./dawn_probe/tvg_on_dawn.sh
#include <dawn/webgpu_cpp.h>
#include <thorvg.h>
#include <cstdio>
#include <vector>

static int errors = 0;

int main() {
    const uint32_t W = 256, H = 256;
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

    // the texture ThorVG draws into: one a renderer could read in a material afterwards
    wgpu::TextureDescriptor like{};
    like.size = {W, H, 1};
    like.format = wgpu::TextureFormat::BGRA8Unorm;   // (what ThorVG's last step writes: blue first; RGBA is refused by its pipeline)
    like.usage = wgpu::TextureUsage::RenderAttachment | wgpu::TextureUsage::TextureBinding | wgpu::TextureUsage::CopySrc;
    wgpu::Texture picture = device.CreateTexture(&like);

    if (tvg::Initializer::init(0) != tvg::Result::Success) { std::printf("FAIL thorvg init\n"); return 1; }
    tvg::WgCanvas* canvas = tvg::WgCanvas::gen();
    if (!canvas) { std::printf("FAIL no WgCanvas (ThorVG built without its WebGPU engine)\n"); return 1; }
    if (canvas->target(device.Get(), instance.Get(), picture.Get(), W, H, tvg::ColorSpace::ABGR8888S, 1) != tvg::Result::Success) { std::printf("FAIL target\n"); return 1; }
    tvg::Shape* disc = tvg::Shape::gen();
    disc->appendCircle(128, 128, 80, 80);
    disc->fill(255, 0, 0, 255);
    canvas->add(disc);
    const bool drawn = canvas->update() == tvg::Result::Success && canvas->draw(true) == tvg::Result::Success && canvas->sync() == tvg::Result::Success;
    if (!drawn) { std::printf("FAIL draw\n"); return 1; }

    // read the texture back
    wgpu::BufferDescriptor room{};
    room.size = W * H * 4;
    room.usage = wgpu::BufferUsage::CopyDst | wgpu::BufferUsage::MapRead;
    wgpu::Buffer back = device.CreateBuffer(&room);
    wgpu::CommandEncoder coder = device.CreateCommandEncoder();
    wgpu::TexelCopyTextureInfo from{};
    from.texture = picture;
    wgpu::TexelCopyBufferInfo to{};
    to.buffer = back;
    to.layout.bytesPerRow = W * 4;
    to.layout.rowsPerImage = H;
    const wgpu::Extent3D all{W, H, 1};
    coder.CopyTextureToBuffer(&from, &to, &all);
    wgpu::CommandBuffer commands = coder.Finish();
    device.GetQueue().Submit(1, &commands);
    bool mapped = false;
    instance.WaitAny(back.MapAsync(wgpu::MapMode::Read, 0, room.size, wgpu::CallbackMode::WaitAnyOnly,
                             [&](wgpu::MapAsyncStatus how, wgpu::StringView) { mapped = how == wgpu::MapAsyncStatus::Success; }), UINT64_MAX);
    if (!mapped) { std::printf("FAIL read back\n"); return 1; }
    const uint8_t* px = (const uint8_t*)back.GetConstMappedRange(0, room.size);
    const auto at = [&](uint32_t x, uint32_t y) { return px + (y * W + x) * 4; };
    const uint8_t* middle = at(128, 128);
    const uint8_t* corner = at(4, 4);
    const uint8_t* rim = at(128 + 79, 128);
    std::printf("(blue green red alpha) middle %d %d %d %d   corner %d %d %d %d   just inside the rim %d %d %d %d   device errors %d\n", middle[0], middle[1], middle[2], middle[3],
                corner[0], corner[1], corner[2], corner[3], rim[0], rim[1], rim[2], rim[3], errors);
    const bool ok = middle[2] > 250 && middle[1] < 5 && middle[0] < 5 && middle[3] > 250 && corner[3] < 5 && errors == 0;   // (bytes are blue, green, red, alpha)
    std::printf(ok ? "PASS: ThorVG drew on the Dawn device\n" : "FAIL: not the picture expected\n");
    return ok ? 0 : 1;
}
