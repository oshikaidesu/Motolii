import Cocoa
import FlutterMacOS

class MainFlutterWindow: NSWindow {
  override func awakeFromNib() {
    let flutterViewController = FlutterViewController()
    self.contentViewController = flutterViewController
    // the lab's window opens large: a window is judged at its real size
    let screen = NSScreen.main?.visibleFrame ?? NSRect(x: 0, y: 0, width: 1512, height: 900)
    self.setFrame(NSRect(x: screen.minX + 20, y: screen.minY + 20, width: min(1480, screen.width - 40), height: min(940, screen.height - 40)), display: true)

    RegisterGeneratedPlugins(registry: flutterViewController)
    GlassTexture.attach(to: flutterViewController)
    StageTexture.attach(to: flutterViewController)

    super.awakeFromNib()
    // after the xib has restored its own frame
    self.setFrame(NSRect(x: screen.minX + 20, y: screen.minY + 20, width: min(1480, screen.width - 40), height: min(940, screen.height - 40)), display: true)
  }
}

/// The live glass (motolii/filament/out/libglass_live.dylib) as a Flutter texture, without a copy: the library draws each
/// frame into a pixel buffer on the GPU, and the last finished buffer is handed to Flutter as it is.
/// Channel "motolii/glass", method "start" {lib, base, width, height, env: {name: value}} -> texture id, or -1.
/// Everything else (sun, camera, pace) the Dart side sets through the library's own C functions.
final class GlassTexture: NSObject, FlutterTexture {
  private typealias FrontFn = @convention(c) () -> UnsafeMutableRawPointer?
  private typealias FramesFn = @convention(c) () -> UInt64
  private typealias EnvFn = @convention(c) (UnsafePointer<CChar>, UnsafePointer<CChar>) -> Void
  private typealias StartFn = @convention(c) (UnsafePointer<CChar>, Int32, Int32) -> Int32

  private static var shared: GlassTexture?
  private let front: FrontFn
  private let frames: FramesFn
  private let textures: FlutterTextureRegistry
  private var id: Int64 = -1
  private var seen: UInt64 = 0
  private var timer: Timer?

  static func attach(to controller: FlutterViewController) {
    let registrar = controller.registrar(forPlugin: "MotoliiGlass")
    let channel = FlutterMethodChannel(name: "motolii/glass", binaryMessenger: registrar.messenger)
    channel.setMethodCallHandler { call, result in
      guard call.method == "start", let a = call.arguments as? [String: Any], let lib = a["lib"] as? String, let base = a["base"] as? String else {
        result(FlutterMethodNotImplemented)
        return
      }
      if shared == nil {
        shared = GlassTexture(
          lib: lib, base: base, width: Int32(a["width"] as? Int ?? 1920), height: Int32(a["height"] as? Int ?? 1080),
          env: a["env"] as? [String: String] ?? [:], textures: registrar.textures)
      } else {
        shared?.resume(base: base)
      }
      result(shared?.id ?? -1)
    }
  }

  private let start: StartFn
  private init?(lib: String, base: String, width: Int32, height: Int32, env: [String: String], textures: FlutterTextureRegistry) {
    guard let h = dlopen(lib, RTLD_NOW), let f = dlsym(h, "glass_front"), let n = dlsym(h, "glass_frames"), let e = dlsym(h, "glass_env"),
      let s = dlsym(h, "glass_start")
    else { return nil }
    front = unsafeBitCast(f, to: FrontFn.self)
    frames = unsafeBitCast(n, to: FramesFn.self)
    start = unsafeBitCast(s, to: StartFn.self)
    self.textures = textures
    super.init()
    let setEnv = unsafeBitCast(e, to: EnvFn.self)
    for (name, value) in env { setEnv(name, value) }
    _ = start(base, width, height)
    id = textures.register(self)
    // tell Flutter whenever the library has finished another frame
    timer = Timer.scheduledTimer(withTimeInterval: 1.0 / 120.0, repeats: true) { [weak self] _ in
      guard let me = self else { return }
      let now = me.frames()
      if now != me.seen {
        me.seen = now
        me.textures.textureFrameAvailable(me.id)
      }
    }
    RunLoop.main.add(timer!, forMode: .common)
  }

  private func resume(base: String) { _ = start(base, 0, 0) }

  /// Flutter releases what it gets here; the library hands the buffer over retained.
  func copyPixelBuffer() -> Unmanaged<CVPixelBuffer>? {
    guard let p = front() else { return nil }
    return Unmanaged<CVPixelBuffer>.fromOpaque(p)
  }
}

/// The present inside's stage (motolii/filament/out/libmotolii.dylib, motolii.h) as a Flutter texture, without a copy:
/// the same manner as GlassTexture. The Dart side opens the document itself (dart:ffi) and makes it live; here the
/// library is only asked for its last finished picture and how many there have been.
/// Channel "motolii/stage", method "start" {lib, handle (the Motolii* as a number)} -> texture id, or -1.
final class StageTexture: NSObject, FlutterTexture {
  private typealias FrontFn = @convention(c) (UnsafeMutableRawPointer?) -> UnsafeMutableRawPointer?
  private typealias FramesFn = @convention(c) (UnsafeMutableRawPointer?) -> UInt64

  private static var shared: StageTexture?
  private let front: FrontFn
  private let frames: FramesFn
  private let inside: UnsafeMutableRawPointer?
  private let textures: FlutterTextureRegistry
  private var id: Int64 = -1
  private var seen: UInt64 = 0
  private var timer: Timer?

  static func attach(to controller: FlutterViewController) {
    let registrar = controller.registrar(forPlugin: "MotoliiStage")
    let channel = FlutterMethodChannel(name: "motolii/stage", binaryMessenger: registrar.messenger)
    channel.setMethodCallHandler { call, result in
      guard call.method == "start", let a = call.arguments as? [String: Any], let lib = a["lib"] as? String, let handle = a["handle"] as? Int else {
        result(FlutterMethodNotImplemented)
        return
      }
      shared?.stop()
      shared = StageTexture(lib: lib, inside: UnsafeMutableRawPointer(bitPattern: handle), textures: registrar.textures)
      result(shared?.id ?? -1)
    }
  }

  private init?(lib: String, inside: UnsafeMutableRawPointer?, textures: FlutterTextureRegistry) {
    guard let h = dlopen(lib, RTLD_NOW), let f = dlsym(h, "motolii_front"), let n = dlsym(h, "motolii_frames") else { return nil }
    front = unsafeBitCast(f, to: FrontFn.self)
    frames = unsafeBitCast(n, to: FramesFn.self)
    self.inside = inside
    self.textures = textures
    super.init()
    id = textures.register(self)
    // tell Flutter whenever the inside has finished another picture (the inside keeps the clock)
    timer = Timer.scheduledTimer(withTimeInterval: 1.0 / 120.0, repeats: true) { [weak self] _ in
      guard let me = self else { return }
      let now = me.frames(me.inside)
      if now != me.seen {
        me.seen = now
        me.textures.textureFrameAvailable(me.id)
      }
    }
    RunLoop.main.add(timer!, forMode: .common)
  }

  private func stop() {
    timer?.invalidate()
    textures.unregisterTexture(id)
  }

  /// Flutter releases what it gets here; the library hands the buffer over retained.
  func copyPixelBuffer() -> Unmanaged<CVPixelBuffer>? {
    guard let p = front(inside) else { return nil }
    return Unmanaged<CVPixelBuffer>.fromOpaque(p)
  }
}
