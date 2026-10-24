// JavaScriptCore (the system's, with its JIT): the same as bench_qjs.c, for the choice of an engine.
//   cc -O2 bench_jsc.c -framework JavaScriptCore -o bench_jsc && ./bench_jsc sunflower.js
#include <JavaScriptCore/JavaScriptCore.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
static double now_ms(void) { struct timespec t; clock_gettime(CLOCK_MONOTONIC, &t); return t.tv_sec * 1e3 + t.tv_nsec / 1e6; }
static JSValueRef get(JSContextRef cx, JSValueRef o, const char* name) {
    JSStringRef s = JSStringCreateWithUTF8CString(name);
    JSValueRef v = JSObjectGetProperty(cx, (JSObjectRef)o, s, NULL);
    JSStringRelease(s);
    return v;
}
int main(int argc, char** argv) {
    FILE* f = fopen(argv[1], "rb"); fseek(f, 0, SEEK_END); long n = ftell(f); rewind(f);
    char* src = malloc(n + 1); fread(src, 1, n, f); src[n] = 0; fclose(f);
    const double t0 = now_ms();
    JSGlobalContextRef cx = JSGlobalContextCreate(NULL);
    JSStringRef code = JSStringCreateWithUTF8CString(src);
    JSValueRef err = NULL;
    JSEvaluateScript(cx, code, NULL, NULL, 1, &err);
    if (err) { JSStringRef e = JSValueToStringCopy(cx, err, NULL); char b[512]; JSStringGetUTF8CString(e, b, sizeof b); printf("error: %s\n", b); return 1; }
    JSObjectRef g = JSContextGetGlobalObject(cx);
    JSValueRef flower = get(cx, g, "flower");
    JSStringRef fs = JSValueToStringCopy(cx, get(cx, g, "features"), NULL);
    char feat[128]; JSStringGetUTF8CString(fs, feat, sizeof feat);
    printf("made in %.2f ms; modern JavaScript: %s (want 4|7|2|k,1|1000)\n", now_ms() - t0, feat);
    JSObjectRef each = (JSObjectRef)get(cx, flower, "each"), all = (JSObjectRef)get(cx, flower, "all");
    double sum = 0, worst = 0;
    for (int p = 0; p < 60; p++) {
        const double a = now_ms();
        for (int i = 0; i < 2000; i++) {
            JSValueRef args[2] = {JSValueMakeNumber(cx, i), JSValueMakeNumber(cx, p / 60.0)};
            JSValueRef c = JSObjectCallAsFunction(cx, each, (JSObjectRef)flower, 2, args, NULL);
            volatile double xv = JSValueToNumber(cx, get(cx, c, "x"), NULL); (void)xv;
        }
        const double d = now_ms() - a; sum += d; if (d > worst) worst = d;
    }
    printf("a call a copy: %.2f ms a picture (worst %.2f) for 2000 copies\n", sum / 60, worst);
    sum = worst = 0;
    for (int p = 0; p < 60; p++) {
        const double a = now_ms();
        JSValueRef args[1] = {JSValueMakeNumber(cx, p / 60.0)};
        JSObjectRef arr = (JSObjectRef)JSObjectCallAsFunction(cx, all, (JSObjectRef)flower, 1, args, NULL);
        volatile float* data = JSObjectGetTypedArrayBytesPtr(cx, arr, NULL); (void)data;
        const double d = now_ms() - a; sum += d; if (d > worst) worst = d;
    }
    printf("one call for all: %.2f ms a picture (worst %.2f) for 2000 copies\n", sum / 60, worst);
    return 0;
}
