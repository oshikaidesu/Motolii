import Cocoa
import FlutterMacOS

class MainFlutterWindow: NSWindow, NSWindowDelegate, NSDraggingDestination {
  private var probeHost: ProbeHost?
  private var closeApproved = false
  private var closePending = false
  override func awakeFromNib() {
    let controller = FlutterViewController()
    // Hover reaches Flutter always, not only while this window is key: a
    // shelf is browsed before it is clicked, and feedback is immediate.
    controller.mouseTrackingMode = .always
    let originalFrame = frame
    contentViewController = controller
    setFrame(originalFrame, display: true)
    RegisterGeneratedPlugins(registry: controller)
    probeHost = ProbeHost.install(controller: controller, window: self)
    setContentSize(NSSize(width: 1280, height: 768))
    minSize = NSSize(width: 1000, height: 620)
    delegate = self
    registerForDraggedTypes([.fileURL])
    super.awakeFromNib()
  }
  private func droppedPaths(_ sender: NSDraggingInfo) -> [String] {
    let urls = sender.draggingPasteboard.readObjects(forClasses: [NSURL.self], options: [.urlReadingFileURLsOnly: true]) as? [URL] ?? []
    return urls.filter { $0.isFileURL }.map { $0.path }
  }
  func draggingEntered(_ sender: NSDraggingInfo) -> NSDragOperation {
    let files = !droppedPaths(sender).isEmpty
    if files { probeHost?.dragHover(true) }
    return files ? .copy : []
  }
  func draggingExited(_ sender: NSDraggingInfo?) { probeHost?.dragHover(false) }
  func prepareForDragOperation(_ sender: NSDraggingInfo) -> Bool { !droppedPaths(sender).isEmpty }
  func performDragOperation(_ sender: NSDraggingInfo) -> Bool {
    probeHost?.dragHover(false)
    let paths = droppedPaths(sender)
    guard !paths.isEmpty else { return false }
    let point = contentView?.convert(sender.draggingLocation, from: nil) ?? sender.draggingLocation
    let height = contentView?.bounds.height ?? frame.height
    probeHost?.filesDropped(paths, point: [point.x, contentView?.isFlipped == true ? point.y : height - point.y])
    return true
  }
  func windowShouldClose(_ sender: NSWindow) -> Bool {
    if closeApproved || ProbeSession.shared.terminationApproved { return true }
    if closePending { return false }
    closePending = true
    ProbeSession.shared.confirmTermination { [weak self] approved in
      guard let self else { return }
      if !approved { self.closePending = false; return }
      ProbeSession.shared.shutdown { [weak self] in
        guard let self else { return }
        self.closeApproved = true
        self.closePending = false
        self.performClose(nil)
      }
    }
    return false
  }
  func windowWillClose(_ notification: Notification) { probeHost?.detachWindow(); probeHost = nil }
}

import Cocoa
import CoreVideo
import Darwin
import FlutterMacOS
import IOSurface
import OSLog
import UniformTypeIdentifiers

private let playbackLog = Logger(subsystem: "dev.motolii.stage5", category: "playback")

private enum ProbeFailure: Error {
  case message(String)
}

private final class ProbeRuntime {
  typealias Open = @convention(c) (UnsafePointer<CChar>) -> UnsafeMutableRawPointer?
  typealias Request = @convention(c) (UnsafeMutableRawPointer, UnsafePointer<CChar>) -> UnsafePointer<CChar>?
  typealias Render = @convention(c) (UnsafeMutableRawPointer, UInt32, UnsafePointer<CChar>) -> Int32
  typealias PlaybackTick = @convention(c) (UnsafeMutableRawPointer) -> Int64
  typealias Close = @convention(c) (UnsafeMutableRawPointer) -> Void
  typealias Wake = @convention(c) (UnsafeMutableRawPointer?) -> Void
  typealias Watch = @convention(c) (UnsafeMutableRawPointer, Wake, UnsafeMutableRawPointer?) -> Int32
  typealias FrameReady = @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<CChar>?, UInt32) -> Void
  typealias SetFrameReady = @convention(c) (UnsafeMutableRawPointer, FrameReady?, UnsafeMutableRawPointer?) -> Int32
  typealias FinishFrames = @convention(c) (UnsafeMutableRawPointer) -> Int32
  /// An IOSurface-backed buffer may cross to the render actor solely to retain
  /// it until Metal has imported the surface. No pixel access occurs here.
  fileprivate struct PlaybackTarget: @unchecked Sendable {
    let view: String
    let surface: UInt32
    let keepAlive: CVPixelBuffer
  }
  /// All Rust/Rerun calls live here.  Flutter's main thread may enqueue work or
  /// publish a completed IOSurface, but it never enters the renderer during
  /// playback.
  private let renderQueue = DispatchQueue(label: "motolii.render", qos: .userInteractive)
  private let renderQueueKey = DispatchSpecificKey<UInt8>()
  private let playbackGate = NSLock()
  private var playbackQueued = false
  private var playbackDropped = 0
  /// Samples are confined to `renderQueue`. They measure CPU time through the
  /// native tick and submit only; GPU completion remains the IOSurface signal.
  private var playbackSubmitUs: [UInt64] = []
  private var library: UnsafeMutableRawPointer?
  /// The native host owns playback pulses; Dart retains this context only for
  /// one-shot commands and still frames. Both use the macOS main thread
  /// (Flutter 3.35+ merges UI and platform threads), so the runtime is never shared.
  fileprivate private(set) var context: UnsafeMutableRawPointer?
  fileprivate private(set) var location: String?
  private var requestFunction: Request?
  fileprivate var renderFunction: Render?
  private var playbackTickFunction: PlaybackTick?
  private var closeFunction: Close?
  /// 描き終わりを待つ同期の口。合図を使わない channel の道だけが使う。
  private var finishFunction: FinishFrames?
  private(set) var rendered = 0

  init() {
    renderQueue.setSpecific(key: renderQueueKey, value: 1)
  }

  private func onRenderQueue<T>(_ body: () throws -> T) throws -> T {
    if DispatchQueue.getSpecific(key: renderQueueKey) != nil { return try body() }
    return try renderQueue.sync(execute: body)
  }

  fileprivate func resetPlaybackStats() {
    renderQueue.async { [weak self] in self?.playbackSubmitUs.removeAll(keepingCapacity: true) }
    playbackGate.lock()
    playbackDropped = 0
    playbackGate.unlock()
  }

  fileprivate func reportPlaybackStats() {
    renderQueue.async { [weak self] in
      guard let self else { return }
      let samples = self.playbackSubmitUs
      self.playbackSubmitUs.removeAll(keepingCapacity: true)
      self.playbackGate.lock()
      let dropped = self.playbackDropped
      self.playbackDropped = 0
      self.playbackGate.unlock()
      guard !samples.isEmpty || dropped > 0 else { return }
      let sorted = samples.sorted()
      let percentile = { (fraction: Double) -> UInt64 in
        guard !sorted.isEmpty else { return 0 }
        return sorted[min(sorted.count - 1, Int((Double(sorted.count - 1) * fraction).rounded(.up)))]
      }
      let message = "PROBE room=playback-native frames=\(samples.count) dropped=\(dropped) cpu-submit-ms median=\(Double(percentile(0.5)) / 1000.0) p90=\(Double(percentile(0.9)) / 1000.0) max=\(Double(sorted.last ?? 0) / 1000.0)"
      playbackLog.info("\(message, privacy: .public)")
    }
  }

  private func dropPlaybackFrame() {
    playbackGate.lock()
    playbackDropped += 1
    playbackGate.unlock()
  }

  func open(path: String) throws -> [String: Any] {
    try onRenderQueue { try openOnRenderQueue(path: path) }
  }

  private func openOnRenderQueue(path: String) throws -> [String: Any] {
    let previous = (library: library, context: context, request: requestFunction, render: renderFunction, tick: playbackTickFunction, close: closeFunction, finish: finishFunction, count: rendered)
    let location = ProcessInfo.processInfo.environment["MOTOLII_NATIVE_LIBRARY"] ?? URL(fileURLWithPath: #filePath).deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent().appendingPathComponent("target/debug/libmotolii_ui.dylib").path
    guard let library = dlopen(location, RTLD_NOW | RTLD_LOCAL) else {
      throw ProbeFailure.message(dlerror().map { String(cString: $0) } ?? "dlopen failed")
    }
    self.library = library
    self.location = location
    context = nil
    requestFunction = nil
    renderFunction = nil
    playbackTickFunction = nil
    closeFunction = nil
    finishFunction = nil
    do {
      func symbol<T>(_ name: String, _: T.Type) throws -> T {
        guard let address = dlsym(library, name) else { throw ProbeFailure.message("Missing symbol: \(name)") }
        return unsafeBitCast(address, to: T.self)
      }
      let start = try symbol("motolii_probe_open", Open.self)
      requestFunction = try symbol("motolii_probe_request", Request.self)
      renderFunction = try symbol("motolii_probe_render", Render.self)
      playbackTickFunction = try symbol("motolii_probe_playback_tick", PlaybackTick.self)
      closeFunction = try symbol("motolii_probe_close", Close.self)
      finishFunction = try symbol("motolii_probe_finish_frames", FinishFrames.self)
      let watch = try symbol("motolii_probe_watch_effects", Watch.self)
      let setFrameReady = try symbol("motolii_probe_set_frame_ready", SetFrameReady.self)
      context = path.withCString { start($0) }
      guard let context else { throw ProbeFailure.message("Rust could not open the document") }
      // vism/ の見張り。Rust は別 thread から起こすので、main へ戻してから棚を読み直す。
      _ = watch(context, { _ in DispatchQueue.main.async { ProbeSession.shared.effectsChanged() } }, nil)
      // GPU completion comes from the render actor.  Only publication crosses
      // back to the main thread; the callback never re-enters Rust.
      _ = setFrameReady(context, { _, view, surface in
        let name = view.map { String(cString: $0) } ?? ""
        DispatchQueue.main.async { ProbeSession.shared.frameReady(name, surface) }
      }, nil)
      rendered = 0
      let reply = try status()
      if let old = previous.context { previous.close?(old) }
      if let old = previous.library { dlclose(old) }
      return reply
    } catch {
      close()
      self.library = previous.library
      context = previous.context
      requestFunction = previous.request
      renderFunction = previous.render
      playbackTickFunction = previous.tick
      closeFunction = previous.close
      finishFunction = previous.finish
      rendered = previous.count
      throw error
    }
  }

  /// The native clock answers only when a new composition frame exists.  No
  /// JSON snapshot crosses into Dart on this path.
  func playbackTick() -> Int64? {
    try? onRenderQueue {
      guard let context, let playbackTickFunction else { return nil }
      let frame = playbackTickFunction(context)
      return frame >= 0 ? frame : nil
    }
  }

  /// Queue exactly one playback submission.  A slow render retains the last
  /// completed picture and makes later display pulses no-ops instead of growing
  /// a backlog on the UI thread.
  fileprivate func enqueuePlayback(
    targets: [PlaybackTarget],
    completion: @escaping (Result<Int64, Error>) -> Void
  ) -> Bool {
    playbackGate.lock()
    guard !playbackQueued else {
      playbackDropped += 1
      playbackGate.unlock()
      return false
    }
    playbackQueued = true
    playbackGate.unlock()
    renderQueue.async { [weak self] in
      guard let self else { return }
      let outcome: Result<Int64?, Error> = Result {
        let started = DispatchTime.now().uptimeNanoseconds
        guard let context = self.context, let tick = self.playbackTickFunction,
              let render = self.renderFunction else { throw ProbeFailure.message("Document is closed") }
        let frame = tick(context)
        guard frame >= 0 else { return nil }
        for target in targets {
          // `keepAlive` holds the IOSurface until Rust has imported it.
          _ = target.keepAlive
          let code = target.view.withCString { render(context, target.surface, $0) }
          if code < 0 { throw ProbeFailure.message("Native playback render failed: \(code)") }
          // Busy means Metal still owns this IOSurface. Do not advance the UI
          // onto a frame that was never submitted; the next host pulse retries.
          if code > 0 {
            self.dropPlaybackFrame()
            return nil
          }
        }
        self.playbackSubmitUs.append((DispatchTime.now().uptimeNanoseconds - started) / 1_000)
        return frame
      }
      self.playbackGate.lock()
      self.playbackQueued = false
      self.playbackGate.unlock()
      DispatchQueue.main.async {
        switch outcome {
        case .success(let frame?): completion(.success(frame))
        case .success(nil): break
        case .failure(let error): completion(.failure(error))
        }
      }
    }
    return true
  }

  func request(_ command: String) throws -> [String: Any] {
    try onRenderQueue { try requestOnRenderQueue(command) }
  }

  private func requestOnRenderQueue(_ command: String) throws -> [String: Any] {
    guard let context, let requestFunction else { throw ProbeFailure.message("Open a document first") }
    guard let pointer = command.withCString({ requestFunction(context, $0) }) else {
      throw ProbeFailure.message("Rust request returned no reply")
    }
    // Copy the reply before another FFI call can invalidate its pointer.
    let data = Data(String(cString: pointer).utf8)
    guard let reply = try JSONSerialization.jsonObject(with: data) as? [String: Any] else {
      throw ProbeFailure.message("Rust reply is not a JSON object")
    }
    return reply
  }

  func status() throws -> [String: Any] { try request("{\"op\":\"status\",\"bootstrap\":true}") }

  /// Every view Rust lists (Camera always; Stage while its tab holds a window) gets its own
  /// surface and one render. Two pictures of one world, both live.
  func render(known: Any? = nil, references: Any? = nil) throws -> ([String: CVPixelBuffer], [String: Any], Int) {
    try onRenderQueue { try renderOnRenderQueue(known: known, references: references) }
  }

  private func renderOnRenderQueue(known: Any? = nil, references: Any? = nil) throws -> ([String: CVPixelBuffer], [String: Any], Int) {
    let state = try request("{\"op\":\"renderInfo\"}")
    guard let views = state["views"] as? [[String: Any]], !views.isEmpty else {
      throw ProbeFailure.message("Document dimensions missing")
    }
    guard let context, let renderFunction else { throw ProbeFailure.message("Document is closed") }
    var buffers: [String: CVPixelBuffer] = [:]
    for entry in views {
      guard let view = entry["view"] as? String,
            let width = (entry["width"] as? NSNumber)?.intValue,
            let height = (entry["height"] as? NSNumber)?.intValue,
            width > 0, height > 0, width <= 16384, height <= 16384 else {
        throw ProbeFailure.message("View dimensions missing or outside probe allocation limit")
      }
      let made = try ProbeRuntime.makeSurface(width: width, height: height)
      let code = view.withCString { renderFunction(context, made.id, $0) }
      guard code == 0 else { throw ProbeFailure.message("Rust render failed for \(view): \(code)") }
      buffers[view] = made.buffer
    }
    // この道は合図を待たずに buffer をそのまま返すので、ここで描き終わりを待つ。
    _ = finishFunction?(context)
    rendered += 1
    var query: [String: Any] = ["op": "status"]
    if let known { query["knownSnapshotId"] = known }
    if let references { query["knownReferenceId"] = references }
    let data = try JSONSerialization.data(withJSONObject: query)
    return (buffers, try request(String(decoding: data, as: UTF8.self)), rendered)
  }

  /// Still-frame channel path. The host owns these surfaces too, so Flutter
  /// never obtains a raw runtime pointer. This is intentionally synchronous:
  /// exact scrub/stop may wait, unlike playback.
  fileprivate func renderInto(
    _ targets: [PlaybackTarget], known: Any? = nil, references: Any? = nil
  ) throws -> ([String: Any], Int) {
    try onRenderQueue {
      guard let context, let renderFunction else { throw ProbeFailure.message("Document is closed") }
      for target in targets {
        _ = target.keepAlive
        var code = target.view.withCString { renderFunction(context, target.surface, $0) }
        // Playback may drop a busy surface. A still frame is exact: wait for
        // that prior submission, then submit this requested frame once.
        if code == 1 {
          _ = finishFunction?(context)
          code = target.view.withCString { renderFunction(context, target.surface, $0) }
        }
        guard code == 0 else { throw ProbeFailure.message("Rust render failed for \(target.view): \(code)") }
      }
      _ = finishFunction?(context)
      rendered += 1
      var query: [String: Any] = ["op": "status"]
      if let known { query["knownSnapshotId"] = known }
      if let references { query["knownReferenceId"] = references }
      let data = try JSONSerialization.data(withJSONObject: query)
      return (try requestOnRenderQueue(String(decoding: data, as: UTF8.self)), rendered)
    }
  }

  static func makeSurface(width: Int, height: Int) throws -> (id: UInt32, buffer: CVPixelBuffer) {
    let properties: [String: Any] = [
      kIOSurfaceWidth as String: width,
      kIOSurfaceHeight as String: height,
      kIOSurfaceBytesPerElement as String: 4,
      kIOSurfacePixelFormat as String: kCVPixelFormatType_32BGRA,
    ]
    guard let surface = IOSurfaceCreate(properties as CFDictionary) else {
      throw ProbeFailure.message("IOSurface allocation failed")
    }
    let attributes = [kCVPixelBufferMetalCompatibilityKey as String: true] as CFDictionary
    var unmanaged: Unmanaged<CVPixelBuffer>?
    let outcome = CVPixelBufferCreateWithIOSurface(kCFAllocatorDefault, surface, attributes, &unmanaged)
    guard outcome == kCVReturnSuccess, let unmanaged else {
      throw ProbeFailure.message("CVPixelBuffer wrapping failed: \(outcome)")
    }
    return (IOSurfaceGetID(surface), unmanaged.takeRetainedValue())
  }

  func close() {
    _ = try? onRenderQueue { closeOnRenderQueue() }
  }

  private func closeOnRenderQueue() {
    if let context { closeFunction?(context) }
    context = nil
    requestFunction = nil
    renderFunction = nil
    playbackTickFunction = nil
    closeFunction = nil
    finishFunction = nil
    if let library { dlclose(library) }
    library = nil
  }
}

private final class ProbeTexture: NSObject, FlutterTexture {
  private let lock = NSLock()
  private var latest: CVPixelBuffer?
  private var published = 0
  private var rasterRequests = 0
  private var hostReleases = 0

  func publish(_ buffer: CVPixelBuffer) {
    lock.lock()
    defer { lock.unlock() }
    if latest != nil { hostReleases += 1 }
    latest = buffer
    published += 1
  }

  func clear() {
    lock.lock()
    defer { lock.unlock() }
    if latest != nil { hostReleases += 1 }
    latest = nil
  }

  func counters() -> [String: Int] {
    lock.lock()
    defer { lock.unlock() }
    return ["publishedFrames": published, "rasterRequests": rasterRequests,
            "hostBufferReleases": hostReleases, "hostRetainedBuffers": latest == nil ? 0 : 1]
  }

  func copyPixelBuffer() -> Unmanaged<CVPixelBuffer>? {
    lock.lock()
    defer { lock.unlock() }
    guard let latest else { return nil }
    rasterRequests += 1
    return Unmanaged.passRetained(latest)
  }
}


final class ProbeSession {
  static let shared = ProbeSession()
  fileprivate let worker = DispatchQueue(label: "motolii.settings-file")
  fileprivate let runtime = ProbeRuntime()
  fileprivate let hosts = NSHashTable<ProbeHost>.weakObjects()
  fileprivate var paneState: [String: Any] = [:]
  fileprivate var epoch: UInt64 = 0
  fileprivate var latest: [String: CVPixelBuffer] = [:]
  /// The surfaces Dart draws each view into: two per view, taken in turn, made
  /// once per size. The Rust callback names the one it just filled.
  fileprivate var surfaces: [String: (width: Int, height: Int, entries: [(id: UInt32, buffer: CVPixelBuffer)])] = [:]
  fileprivate var state: [String: Any] = [:]
  fileprivate var windows: [String: PanelFlutterWindow] = [:]
  /// The native host, not Flutter's `Ticker`, owns playback cadence.  The
  /// Flutter side receives a texture availability signal and a small playhead
  /// integer only.
  private var playbackTimer: DispatchSourceTimer?
  private var playbackSurfaceFlip = false
  private var confirming = false
  var terminationApproved = false

  /// 効果の file が変わった。棚を読み直すのは main の窓(Dart が reloadEffects を送り、絵を描き直し、他の窓へ配る)。
  fileprivate func effectsChanged() {
    precondition(Thread.isMainThread)
    guard let main = hosts.allObjects.first(where: { $0.isMain && !$0.closed && $0.attached }) else { return }
    main.channel.invokeMethod("effectsChanged", arguments: nil)
  }

  /// The GPU finished `view` in the surface `id`, and Rust said so on this thread,
  /// inside Dart's render call — this frame's picture when it was waited for, the
  /// previous one while playing. The main window's texture learns it now; the other
  /// windows learn it with the status Dart broadcasts next, so their picture and its
  /// size travel as one step.
  fileprivate func frameReady(_ view: String, _ id: UInt32) {
    precondition(Thread.isMainThread)
    guard let buffer = surfaces[view]?.entries.first(where: { $0.id == id })?.buffer else { return }
    latest[view] = buffer
    for host in hosts.allObjects where !host.closed && host.attached && host.isMain {
      try? host.ensureTexture(view)
      host.publish(view, buffer)
    }
  }

  /// Surfaces for the views Dart lists, at their sizes; kept when the size holds,
  /// remade when it moves, dropped when a view is no longer listed.
  fileprivate func ensureSurfaces(_ views: [[String: Any]]) throws -> [String: Any] {
    precondition(Thread.isMainThread)
    var next: [String: (width: Int, height: Int, entries: [(id: UInt32, buffer: CVPixelBuffer)])] = [:]
    var reply: [String: Any] = [:]
    for entry in views {
      guard let view = entry["view"] as? String,
            let width = (entry["width"] as? NSNumber)?.intValue,
            let height = (entry["height"] as? NSNumber)?.intValue,
            width > 0, height > 0, width <= 16384, height <= 16384 else {
        throw ProbeFailure.message("View dimensions missing or outside probe allocation limit")
      }
      if let held = surfaces[view], held.width == width, held.height == height {
        next[view] = held
      } else {
        next[view] = (width, height, try (0..<2).map { _ in try ProbeRuntime.makeSurface(width: width, height: height) })
      }
      reply[view] = ["width": width, "height": height, "ids": next[view]!.entries.map { Int($0.id) }]
    }
    surfaces = next
    return ["surfaces": reply]
  }

  fileprivate func targets(for views: [[String: Any]], flip: Bool = false) -> [ProbeRuntime.PlaybackTarget] {
    views.compactMap { entry in
      guard let view = entry["view"] as? String,
            let surface = surfaces[view], !surface.entries.isEmpty else { return nil }
      let entry = surface.entries[flip ? 1 % surface.entries.count : 0]
      return ProbeRuntime.PlaybackTarget(view: view, surface: entry.id, keepAlive: entry.buffer)
    }
  }

  fileprivate func broadcast(_ state: [String: Any], buffers: [String: CVPixelBuffer] = [:], origin: ProbeHost? = nil, frameReady: Bool = false, frameOnly: Bool = false, effectsReloaded: Bool = false) {
    precondition(Thread.isMainThread)
    self.state.merge(state) { _, next in next }
    latest.merge(buffers) { _, next in next }
    let fresh = frameReady ? latest : buffers
    for host in hosts.allObjects where !host.closed && host.attached {
      do {
        try host.ensureTexture(ProbeHost.outputView)
        if host !== origin {
          var event = host.envelope(state, frameReady: !fresh.isEmpty)
          event["frameOnly"] = frameOnly
          event["effectsReloaded"] = effectsReloaded
          // The picture and its size travel as one step: Dart learns the size
          // (envelope) first, and only then is the frame made available to the
          // raster thread. A resized Stage never paints an old frame stretched.
          let attachmentID = host.attachmentID
          host.channel.invokeMethod("documentChanged", arguments: event) { [weak host] _ in
            guard let host, !host.closed, host.attached, host.attachmentID == attachmentID else { return }
            for (view, buffer) in fresh { host.publish(view, buffer) }
          }
        } else {
          for (view, buffer) in buffers { host.publish(view, buffer) }
        }
      } catch {
        host.channel.invokeMethod("nativeError", arguments: String(describing: error))
      }
    }
  }

  fileprivate func clearFrames() {
    latest = [:]
    surfaces = [:]
    for host in hosts.allObjects { host.detachTexture() }
  }

  fileprivate func startPlayback() {
    precondition(Thread.isMainThread)
    guard playbackTimer == nil else { return }
    runtime.resetPlaybackStats()
    // Flutter's run loop can be idle while its skin has no frame to build.
    // The transport therefore cannot use a RunLoop Timer: cadence belongs to
    // the native host, and each pulse merely queues Rust/Rerun work on the
    // serial actor.  The main hop only snapshots IOSurface ownership and
    // publishes a finished texture; it never enters the renderer.
    let timer = DispatchSource.makeTimerSource(queue: .global(qos: .userInteractive))
    timer.schedule(deadline: .now(), repeating: .milliseconds(16), leeway: .milliseconds(1))
    timer.setEventHandler { [weak self] in
      DispatchQueue.main.async { self?.playbackPulse() }
    }
    playbackTimer = timer
    timer.resume()
  }

  fileprivate func stopPlayback() {
    playbackTimer?.setEventHandler {}
    playbackTimer?.cancel()
    playbackTimer = nil
    runtime.reportPlaybackStats()
  }

  private func playbackPulse() {
    precondition(Thread.isMainThread)
    playbackSurfaceFlip.toggle()
    let views: [[String: Any]] = surfaces.keys.map { ["view": $0] }
    let targets = self.targets(for: views, flip: playbackSurfaceFlip)
    guard !targets.isEmpty else { return }
    _ = runtime.enqueuePlayback(targets: targets) { [weak self] outcome in
      guard let self else { return }
      switch outcome {
      case .success(let frame):
        // This wakes only frame listeners (timeline/playhead). It carries no
        // Document snapshot and never asks Flutter to render the composition.
        for host in self.hosts.allObjects where !host.closed && host.attached {
          host.channel.invokeMethod("playbackFrame", arguments: ["frame": frame])
        }
      case .failure(let error):
        self.stopPlayback()
        for host in self.hosts.allObjects where !host.closed && host.attached {
          host.channel.invokeMethod("nativeError", arguments: String(describing: error))
        }
      }
    }
  }

  func confirmTermination(_ completion: @escaping (Bool) -> Void) {
    precondition(Thread.isMainThread)
    if terminationApproved { completion(true); return }
    guard !confirming else { completion(false); return }
    confirming = true
    let group = DispatchGroup()
    for host in hosts.allObjects where !host.closed && host.attached { group.enter(); host.channel.invokeMethod("flushEditors", arguments: nil) { _ in group.leave() } }
    group.notify(queue: .main) {
      if let status = try? self.runtime.status() { self.state = status }
      guard self.state["dirty"] as? Bool == true else {
        self.confirming = false; completion(true); return
      }
      guard let main = self.hosts.allObjects.first(where: { $0.isMain && !$0.closed && $0.attached }) else {
        self.confirming = false; completion(false); return
      }
      main.window?.deminiaturize(nil)
      main.window?.makeKeyAndOrderFront(nil)
      main.channel.invokeMethod("documentChanged", arguments: main.envelope(self.state))
      main.channel.invokeMethod("confirmClose", arguments: nil) { reply in
        self.confirming = false
        completion((reply as? Bool) == true)
      }
    }
  }

  func shutdown(_ completion: @escaping () -> Void) {
    precondition(Thread.isMainThread)
    terminationApproved = true
    stopPlayback()
    epoch &+= 1
    clearFrames()
    state = [:]
    runtime.close()
    for window in Array(windows.values) { window.close() }
    completion()
  }

  fileprivate func openPanelWindow(_ panels: [String]) -> [String: Any] {
    let id = UUID().uuidString
    let window = PanelFlutterWindow(identifier: id, panels: panels)
    windows[id] = window
    window.center()
    window.makeKeyAndOrderFront(nil)
    return ["id": id, "panels": panels, "main": false]
  }
}

final class ProbeHost: NSObject {
  private let session = ProbeSession.shared
  private let registry: FlutterTextureRegistry
  fileprivate let channel: FlutterMethodChannel
  /// One Flutter texture per view this window shows: "Camera" (the output) and "User" (the Stage).
  static let outputView = "Camera"
  private var textures: [String: ProbeTexture] = [:]
  private var textureIDs: [String: Int64] = [:]
  fileprivate weak var window: NSWindow?
  fileprivate var closed = false
  private var attachment = WindowAttachment()
  fileprivate var attached: Bool { attachment.id != nil }
  fileprivate var attachmentID: Int? { attachment.id }
  let id: String
  let panels: [String]
  let isMain: Bool
  private var rendering = false

  static func install(controller: FlutterViewController, window: NSWindow? = nil,
                      id: String = "main", panels: [String] = [], main: Bool = true) -> ProbeHost {
    ProbeHost(controller: controller, window: window, id: id, panels: panels, main: main)
  }

  private init(controller: FlutterViewController, window: NSWindow?, id: String, panels: [String], main: Bool) {
    registry = controller.engine
    channel = FlutterMethodChannel(name: "motolii/probe", binaryMessenger: controller.engine.binaryMessenger)
    self.window = window
    self.id = id
    self.panels = panels
    isMain = main
    super.init()
    session.hosts.add(self)
    channel.setMethodCallHandler { [weak self] call, result in
      guard let self, !self.closed else {
        result(FlutterError(code: "closed", message: "Window detached", details: nil)); return
      }
      self.handle(call, result)
    }
  }

  fileprivate func ensureTexture(_ view: String) throws {
    guard textureIDs[view] == nil else { return }
    let next = ProbeTexture()
    let id = registry.register(next)
    guard id != 0 else { throw ProbeFailure.message("Flutter texture registration failed") }
    textures[view] = next
    textureIDs[view] = id
    if let latest = session.latest[view] { publish(view, latest) }
  }

  fileprivate func publish(_ view: String, _ buffer: CVPixelBuffer) {
    guard let texture = textures[view], let textureID = textureIDs[view] else { return }
    texture.publish(buffer)
    registry.textureFrameAvailable(textureID)
  }

  fileprivate func envelope(_ status: [String: Any], frameReady: Bool = false) -> [String: Any] {
    var reply: [String: Any] = ["status": status, "windowId": id, "frameReady": frameReady]
    reply["runtimeEpoch"] = session.epoch
    if let attachmentID = attachment.id { reply["attachmentId"] = attachmentID }
    if let context = session.runtime.context, let library = session.runtime.location {
      reply["context"] = Int(bitPattern: context)
      reply["library"] = library
    }
    if let textureID = textureIDs[ProbeHost.outputView] { reply["textureId"] = textureID }
    reply["textureIds"] = textureIDs
    if let width = status["width"] { reply["width"] = width }
    if let height = status["height"] { reply["height"] = height }
    if let texture = textures[ProbeHost.outputView] { reply.merge(texture.counters()) { _, new in new } }
    return reply
  }

  fileprivate func detachTexture() {
    precondition(Thread.isMainThread)
    for id in textureIDs.values { registry.unregisterTexture(id) }
    for texture in textures.values { texture.clear() }
    textures = [:]
    textureIDs = [:]
  }

  func filesDropped(_ paths: [String], point: [CGFloat]) {
    guard !closed else { return }
    channel.invokeMethod("filesDropped", arguments: ["paths": paths, "point": point])
  }

  /// Files are being carried over the window (or just left it).
  func dragHover(_ active: Bool) {
    guard !closed else { return }
    channel.invokeMethod("dragHover", arguments: ["active": active])
  }

  func detachWindow() {
    guard !closed else { return }
    closed = true
    attachment.invalidate()
    channel.setMethodCallHandler(nil)
    detachTexture()
    session.hosts.remove(self)
    for host in session.hosts.allObjects where host.isMain && !host.closed && host.attached {
      host.channel.invokeMethod("windowClosed", arguments: ["id": id, "panels": panels])
    }
  }

  private func openPicker(_ panel: NSOpenPanel, result: @escaping FlutterResult, multiple: Bool) {
    let completion: (NSApplication.ModalResponse) -> Void = { response in
      guard response == .OK else { result(nil); return }
      if multiple { result(panel.urls.map { $0.path }) } else { result(panel.url?.path) }
    }
    if let window { panel.beginSheetModal(for: window, completionHandler: completion) }
    else { panel.begin(completionHandler: completion) }
  }

  private func handle(_ call: FlutterMethodCall, _ result: @escaping FlutterResult) {
    precondition(Thread.isMainThread)
    let args = call.arguments as? [String: Any] ?? [:]
    switch call.method {
    case "readSettings", "writeSettings":
      session.worker.async {
        let outcome: Result<Any, Error> = Result {
          let support = try FileManager.default.url(for: .applicationSupportDirectory, in: .userDomainMask, appropriateFor: nil, create: true)
          let directory = support.appendingPathComponent("MotoliiStage5", isDirectory: true)
          let file = directory.appendingPathComponent("layout.json")
          if call.method == "readSettings" {
            guard FileManager.default.fileExists(atPath: file.path) else { return [String: Any]() }
            guard let settings = try JSONSerialization.jsonObject(with: Data(contentsOf: file)) as? [String: Any] else {
              throw ProbeFailure.message("Saved layout is not a JSON object")
            }
            return settings
          }
          let data = try JSONSerialization.data(withJSONObject: args, options: [.sortedKeys])
          try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
          try data.write(to: file, options: .atomic)
          return true
        }
        DispatchQueue.main.async {
          do { result(try outcome.get()) }
          catch { self.fail(result, String(describing: error)) }
        }
      }
    case "openWeb":
      guard let raw = args["url"] as? String, let url = URL(string: raw), ["http", "https"].contains(url.scheme?.lowercased() ?? "") else { fail(result, "Enter an http or https URL"); return }
      result(NSWorkspace.shared.open(url))
    case "flushEditors":
      let group = DispatchGroup()
      for host in session.hosts.allObjects where !host.closed && host.attached { group.enter(); host.channel.invokeMethod("flushEditors", arguments: nil) { _ in group.leave() } }
      group.notify(queue: .main) { result(true) }
    case "placePanel":
      guard let main = session.hosts.allObjects.first(where: { $0.isMain && !$0.closed && $0.attached }) else { fail(result, "Main window unavailable"); return }
      main.channel.invokeMethod("placePanel", arguments: args) { value in result(value) }
    case "setPaneState":
      session.paneState = args
      for host in session.hosts.allObjects where !host.closed && host.attached { host.channel.invokeMethod("paneState", arguments: args) }
      result(true)
    case "windowInfo":
      result(["id": id, "panels": panels, "main": isMain, "paneState": session.paneState, "panelWindows": session.windows.count])
    case "ensureSurfaces":
      guard let views = args["views"] as? [[String: Any]] else { fail(result, "ensureSurfaces requires views"); return }
      do {
        for view in views { if let name = view["view"] as? String { try ensureTexture(name) } }
        var reply = try session.ensureSurfaces(views)
        reply["textureIds"] = textureIDs
        if let textureID = textureIDs[ProbeHost.outputView] { reply["textureId"] = textureID }
        result(reply)
      } catch { fail(result, String(describing: error)) }
    case "broadcast":
      // The main window drove the runtime itself; the other windows learn the outcome here.
      guard let text = args["status"] as? String, let data = text.data(using: .utf8),
            let status = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any] else { fail(result, "broadcast requires a status JSON string"); return }
      session.broadcast(status, origin: self, frameReady: args["frameReady"] as? Bool == true, frameOnly: args["frameOnly"] as? Bool == true)
      result(true)
    case "focusWindow", "closeWindow":
      guard let requestedID = args["id"] as? String,
            let host = session.hosts.allObjects.first(where: { $0.id == requestedID && !$0.closed }),
            let target = host.window else { fail(result, "Window not found"); return }
      if call.method == "focusWindow" {
        target.deminiaturize(nil)
        target.makeKeyAndOrderFront(nil)
        result(true)
      } else if !host.attached {
        target.performClose(nil)
        result(["requested": true])
      } else {
        host.channel.invokeMethod("flushEditors", arguments: nil) { _ in target.performClose(nil); result(["requested": true]) }
      }
    case "openPanelWindow":
      let requested = (args["panels"] as? [String] ?? []).filter { !$0.isEmpty }
      guard !requested.isEmpty else { fail(result, "No supported panels requested"); return }
      result(session.openPanelWindow(requested))
    case "attach":
      let view = args["view"] as? String ?? ProbeHost.outputView
      perform(result, work: { try self.session.runtime.status() }) { status in
        guard self.attachment.attach(reusing: (args["attachmentId"] as? NSNumber)?.intValue) != nil else {
          throw ProbeFailure.message("Stale UI attachment")
        }
        try self.ensureTexture(view)
        return self.envelope(status)
      }
    case "detach":
      let detached = (args["attachmentId"] as? NSNumber).map { attachment.detach($0.intValue) } ?? false
      if detached { detachTexture() }
      result(["detached": detached, "windowId": id])
    case "pickOpen", "pickImport":
      let panel = NSOpenPanel()
      let importing = call.method == "pickImport"
      panel.canChooseDirectories = importing
      panel.canChooseFiles = true
      panel.allowsMultipleSelection = importing
      if importing {
        let types = ((args["extensions"] as? [String]) ?? []).compactMap { UTType(filenameExtension: $0) }
        if !types.isEmpty { panel.allowedContentTypes = types + [.folder] }
      } else {
        panel.allowedContentTypes = [UTType(filenameExtension: "rrd") ?? .data]
      }
      openPicker(panel, result: result, multiple: importing)
    case "pickSave", "pickExport":
      let panel = NSSavePanel()
      panel.nameFieldStringValue = args["name"] as? String ?? "Untitled.rrd"
      panel.allowedContentTypes = [UTType(filenameExtension: call.method == "pickExport" ? "mp4" : "rrd") ?? .data]
      let completion: (NSApplication.ModalResponse) -> Void = { response in result(response == .OK ? panel.url?.path : nil) }
      if let window { panel.beginSheetModal(for: window, completionHandler: completion) }
      else { panel.begin(completionHandler: completion) }
    case "reveal":
      guard let path = args["path"] as? String else { fail(result, "Missing file path"); return }
      NSWorkspace.shared.activateFileViewerSelecting([URL(fileURLWithPath: path)])
      result(true)
    case "openFile":
      // The file in whatever the system opens it with; nothing of ours.
      guard let path = args["path"] as? String else { fail(result, "Missing file path"); return }
      result(NSWorkspace.shared.open(URL(fileURLWithPath: path)))
    case "open":
      guard isMain else { fail(result, "Open documents from the main window"); return }
      guard let path = args["path"] as? String else { fail(result, "open requires path"); return }
      if let expected = (args["attachmentId"] as? NSNumber)?.intValue, attachment.id != expected {
        fail(result, "Stale UI attachment"); return
      }
      session.terminationApproved = false
      session.stopPlayback()
      rendering = false
      perform(result, work: {
        let status = try self.session.runtime.open(path: path)
        return status
      }) { status in
        guard self.attachment.attach(reusing: (args["attachmentId"] as? NSNumber)?.intValue) != nil else {
          throw ProbeFailure.message("Stale UI attachment")
        }
        self.session.epoch &+= 1
        self.session.clearFrames()
        self.session.broadcast(status, origin: self)
        return self.envelope(status)
      }
    case "noteClipboard":
      if let image = NSImage(pasteboard: .general), let tiff = image.tiffRepresentation, let bitmap = NSBitmapImageRep(data: tiff), let png = bitmap.representation(using: .png, properties: [:]) {
        result(["png": png.base64EncodedString()])
      } else { result(["text": NSPasteboard.general.string(forType: .string) ?? ""]) }
    case "easeModel":
      var query = args
      query["op"] = "easeModel"
      guard let data = try? JSONSerialization.data(withJSONObject: query), let command = String(data: data, encoding: .utf8) else { fail(result, "Invalid easing model query"); return }
      perform(result, work: { try self.session.runtime.request(command) }) { $0 }
    case "request":
      guard var command = args["command"] as? String else { fail(result, "request requires JSON command string"); return }
      var parsed: [String: Any]?
      if let data = command.data(using: .utf8) { parsed = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any] }
      // A specimen or a measurement answers the window that asked; only a
      // document change is worth waking the others with.
      let asked = ["visualSample", "renderInfo", "easeModel"].contains(parsed?["op"] as? String ?? "")
      if args["deferSnapshot"] != nil || args["knownSnapshotId"] != nil || args["knownReferenceId"] != nil {
        guard var query = parsed else { fail(result, "Invalid command JSON"); return }
        for key in ["deferSnapshot", "knownSnapshotId", "knownReferenceId"] { if let value = args[key] { query[key] = value } }
        guard let data = try? JSONSerialization.data(withJSONObject: query) else { fail(result, "Invalid snapshot context"); return }
        command = String(decoding: data, as: UTF8.self)
      }
      perform(result, work: {
        let status = try self.session.runtime.request(command)
        return status
      }) { status in
        let operation = parsed?["op"] as? String
        if operation == "play" { self.session.startPlayback() }
        if operation == "pause" { self.session.stopPlayback() }
        if !asked, status["error"] == nil { self.session.broadcast(status, origin: self) }
        return status
      }
    case "render":
      guard !rendering else { fail(result, "This window already has a render pending"); return }
      rendering = true
      perform(result, isRender: true, work: {
        if args["playing"] as? Bool == true {
          _ = try self.session.runtime.request("{\"op\":\"tick\",\"quiet\":true}")
        } else if let frame = args["frame"] as? NSNumber {
          _ = try self.session.runtime.request("{\"op\":\"seek\",\"quiet\":true,\"frame\":\(frame.int64Value)}")
        }
        return try self.session.runtime.request("{\"op\":\"renderInfo\"}")
      }) { info in
        guard let views = info["views"] as? [[String: Any]], !views.isEmpty else {
          throw ProbeFailure.message("Document dimensions missing")
        }
        for view in views { if let name = view["view"] as? String { try self.ensureTexture(name) } }
        _ = try self.session.ensureSurfaces(views)
        let targets = self.session.targets(for: views)
        guard !targets.isEmpty else { throw ProbeFailure.message("No render surface") }
        let (status, rendered) = try self.session.runtime.renderInto(
          targets, known: args["knownSnapshotId"], references: args["knownReferenceId"]
        )
        let buffers = Dictionary(uniqueKeysWithValues: targets.map { ($0.view, $0.keepAlive) })
        self.session.broadcast(status, buffers: buffers, origin: self, frameReady: true, frameOnly: args["frame"] != nil || args["playing"] as? Bool == true)
        var reply = self.envelope(status, frameReady: true)
        reply["renderedFrames"] = rendered
        reply["frameOnly"] = args["frame"] != nil || args["playing"] as? Bool == true
        return reply
      }
    case "close":
      if !isMain {
        detachTexture()
        result(["detached": true, "windowId": id])
        return
      }
      session.epoch &+= 1
      session.stopPlayback()
      rendering = false
      session.clearFrames()
      session.state = [:]
      perform(result, work: { self.session.runtime.close(); return ["closed": true] }) { state in
        for host in self.session.hosts.allObjects where !host.closed && host.attached {
          host.channel.invokeMethod("documentClosed", arguments: nil)
        }
        return state
      }
    default:
      result(FlutterMethodNotImplemented)
    }
  }

  /// Runtime work runs here, on the main thread, where Dart's own calls run too.
  private func perform<T>(_ result: FlutterResult, isRender: Bool = false,
                          work: () throws -> T, finish: (T) throws -> Any) {
    precondition(Thread.isMainThread)
    let outcome = Result { try work() }
    if isRender { rendering = false }
    do { result(try finish(outcome.get())) }
    catch { fail(result, String(describing: error)) }
  }

  private func fail(_ result: FlutterResult, _ message: String) {
    result(FlutterError(code: "probe", message: message, details: nil))
  }
}

final class PanelFlutterWindow: NSWindow, NSWindowDelegate, NSDraggingDestination {
  private var host: ProbeHost?
  private let panelID: String
  private var closeApproved = false
  private var closePending = false
  init(identifier: String, panels: [String]) {
    panelID = identifier
    super.init(contentRect: NSRect(x: 0, y: 0, width: 760, height: 540),
               styleMask: [.titled, .closable, .miniaturizable, .resizable], backing: .buffered, defer: false)
    isReleasedWhenClosed = false
    title = "Motolii · " + panels.joined(separator: " / ")
    minSize = NSSize(width: 320, height: 220)
    delegate = self
    let controller = FlutterViewController()
    controller.mouseTrackingMode = .always
    contentViewController = controller
    RegisterGeneratedPlugins(registry: controller)
    host = ProbeHost.install(controller: controller, window: self, id: identifier, panels: panels, main: false)
    registerForDraggedTypes([.fileURL])
    setContentSize(NSSize(width: 760, height: 540))
  }
  private func paths(_ sender: NSDraggingInfo) -> [String] {
    let urls = sender.draggingPasteboard.readObjects(forClasses: [NSURL.self], options: [.urlReadingFileURLsOnly: true]) as? [URL] ?? []
    return urls.filter { $0.isFileURL }.map { $0.path }
  }
  func draggingEntered(_ sender: NSDraggingInfo) -> NSDragOperation {
    let files = !paths(sender).isEmpty
    if files { host?.dragHover(true) }
    return files ? .copy : []
  }
  func draggingExited(_ sender: NSDraggingInfo?) { host?.dragHover(false) }
  func performDragOperation(_ sender: NSDraggingInfo) -> Bool {
    host?.dragHover(false)
    let files = paths(sender); guard !files.isEmpty else { return false }
    let point = contentView?.convert(sender.draggingLocation, from: nil) ?? sender.draggingLocation
    host?.filesDropped(files, point: [point.x, contentView?.isFlipped == true ? point.y : (contentView?.bounds.height ?? frame.height) - point.y])
    return true
  }
  func windowShouldClose(_ sender: NSWindow) -> Bool {
    if closeApproved || host?.attached != true { return true }
    if closePending { return false }
    closePending = true
    host?.channel.invokeMethod("flushEditors", arguments: nil) { [weak self] _ in
      self?.closeApproved = true
      self?.performClose(nil)
    }
    return false
  }
  func windowWillClose(_ notification: Notification) {
    host?.detachWindow()
    host = nil
    ProbeSession.shared.windows.removeValue(forKey: panelID)
  }
}
