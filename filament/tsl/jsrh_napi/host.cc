// Motolii's own QuickJS (QuickJS-ng 0.17, Homebrew's libqjs, as script.cpp uses it) given Node-API by
// BabylonJS/JsRuntimeHost (980cfb0, MIT: Napi::Attach on an existing JSContext, Core/Node-API/Source/env_quickjs.cc),
// with jsrh_qjs017_dawn.patch; loads Node-API addons (Dawn's dawn.node) and runs an ES module. main as QuickJS's
// own qjs.c runs a module; an addon loaded as Node's process.dlopen does (dlopen, napi_register_module_v1);
// napi_fatal_error as Node's src/node_api.cc says it.   ./build.sh && ../out/jsrh_napi/qjs_napi <module.mjs>
#include <napi/env.h>
#include <napi/js_native_api.h>
#include <quickjs.h>
#include <quickjs-libc.h>
#include <dlfcn.h>
#include <cstdio>
#include <cstdlib>
#include <cstring>

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
  JSRuntime* rt = JS_NewRuntime();
  js_std_init_handlers(rt);
  JSContext* ctx = JS_NewContext(rt);
  js_init_module_std(ctx, "qjs:std");
  js_init_module_os(ctx, "qjs:os");
  JS_SetModuleLoaderFunc2(rt, nullptr, js_module_loader, nullptr, nullptr);
  js_std_add_helpers(ctx, argc - 1, argv + 1);
  Napi::Env env = Napi::Attach(ctx);
  napi_handle_scope scope;
  napi_open_handle_scope(env, &scope);
  napi_value global, fn;
  napi_get_global(env, &global);
  napi_create_function(env, "loadAddon", NAPI_AUTO_LENGTH, load_addon, nullptr, &fn);
  napi_set_named_property(env, global, "loadAddon", fn);
  return (eval_module(ctx, argv[1]) || js_std_loop(ctx)) ? 1 : 0;
}
