// video.h: a film from a file (an .mp4 or anything FFmpeg reads), put in the scene as a thing like any other: a flat
// sheet whose look is the film's picture at a moment (flat.mat, as for ThorVG's pictures: so it can be cut by mattes,
// seen through glass, and keyed: `green`).
// FIRST STEP, the plainest way: the ffmpeg program reads the file (with the Mac's video decoder) and pipes its
// pictures here as plain numbers, and each is copied to the GPU. To be replaced by pictures that stay on the GPU.
#include <cstdio>
#include <string>

struct Video {
    FILE* pipe = nullptr;
    std::string file;
    int w = 0, h = 0, shown = -1;      // shown: which of its pictures is on the sheet
    float each = 0.f, seconds = 0.f;   // pictures a second, and how long it is
    Texture* texture = nullptr;
    Mesh* sheet = nullptr;             // the thing in the scene: moved like any other (kit.h: move)
    MaterialInstance* look = nullptr;
};

// The film in `file` put in the scene: a sheet `high` tall (as wide as the film makes it) with its middle at `place`,
// facing along the place's z.
static Video* video(Stage& s, const char* file, const mat4f& place, float high) {
    Video* v = new Video;
    v->file = file;
    int num = 0, den = 1, count = 0;
    FILE* asked = popen(("ffprobe -v error -select_streams v:0 -count_packets -show_entries stream=width,height,r_frame_rate,nb_read_packets -of csv=p=0 '" + v->file + "'").c_str(), "r");
    if (!asked || fscanf(asked, "%d,%d,%d/%d,%d", &v->w, &v->h, &num, &den, &count) < 5 || v->w <= 0 || v->h <= 0 || num <= 0 || count <= 0) return nullptr;
    pclose(asked);
    v->each = (float)num / (float)den;
    v->seconds = (float)count / v->each;
    v->texture = Texture::Builder().width((uint32_t)v->w).height((uint32_t)v->h).levels(1).sampler(Texture::Sampler::SAMPLER_2D).format(Texture::InternalFormat::RGBA8).build(*s.engine);
    v->sheet = new Mesh;
    for (float2 c : {float2{-1, -1}, float2{1, -1}, float2{1, 1}, float2{-1, 1}}) {
        v->sheet->pos.push_back({0.5f * high * (float)v->w / (float)v->h * c.x, 0.5f * high * c.y, 0.f});
        v->sheet->nrm.push_back({0, 0, 1});
        v->sheet->uv.push_back(c * 0.5f + 0.5f);   // (a film's first row is its top: Filament turns a sheet's v over itself)
    }
    v->sheet->idx = {0, 1, 2, 0, 2, 3};
    v->look = add(s, v->sheet, "flat.filamat", {place});
    if (!v->look) return nullptr;
    v->look->setParameter("picture", v->texture, TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR));
    return v;
}

// A green screen: where the film is this colour it is clear. near: how near to the colour counts as the screen itself
// (in the two colour numbers of video, where pure green is 0.6 from grey: the screen's own noise must fit in it);
// soft: over how much more the film comes in (a point half screen, half grey is 0.3 from the screen);
// spill: how far from the screen its colour is still taken out of what stays.
static void green(Video* v, float3 colour = float3{0.f, 1.f, 0.f}, float near = 0.12f, float soft = 0.3f, float spill = 0.9f) {
    v->look->setParameter("key", float4{colour, near});
    v->look->setParameter("keySoft", float2{soft, spill});
}

// The film at its moment, `seconds` from its start (it starts again when it ends), for the picture that comes next.
static void video_at(Stage& s, Video* v, float seconds) {
    const int want = (int)std::floor(std::fmod(seconds, v->seconds) * v->each);
    if (want == v->shown) return;
    if (want < v->shown && v->pipe) { pclose(v->pipe); v->pipe = nullptr; }   // (from its start again)
    if (!v->pipe) {
        v->pipe = popen(("ffmpeg -loglevel quiet -hwaccel videotoolbox -i '" + v->file + "' -f rawvideo -pix_fmt rgba -").c_str(), "r");
        v->shown = -1;
    }
    const size_t size = (size_t)v->w * (size_t)v->h * 4;
    uint8_t* picture = (uint8_t*)malloc(size);
    while (v->pipe && v->shown < want) {
        if (fread(picture, 1, size, v->pipe) != size) { free(picture); return; }
        ++v->shown;
    }
    v->texture->setImage(*s.engine, 0, backend::PixelBufferDescriptor(picture, size, backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE,
                                                                      [](void* buffer, size_t, void*) { free(buffer); }));
}
