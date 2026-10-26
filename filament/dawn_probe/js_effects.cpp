// js_effects.cpp: a web effect's own renderer (shader-effects-inc/shaders, npm shaders 4.0.5) drawing inside this
// process on Filament's own Dawn device. JS runs on Motolii's QuickJS with JsRuntimeHost's Node-API (as
// tsl/jsrh_napi/host.cc runs it); Dawn's JS binding (dawn.node with dawn_lend.patch) is given Filament's Dawn proc
// table (motolii_dawn_procs) and lent Filament's device and a texture of it (wrapDevice, wrapTexture); the effect is
// drawn by tsl/jsrh_napi/shaders.ts. After it, the texture is read back here, on the same device.
//   ./dawn_probe/js_effects.sh <preset.json> <w> <h> <frames> <out.rgba>
#include "gpu_door.h"
#include <filament/Engine.h>
#include <dawn/native/DawnNative.h>
#include <dawn/webgpu_cpp.h>
#include <napi/env.h>
#include <napi/js_native_api.h>
#include <quickjs.h>
#include <quickjs-libc.h>
#include <dlfcn.h>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <string>

// Filament's Dawn as Dawn's JS binding should call it (dawn_lend.patch, Module.cpp)
extern "C" __attribute__((visibility("default"))) const DawnProcTable* motolii_dawn_procs() { return &dawn::native::GetProcs(); }

// (as tsl/jsrh_napi/host.cc)
extern "C" __attribute__((noreturn)) void napi_fatal_error(const char* location, size_t location_len, const char* message, size_t message_len) {
  std::fprintf(stderr, "FATAL ERROR: %.*s %.*s\n", (int)(location_len == NAPI_AUTO_LENGTH ? std::strlen(location) : location_len), location,
               (int)(message_len == NAPI_AUTO_LENGTH ? std::strlen(message) : message_len), message);
  std::abort();
}

typedef napi_value (*napi_init)(napi_env, napi_value);

static napi_value load_addon(napi_env env, napi_callback_info info) {
  size_t argc = 1;
  napi_value arg;
  napi_get_cb_info(env, info, &argc, &arg, nullptr, nullptr);
  char path[4096];
  size_t len;
  napi_get_value_string_utf8(env, arg, path, sizeof(path), &len);
  void* lib = dlopen(path, RTLD_NOW | RTLD_LOCAL);
  if (!lib) { napi_throw_error(env, nullptr, dlerror()); return nullptr; }
  auto init = (napi_init)dlsym(lib, "napi_register_module_v1");
  if (!init) { napi_throw_error(env, nullptr, "napi_register_module_v1 not found"); return nullptr; }
  napi_value exports;
  napi_create_object(env, &exports);
  napi_value r = init(env, exports);
  return r ? r : exports;
}

static int eval_module(JSContext* ctx, const char* filename) {
  size_t buf_len;
  uint8_t* buf = js_load_file(ctx, &buf_len, filename);
  if (!buf) { std::perror(filename); return -1; }
  JSValue val = JS_Eval(ctx, (const char*)buf, buf_len, filename, JS_EVAL_TYPE_MODULE | JS_EVAL_FLAG_COMPILE_ONLY);
  js_free(ctx, buf);
  if (!JS_IsException(val)) {
    js_module_set_import_meta(ctx, val, true, true);
    val = JS_EvalFunction(ctx, val);
  }
  val = js_std_await(ctx, val);
  int ret = JS_IsException(val) ? -1 : 0;
  if (ret) js_std_dump_error(ctx);
  JS_FreeValue(ctx, val);
  return ret;
}

int main(int argc, char** argv) {
  if (argc < 6) { std::printf("usage: js_effects <preset.json> <w> <h> <frames> <out.rgba>\n"); return 2; }
  const uint32_t W = (uint32_t)std::atoi(argv[2]), H = (uint32_t)std::atoi(argv[3]);
  filament::Engine* engine = filament::Engine::Builder().backend(filament::Engine::Backend::WEBGPU).build();
  if (!engine || !door::device(engine)) { std::printf("FAIL no Filament WebGPU device\n"); return 1; }
  WGPUDevice device = (WGPUDevice)door::device(engine);
  WGPUInstance instance = (WGPUInstance)door::instance(engine);
  wgpu::TextureDescriptor td{};
  td.size = {W, H, 1};
  td.format = wgpu::TextureFormat::RGBA8Unorm;
  td.usage = wgpu::TextureUsage::RenderAttachment | wgpu::TextureUsage::TextureBinding | wgpu::TextureUsage::CopySrc;
  wgpu::Texture target = wgpu::Device(device).CreateTexture(&td);
  engine->flushAndWait();   // (Filament's own work done before the effect uses its device on this thread)

  JSRuntime* rt = JS_NewRuntime();
  js_std_init_handlers(rt);
  JSContext* ctx = JS_NewContext(rt);
  js_init_module_std(ctx, "qjs:std");
  js_init_module_os(ctx, "qjs:os");
  JS_SetModuleLoaderFunc2(rt, nullptr, js_module_loader, nullptr, nullptr);
  js_std_add_helpers(ctx, 0, nullptr);
  Napi::Env env = Napi::Attach(ctx);
  napi_handle_scope scope;
  napi_open_handle_scope(env, &scope);
  napi_value global, fn;
  napi_get_global(env, &global);
  napi_create_function(env, "loadAddon", NAPI_AUTO_LENGTH, load_addon, nullptr, &fn);
  napi_set_named_property(env, global, "loadAddon", fn);
  char lend[1024];
  std::snprintf(lend, sizeof lend, "globalThis.motolii = { device: %llun, instance: %llun, texture: %llun, args: ['%s', '%s', '%s', '%s', '%s.js'] }",
                (unsigned long long)(uintptr_t)device, (unsigned long long)(uintptr_t)instance, (unsigned long long)(uintptr_t)target.Get(),
                argv[1], argv[2], argv[3], argv[4], argv[5]);
  JS_FreeValue(ctx, JS_Eval(ctx, lend, std::strlen(lend), "<motolii>", JS_EVAL_TYPE_GLOBAL));
  const int failed = eval_module(ctx, std::getenv("EFFECT_JS")) || js_std_loop(ctx);

  // the picture read back here, on Filament's device (a row of a multiple of 256 bytes: WebGPU's copy rule)
  wgpu::Device dev(device);
  const uint32_t stride = (W * 4 + 255) & ~255u;
  wgpu::BufferDescriptor room{};
  room.size = (uint64_t)stride * H;
  room.usage = wgpu::BufferUsage::CopyDst | wgpu::BufferUsage::MapRead;
  wgpu::Buffer back = dev.CreateBuffer(&room);
  wgpu::TexelCopyTextureInfo src{};
  src.texture = target;
  wgpu::TexelCopyBufferInfo dst{};
  dst.buffer = back;
  dst.layout.bytesPerRow = stride;
  dst.layout.rowsPerImage = H;
  wgpu::Extent3D all{W, H, 1};
  wgpu::CommandEncoder coder = dev.CreateCommandEncoder();
  coder.CopyTextureToBuffer(&src, &dst, &all);
  wgpu::CommandBuffer done = coder.Finish();
  dev.GetQueue().Submit(1, &done);
  bool mapped = false;
  (void)back.MapAsync(wgpu::MapMode::Read, 0, room.size, wgpu::CallbackMode::AllowProcessEvents,
                      [&](wgpu::MapAsyncStatus how, wgpu::StringView) { mapped = how == wgpu::MapAsyncStatus::Success; });
  while (!mapped) wgpuInstanceProcessEvents(instance);
  const char* got = (const char*)back.GetConstMappedRange(0, room.size);
  std::ofstream out(argv[5], std::ios::binary);
  for (uint32_t y = 0; y < H; ++y) out.write(got + (size_t)y * stride, W * 4);
  std::printf("%s: read back on Filament's device %p\n", failed ? "FAIL" : "PASS", (void*)device);
  return failed;
}
