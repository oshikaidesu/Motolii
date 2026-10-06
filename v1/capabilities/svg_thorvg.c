// CAPABILITY: SVG = ThorVG (its WebGPU engine). The SVG file stays an SVG; ThorVG draws it on the GPU into a texture,
// and the Metal texture handle is what leaves this file. wgpu_device.c (device + target texture) is the verified
// wgpu-native sequence from the old repo, unchanged.
#include "wgpu_device.c"
#include "thorvg_capi.h"

static Tvg_Canvas g_canvas;
static Tvg_Paint  g_picture;
static float      g_w, g_h;

void* motolii_svg_open(const char* svg_path, uint32_t w, uint32_t h)
{
    char     error[256] = { 0 };
    BNative* native = b_native_open(w, h, error, sizeof(error));
    if (!native) { fprintf(stderr, "svg: %s\n", error); return NULL; }
    tvg_engine_init(0);
    g_canvas = tvg_wgcanvas_create(TVG_ENGINE_OPTION_DEFAULT);
    if (!g_canvas) { fprintf(stderr, "svg: no wg canvas\n"); return NULL; }
    Tvg_Result r = tvg_wgcanvas_set_target(g_canvas, native->device, native->instance, native->texture, w, h, TVG_COLORSPACE_ABGR8888S, 1);
    g_picture = tvg_picture_new();
    Tvg_Result l = tvg_picture_load(g_picture, svg_path);
    tvg_picture_set_size(g_picture, (float)w, (float)h);
    tvg_canvas_add(g_canvas, g_picture);
    g_w = (float)w; g_h = (float)h;
    fprintf(stderr, "svg: target %d load %d\n", (int)r, (int)l);
    return wgpuTextureGetNativeMetalTexture(native->texture);
}

// one GPU draw of the SVG; `scale` is a value from the Core
void motolii_svg_draw(float scale)
{
    tvg_picture_set_size(g_picture, g_w * scale, g_h * scale);
    tvg_paint_translate(g_picture, g_w * (1.0f - scale) * 0.5f, g_h * (1.0f - scale) * 0.5f);
    tvg_canvas_update(g_canvas);
    tvg_canvas_draw(g_canvas, true);
    tvg_canvas_sync(g_canvas);
}
