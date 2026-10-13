// What GStreamer makes of any film file when it is left to choose the parts itself (decodebin3): which decoder, what
// kind of picture comes out, whether it is still on the GPU (an IOSurface), how its pictures are spaced in time,
// which way up it is meant to be, and how fast it is read.     any <file>...
#include <gst/gst.h>
#include <gst/app/gstappsink.h>
#include <CoreVideo/CoreVideo.h>
#include <IOSurface/IOSurface.h>
typedef struct { GstMeta meta; CVBufferRef cvbuf; CVPixelBufferRef pixbuf; } CoreVideoMeta;
static GstElement* g_out; static GstElement* g_pipe; static int g_sound = 0;
static void pad_added(GstElement* d, GstPad* pad, gpointer unused) {
    gchar* name = gst_pad_get_name(pad);
    GstElement* to = NULL;
    if (g_str_has_prefix(name, "video") && !gst_pad_is_linked(gst_element_get_static_pad(g_out, "sink"))) to = g_out;
    else { to = gst_element_factory_make("fakesink", NULL); g_object_set(to, "sync", FALSE, NULL); gst_bin_add(GST_BIN(g_pipe), to); gst_element_sync_state_with_parent(to); g_sound += g_str_has_prefix(name, "audio"); }
    GstPad* sink = gst_element_get_static_pad(to, "sink");
    gst_pad_link(pad, sink);
    g_free(name);
}
int main(int argc, char** argv) {
    gst_init(&argc, &argv);
    g_print("%-24s %-14s %-9s %-11s %-5s %-22s %-9s %-6s %s\n", "file", "decoder", "picture", "size", "GPU", "pictures: gaps (ms)", "turned", "sound", "read");
    for (int a = 1; a < argc; ++a) {
        g_pipe = gst_pipeline_new(NULL); g_sound = 0;
        GstElement* src = gst_element_factory_make("filesrc", NULL), *dec = gst_element_factory_make("decodebin3", NULL);
        g_out = gst_element_factory_make("appsink", NULL);
        g_object_set(src, "location", argv[a], NULL);
        g_object_set(g_out, "sync", FALSE, "max-buffers", 4, NULL);
        gst_bin_add_many(GST_BIN(g_pipe), src, dec, g_out, NULL);
        gst_element_link(src, dec);
        g_signal_connect(dec, "pad-added", G_CALLBACK(pad_added), NULL);
        gint64 t0 = g_get_monotonic_time();
        gst_element_set_state(g_pipe, GST_STATE_PLAYING);
        int n = 0, surfaces = 0; int w = 0, h = 0; const char* fmt = "?"; char format[32] = "?";
        GstClockTime last = GST_CLOCK_TIME_NONE; double least = 1e9, most = 0, first = -1, end = 0;
        for (GstSample* s; (s = gst_app_sink_pull_sample(GST_APP_SINK(g_out))); gst_sample_unref(s), ++n) {
            GstBuffer* b = gst_sample_get_buffer(s);
            GType api = g_type_from_name("GstCoreVideoMetaAPI");
            CoreVideoMeta* m = api ? (CoreVideoMeta*)gst_buffer_get_meta(b, api) : NULL;
            if (m && m->pixbuf && CVPixelBufferGetIOSurface(m->pixbuf)) ++surfaces;
            if (n == 0) { GstStructure* c = gst_caps_get_structure(gst_sample_get_caps(s), 0); gst_structure_get_int(c, "width", &w); gst_structure_get_int(c, "height", &h); fmt = gst_structure_get_string(c, "format"); g_strlcpy(format, fmt ? fmt : "?", sizeof format); }
            GstClockTime pts = GST_BUFFER_PTS(b);
            if (GST_CLOCK_TIME_IS_VALID(pts)) { if (first < 0) first = pts / 1e9; end = pts / 1e9; if (GST_CLOCK_TIME_IS_VALID(last)) { double d = (double)(pts - last) / 1e6; if (d < least) least = d; if (d > most) most = d; } last = pts; }
        }
        double took = (g_get_monotonic_time() - t0) / 1e6;
        // which decoder it chose, and which way up the film says it is
        char decoder[64] = "none"; char turned[32] = "-";
        GstIterator* it = gst_bin_iterate_recurse(GST_BIN(g_pipe)); GValue v = G_VALUE_INIT;
        while (gst_iterator_next(it, &v) == GST_ITERATOR_OK) {
            GstElementFactory* f = gst_element_get_factory(GST_ELEMENT(g_value_get_object(&v)));
            if (f && strstr(gst_element_factory_get_metadata(f, GST_ELEMENT_METADATA_KLASS), "Decoder/Video")) g_strlcpy(decoder, GST_OBJECT_NAME(f), sizeof decoder);
            g_value_reset(&v);
        }
        gst_iterator_free(it);
        GstBus* bus = gst_element_get_bus(g_pipe); char trouble[160] = "";
        for (GstMessage* msg; (msg = gst_bus_pop(bus)); gst_message_unref(msg)) {
            if (GST_MESSAGE_TYPE(msg) == GST_MESSAGE_TAG) { GstTagList* tags; gst_message_parse_tag(msg, &tags); gchar* o = NULL; if (gst_tag_list_get_string(tags, "image-orientation", &o)) { g_strlcpy(turned, o, sizeof turned); g_free(o); } gst_tag_list_unref(tags); }
            if (GST_MESSAGE_TYPE(msg) == GST_MESSAGE_ERROR && !trouble[0]) { GError* e; gst_message_parse_error(msg, &e, NULL); g_strlcpy(trouble, e->message, sizeof trouble); g_error_free(e); }
        }
        gst_object_unref(bus);
        gchar* base = g_path_get_basename(argv[a]); char size[24], gaps[40], read[40];
        g_snprintf(size, sizeof size, "%dx%d", w, h);
        g_snprintf(gaps, sizeof gaps, "%d: %.0f to %.0f", n, n > 1 ? least : 0, n > 1 ? most : 0);
        g_snprintf(read, sizeof read, "%.2f s (%.0fx)", took, end > first && took > 0 ? (end - first) / took : 0);
        g_print("%-24s %-14s %-9s %-11s %-5s %-22s %-9s %-6s %s %s\n", base, decoder, format, size, surfaces == n && n ? "yes" : (surfaces ? "some" : "NO"), gaps, turned, g_sound ? "yes" : "-", read, trouble);
        g_free(base);
        gst_element_set_state(g_pipe, GST_STATE_NULL);
        gst_object_unref(g_pipe);
    }
    return 0;
}
