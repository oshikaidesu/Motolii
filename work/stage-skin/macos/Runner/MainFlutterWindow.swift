import Cocoa
import FlutterMacOS
import AVFoundation

// Probe: a FlutterTexture whose pixels are the IOSurface-backed CVPixelBuffer the decoder hands out. No copy by us.
final class VideoTexture: NSObject, FlutterTexture {
  let player: AVPlayer
  let output: AVPlayerItemVideoOutput
  var registry: FlutterTextureRegistry?
  var id: Int64 = 0
  var link: CVDisplayLink?
  var handed = 0
  var t0 = CACurrentMediaTime()
  init(path: String) {
    let item = AVPlayerItem(url: URL(fileURLWithPath: path))
    output = AVPlayerItemVideoOutput(pixelBufferAttributes: [
      kCVPixelBufferPixelFormatTypeKey as String: kCVPixelFormatType_32BGRA,
      kCVPixelBufferMetalCompatibilityKey as String: true,
      kCVPixelBufferIOSurfacePropertiesKey as String: [:] as [String: Any]])
    item.add(output)
    player = AVPlayer(playerItem: item)
    super.init()
  }
  func copyPixelBuffer() -> Unmanaged<CVPixelBuffer>? {
    let now = output.itemTime(forHostTime: CACurrentMediaTime())
    guard let pb = output.copyPixelBuffer(forItemTime: now, itemTimeForDisplay: nil) else { return nil }
    handed += 1
    return Unmanaged.passRetained(pb)
  }
  func start(registry: FlutterTextureRegistry) {
    self.registry = registry
    id = registry.register(self)
    player.rate = 2.0
    CVDisplayLinkCreateWithActiveCGDisplays(&link)
    let me = Unmanaged.passUnretained(self).toOpaque()
    CVDisplayLinkSetOutputCallback(link!, { (_, _, _, _, _, ctx) -> CVReturn in
      let s = Unmanaged<VideoTexture>.fromOpaque(ctx!).takeUnretainedValue()
      s.registry?.textureFrameAvailable(s.id)
      return kCVReturnSuccess
    }, me)
    CVDisplayLinkStart(link!)
  }
}

class MainFlutterWindow: NSWindow {
  var tex: VideoTexture?
  override func awakeFromNib() {
    let flutterViewController = FlutterViewController()
    let windowFrame = self.frame
    self.contentViewController = flutterViewController
    self.setFrame(windowFrame, display: true)

    RegisterGeneratedPlugins(registry: flutterViewController)

    let ch = FlutterMethodChannel(name: "skin/tex", binaryMessenger: flutterViewController.engine.binaryMessenger)
    ch.setMethodCallHandler { [weak self] call, result in
      guard let self = self else { return }
      if call.method == "create", let path = call.arguments as? String {
        let t = VideoTexture(path: path)
        t.start(registry: flutterViewController.engine)
        self.tex = t
        result(t.id)
      } else if call.method == "stats", let t = self.tex {
        result(["handed": t.handed, "secs": CACurrentMediaTime() - t.t0])
      } else { result(FlutterMethodNotImplemented) }
    }
    super.awakeFromNib()
  }
}
