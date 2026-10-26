// What Dawn's dawn.node expects of its host (ManualConstructors.cpp:30-77, AsyncRunner.cpp:68-89), and what Node gives:
// DOMException (core-js), Event and EventTarget (event-target-shim, MIT), setImmediate (a qjs:os timer of 0).
import "core-js/actual/dom-exception"
import { Event, EventTarget } from "event-target-shim"
import * as os from "qjs:os"
Object.assign(globalThis, { Event, EventTarget, setImmediate: (f: Function, ...a: any[]) => os.setTimeout(() => f(...a), 0) })
