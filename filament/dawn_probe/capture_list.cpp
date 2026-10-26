// capture_list.cpp: lists a Dawn capture (tsl/dawn_capture.sh) one line per root command, to see what one frame of an
// effect is: what is made once, what is written each frame, what is sent. Built on Dawn's own walker and its own
// SurfaceDiscoveryVisitor (src/dawn/replay/SurfaceDiscovery.h), only the four looks below changed.
//   ./dawn_probe/capture_list.sh <capture>   (<capture>.commands, .contents)
#include "src/dawn/replay/Capture.h"
#include "src/dawn/replay/SurfaceDiscovery.h"
#include <cstdio>
#include <fstream>
#include <sstream>

using namespace dawn::replay;

// What one command buffer does, as one line: its passes, where each draws, which pipeline and bind groups, the draws
// (walked by Dawn's own ProcessEncoderCommands / ProcessRenderPassCommands, CaptureWalker.cpp)
struct PassSigner : RenderPassVisitor {
    std::string* line;
    void add(const std::string& s) { *line += s + " "; }
    template <typename T>
    void sign(const T& d, const char* name) {
        if constexpr (std::is_same_v<T, schema::CommandBufferCommandSetRenderPipelineCmdData>) add("pipe" + std::to_string(d.pipelineId));
        else if constexpr (std::is_same_v<T, schema::CommandBufferCommandSetBindGroupCmdData>) add("bg" + std::to_string(d.index) + "=" + std::to_string(d.bindGroupId));
        else if constexpr (std::is_same_v<T, schema::CommandBufferCommandDrawCmdData>) add("draw" + std::to_string(d.vertexCount));
        else add(name);
    }
#define SIGN(NAME)                                                                          \
    VisitResult operator()(const schema::CommandBufferCommand##NAME##CmdData& d) override { \
        sign(d, #NAME);                                                                     \
        return VisitStatus::Continue;                                                       \
    }
    DAWN_REPLAY_RENDER_PASS_COMMANDS(SIGN)
#undef SIGN
};

struct Signer : EncoderVisitor {
    std::string line;
    PassSigner pass;
    void add(const std::string& s) { line += s + " "; }
    ResultOrError<ComputePassVisitor*> BeginComputePass(const schema::CommandBufferCommandBeginComputePassCmdData&) override {
        return DAWN_INTERNAL_ERROR("compute pass not listed");
    }
    ResultOrError<RenderPassVisitor*> BeginRenderPass(const schema::CommandBufferCommandBeginRenderPassCmdData& d) override {
        add("pass->" + (d.colorAttachments.empty() ? std::string("-") : std::to_string(d.colorAttachments[0].viewId)));
        pass.line = &line;
        return static_cast<RenderPassVisitor*>(&pass);
    }
#define SIGN_ENCODER(NAME)                                                                  \
    VisitResult operator()(const schema::CommandBufferCommand##NAME##CmdData&) override { \
        add(#NAME);                                                                         \
        return VisitStatus::Continue;                                                       \
    }
    DAWN_REPLAY_ENCODER_NON_CREATION_COMMANDS(SIGN_ENCODER)
#undef SIGN_ENCODER
};

struct Lister : SurfaceDiscoveryVisitor {
    using SurfaceDiscoveryVisitor::operator();
    // the written bytes are read from the content stream in order, as Replay.cpp's ReadContentInto* read them
    ReadHead* content = nullptr;
    void SetContentReadHead(ReadHead* readHead) override { content = readHead; }
    const uint32_t* take(uint64_t size) {
        auto got = content->GetData(size_t(size));
        return got.IsSuccess() ? got.AcquireSuccess() : nullptr;
    }
    VisitResult operator()(const CreateResourceData& data) override {
        if (auto* cb = std::get_if<CommandBufferData>(&data.data)) {
            Signer signer;
            VisitResult r = ProcessEncoderCommands(cb->readHead, &signer);
            std::printf("commands %s\n", signer.line.c_str());
            return r;
        }
        std::printf("make %d #%llu %s\n", (int)data.resource.type, (unsigned long long)data.resource.id, data.resource.label.c_str());
        return SurfaceDiscoveryVisitor::operator()(data);
    }
    VisitResult operator()(const schema::RootCommandWriteBufferCmdData& data) override {
        const uint32_t* bytes = take(data.size);
        std::printf("write buffer #%llu +%llu %llu bytes", (unsigned long long)data.bufferId,
                    (unsigned long long)data.bufferOffset, (unsigned long long)data.size);
        if (bytes && data.size == 4) std::printf(" = %g", *(const float*)bytes);
        std::printf("\n");
        return VisitStatus::Continue;
    }
    VisitResult operator()(const schema::RootCommandWriteTextureCmdData& data) override {
        take((data.dataSize + 3) & ~3ull);
        std::printf("write texture #%llu %llu bytes\n", (unsigned long long)data.destination.textureId, (unsigned long long)data.dataSize);
        return VisitStatus::Continue;
    }
    VisitResult operator()(const schema::RootCommandInitTextureCmdData& data) override {
        take((data.dataSize + 3) & ~3ull);
        std::printf("init texture %llu bytes\n", (unsigned long long)data.dataSize);
        return VisitStatus::Continue;
    }
    VisitResult operator()(const schema::RootCommandQueueSubmitCmdData& data) override {
        std::printf("submit %zu\n", data.commandBuffers.size());
        return VisitStatus::Continue;
    }
};

static std::string slurp(const std::string& path) {
    std::ifstream f(path, std::ios::binary);
    std::stringstream s;
    s << f.rdbuf();
    return s.str();
}

int main(int argc, char** argv) {
    const std::string commands = slurp(std::string(argv[1]) + ".commands"), contents = slurp(std::string(argv[1]) + ".contents");
    std::istringstream command_stream(commands), content_stream(contents);
    auto capture = Capture::Create(command_stream, commands.size(), content_stream, contents.size());
    Lister lister;
    return capture->Walk(lister) ? 0 : 1;
}
