// QuickJS-ng: the script made once, then each of 60 pictures asks it for every copy, two ways (a call a copy; one
// call for all, a typed array back). The time a picture, and whether modern JavaScript ran.
#include <quickjs.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
static double now_ms(void) { struct timespec t; clock_gettime(CLOCK_MONOTONIC, &t); return t.tv_sec * 1e3 + t.tv_nsec / 1e6; }
int main(int argc, char** argv) {
    FILE* f = fopen(argv[1], "rb"); fseek(f, 0, SEEK_END); long n = ftell(f); rewind(f);
    char* src = malloc(n + 1); fread(src, 1, n, f); src[n] = 0; fclose(f);
    const double t0 = now_ms();
    JSRuntime* rt = JS_NewRuntime(); JSContext* cx = JS_NewContext(rt);
    JSValue r = JS_Eval(cx, src, n, argv[1], JS_EVAL_TYPE_GLOBAL);
    if (JS_IsException(r)) { JSValue e = JS_GetException(cx); const char* s = JS_ToCString(cx, e); printf("error: %s\n", s); return 1; }
    JS_FreeValue(cx, r);
    JSValue g = JS_GetGlobalObject(cx), flower = JS_GetPropertyStr(cx, g, "flower"), feat = JS_GetPropertyStr(cx, g, "features");
    const char* fs = JS_ToCString(cx, feat);
    printf("made in %.2f ms; modern JavaScript: %s (want 4|7|2|k,1|1000)\n", now_ms() - t0, fs);
    JSValue each = JS_GetPropertyStr(cx, flower, "each"), all = JS_GetPropertyStr(cx, flower, "all");
    double sum = 0, worst = 0;
    for (int p = 0; p < 60; p++) {
        const double a = now_ms();
        for (int i = 0; i < 2000; i++) {
            JSValue args[2] = {JS_NewInt32(cx, i), JS_NewFloat64(cx, p / 60.0)};
            JSValue c = JS_Call(cx, each, flower, 2, args);
            JSValue x = JS_GetPropertyStr(cx, c, "x"); double xv; JS_ToFloat64(cx, &xv, x); JS_FreeValue(cx, x);
            JS_FreeValue(cx, c);
        }
        const double d = now_ms() - a; sum += d; if (d > worst) worst = d;
    }
    printf("a call a copy: %.2f ms a picture (worst %.2f) for 2000 copies\n", sum / 60, worst);
    sum = worst = 0;
    for (int p = 0; p < 60; p++) {
        const double a = now_ms();
        JSValue args[1] = {JS_NewFloat64(cx, p / 60.0)};
        JSValue arr = JS_Call(cx, all, flower, 1, args);
        size_t off, len, bpe; JSValue buf = JS_GetTypedArrayBuffer(cx, arr, &off, &len, &bpe);
        size_t size; uint8_t* data = JS_GetArrayBuffer(cx, &size, buf);
        (void)data; JS_FreeValue(cx, buf); JS_FreeValue(cx, arr);
        const double d = now_ms() - a; sum += d; if (d > worst) worst = d;
    }
    printf("one call for all: %.2f ms a picture (worst %.2f) for 2000 copies\n", sum / 60, worst);
    return 0;
}
