// Can an app take the Mac decoder's picture out of GStreamer still on the GPU? The applemedia plugin puts a meta on
// its buffers (GstCoreVideoMeta: the buffer's CVPixelBuffer); its header is not installed, so it is asked for by
// name and read by its layout as the plugin's source has it (sys/applemedia/corevideobuffer.h).
#include <gst/gst.h>
#include <gst/app/gstappsink.h>
#include <CoreVideo/CoreVideo.h>
#include <IOSurface/IOSurface.h>
typedef struct { GstMeta meta; CVBufferRef cvbuf; CVPixelBufferRef pixbuf; } CoreVideoMeta;
int main(int argc, char** argv) {
    gst_init(&argc, &argv);
    gchar* line = g_strdup_printf("filesrc location=\"%s\" ! qtdemux ! h264parse ! vtdec_hw ! video/x-raw,format=NV12 ! appsink name=out sync=false max-buffers=4", argv[1]);
    GError* err = NULL;
    GstElement* pipe = gst_parse_launch(line, &err);
    if (!pipe) { g_printerr("no pipeline: %s\n", err->message); return 1; }
    GstAppSink* out = GST_APP_SINK(gst_bin_get_by_name(GST_BIN(pipe), "out"));
    gst_element_set_state(pipe, GST_STATE_PLAYING);
    int n = 0, with_surface = 0;
    uint32_t seen[64]; int kinds = 0;
    for (GstSample* s; (s = gst_app_sink_pull_sample(out)); gst_sample_unref(s), ++n) {
        GstBuffer* b = gst_sample_get_buffer(s);
        GType api = g_type_from_name("GstCoreVideoMetaAPI");
        CoreVideoMeta* m = api ? (CoreVideoMeta*)gst_buffer_get_meta(b, api) : NULL;
        IOSurfaceRef io = (m && m->pixbuf) ? CVPixelBufferGetIOSurface(m->pixbuf) : NULL;
        if (io) {
            ++with_surface;
            uint32_t id = IOSurfaceGetID(io); int k = 0;
            while (k < kinds && seen[k] != id) ++k;
            if (k == kinds && kinds < 64) seen[kinds++] = id;
        }
        if (n == 0) {
            GstMemory* mem = gst_buffer_peek_memory(b, 0);
            g_print("first buffer: meta %s, pixel buffer %s, IOSurface %s; memory kind \"%s\"\n", m ? "yes" : "NO", (m && m->pixbuf) ? "yes" : "NO", io ? "yes" : "NO", mem->allocator ? mem->allocator->mem_type : "?");
            if (m && m->pixbuf) {
                OSType f = CVPixelBufferGetPixelFormatType(m->pixbuf);
                g_print("  %zu x %zu, format '%c%c%c%c', planes %zu, Metal-compatible: %s\n", CVPixelBufferGetWidth(m->pixbuf), CVPixelBufferGetHeight(m->pixbuf),
                        (char)(f >> 24), (char)(f >> 16), (char)(f >> 8), (char)f, CVPixelBufferGetPlaneCount(m->pixbuf), io ? "an IOSurface is what Metal and Dawn share" : "-");
            }
        }
    }
    g_print("%d pictures pulled, %d with an IOSurface, %d different surfaces in turn (a pool)\n", n, with_surface, kinds);
    gst_element_set_state(pipe, GST_STATE_NULL);
    return 0;
}
