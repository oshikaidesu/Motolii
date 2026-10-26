// replay_on_dawn.cpp: plays back what a three.js effect did on the GPU, without three.js or any JavaScript. The run was
// recorded by Dawn's own capture (its webgpu backend, tsl/dawn_capture.sh); Dawn's own replay (src/dawn/replay) plays it
// on the Dawn Motolii draws with, the way Dawn's DawnTest.cpp Recorder does (Capture::Create, Replay::Create, Play), and
// the texture named <label> is read back as CaptureAndReplayTests.cpp ExpectTextureEQ finds it (GetObjectByLabel).
// Device and reading back as tsl_on_dawn.cpp; every feature the adapter has, as three.js asks (three.webgpu.js 83954).
//   ./dawn_probe/replay_on_dawn.sh <capture> <label> <width> <height> <out.rgba>   (<capture>.commands, .contents)
#include <dawn/replay/Replay.h>
#include <dawn/webgpu_cpp.h>
#include <cstdio>
#include <fstream>
#include <sstream>
#include <string>

static int errors = 0;

static std::string slurp(const std::string& path) {
    std::ifstream f(path, std::ios::binary);
    std::stringstream s;
    s << f.rdbuf();
    return s.str();
}

int main(int argc, char** argv) {
    if (argc < 6) { std::printf("usage: replay_on_dawn <capture> <label> <width> <height> <out.rgba>\n"); return 2; }
    const std::string capture = argv[1], label = argv[2];
    const uint32_t W = (uint32_t)std::atoi(argv[3]), H = (uint32_t)std::atoi(argv[4]);
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
    wgpu::SupportedFeatures features;
    adapter.GetFeatures(&features);
    wgpu::DeviceDescriptor device_wish{};
    device_wish.requiredFeatureCount = features.featureCount;
    device_wish.requiredFeatures = features.features;
    device_wish.SetUncapturedErrorCallback([](const wgpu::Device&, wgpu::ErrorType, wgpu::StringView said) {
        ++errors;
        std::printf("DEVICE ERROR: %.*s\n", (int)said.length, said.data);
    });
    wgpu::Device device;
    instance.WaitAny(adapter.RequestDevice(&device_wish, wgpu::CallbackMode::WaitAnyOnly,
                             [&](wgpu::RequestDeviceStatus, wgpu::Device got, wgpu::StringView) { device = got; }), UINT64_MAX);
    if (!device) { std::printf("FAIL no device\n"); return 1; }

    // the recording, played back
    const std::string commands = slurp(capture + ".commands"), contents = slurp(capture + ".contents");
    std::istringstream command_stream(commands), content_stream(contents);
    auto recorded = dawn::replay::Capture::Create(command_stream, commands.size(), content_stream, contents.size());
    if (!recorded) { std::printf("FAIL capture not read\n"); return 1; }
    auto replay = dawn::replay::Replay::Create(device, std::move(recorded));
    if (!replay) { std::printf("FAIL replay\n"); return 1; }
    // PlayFrame(): up to the next frame boundary, which is a surface's Present (Replay.cpp:1510-1515)
    int frames = 0;
    for (dawn::replay::ReplayStatus s; (s = replay->PlayFrame()) == dawn::replay::ReplayStatus::Continuing;) ++frames;
    std::printf("PlayFrame returned Continuing %d times\n", frames);

    // the picture it drew, read back (a row copied to a buffer takes a multiple of 256 bytes: WebGPU's copy rule)
    wgpu::Texture drawn = replay->GetObjectByLabel<wgpu::Texture>(label);
    if (!drawn) { std::printf("FAIL no texture %s\n", label.c_str()); return 1; }
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
    wgpu::Extent3D all{W, H, 1};
    wgpu::CommandEncoder coder = device.CreateCommandEncoder();
    coder.CopyTextureToBuffer(&from, &to, &all);
    wgpu::CommandBuffer done = coder.Finish();
    device.GetQueue().Submit(1, &done);
    bool mapped = false;
    instance.WaitAny(back.MapAsync(wgpu::MapMode::Read, 0, room.size, wgpu::CallbackMode::WaitAnyOnly,
                             [&](wgpu::MapAsyncStatus how, wgpu::StringView) { mapped = how == wgpu::MapAsyncStatus::Success; }), UINT64_MAX);
    if (!mapped) { std::printf("FAIL read back\n"); return 1; }
    const char* got = (const char*)back.GetConstMappedRange(0, room.size);
    std::ofstream out(argv[5], std::ios::binary);
    for (uint32_t y = 0; y < H; ++y) out.write(got + (size_t)y * stride, W * 4);
    std::printf("%s: replayed %zu + %zu bytes, %u x %u read back, device errors %d\n", errors ? "FAIL" : "PASS",
                commands.size(), contents.size(), W, H, errors);
    return errors ? 1 : 0;
}
