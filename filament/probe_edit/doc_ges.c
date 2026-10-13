// (a probe: can GStreamer Editing Services be the document, never playing? and GStreamer's controller the keys?)
//   cc doc_ges.c $(pkg-config --cflags --libs gst-editing-services-1.0 gstreamer-controller-1.0) -o ../out/doc_ges && ../out/doc_ges <film>
#include <ges/ges.h>
#include <gst/controller/gstinterpolationcontrolsource.h>
#include <gst/controller/gstdirectcontrolbinding.h>
#include <stdio.h>
#include <time.h>
static double ms(void) { struct timespec t; clock_gettime(CLOCK_MONOTONIC, &t); return t.tv_sec * 1e3 + t.tv_nsec / 1e6; }
int main(int argc, char** argv) {
    gst_init(&argc, &argv);
    // 1. keys alone: a value over time, nothing else (no element, no pipeline)
    GstControlSource* keys = gst_interpolation_control_source_new();
    GstTimedValueControlSource* k = GST_TIMED_VALUE_CONTROL_SOURCE(keys);
    gst_timed_value_control_source_set(k, 0 * GST_SECOND, 0.0);
    gst_timed_value_control_source_set(k, 1 * GST_SECOND, 1.0);
    gst_timed_value_control_source_set(k, 3 * GST_SECOND, -0.5);
    for (int mode = GST_INTERPOLATION_MODE_NONE; mode <= GST_INTERPOLATION_MODE_CUBIC_MONOTONIC; ++mode) {
        g_object_set(keys, "mode", mode, NULL);
        printf("keys alone, mode %d:", mode);
        for (double t = 0; t <= 3.01; t += 0.5) { gdouble v = 0; gst_control_source_get_value(keys, (GstClockTime)(t * GST_SECOND), &v); printf(" %.3f", v); }
        printf("\n");
    }
    double t0 = ms(); gdouble v = 0;
    for (int i = 0; i < 1000000; ++i) gst_control_source_get_value(keys, (GstClockTime)(i % 3000) * GST_MSECOND, &v);
    printf("keys alone: %.3f microseconds a reading\n", (ms() - t0) / 1000.0);   // (a million readings)

    // 2. GES as the document: a timeline, a layer, a film as a clip; times changed; our own values hung on it; saved; read again
    t0 = ms();
    ges_init();
    printf("ges_init %.1f ms\n", ms() - t0);
    GESTimeline* tl = ges_timeline_new_audio_video();
    GESLayer* layer = ges_timeline_append_layer(tl);
    gchar* uri = gst_filename_to_uri(argv[1], NULL);
    t0 = ms();
    GESUriClip* clip = ges_uri_clip_new(uri);
    printf("a film as a clip: %.1f ms (%s)\n", ms() - t0, clip ? "made" : "NOT made");
    if (!clip) return 1;
    ges_layer_add_clip(layer, GES_CLIP(clip));
    g_object_set(clip, "start", 1500 * GST_MSECOND, "in-point", 2 * GST_SECOND, "duration", 1 * GST_SECOND, NULL);
    ges_meta_container_set_double(GES_META_CONTAINER(clip), "z", -2.5);
    ges_meta_container_set_string(GES_META_CONTAINER(clip), "matte", "thing-7");
    // what can be keyed on it: its children's properties
    guint n = 0;
    GParamSpec** props = ges_timeline_element_list_children_properties(GES_TIMELINE_ELEMENT(clip), &n);
    printf("what a film clip lets one set (and key):");
    for (guint i = 0; i < n; ++i) printf(" %s", props[i]->name);
    printf("\n");
    GList* els = ges_clip_find_track_elements(GES_CLIP(clip), NULL, GES_TRACK_TYPE_VIDEO, GES_TYPE_VIDEO_SOURCE);
    if (els) {
        GstControlSource* a = gst_interpolation_control_source_new();
        g_object_set(a, "mode", GST_INTERPOLATION_MODE_LINEAR, NULL);
        gst_timed_value_control_source_set(GST_TIMED_VALUE_CONTROL_SOURCE(a), 2 * GST_SECOND, 0.0);
        gst_timed_value_control_source_set(GST_TIMED_VALUE_CONTROL_SOURCE(a), 3 * GST_SECOND, 1.0);
        printf("keys on its clearness (alpha): %s\n", ges_track_element_set_control_source(GES_TRACK_ELEMENT(els->data), a, "alpha", "direct") ? "taken" : "REFUSED");
    } else printf("no picture part in the clip\n");
    gchar* out = gst_filename_to_uri("../out/doc_probe.xges", NULL);
    GError* err = NULL;
    t0 = ms();
    gboolean ok = ges_timeline_save_to_uri(tl, out, NULL, TRUE, &err);
    printf("saved: %s, %.1f ms\n", ok ? "yes" : err->message, ms() - t0);
    t0 = ms();
    GESTimeline* again = ges_timeline_new_from_uri(out, &err);
    // (reading is done by the main loop: wait for it)
    GMainContext* ctx = g_main_context_default();
    for (int i = 0; i < 400 && !ges_timeline_get_layers(again); ++i) { g_main_context_iteration(ctx, FALSE); g_usleep(5000); }
    for (int i = 0; i < 400; ++i) { g_main_context_iteration(ctx, FALSE); GList* ls = ges_timeline_get_layers(again); if (ls && ges_layer_get_clips(ls->data)) break; g_usleep(5000); }
    printf("read again: %.1f ms\n", ms() - t0);
    for (GList* l = ges_timeline_get_layers(again); l; l = l->next)
        for (GList* c = ges_layer_get_clips(l->data); c; c = c->next) {
            guint64 s, in, d; gdouble z = 0;
            g_object_get(c->data, "start", &s, "in-point", &in, "duration", &d, NULL);
            ges_meta_container_get_double(GES_META_CONTAINER(c->data), "z", &z);
            printf("  a clip: start %.2f in-point %.2f duration %.2f  z %.2f  matte %s\n", s / 1e9, in / 1e9, d / 1e9, z, ges_meta_container_get_string(GES_META_CONTAINER(c->data), "matte"));
        }
    return 0;
}
