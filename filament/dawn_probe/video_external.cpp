// video_external.cpp: a film's picture, as GStreamer decodes it, read by a web effect's own WGSL that asks for a
// `texture_external` (shader-effects-inc/shaders' VideoTexture, as shaderport writes it out), on Filament's own Dawn
// device. Nothing in between is ours:
//  - the picture: GStreamer (filesrc ! decodebin3 ! appsink NV12), its IOSurface shared with Filament's device as
//    Motolii's video.h shares it (SharedTextureMemory, R8BG8Biplanar420Unorm);
//  - texture_external: Dawn's ExternalTexture over the picture's two planes, as Dawn's own
//    src/dawn/tests/end2end/ExternalTextureTests.cpp InitExternalTextureDescriptor (108-136) makes one, its numbers by
//    Dawn's ComputeExternalTextureParams (src/dawn/common/ExternalTextureParams.h): what a browser's
//    importExternalTexture is, in Chrome;
//  - the drawing: the effect's fragment as written, the vertex its authors draw it with (typegpu 0.12.3
//    common/fullScreenTriangle.js, MIT), the pipeline with the shader's own layout ("auto"), on the device gpu_door lends.
// The check: GStreamer's own videoconvert of the same picture to RGBA, beside it.
//   ./dawn_probe/video_external.sh <film> <effect.wgsl> <out folder>   (writes effect.rgba, gstreamer.rgba)
#include "gpu_door.h"
#include <filament/Engine.h>
#include <dawn/webgpu_cpp.h>
#include "src/dawn/common/ExternalTextureParams.h"
#include <gst/gst.h>
#include <gst/app/gstappsink.h>
#include <gst/video/video.h>
#include <CoreVideo/CoreVideo.h>
#include <IOSurface/IOSurface.h>
#include <cstdio>
#include <cstring>
#include <fstream>
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

static GstSample* first_picture(const char* film, const char* caps, GstElement** pipe) {
    gchar* launch = g_strdup_printf("filesrc location=\"%s\" ! decodebin3 ! %sappsink name=out sync=false caps=%s", film,
                                    std::strstr(caps, "RGBA") ? "videoconvert ! " : "", caps);
    *pipe = gst_parse_launch(launch, nullptr);
    g_free(launch);
    gst_element_set_state(*pipe, GST_STATE_PAUSED);
    gst_element_get_state(*pipe, nullptr, nullptr, 5 * GST_SECOND);
    GstElement* out = gst_bin_get_by_name(GST_BIN(*pipe), "out");
    GstSample* got = gst_app_sink_pull_preroll(GST_APP_SINK(out));
    gst_object_unref(out);
    return got;
}

int main(int argc, char** argv) {
    if (argc < 4) { std::printf("usage: video_external <film> <effect.wgsl> <out folder>\n"); return 2; }
    gst_init(&argc, &argv);
    const std::string outDir = std::string(argv[3]) + "/";
    std::ifstream wf(argv[2]);
    std::stringstream ws;
    ws << wf.rdbuf();
    const std::string fragment = ws.str();

    // GStreamer's own RGBA of the picture: the answer to check against
    GstElement* plain = nullptr;
    GstSample* rgba = first_picture(argv[1], "video/x-raw,format=RGBA", &plain);
    GstVideoInfo info;
    gst_video_info_from_caps(&info, gst_sample_get_caps(rgba));
    const uint32_t W = GST_VIDEO_INFO_WIDTH(&info), H = GST_VIDEO_INFO_HEIGHT(&info);
    {
        GstVideoFrame f;
        gst_video_frame_map(&f, &info, gst_sample_get_buffer(rgba), GST_MAP_READ);
        std::ofstream o(outDir + "gstreamer.rgba", std::ios::binary);
        for (uint32_t y = 0; y < H; ++y) o.write((const char*)GST_VIDEO_FRAME_PLANE_DATA(&f, 0) + y * GST_VIDEO_FRAME_PLANE_STRIDE(&f, 0), W * 4);
        gst_video_frame_unmap(&f);
    }

    // the picture as the machine's decoder leaves it: an IOSurface (NV12)
    GstElement* reader = nullptr;
    GstSample* picture = first_picture(argv[1], "video/x-raw,format=NV12", &reader);
    const GType api = g_type_from_name("GstCoreVideoMetaAPI");
    auto* meta = api ? (VideoCoreVideoMeta*)gst_buffer_get_meta(gst_sample_get_buffer(picture), api) : nullptr;
    IOSurfaceRef surface = meta && meta->pixbuf ? CVPixelBufferGetIOSurface(meta->pixbuf) : nullptr;
    if (!surface) { std::printf("FAIL the decoder left no IOSurface\n"); return 1; }
    const bool full = CVPixelBufferGetPixelFormatType(meta->pixbuf) == kCVPixelFormatType_420YpCbCr8BiPlanarFullRange;

    filament::Engine* engine = filament::Engine::Builder().backend(filament::Engine::Backend::WEBGPU).build();
    if (!engine || !door::device(engine)) { std::printf("FAIL no Filament WebGPU device\n"); return 1; }
    int errors = 0;
    bool done = false;
    std::vector<char> drawn((size_t)W * H * 4);
    door::on_driver(engine, [&] {
        wgpu::Device device((WGPUDevice)door::device(engine));
        wgpu::Instance instance((WGPUInstance)door::instance(engine));
        device.PushErrorScope(wgpu::ErrorFilter::Validation);

        // shared as video.h shares it
        wgpu::SharedTextureMemoryIOSurfaceDescriptor from{};
        from.ioSurface = surface;
        wgpu::SharedTextureMemoryDescriptor what{};
        what.nextInChain = &from;
        wgpu::SharedTextureMemory memory = device.ImportSharedTextureMemory(&what);
        wgpu::TextureDescriptor like{};
        like.size = {W, H, 1};
        like.format = wgpu::TextureFormat::R8BG8Biplanar420Unorm;
        like.usage = wgpu::TextureUsage::TextureBinding;
        wgpu::Texture shared = memory.CreateTexture(&like);
        wgpu::SharedTextureMemoryBeginAccessDescriptor lend{};
        lend.initialized = true;
        memory.BeginAccess(shared, &lend);

        // texture_external over its two planes, as ExternalTextureTests.cpp InitExternalTextureDescriptor
        wgpu::TextureViewDescriptor light{}, colour{};
        light.aspect = wgpu::TextureAspect::Plane0Only;
        colour.aspect = wgpu::TextureAspect::Plane1Only;
        dawn::ExternalTextureColorSpaceParams params{};
        dawn::ComputeExternalTextureParams(
            {.primaries = wgpu::ColorSpacePrimariesDawn::Rec709, .transfer = wgpu::ColorSpaceTransferDawn::SMPTE_170M,
             .yCbCrRange = full ? wgpu::ColorSpaceYCbCrRangeDawn::Full : wgpu::ColorSpaceYCbCrRangeDawn::Narrow,
             .yCbCrMatrix = wgpu::ColorSpaceYCbCrMatrixDawn::Rec709},
            wgpu::PredefinedColorSpace::SRGB, &params);
        wgpu::ExternalTextureDescriptor ext{};
        ext.plane0 = shared.CreateView(&light);
        ext.plane1 = shared.CreateView(&colour);
        ext.yuvToRgbConversionMatrix = params.yuvToRgbConversionMatrix.data();
        ext.gamutConversionMatrix = params.gamutConversionMatrix.data();
        ext.srcTransferFunctionParameters = params.srcTransferFunction.data();
        ext.dstTransferFunctionParameters = params.dstTransferFunction.data();
        ext.doYuvToRgbConversionOnly = true;   // (the colour as the film has it, as GStreamer's videoconvert gives it)
        ext.cropOrigin = {0, 0};
        ext.cropSize = {W, H};
        ext.apparentSize = {W, H};
        wgpu::ExternalTexture video = device.CreateExternalTexture(&ext);

        // the effect's own WGSL, its own layout
        const auto module = [&](const char* code) {
            wgpu::ShaderSourceWGSL wgsl{};
            wgsl.code = code;
            wgpu::ShaderModuleDescriptor made{};
            made.nextInChain = &wgsl;
            return device.CreateShaderModule(&made);
        };
        wgpu::ShaderModule vs = module(kVertex), fs = module(fragment.c_str());
        wgpu::ColorTargetState target{};
        target.format = wgpu::TextureFormat::RGBA8Unorm;
        wgpu::FragmentState fragState{};
        fragState.module = fs;
        fragState.targetCount = 1;
        fragState.targets = &target;
        wgpu::RenderPipelineDescriptor pd{};
        pd.vertex.module = vs;
        pd.fragment = &fragState;
        wgpu::RenderPipeline pipeline = device.CreateRenderPipeline(&pd);

        // its groups as the composer pins them (uniforms 0, textures 1, samplers 2: composer.ts BIND_GROUPS)
        wgpu::BufferDescriptor ub{};
        ub.size = 256;   // (its knobs do not change this effect's picture: time is read and dropped, the fit is fixed)
        ub.usage = wgpu::BufferUsage::Uniform | wgpu::BufferUsage::CopyDst;
        wgpu::Buffer knobs = device.CreateBuffer(&ub);
        wgpu::SamplerDescriptor sd{};
        sd.magFilter = sd.minFilter = wgpu::FilterMode::Linear;
        wgpu::Sampler linearClamp = device.CreateSampler(&sd);
        const auto group = [&](uint32_t g, wgpu::BindGroupEntry e) {
            wgpu::BindGroupDescriptor bd{};
            bd.layout = pipeline.GetBindGroupLayout(g);
            bd.entryCount = 1;
            bd.entries = &e;
            return device.CreateBindGroup(&bd);
        };
        wgpu::BindGroupEntry e0{}, e1{}, e2{};
        e0.buffer = knobs;
        wgpu::ExternalTextureBindingEntry asExternal{};
        asExternal.externalTexture = video;
        e1.nextInChain = &asExternal;
        e2.sampler = linearClamp;
        wgpu::BindGroup g0 = group(0, e0), g1 = group(1, e1), g2 = group(2, e2);

        wgpu::TextureDescriptor od{};
        od.size = {W, H, 1};
        od.format = wgpu::TextureFormat::RGBA8Unorm;
        od.usage = wgpu::TextureUsage::RenderAttachment | wgpu::TextureUsage::CopySrc;
        wgpu::Texture out = device.CreateTexture(&od);
        wgpu::RenderPassColorAttachment at{};
        at.view = out.CreateView();
        at.loadOp = wgpu::LoadOp::Clear;
        at.storeOp = wgpu::StoreOp::Store;
        wgpu::RenderPassDescriptor rp{};
        rp.colorAttachmentCount = 1;
        rp.colorAttachments = &at;
        wgpu::CommandEncoder coder = device.CreateCommandEncoder();
        wgpu::RenderPassEncoder pass = coder.BeginRenderPass(&rp);
        pass.SetPipeline(pipeline);
        pass.SetBindGroup(0, g0);
        pass.SetBindGroup(1, g1);
        pass.SetBindGroup(2, g2);
        pass.Draw(3);
        pass.End();

        // read back (rows of a multiple of 256 bytes: WebGPU's copy rule), as tsl_on_dawn.cpp
        const uint32_t stride = (W * 4 + 255) & ~255u;
        wgpu::BufferDescriptor room{};
        room.size = (uint64_t)stride * H;
        room.usage = wgpu::BufferUsage::CopyDst | wgpu::BufferUsage::MapRead;
        wgpu::Buffer back = device.CreateBuffer(&room);
        wgpu::TexelCopyTextureInfo src{};
        src.texture = out;
        wgpu::TexelCopyBufferInfo dst{};
        dst.buffer = back;
        dst.layout.bytesPerRow = stride;
        dst.layout.rowsPerImage = H;
        wgpu::Extent3D all{W, H, 1};
        coder.CopyTextureToBuffer(&src, &dst, &all);
        wgpu::CommandBuffer commands = coder.Finish();
        device.GetQueue().Submit(1, &commands);
        device.PopErrorScope(wgpu::CallbackMode::AllowProcessEvents,
            [&](wgpu::PopErrorScopeStatus, wgpu::ErrorType type, wgpu::StringView said) {
                if (type != wgpu::ErrorType::NoError) { ++errors; std::printf("DEVICE ERROR: %.*s\n", (int)said.length, said.data); }
            });
        bool mapped = false;
        back.MapAsync(wgpu::MapMode::Read, 0, room.size, wgpu::CallbackMode::AllowProcessEvents,
            [&](wgpu::MapAsyncStatus how, wgpu::StringView) { mapped = how == wgpu::MapAsyncStatus::Success; done = true; });
        while (!done) instance.ProcessEvents();
        if (mapped) {
            const char* got = (const char*)back.GetConstMappedRange(0, room.size);
            for (uint32_t y = 0; y < H; ++y) std::memcpy(drawn.data() + (size_t)y * W * 4, got + (size_t)y * stride, W * 4);
        } else ++errors;
        wgpu::SharedTextureMemoryEndAccessState ended{};
        memory.EndAccess(shared, &ended);
    });
    engine->flushAndWait();
    std::ofstream(outDir + "effect.rgba", std::ios::binary).write(drawn.data(), (std::streamsize)drawn.size());
    std::printf("%s: %u x %u, %s range, device errors %d\n", errors ? "FAIL" : "PASS", W, H, full ? "full" : "video", errors);
    gst_sample_unref(picture), gst_sample_unref(rgba);
    gst_element_set_state(reader, GST_STATE_NULL), gst_element_set_state(plain, GST_STATE_NULL);
    filament::Engine::destroy(&engine);
    return errors ? 1 : 0;
}
