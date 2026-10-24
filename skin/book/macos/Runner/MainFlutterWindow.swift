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

/// A library's own picture as a Flutter texture, without a copy: the library draws each frame into a pixel buffer on
/// the GPU, and its last finished buffer is handed to Flutter as it is. Flutter is told whenever another is finished.
/// front: the last finished buffer, retained (Flutter releases it); frames: how many have been finished. Whoever knows
/// another is finished says so (check).
final class LiveTexture: NSObject, FlutterTexture {
  private let front: () -> UnsafeMutableRawPointer?
  private let frames: () -> UInt64
  private let textures: FlutterTextureRegistry
  private(set) var id: Int64 = -1
  private var seen: UInt64 = 0

  init(textures: FlutterTextureRegistry, front: @escaping () -> UnsafeMutableRawPointer?, frames: @escaping () -> UInt64) {
    self.front = front
    self.frames = frames
    self.textures = textures
    super.init()
    id = textures.register(self)
  }

  func check() {
    let now = frames()
    if now != seen {
      seen = now
      textures.textureFrameAvailable(id)
    }
  }

  func stop() {
    textures.unregisterTexture(id)
  }

  func copyPixelBuffer() -> Unmanaged<CVPixelBuffer>? {
    guard let p = front() else { return nil }
    return Unmanaged<CVPixelBuffer>.fromOpaque(p)
  }
}

/// The live glass (motolii/filament/out/libglass_live.dylib).
/// Channel "motolii/glass", method "start" {lib, base, width, height, env: {name: value}} -> texture id, or -1.
/// Everything else (sun, camera, pace) the Dart side sets through the library's own C functions.
enum GlassTexture {
  private typealias FrontFn = @convention(c) () -> UnsafeMutableRawPointer?
  private typealias FramesFn = @convention(c) () -> UInt64
  private typealias EnvFn = @convention(c) (UnsafePointer<CChar>, UnsafePointer<CChar>) -> Void
  private typealias StartFn = @convention(c) (UnsafePointer<CChar>, Int32, Int32) -> Int32

  private static var shared: LiveTexture?
  private static var start: StartFn?
  private static var timer: Timer?   // (the glass study keeps its own clock: it says nothing when a picture is finished)

  static func attach(to controller: FlutterViewController) {
    let registrar = controller.registrar(forPlugin: "MotoliiGlass")
    let channel = FlutterMethodChannel(name: "motolii/glass", binaryMessenger: registrar.messenger)
    channel.setMethodCallHandler { call, result in
      guard call.method == "start", let a = call.arguments as? [String: Any], let lib = a["lib"] as? String, let base = a["base"] as? String else {
        result(FlutterMethodNotImplemented)
        return
      }
      if shared != nil {
        _ = start?(base, 0, 0)
      } else if let h = dlopen(lib, RTLD_NOW), let f = dlsym(h, "glass_front"), let n = dlsym(h, "glass_frames"), let e = dlsym(h, "glass_env"),
        let s = dlsym(h, "glass_start")
      {
        let front = unsafeBitCast(f, to: FrontFn.self), frames = unsafeBitCast(n, to: FramesFn.self), setEnv = unsafeBitCast(e, to: EnvFn.self)
        start = unsafeBitCast(s, to: StartFn.self)
        for (name, value) in a["env"] as? [String: String] ?? [:] { setEnv(name, value) }
        _ = start?(base, Int32(a["width"] as? Int ?? 1920), Int32(a["height"] as? Int ?? 1080))
        shared = LiveTexture(textures: registrar.textures, front: { front() }, frames: { frames() })
        timer = Timer.scheduledTimer(withTimeInterval: 1.0 / 120.0, repeats: true) { _ in shared?.check() }
        RunLoop.main.add(timer!, forMode: .common)
      }
      result(shared?.id ?? -1)
    }
  }
}

/// The present inside's stage (motolii/filament/out/libmotolii.dylib, motolii.h). The Dart side opens the document
/// itself (dart:ffi) and makes it live; here the library is only asked for its last finished picture and how many
/// there have been (the inside keeps the clock).
/// Channel "motolii/stage", method "start" {lib, handle (the Motolii* as a number), and which picture: front and frames,
/// the names of its two calls ("motolii_front" and "motolii_frames", the stage's own, unless told)} -> texture id, or -1.
/// Each picture its own texture: the stage's, and a card's small one of a thing alone ("motolii_solo_front").
/// The inside tells when a picture is finished (motolii_finished, on its own thread); Flutter is told then, on the main thread.
enum StageTexture {
  private typealias FrontFn = @convention(c) (UnsafeMutableRawPointer?) -> UnsafeMutableRawPointer?
  private typealias FramesFn = @convention(c) (UnsafeMutableRawPointer?) -> UInt64
  private typealias FinishedFn = @convention(c) (UnsafeMutableRawPointer?, @convention(c) (UnsafeMutableRawPointer?) -> Void, UnsafeMutableRawPointer?) -> Void

  private static var shared: [String: LiveTexture] = [:]

  static func attach(to controller: FlutterViewController) {
    let registrar = controller.registrar(forPlugin: "MotoliiStage")
    let channel = FlutterMethodChannel(name: "motolii/stage", binaryMessenger: registrar.messenger)
    channel.setMethodCallHandler { call, result in
      guard call.method == "start", let a = call.arguments as? [String: Any], let lib = a["lib"] as? String, let handle = a["handle"] as? Int else {
        result(FlutterMethodNotImplemented)
        return
      }
      let frontName = a["front"] as? String ?? "motolii_front", framesName = a["frames"] as? String ?? "motolii_frames"
      shared[frontName]?.stop()
      shared[frontName] = nil
      if let h = dlopen(lib, RTLD_NOW), let f = dlsym(h, frontName), let n = dlsym(h, framesName) {
        let front = unsafeBitCast(f, to: FrontFn.self), frames = unsafeBitCast(n, to: FramesFn.self), inside = UnsafeMutableRawPointer(bitPattern: handle)
        shared[frontName] = LiveTexture(textures: registrar.textures, front: { front(inside) }, frames: { frames(inside) })
        if let t = dlsym(h, "motolii_finished") {
          unsafeBitCast(t, to: FinishedFn.self)(inside, { _ in DispatchQueue.main.async { StageTexture.shared.values.forEach { $0.check() } } }, nil)
        }
      }
      result(shared[frontName]?.id ?? -1)
    }
  }
}
