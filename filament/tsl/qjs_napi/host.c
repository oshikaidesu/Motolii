// A QuickJS that loads Node-API addons (Dawn's dawn.node) and runs an ES module: NativeScript's Node-API over QuickJS
// (NativeScript/runtimes 5b697b3, NativeScript/napi/quickjs: Apache-2.0, its QuickJS-ng fork MIT), with
// ns_finalize_once.patch (NativeScript/windows's own fix). main as QuickJS's own qjs.c runs a module (eval_buf);
// an addon loaded as Node's process.dlopen does (dlopen, napi_register_module_v1); napi_fatal_error as Node's
// src/node_api.cc says it.   ./build.sh && ../out/qjs_napi/qjs_napi <module.mjs> [args]
#include "quickjs-api.c"
#include "quickjs-libc.h"
#include <dlfcn.h>
#include <stdio.h>

NAPI_EXTERN __attribute__((noreturn)) void NAPI_CDECL napi_fatal_error(const char* location, size_t location_len,
                                                                      const char* message, size_t message_len) {
  fprintf(stderr, "FATAL ERROR: %.*s %.*s\n", (int)(location_len == NAPI_AUTO_LENGTH ? strlen(location) : location_len), location,
          (int)(message_len == NAPI_AUTO_LENGTH ? strlen(message) : message_len), message);
  abort();
}

typedef napi_value (*napi_init)(napi_env, napi_value);

static napi_value load_addon(napi_env env, napi_callback_info info) {
  size_t argc = 1;
  napi_value arg;
  napi_get_cb_info(env, info, &argc, &arg, NULL, NULL);
  char path[4096];
  size_t len;
  napi_get_value_string_utf8(env, arg, path, sizeof(path), &len);
  void* lib = dlopen(path, RTLD_NOW | RTLD_LOCAL);
  if (!lib) { napi_throw_error(env, NULL, dlerror()); return NULL; }
  napi_init init = (napi_init)dlsym(lib, "napi_register_module_v1");
  if (!init) { napi_throw_error(env, NULL, "napi_register_module_v1 not found"); return NULL; }
  napi_value exports;
  napi_create_object(env, &exports);
  napi_value r = init(env, exports);
  return r ? r : exports;
}

static int eval_module(JSContext* ctx, const char* filename) {
  size_t buf_len;
  uint8_t* buf = js_load_file(ctx, &buf_len, filename);
  if (!buf) { perror(filename); return -1; }
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
  napi_runtime runtime;
  napi_env env;
  qjs_create_runtime(&runtime);
  qjs_create_napi_env(&env, runtime);
  JSContext* ctx = env->context;
  JSRuntime* rt = JS_GetRuntime(ctx);
  js_std_init_handlers(rt);
  js_init_module_std(ctx, "qjs:std");
  js_init_module_os(ctx, "qjs:os");
  JS_SetModuleLoaderFunc(rt, NULL, js_module_loader, NULL);
  js_std_add_helpers(ctx, argc - 1, argv + 1);
  napi_handle_scope scope;
  napi_open_handle_scope(env, &scope);
  napi_value global, fn;
  napi_get_global(env, &global);
  napi_create_function(env, "loadAddon", NAPI_AUTO_LENGTH, load_addon, NULL, &fn);
  napi_set_named_property(env, global, "loadAddon", fn);
  // inside JS for the whole run, as NativeScript's napi_run_script entry counts it (js_enter/js_exit): jobs run
  // between tasks (js_std_await, js_std_loop), never in the middle of a Node-API call
  js_enter(env);
  return (eval_module(ctx, argv[1]) || js_std_loop(ctx)) ? 1 : 0;
}
