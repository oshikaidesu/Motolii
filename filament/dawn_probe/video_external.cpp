// video_external.cpp: a film, as GStreamer decodes it, put through a web effect exactly as its authors' composer
// wrote it (shader-effects-inc/shaders, written out by shaderport's exporter: tsl/shaderport_export.sh), frame after
// frame, on Filament's own Dawn device. Nothing in between is ours:
//  - each picture: GStreamer (filesrc ! decodebin3 ! appsink NV12), its IOSurface shared with Filament's device as
//    Motolii's video.h shares it (SharedTextureMemory, R8BG8Biplanar420Unorm, kept per surface as video.h keeps them);
//  - texture_external: Dawn's ExternalTexture over the picture's two planes, as Dawn's own
//    src/dawn/tests/end2end/ExternalTextureTests.cpp InitExternalTextureDescriptor (108-136) makes one, its numbers by
//    Dawn's ComputeExternalTextureParams (src/dawn/common/ExternalTextureParams.h): what a browser's
//    importExternalTexture is, in Chrome;
//  - the passes: the composer's own WGSL in its own order (its rtt passes, then the final one, manifest.json), the
//    vertex its authors draw them with (typegpu 0.12.3 common/fullScreenTriangle.js, MIT), each pipeline with the
//    shader's own layout ("auto"), the composer's own formats (rtt rgba16float: passManager.ts:106), each binding
//    found by the name the shader gives it (uniforms, video_N, rtt_N, the four shared samplers of composer.ts:52-59);
//  - each frame's uniforms: the bytes the exporter packed for that frame (<effect>__frames.bin).
// Out: the frames to ffmpeg (an .mp4), and the time a frame takes on the GPU.
//   ./dawn_probe/video_external.sh <film> <export folder> <effect> <frames> <out.mp4>
#include "gpu_door.h"
#include <filament/Engine.h>
#include <dawn/webgpu_cpp.h>
#include "src/dawn/common/ExternalTextureParams.h"
#include <gst/gst.h>
#include <gst/app/gstappsink.h>
#include <gst/video/video.h>
#include <CoreVideo/CoreVideo.h>
#include <IOSurface/IOSurface.h>
#include <algorithm>
#include <chrono>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <map>
#include <regex>
#include <sstream>
#include <string>
#include <vector>

struct VideoCoreVideoMeta { GstMeta meta; CVBufferRef cvbuf; CVPixelBufferRef pixbuf; };   // (as video.h reads it)

static const char* kVertex = R"(
struct Out { @builtin(position) pos: vec4f, @location(0) uv: vec2f }
@vertex fn fullScreenTriangle(@builtin(vertex_index) vertexIndex: u32) -> Out {
  const pos = array<vec2f, 3>(vec2f(-1, -1), vec2f(3, -1), vec2f(-1, 3));
  const uv = array<vec2f, 3>(vec2f(0, 1), vec2f(2, 1), vec2f(0, -1));
  return Out(vec4f(pos[vertexIndex], 0, 1), uv[vertexIndex]);
}
)";

static std::string slurp(const std::string& path) {
    std::ifstream f(path, std::ios::binary);
    std::stringstream s;
    s << f.rdbuf();
    return s.str();
}

struct Shared { wgpu::SharedTextureMemory memory; wgpu::Texture texture; };
struct Pass {
    std::string key;          // "rtt_rtt_0" writes rtt_0; "final" writes the frame
    wgpu::RenderPipeline pipeline;
    struct Use { uint32_t group, binding; std::string name, type; };
    std::vector<Use> uses;    // as the shader declares them
    uint32_t groups = 0;
};

int main(int argc, char** argv) {
    if (argc < 6) { std::printf("usage: video_external <film> <export folder> <effect> <frames> <out.mp4>\n"); return 2; }
    gst_init(&argc, &argv);
    const std::string dir = std::string(argv[2]) + "/", effect = argv[3];
    const int frames = std::atoi(argv[4]);

    // the composer's passes: its rtt passes (they write what the final one reads), then the final one
    std::vector<std::string> keys;
    {
        const std::string manifest = slurp(dir + "manifest.json");
        const size_t at = manifest.find("\"name\": \"" + effect + "\"");
        if (at == std::string::npos) { std::printf("FAIL %s is not in manifest.json\n", effect.c_str()); return 1; }
        const size_t end = manifest.find("\"props\"", at);
        std::regex key("\"key\": \"([a-z_0-9]+)\"");
        for (std::sregex_iterator i(manifest.begin() + (long)at, manifest.begin() + (long)end, key), e; i != e; ++i)
            if ((*i)[1] != "final") keys.push_back((*i)[1]);
        keys.push_back("final");
    }
    const std::string uniformBytes = slurp(dir + effect + "__frames.bin");
    const size_t stride = uniformBytes.size() / (size_t)std::max(1, frames);

    gchar* launch = g_strdup_printf("filesrc location=\"%s\" ! decodebin3 ! appsink name=out sync=false max-buffers=4 caps=video/x-raw,format=NV12", argv[1]);
    GstElement* reader = gst_parse_launch(launch, nullptr);
    g_free(launch);
    GstElement* sinkElement = gst_bin_get_by_name(GST_BIN(reader), "out");
    GstAppSink* sink = GST_APP_SINK(sinkElement);
    gst_element_set_state(reader, GST_STATE_PLAYING);
    GstSample* first = gst_app_sink_pull_sample(sink);
    if (!first) { std::printf("FAIL no picture\n"); return 1; }
    GstVideoInfo info;
    gst_video_info_from_caps(&info, gst_sample_get_caps(first));
    const uint32_t W = GST_VIDEO_INFO_WIDTH(&info), H = GST_VIDEO_INFO_HEIGHT(&info);

    gchar* encode = g_strdup_printf("ffmpeg -loglevel error -y -f rawvideo -pix_fmt rgba -s %ux%u -r 60 -i - -pix_fmt yuv420p -crf 18 \"%s\"", W, H, argv[5]);
    FILE* ffmpeg = popen(encode, "w");
    g_free(encode);

    filament::Engine* engine = filament::Engine::Builder().backend(filament::Engine::Backend::WEBGPU).build();
    if (!engine || !door::device(engine)) { std::printf("FAIL no Filament WebGPU device\n"); return 1; }
    int errors = 0, drawnFrames = 0;
    double setupMs = 0, gpuMs = 0, gpuWorst = 0;
    door::on_driver(engine, [&] {
        wgpu::Device device((WGPUDevice)door::device(engine));
        wgpu::Instance instance((WGPUInstance)door::instance(engine));
        wgpu::Queue queue = device.GetQueue();
        device.PushErrorScope(wgpu::ErrorFilter::Validation);
        const auto t0 = std::chrono::steady_clock::now();

        // made once: the passes, what they draw into, the uniforms, the samplers
        const auto module = [&](const std::string& code) {
            wgpu::ShaderSourceWGSL wgsl{};
            wgsl.code = code.c_str();
            wgpu::ShaderModuleDescriptor made{};
            made.nextInChain = &wgsl;
            return device.CreateShaderModule(&made);
        };
        wgpu::ShaderModule vs = module(kVertex);
        std::vector<Pass> passes;
        for (const std::string& key : keys) {
            Pass p;
            p.key = key;
            const std::string code = slurp(dir + effect + "__" + key + ".wgsl");
            std::regex use("@group\\((\\d+)\\) @binding\\((\\d+)\\) var(?:<[a-z]+>)? ([A-Za-z_0-9]+): ([A-Za-z_0-9<>]+);");
            for (std::sregex_iterator i(code.begin(), code.end(), use), e; i != e; ++i) {
                p.uses.push_back({(uint32_t)std::stoul((*i)[1]), (uint32_t)std::stoul((*i)[2]), (*i)[3], (*i)[4]});
                p.groups = std::max(p.groups, p.uses.back().group + 1);
            }
            wgpu::ShaderModule fs = module(code);
            wgpu::ColorTargetState target{};
            target.format = key == "final" ? wgpu::TextureFormat::RGBA8Unorm : wgpu::TextureFormat::RGBA16Float;
            wgpu::FragmentState fragState{};
            fragState.module = fs;
            fragState.targetCount = 1;
            fragState.targets = &target;
            wgpu::RenderPipelineDescriptor pd{};
            pd.vertex.module = vs;
            pd.fragment = &fragState;
            p.pipeline = device.CreateRenderPipeline(&pd);
            passes.push_back(p);
        }
        const auto texture = [&](wgpu::TextureFormat format, wgpu::TextureUsage usage) {
            wgpu::TextureDescriptor td{};
            td.size = {W, H, 1};
            td.format = format;
            td.usage = usage;
            return device.CreateTexture(&td);
        };
        std::map<std::string, wgpu::Texture> rtt;   // rtt_0 ... as the composer names them
        for (const Pass& p : passes)
            if (p.key != "final") rtt[p.key.substr(4)] = texture(wgpu::TextureFormat::RGBA16Float, wgpu::TextureUsage::RenderAttachment | wgpu::TextureUsage::TextureBinding);
        wgpu::Texture out = texture(wgpu::TextureFormat::RGBA8Unorm, wgpu::TextureUsage::RenderAttachment | wgpu::TextureUsage::CopySrc);
        wgpu::BufferDescriptor ub{};
        ub.size = (stride + 255) & ~size_t(255);
        ub.usage = wgpu::BufferUsage::Uniform | wgpu::BufferUsage::CopyDst;
        wgpu::Buffer uniforms = device.CreateBuffer(&ub);
        std::map<std::string, wgpu::Sampler> samplers;   // composer.ts SHARED_SAMPLER_LAYOUT
        for (const auto& [name, filter, wrap] : std::vector<std::tuple<std::string, wgpu::FilterMode, wgpu::AddressMode>>{
                 {"linearClamp", wgpu::FilterMode::Linear, wgpu::AddressMode::ClampToEdge}, {"nearestClamp", wgpu::FilterMode::Nearest, wgpu::AddressMode::ClampToEdge},
                 {"linearRepeat", wgpu::FilterMode::Linear, wgpu::AddressMode::Repeat}, {"nearestRepeat", wgpu::FilterMode::Nearest, wgpu::AddressMode::Repeat}}) {
            wgpu::SamplerDescriptor sd{};
            sd.magFilter = sd.minFilter = filter;
            sd.addressModeU = sd.addressModeV = sd.addressModeW = wrap;
            samplers[name] = device.CreateSampler(&sd);
        }
        const uint32_t rowStride = (W * 4 + 255) & ~255u;
        wgpu::BufferDescriptor room{};
        room.size = (uint64_t)rowStride * H;
        room.usage = wgpu::BufferUsage::CopyDst | wgpu::BufferUsage::MapRead;
        wgpu::Buffer back = device.CreateBuffer(&room);
        dawn::ExternalTextureColorSpaceParams params{};
        std::map<IOSurfaceID, Shared> seen;
        const auto wait = [&](wgpu::Future f) {
            while (instance.WaitAny(f, 0) != wgpu::WaitStatus::Success) instance.ProcessEvents();
        };
        setupMs = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();

        std::vector<char> rows((size_t)W * H * 4);
        GstSample* picture = first;
        for (int f = 0; f < frames && picture; ++f) {
            auto* meta = (VideoCoreVideoMeta*)gst_buffer_get_meta(gst_sample_get_buffer(picture), g_type_from_name("GstCoreVideoMetaAPI"));
            IOSurfaceRef surface = meta && meta->pixbuf ? CVPixelBufferGetIOSurface(meta->pixbuf) : nullptr;
            if (!surface) { ++errors; std::printf("FAIL the decoder left no IOSurface\n"); break; }
            if (f == 0) {
                const bool full = CVPixelBufferGetPixelFormatType(meta->pixbuf) == kCVPixelFormatType_420YpCbCr8BiPlanarFullRange;
                dawn::ComputeExternalTextureParams(
                    {.primaries = wgpu::ColorSpacePrimariesDawn::Rec709, .transfer = wgpu::ColorSpaceTransferDawn::SMPTE_170M,
                     .yCbCrRange = full ? wgpu::ColorSpaceYCbCrRangeDawn::Full : wgpu::ColorSpaceYCbCrRangeDawn::Narrow,
                     .yCbCrMatrix = wgpu::ColorSpaceYCbCrMatrixDawn::Rec709},
                    wgpu::PredefinedColorSpace::SRGB, &params);
            }
            const auto a = std::chrono::steady_clock::now();
            // the decoder's surface, shared once and kept (as video.h keeps them)
            Shared& s = seen[IOSurfaceGetID(surface)];
            if (!s.texture) {
                wgpu::SharedTextureMemoryIOSurfaceDescriptor from{};
                from.ioSurface = surface;
                wgpu::SharedTextureMemoryDescriptor what{};
                what.nextInChain = &from;
                s.memory = device.ImportSharedTextureMemory(&what);
                wgpu::TextureDescriptor like{};
                like.size = {W, H, 1};
                like.format = wgpu::TextureFormat::R8BG8Biplanar420Unorm;
                like.usage = wgpu::TextureUsage::TextureBinding;
                s.texture = s.memory.CreateTexture(&like);
                wgpu::SharedTextureMemoryBeginAccessDescriptor lend{};
                lend.initialized = true;
                s.memory.BeginAccess(s.texture, &lend);
            }
            wgpu::TextureViewDescriptor light{}, colour{};
            light.aspect = wgpu::TextureAspect::Plane0Only;
            colour.aspect = wgpu::TextureAspect::Plane1Only;
            wgpu::ExternalTextureDescriptor ext{};
            ext.plane0 = s.texture.CreateView(&light);
            ext.plane1 = s.texture.CreateView(&colour);
            ext.yuvToRgbConversionMatrix = params.yuvToRgbConversionMatrix.data();
            ext.gamutConversionMatrix = params.gamutConversionMatrix.data();
            ext.srcTransferFunctionParameters = params.srcTransferFunction.data();
            ext.dstTransferFunctionParameters = params.dstTransferFunction.data();
            ext.doYuvToRgbConversionOnly = true;   // (the colour as the film has it)
            ext.cropOrigin = {0, 0};
            ext.cropSize = {W, H};
            ext.apparentSize = {W, H};
            wgpu::ExternalTexture video = device.CreateExternalTexture(&ext);

            queue.WriteBuffer(uniforms, 0, uniformBytes.data() + (size_t)f * stride, stride);
            wgpu::CommandEncoder coder = device.CreateCommandEncoder();
            for (Pass& p : passes) {
                std::vector<std::vector<wgpu::BindGroupEntry>> entries(p.groups);
                std::vector<wgpu::ExternalTextureBindingEntry> externals(p.uses.size());
                for (size_t i = 0; i < p.uses.size(); ++i) {
                    const Pass::Use& u = p.uses[i];
                    wgpu::BindGroupEntry e{};
                    e.binding = u.binding;
                    if (u.type == "texture_external") {
                        externals[i].externalTexture = video;
                        e.nextInChain = &externals[i];
                    } else if (u.name == "uniforms") e.buffer = uniforms;
                    else if (samplers.count(u.name)) e.sampler = samplers[u.name];
                    else if (rtt.count(u.name)) e.textureView = rtt[u.name].CreateView();
                    else { ++errors; std::printf("FAIL %s: no %s for %s\n", p.key.c_str(), u.type.c_str(), u.name.c_str()); }
                    entries[u.group].push_back(e);
                }
                wgpu::RenderPassColorAttachment at{};
                at.view = p.key == "final" ? out.CreateView() : rtt[p.key.substr(4)].CreateView();
                at.loadOp = wgpu::LoadOp::Clear;
                at.storeOp = wgpu::StoreOp::Store;
                wgpu::RenderPassDescriptor rp{};
                rp.colorAttachmentCount = 1;
                rp.colorAttachments = &at;
                wgpu::RenderPassEncoder pass = coder.BeginRenderPass(&rp);
                pass.SetPipeline(p.pipeline);
                for (uint32_t g = 0; g < p.groups; ++g) {
                    wgpu::BindGroupDescriptor bd{};
                    bd.layout = p.pipeline.GetBindGroupLayout(g);
                    bd.entryCount = entries[g].size();
                    bd.entries = entries[g].data();
                    pass.SetBindGroup(g, device.CreateBindGroup(&bd));
                }
                pass.Draw(3);
                pass.End();
            }
            wgpu::CommandBuffer drawn = coder.Finish();
            queue.Submit(1, &drawn);
            wait(queue.OnSubmittedWorkDone(wgpu::CallbackMode::WaitAnyOnly, [](wgpu::QueueWorkDoneStatus, wgpu::StringView) {}));
            const double ms = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - a).count();
            gpuMs += ms;
            gpuWorst = std::max(gpuWorst, ms);

            // read back for the film (not part of the time above)
            wgpu::CommandEncoder copy = device.CreateCommandEncoder();
            wgpu::TexelCopyTextureInfo src{};
            src.texture = out;
            wgpu::TexelCopyBufferInfo dst{};
            dst.buffer = back;
            dst.layout.bytesPerRow = rowStride;
            dst.layout.rowsPerImage = H;
            wgpu::Extent3D all{W, H, 1};
            copy.CopyTextureToBuffer(&src, &dst, &all);
            wgpu::CommandBuffer copied = copy.Finish();
            queue.Submit(1, &copied);
            wait(back.MapAsync(wgpu::MapMode::Read, 0, room.size, wgpu::CallbackMode::WaitAnyOnly, [](wgpu::MapAsyncStatus, wgpu::StringView) {}));
            const char* got = (const char*)back.GetConstMappedRange(0, room.size);
            for (uint32_t y = 0; y < H; ++y) std::memcpy(rows.data() + (size_t)y * W * 4, got + (size_t)y * rowStride, W * 4);
            back.Unmap();
            fwrite(rows.data(), 1, rows.size(), ffmpeg);
            ++drawnFrames;
            gst_sample_unref(picture);
            picture = f + 1 < frames ? gst_app_sink_pull_sample(sink) : nullptr;
        }
        if (picture) gst_sample_unref(picture);
        for (auto& [id, s] : seen) {
            wgpu::SharedTextureMemoryEndAccessState ended{};
            s.memory.EndAccess(s.texture, &ended);
        }
        bool popped = false;
        wait(device.PopErrorScope(wgpu::CallbackMode::WaitAnyOnly,
            [&](wgpu::PopErrorScopeStatus, wgpu::ErrorType type, wgpu::StringView said) {
                popped = true;
                if (type != wgpu::ErrorType::NoError) { ++errors; std::printf("DEVICE ERROR: %.*s\n", (int)said.length, said.data); }
            }));
    });
    engine->flushAndWait();
    pclose(ffmpeg);
    std::printf("%s: %s, %d frames %u x %u, made once %.1f ms, a frame on the GPU %.2f ms (worst %.2f), device errors %d\n",
                errors ? "FAIL" : "PASS", effect.c_str(), drawnFrames, W, H, setupMs, drawnFrames ? gpuMs / drawnFrames : 0.0, gpuWorst, errors);
    gst_element_set_state(reader, GST_STATE_NULL);
    gst_object_unref(sinkElement);
    filament::Engine::destroy(&engine);
    return errors ? 1 : 0;
}
