// kit_draw.h: part of kit.h. Read only through kit.h.
// A film moves things between its pictures: every thing made of mesh `m` goes where `to` says (told which of the
// mesh's places it is, and that place). Poses only: nothing is worked out here that a picture is made of.
static void move(Stage& s, const Mesh* m, const std::function<mat4f(int, const mat4f&)>& to, const mat4f* eye = nullptr) {
    auto& tm = s.engine->getTransformManager();
    for (const Stage::Made& d : s.made) {
        if (d.of != m) continue;
        if (!d.many) { tm.setTransform(tm.getInstance(d.thing), to(d.nth, d.place)); continue; }
        std::vector<mat4f> all((size_t)d.count);   // (a group drawn in one go: its places told at once, those further off first when an eye is given, as flat things lie)
        for (int k = 0; k < d.count; ++k) all[(size_t)k] = to(d.nth + k, d.place);
        if (eye) std::stable_sort(all.begin(), all.end(), [eye](const mat4f& a, const mat4f& b) { return (*eye * a[3]).z < (*eye * b[3]).z; });
        d.many->setLocalTransforms(all.data(), all.size());
    }
}

// A film of the stage, 60 pictures a second, written by ffmpeg, its kind by the file's ending as ffmpeg takes one: .mp4
// H.264; .mov HEVC by the Mac's own encoder (VideoToolbox, tagged hvc1 as QuickTime plays it); .gif by ffmpeg's palette
// (palettegen, paletteuse: its own way to a good GIF) at 15 a second, as Alight Motion writes one; .png a picture each,
// name_0001.png on, .jpg the same as JPEGs (no sound in the last three). Before each picture `pose` is told how far through
// the film it is (0 up to 1): where the end meets the start, the film loops. What is heard is the things' own sound
// (Stage::sounding), cut, sped, gone round, made as loud as it says (ffmpeg's volume) and mixed by ffmpeg as each thing says.
// How it is written, as Alight Motion's export asks (0: as it was): how high (ffmpeg's scale, the shape kept, Lanczos),
// how many pictures a second (ffmpeg's -r: some of the 60 left out), how many megabits a second (-b:v, in place of
// x264's -crf 17 or VideoToolbox's -q:v 65).
// blur: so many moments drawn for each picture and laid one on another (After Effects' and Blender's motion blur: a
// 180° shutter round the picture's own moment, as After Effects' shutter angle 180 and phase -90): what moves is smeared
// along its way, what stays is as sharp as before.
// alpha: where nothing is drawn, clear (Filament's own: the stage's view laid as translucent, its pictures as they come
// out times their alpha, made plain again for ffmpeg); kept by .png and .gif, a .mov's HEVC with its alpha
// (VideoToolbox's own, as QuickTime and Final Cut read it); .mp4 and .jpg have none.
struct FilmWay { int high = 0, rate = 0; double mbps = 0.0; int blur = 0; bool alpha = false; };
static bool film(Stage& s, const char* path, int pictures, const std::function<void(double)>& pose, const FilmWay& said = {}) {
    const auto quoted = [](std::string name) {   // (a file's name as a shell takes it)
        for (size_t at = 0; (at = name.find('\'', at)) != std::string::npos; at += 4) name.replace(at, 1, "'\\''");
        return "'" + name + "'";
    };
    std::string files, each, all;
    int n = 0;
    for (const auto& [who, tells] : s.sounding) {
        const Stage::Sound a = tells();
        const double fast = std::fabs(a.speed);
        std::string loud;   // (keyed: ffmpeg's asendcmd telling its volume each 1/30 s, counted from when it comes on)
        for (size_t i = 0; i < a.loud.size(); ++i)
            if (i == 0 || a.loud[i] != a.loud[i - 1]) loud += (loud.empty() ? "" : ";") + std::to_string(i / 30.0) + " volume@v" + std::to_string(n + 1) + " volume " + std::to_string(a.loud[i]);
        std::string halved;   // (ffmpeg's atempo goes no slower than half at a time)
        double left = fast;
        for (; left < 0.5; left /= 0.5) halved += "atempo=0.5,";
        std::vector<char> how(700 + loud.size());   // (room for its volume's commands)
        std::snprintf(how.data(), how.size(), "[%d:a]aresample=48000,atrim=start=%g:end=%g,asetpts=PTS-STARTPTS,%s%satempo=%g,aloop=loop=-1:size=%ld,atrim=start=%g%s%s,asetpts=PTS-STARTPTS,%svolume@v%d=%g,adelay=delays=%ld:all=1[a%d];",
                      n + 1, a.from, a.to, a.speed < 0.0 ? "areverse," : "", halved.c_str(), left, std::lround((a.to - a.from) / fast * 48000.0), a.into,
                      a.lasts < 0.0 ? "" : ":duration=", a.lasts < 0.0 ? "" : std::to_string(a.lasts).c_str(), loud.empty() ? "" : ("asendcmd=c='" + loud + "',").c_str(), n + 1,
                      a.loud.empty() ? a.gain : a.loud[0], std::lround(a.on * 1000.0), n + 1);
        files += " -i " + quoted(a.file);
        each += how.data();
        all += "[a" + std::to_string(++n) + "]";
    }
    const std::string ending = std::filesystem::path(path).extension().string();
    const bool silent = ending == ".gif" || ending == ".png" || ending == ".jpg";
    const std::string sound = n && !silent ? files + " -filter_complex \"" + each + all + "amix=inputs=" + std::to_string(n) + ":normalize=0:duration=longest,alimiter[a]\" -map 0:v -map '[a]' -c:a aac -b:a 160k -t " + std::to_string(pictures / 60.0) : "";
    const std::string scale = said.high > 0 ? "scale=-2:" + std::to_string(said.high) + ":flags=lanczos" : "", mbps = std::to_string(said.mbps) + "M";
    const std::string sized = scale.empty() || ending == ".gif" ? "" : " -vf " + scale, rate = said.rate > 0 && ending != ".gif" ? " -r " + std::to_string(said.rate) : "";
    const std::string way = ending == ".gif" ? " -an -vf \"fps=15," + (scale.empty() ? "" : scale + ",") + "split[a][b];[a]palettegen[p];[b][p]paletteuse\" " + quoted(path)
                            : ending == ".png" ? " -an" + sized + rate + " -c:v png " + quoted(std::filesystem::path(path).replace_extension().string() + "_%04d.png")
                            : ending == ".jpg" ? " -an" + sized + rate + " -c:v mjpeg -q:v 2 -pix_fmt yuvj420p " + quoted(std::filesystem::path(path).replace_extension().string() + "_%04d.jpg")   // (ffmpeg's JPEG, -q:v 2 its best but one)
                            : ending == ".mov" ? sized + rate + " -c:v hevc_videotoolbox " + (said.mbps > 0.0 ? "-b:v " + mbps : std::string("-q:v 65")) + (said.alpha ? " -alpha_quality 0.75 -pix_fmt bgra" : " -pix_fmt yuv420p") + " -tag:v hvc1 -movflags +faststart " + quoted(path)
                                               : sized + rate + " -c:v libx264 " + (said.mbps > 0.0 ? "-b:v " + mbps + " -maxrate " + mbps + " -bufsize " + std::to_string(2.0 * said.mbps) + "M" : std::string("-crf 17")) + " -g 60 -pix_fmt yuv420p -movflags +faststart " + quoted(path);
    const std::string cmd = "ffmpeg -loglevel error -y -f rawvideo -pix_fmt rgba -s " + std::to_string(s.w) + "x" + std::to_string(s.h) + " -r 60 -i -" + sound + way;
    std::signal(SIGPIPE, SIG_IGN);   // (if ffmpeg gives up, writing to it fails and that is told, instead of this program ending with it)
    FILE* to = popen(cmd.c_str(), "w");
    if (!to) return false;
    std::vector<uint8_t> pic((size_t)s.w * s.h * 4);
    s.by_clock = false;
    if (said.alpha) s.view->setBlendMode(View::BlendMode::TRANSLUCENT);   // (Filament keeps the alpha through its post-processing so: View.h)
    const int samples = std::max(1, said.blur);
    std::vector<uint32_t> sum(samples > 1 ? pic.size() : 0);
    for (int i = 0; i < pictures; ++i) {
        if (samples > 1) std::fill(sum.begin(), sum.end(), 0u);
        for (int k = 0; k < samples; ++k) {
            const double at = samples > 1 ? std::max(0.0, i + 0.5 * ((k + 0.5) / samples - 0.5)) : (double)i;   // (its moment, or one of those round it)
            time_is(s, at / 60.0);
            pose(at / pictures);
            draw(s, &pic);
            if (samples > 1) for (size_t j = 0; j < pic.size(); ++j) sum[j] += pic[j];
        }
        if (samples > 1) for (size_t j = 0; j < pic.size(); ++j) pic[j] = (uint8_t)((sum[j] + samples / 2) / samples);
        if (said.alpha)   // (as plain colour, as ffmpeg's rgba is, not times its alpha as Filament gives it)
            for (size_t j = 0; j < pic.size(); j += 4)
                if (const int a = pic[j + 3]; a > 0 && a < 255) for (int c = 0; c < 3; ++c) pic[j + c] = (uint8_t)std::min(255, (pic[j + c] * 255 + a / 2) / a);
        if (std::fwrite(pic.data(), 1, pic.size(), to) != pic.size()) break;
    }
    if (said.alpha) s.view->setBlendMode(View::BlendMode::OPAQUE);
    return pclose(to) == 0;
}

// The same film played without being kept, to see what a picture costs. Two measures.
// Flat out, the GPU waited for after every fourth picture: pictures a second. Steady to a few hundredths on Metal, so
// good for telling two versions apart there; NOT for comparing backends: how often one waits moves it by a
// millisecond or two, and differently on each (waiting after every picture or every second one, WebGPU and Metal
// read alike; after every fourth, WebGPU reads a millisecond or two dearer; seldom or never, both are held back).
// (And at 60 a second the GPU slows its clock, so a picture's own time cannot be read off a paced run either.)
static void clock(Stage& s, int pictures, const std::function<void(float)>& pose) {
    s.by_clock = true;
    const auto from = std::chrono::steady_clock::now();
    for (int i = 0; i < pictures; ++i) {
        time_is(s, (double)i / 60.0);
        pose((float)i / (float)pictures);
        draw(s, nullptr, i % 4 == 3);
    }
    s.engine->flushAndWait();
    const double took = std::chrono::duration<double>(std::chrono::steady_clock::now() - from).count();
    std::printf("%.0f pictures a second, %.2f ms each (%.1f times 60)\n", pictures / took, 1000.0 * took / pictures, pictures / took / 60.0);
    for (const auto& word : s.said_after) word();
    // And by the clock, as a screen asks for them: on every tick Filament is asked once whether it will draw (it says
    // no when it is behind), nothing is waited for. Pictures drawn and pictures left out, at 60, 120 and 240 a second.
    for (int rate : {60, 120, 240}) {
        int drawn = 0, left_out = 0, late = 0;   // (late: begun a whole tick after its time, the moment's own work too slow; Filament does not count those)
        const auto start = std::chrono::steady_clock::now();
        for (int i = 0; i < 2 * rate; ++i) {   // two seconds
            const auto due = start + std::chrono::microseconds((long long)i * 1000000 / rate);
            if (std::chrono::steady_clock::now() > due + std::chrono::microseconds(1000000 / rate)) ++late;
            std::this_thread::sleep_until(due);
            time_is(s, (double)i / (double)rate);
            pose((float)(i % (pictures * rate / 60)) / (float)(pictures * rate / 60));
            if (!begins(s)) { ++left_out; continue; }
            pictures_of(s);
            s.renderer->endFrame();
            ++drawn;
        }
        const auto asked = std::chrono::steady_clock::now();
        s.engine->flushAndWait();
        std::printf("at %d a second: %d drawn, %d left out, %d late; the last was done %.1f ms after it was asked for, the two seconds took %.2f\n", rate, drawn, left_out, late,
                    std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - asked).count(), std::chrono::duration<double>(asked - start).count());
        for (const auto& word : s.said_after) word();
    }
}

// The stage played on and on by the clock at 60 a second, with no jump in time (the film goes round as often as it
// takes): how it holds up over minutes. A line a minute: pictures drawn and left out (Filament says no when it is
// behind), the longest this thread took over one picture, and how much memory the program holds.
static void soak(Stage& s, double seconds, int pictures, const std::function<void(float)>& pose) {
    s.by_clock = true;
    const auto start = std::chrono::steady_clock::now();
    const long ticks = (long)(seconds * 60.0);
    int drawn = 0, left_out = 0, all_drawn = 0, all_left = 0;
    double longest = 0.0;
    for (long i = 0; i < ticks; ++i) {
        std::this_thread::sleep_until(start + std::chrono::microseconds(i * 1000000 / 60));
        const auto began = std::chrono::steady_clock::now();
        time_is(s, (double)i / 60.0);
        pose((float)((double)i / (double)pictures));   // (on past 1: time does not go back)
        if (begins(s)) {
            pictures_of(s);
            s.renderer->endFrame();
            ++drawn;
        } else ++left_out;
        longest = std::max(longest, std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - began).count());
        if ((i + 1) % 3600 == 0 || i + 1 == ticks) {
            mach_task_basic_info_data_t info;
            mach_msg_type_number_t count = MACH_TASK_BASIC_INFO_COUNT;
            task_info(mach_task_self(), MACH_TASK_BASIC_INFO, (task_info_t)&info, &count);
            std::printf("minute %2ld: %d drawn, %d left out; longest %.1f ms; memory %.0f MB\n", (i + 1 + 3599) / 3600, drawn, left_out, longest, (double)info.resident_size / 1048576.0);
            std::fflush(stdout);
            for (const auto& word : s.said_after) word();
            all_drawn += drawn; all_left += left_out;
            drawn = left_out = 0;
            longest = 0.0;
        }
    }
    s.engine->flushAndWait();
    std::printf("in all: %d drawn, %d left out, over %.0f seconds\n", all_drawn, all_left, seconds);
}

static bool shoot(Stage& s, const char* path) {
    static std::vector<uint8_t> pic;
    pic.resize((size_t)s.w * s.h * 4);
    for (int i = 0; i < 3; ++i) draw(s, i == 2 ? &pic : nullptr);
    uint8_t* plane = pic.data();
    NSBitmapImageRep* rep = [[NSBitmapImageRep alloc] initWithBitmapDataPlanes:&plane pixelsWide:s.w pixelsHigh:s.h bitsPerSample:8 samplesPerPixel:4
                                                                        hasAlpha:YES isPlanar:NO colorSpaceName:NSDeviceRGBColorSpace bytesPerRow:s.w * 4 bitsPerPixel:32];
    return [[rep representationUsingType:NSBitmapImageFileTypePNG properties:@{}] writeToFile:@(path) atomically:YES];
}

// The sun turned to shine another way (a film's pose): the shade and the light the glass throws go with it.
static void sun_to(Stage& s, float3 way) {
    auto& lights = s.engine->getLightManager();
    lights.setDirection(lights.getInstance(s.sun), way);
    for (MaterialInstance* mi : s.by_sun) mi->setParameter("sunWay", way);
}

// What a picture's program does once its stage is set: the still (out/<name>.png); or, asked for `film`, the film
// (out/<name>.mp4); or, asked for `time`, the film played for its cost.
static int show(Stage& s, int argc, char** argv, const char* name, int pictures, const std::function<void(float)>& pose) {
    const std::string still = std::string("out/") + name + ".png", moving = std::string("out/") + name + ".mp4";
    if (argc > 1 && !strcmp(argv[1], "time")) return clock(s, pictures, pose), 0;
    if (argc > 1 && !strcmp(argv[1], "soak")) return soak(s, argc > 2 ? atof(argv[2]) : 60.0, pictures, pose), 0;   // (soak 600: ten minutes)
    const bool as_film = argc > 1 && !strcmp(argv[1], "film");
    if (!as_film) {   // (a still: the stage three tenths of the way through its film)
        s.by_clock = false;
        time_is(s, 0.3 * (double)pictures / 60.0);
        pose(0.3f);
    }
    if (!(as_film ? film(s, moving.c_str(), pictures, pose) : shoot(s, still.c_str()))) return fail(as_film ? "film" : "png");
    std::printf("wrote %s\n", (as_film ? moving : still).c_str());
    return 0;
}
