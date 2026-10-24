// V8 (Homebrew's, as Cavalry's own engine): the same as bench_qjs.c, for the choice of an engine.
//   clang++ -std=c++20 -O2 bench_v8.cpp -I/opt/homebrew/opt/v8/include -L/opt/homebrew/opt/v8/lib -lv8 -lv8_libplatform -lv8_libbase \
//     -DV8_COMPRESS_POINTERS -Wl,-rpath,/opt/homebrew/opt/v8/lib -o bench_v8 && ./bench_v8 sunflower.js
#include <libplatform/libplatform.h>
#include <v8.h>
#include <cstdio>
#include <fstream>
#include <sstream>
#include <time.h>
static double now_ms() { timespec t; clock_gettime(CLOCK_MONOTONIC, &t); return t.tv_sec * 1e3 + t.tv_nsec / 1e6; }
int main(int argc, char** argv) {
    std::ifstream in(argv[1]);
    std::stringstream ss; ss << in.rdbuf();
    const std::string src = ss.str();
    const double t0 = now_ms();
    v8::V8::InitializeICUDefaultLocation(argv[0]);
    std::unique_ptr<v8::Platform> platform = v8::platform::NewDefaultPlatform();
    v8::V8::InitializePlatform(platform.get());
    v8::V8::Initialize();
    v8::Isolate::CreateParams params;
    params.array_buffer_allocator = v8::ArrayBuffer::Allocator::NewDefaultAllocator();
    v8::Isolate* iso = v8::Isolate::New(params);
    {
        v8::Isolate::Scope is(iso);
        v8::HandleScope hs(iso);
        v8::Local<v8::Context> cx = v8::Context::New(iso);
        v8::Context::Scope cs(cx);
        auto str = [&](const char* s) { return v8::String::NewFromUtf8(iso, s).ToLocalChecked(); };
        v8::TryCatch tc(iso);
        auto script = v8::Script::Compile(cx, str(src.c_str()));
        if (script.IsEmpty() || script.ToLocalChecked()->Run(cx).IsEmpty()) {
            v8::String::Utf8Value e(iso, tc.Exception()); std::printf("error: %s\n", *e); return 1;
        }
        auto g = cx->Global();
        auto flower = g->Get(cx, str("flower")).ToLocalChecked().As<v8::Object>();
        v8::String::Utf8Value feat(iso, g->Get(cx, str("features")).ToLocalChecked());
        std::printf("made in %.2f ms; modern JavaScript: %s (want 4|7|2|k,1|1000)\n", now_ms() - t0, *feat);
        auto each = flower->Get(cx, str("each")).ToLocalChecked().As<v8::Function>();
        auto all = flower->Get(cx, str("all")).ToLocalChecked().As<v8::Function>();
        auto xname = str("x");
        double sum = 0, worst = 0;
        for (int p = 0; p < 60; p++) {
            const double a = now_ms();
            for (int i = 0; i < 2000; i++) {
                v8::HandleScope one(iso);
                v8::Local<v8::Value> args[2] = {v8::Integer::New(iso, i), v8::Number::New(iso, p / 60.0)};
                auto c = each->Call(cx, flower, 2, args).ToLocalChecked().As<v8::Object>();
                volatile double xv = c->Get(cx, xname).ToLocalChecked()->NumberValue(cx).FromJust(); (void)xv;
            }
            const double d = now_ms() - a; sum += d; if (d > worst) worst = d;
        }
        std::printf("a call a copy: %.2f ms a picture (worst %.2f) for 2000 copies\n", sum / 60, worst);
        sum = worst = 0;
        for (int p = 0; p < 60; p++) {
            const double a = now_ms();
            v8::HandleScope one(iso);
            v8::Local<v8::Value> args[1] = {v8::Number::New(iso, p / 60.0)};
            auto arr = all->Call(cx, flower, 1, args).ToLocalChecked().As<v8::Float32Array>();
            volatile void* data = arr->Buffer()->Data(); (void)data;
            const double d = now_ms() - a; sum += d; if (d > worst) worst = d;
        }
        std::printf("one call for all: %.2f ms a picture (worst %.2f) for 2000 copies\n", sum / 60, worst);
    }
    return 0;
}
