// (a check, not a scene: which picture a film holds as the stage's time goes on, for cuts and speeds)
#include "../kit.h"
#include "../video.h"
int main(int argc, char** argv) {
    Stage s = open_stage(540, 960);
    s.camera->lookAt({0, 0, 4}, {0, 0, 0}, {0, 1, 0});
    s.camera->setProjection(30.0, 540.0 / 960.0, 0.05, 60.0, Camera::Fov::VERTICAL);
    const char* file = argv[1];
    struct Case { const char* name; float from, to, speed; };
    const Case cases[] = {{"whole, as it is", 0, -1, 1}, {"twice as fast", 0, -1, 2}, {"half as fast", 0, -1, 0.5f}, {"cut 2 to 3", 2, 3, 1}, {"backwards", 0, -1, -1}, {"cut 2 to 3, backwards, half as fast", 2, 3, -0.5f}, {"cut 2 to 4, backwards, twice as fast", 2, 4, -2}};
    for (const Case& c : cases) {
        Video* v = video(s, file, mat4f{}, 1.f, c.from, c.to, c.speed);
        if (!v) { std::printf("%-38s could not be shown: %s\n", c.name, video_trouble); continue; }
        std::printf("%-38s shown for %.2f s:", c.name, v->seconds);
        for (int k = 0; k < 14; ++k) {
            const float at = 0.02f + 0.25f * (float)k;
            video_at(s, v, at);
            std::printf("  %.2f", (double)GST_BUFFER_PTS(gst_sample_get_buffer(v->held)) / 1e9);
        }
        GstBus* bus = gst_element_get_bus(v->reader);
        while (GstMessage* m = gst_bus_pop(bus)) { if (GST_MESSAGE_TYPE(m) & (GST_MESSAGE_ERROR | GST_MESSAGE_EOS | GST_MESSAGE_SEGMENT_DONE | GST_MESSAGE_WARNING)) std::printf(" [%s]", GST_MESSAGE_TYPE_NAME(m)); gst_message_unref(m); }
        std::printf("   (every quarter second of the stage)\n");
        gst_element_set_state(v->reader, GST_STATE_NULL);
    }
    return 0;
}
