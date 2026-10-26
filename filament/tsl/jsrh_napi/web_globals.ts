// What Dawn's dawn.node (ManualConstructors.cpp:30-77, AsyncRunner.cpp:68-89) and the effects' renderer expect of a
// host, and what a browser or Node gives, filled with existing ones: DOMException, URL, URLSearchParams and
// structuredClone (core-js), Event and EventTarget (event-target-shim, MIT), AbortController (abort-controller, MIT),
// TextEncoder and TextDecoder (fast-text-encoding, Apache-2.0, Google); the timers on QuickJS's own (qjs:os).
import "core-js/actual/dom-exception"
import "core-js/actual/url"
import "core-js/actual/url-search-params"
import "core-js/actual/structured-clone"
import "fast-text-encoding"
import { Event, EventTarget } from "event-target-shim"
import { AbortController, AbortSignal } from "abort-controller"
import * as os from "qjs:os"
Object.assign(globalThis, {
  Event, EventTarget, AbortController, AbortSignal,
  setImmediate: (f: Function, ...a: any[]) => os.setTimeout(() => f(...a), 0),
  setTimeout: (f: Function, ms?: number, ...a: any[]) => os.setTimeout(() => f(...a), ms ?? 0),
  clearTimeout: (t: any) => t !== undefined && os.clearTimeout(t),
  setInterval: (f: Function, ms?: number, ...a: any[]) => { const box: any = {}; const tick = () => { box.t = os.setTimeout(tick, ms ?? 0); f(...a) }; box.t = os.setTimeout(tick, ms ?? 0); return box },
  clearInterval: (box: any) => box && os.clearTimeout(box.t),
})
